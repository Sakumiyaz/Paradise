//! Unix socket server para comunicación con orchestrator
//! Socket: ~/.eden/eden_cell.sock (protegido, no en /tmp)
//! Protocolo: JSON {input, contexto} -> {respuesta, confianza, energia}
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::cell::{Cell, CellConfig, CellRequest};
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::path::PathBuf;

/// Obtiene la ruta del socket en directorio protegido (~/.eden/)
fn get_socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ubuntu".to_string());
    PathBuf::from(home).join(".eden").join("eden_cell.sock")
}

pub struct CellServer {
    cell: Cell,
    socket_path: PathBuf,
}

impl CellServer {
    pub fn new(config: CellConfig) -> Result<Self, String> {
        let cell = Cell::new(config)?;

        let socket_path = get_socket_path();

        // Crear directorio ~/.eden si no existe
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Error creando directorio {}: {}", parent.display(), e))?;
            }
        }

        // Eliminar socket anterior si existe (ignoramos error si no existe)
        let _ = std::fs::remove_file(&socket_path);

        Ok(CellServer {
            cell,
            socket_path,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| format!("Error creando socket {}: {}", self.socket_path.display(), e))?;

        println!("[EDEN_CORE] Socket server activo en {}", self.socket_path.display());
        println!("[EDEN_CORE] Esperando conexiones del orchestrator...");

        // Configurar permisos del socket a 0o600 (solo owner)
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&self.socket_path).map(|m| m.permissions()) {
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&self.socket_path, perms);
        }

        loop {
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    if let Err(e) = self.handle_connection(&mut stream) {
                        eprintln!("[EDEN_CORE] Error en conexión: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("[EDEN_CORE] Error aceptando conexión: {}", e);
                }
            }
        }
    }

    fn handle_connection(&mut self, stream: &mut UnixStream) -> Result<(), String> {
        loop {
            let mut buffer = [0u8; 4096];

            // Leer request JSON
            let n = match stream.read(&mut buffer) {
                Ok(0) => return Ok(()), // Cliente cerró conexión
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[EDEN_CORE] Error leyendo socket: {}", e);
                    return Err(format!("Error leyendo socket: {}", e));
                }
            };

            let data = String::from_utf8_lossy(&buffer[..n]);
            let request: CellRequest = match serde_json::from_str(&data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[EDEN_CORE] Error parseando JSON: {}", e);
                    continue;
                }
            };

            // Procesar request
            let response = self.cell.process_socket_request(&request)?;

            // Serializar response
            let json_response = serde_json::to_string(&response)
                .map_err(|e| format!("Error serializando response: {}", e))?;

            // Enviar response
            if let Err(e) = stream.write_all(json_response.as_bytes()) {
                eprintln!("[EDEN_CORE] Error escribiendo response: {}", e);
                return Err(format!("Error escribiendo response: {}", e));
            }

            // Guardar estado completo (snapshot del ciclo)
            if let Err(e) = self.cell.save_state() {
                eprintln!("[EDEN_CORE] Error guardando estado: {}", e);
            }
        }
    }

    pub fn get_cell_state(&self) -> (i32, u64, usize) {
        (self.cell.get_energy(), self.cell.get_cycles(), self.cell.get_patterns_count())
    }
}

impl Drop for CellServer {
    fn drop(&mut self) {
        // Limpiar socket al salir
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
