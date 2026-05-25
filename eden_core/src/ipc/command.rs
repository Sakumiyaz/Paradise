//! # EdenCommand: Protocolo de Comunicación Bidireccional
//!
//! Wrapper de alto nivel sobre el socket Unix para manejar:
//! - Eventos de Rust a Python (nacimientos, muertes, Nomos)
//! - Comandos de Python a Rust (inyección de energía, escoria)
//!
//! ## Uso
//!
//! ```rust,no_run
//! use eden_core::ipc::{Comando, EdenCommand, Evento};
//!
//! # fn main() -> std::io::Result<()> {
//! let mut cmd = EdenCommand::server("/tmp/eden_core.sock")?;
//!
//! // Enviar evento a Python
//! cmd.enviar_evento(Evento::NacioAuton { id: 1, x: 10.0, y: 20.0 })?;
//!
//! // Recibir comandos de Python (non-blocking)
//! while let Some(comando) = cmd.recibir_comando()? {
//!     match comando {
//!         Comando::InyectarEnergon { x, y, cantidad } => {
//!             let _ = (x, y, cantidad);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![allow(dead_code)]

use super::channel::Channel;
use super::message::{Message, MessageType};
use std::io;
use std::path::Path;

/// Ruta del socket por defecto
const SOCKET_PATH: &str = "/tmp/eden_core.sock";

/// Eventos que Rust envía a Python
#[derive(Debug, Clone)]
pub enum Evento {
    /// Un Auton nació
    NacioAuton { id: u64, x: f64, y: f64 },
    /// Un Auton murió
    MurioAuton { id: u64, causa: String },
    /// Se detectó un Nomos
    NomoFormado { tipo: String, x: f64, y: f64 },
    /// Bifurcación de Auton
    Bifurcacion { padre_id: u64, hijo_id: u64 },
    /// Estado del ecosistema
    EcosistemaState {
        autons_vivos: u32,
        energia_total: f64,
        escoria_total: f64,
        densidad_promedio: f64,
    },
}

impl Evento {
    /// Convierte evento a Message
    pub fn to_message(&self) -> io::Result<Message> {
        match self {
            Evento::NacioAuton { id, x, y } => Message::evento_nacio_auton(*id, *x, *y),
            Evento::MurioAuton { id, causa } => Message::evento_murio_auton(*id, causa),
            Evento::NomoFormado { tipo, x, y } => Message::evento_nomo_formado(tipo, *x, *y),
            Evento::Bifurcacion { padre_id, hijo_id } => {
                Message::evento_bifurcacion(*padre_id, *hijo_id)
            }
            Evento::EcosistemaState {
                autons_vivos,
                energia_total,
                escoria_total,
                densidad_promedio,
            } => Message::evento_ecosistema_state(
                *autons_vivos,
                *energia_total,
                *escoria_total,
                *densidad_promedio,
            ),
        }
    }
}

/// Comandos que Python envía a Rust
#[derive(Debug, Clone)]
pub enum Comando {
    /// Inyectar energía en coordenadas
    InyectarEnergon { x: f64, y: f64, cantidad: f64 },
    /// Aumentar escoria en radio
    AumentarEscoria {
        x: f64,
        y: f64,
        radio: f64,
        cantidad: f64,
    },
    /// Forzar bifurcación de Auton
    ForzarBifurcacion { id: u64 },
    /// Eliminar un Auton
    EliminarAuton { id: u64 },
    /// Pausar/reanudar simulación
    PausarSimulacion { pausar: bool },
    /// Desconocido
    Desconocido { tipo: u8, payload: Vec<u8> },
}

impl Comando {
    /// Parsea un Message a Comando
    pub fn from_message(msg: &Message) -> Option<Self> {
        let msg_type = msg.msg_type()?;

        if !msg_type.is_command() {
            return None;
        }

        let payload = &msg.payload;
        let json_str = String::from_utf8_lossy(payload);

        // Simple JSON parsing sin dependencia externa
        match msg_type {
            MessageType::CmdInyectarEnergon => {
                // Parse: {"x":..., "y":..., "cantidad":...}
                let x = json_extract_f64(&json_str, "x")?;
                let y = json_extract_f64(&json_str, "y")?;
                let cantidad = json_extract_f64(&json_str, "cantidad")?;
                Some(Comando::InyectarEnergon { x, y, cantidad })
            }
            MessageType::CmdAumentarEscoria => {
                let x = json_extract_f64(&json_str, "x")?;
                let y = json_extract_f64(&json_str, "y")?;
                let radio = json_extract_f64(&json_str, "radio")?;
                let cantidad = json_extract_f64(&json_str, "cantidad")?;
                Some(Comando::AumentarEscoria {
                    x,
                    y,
                    radio,
                    cantidad,
                })
            }
            MessageType::CmdForzarBifurcacion => {
                let id = json_extract_u64(&json_str, "id")?;
                Some(Comando::ForzarBifurcacion { id })
            }
            MessageType::CmdEliminarAuton => {
                let id = json_extract_u64(&json_str, "id")?;
                Some(Comando::EliminarAuton { id })
            }
            MessageType::CmdPausarSimulacion => {
                let pausar = json_extract_bool(&json_str, "pausar")?;
                Some(Comando::PausarSimulacion { pausar })
            }
            _ => Some(Comando::Desconocido {
                tipo: msg_type as u8,
                payload: payload.clone(),
            }),
        }
    }
}

/// Extrae un f64 de JSON simple
fn json_extract_f64(json: &str, key: &str) -> Option<f64> {
    let search = format!("\"{}\":", key);
    let start = json.find(&search)? + search.len();

    // Skip whitespace
    let mut pos = start;
    while pos < json.len() && json.is_char_boundary(pos) {
        let c = json[pos..].chars().next()?;
        if c == ' ' || c == '\t' || c == '\n' {
            pos += c.len_utf8();
        } else {
            break;
        }
    }

    // Extract number
    let end = json[pos..].find(|c: char| !c.is_numeric() && c != '.' && c != '-')? + pos;
    let num_str = &json[pos..end];
    num_str.parse().ok()
}

/// Extrae un u64 de JSON simple
fn json_extract_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{}\":", key);
    let start = json.find(&search)? + search.len();

    let mut pos = start;
    while pos < json.len() && json.is_char_boundary(pos) {
        let c = json[pos..].chars().next()?;
        if c == ' ' || c == '\t' || c == '\n' {
            pos += c.len_utf8();
        } else {
            break;
        }
    }

    let end = json[pos..].find(|c: char| !c.is_numeric())? + pos;
    let num_str = &json[pos..end];
    num_str.parse().ok()
}

/// Extrae un bool de JSON simple
fn json_extract_bool(json: &str, key: &str) -> Option<bool> {
    let search = format!("\"{}\":", key);
    let start = json.find(&search)? + search.len();

    let mut pos = start;
    while pos < json.len() && json.is_char_boundary(pos) {
        let c = json[pos..].chars().next()?;
        if c == ' ' || c == '\t' || c == '\n' {
            pos += c.len_utf8();
        } else {
            break;
        }
    }

    let remaining = &json[pos..];
    if remaining.starts_with("true") {
        Some(true)
    } else if remaining.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

/// EdenCommand: Wrapper bidireccional para el socket Unix
pub struct EdenCommand {
    /// Canal de comunicación
    channel: Channel,
    /// Buffer de comandos pendientes
    comando_buffer: Vec<Comando>,
}

impl EdenCommand {
    /// Crea un servidor (Rust escucha, Python conecta)
    pub fn server<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        // Guardar el path como string antes de pasarlo
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let path_for_remove = std::path::PathBuf::from(&path_str);

        // Eliminar archivo viejo si existe
        if path_for_remove.exists() {
            std::fs::remove_file(&path_for_remove).ok();
        }

        let channel = Channel::bind(path)?;

        Ok(EdenCommand {
            channel,
            comando_buffer: Vec::new(),
        })
    }

    /// Crea un cliente (Python escucha, Rust conecta)
    pub fn client<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let channel = Channel::connect(path)?;
        Ok(EdenCommand {
            channel,
            comando_buffer: Vec::new(),
        })
    }

    /// Crea conexión por defecto al socket estándar
    pub fn default() -> io::Result<Self> {
        Self::client(SOCKET_PATH)
    }

    /// Envía un evento a Python
    pub fn enviar_evento(&mut self, evento: Evento) -> io::Result<()> {
        let msg = evento.to_message()?;
        self.channel.send(&msg)?;
        Ok(())
    }

    /// Recibe comandos de Python (non-blocking, retorna None si no hay datos)
    pub fn recibir_comando(&mut self) -> io::Result<Option<Comando>> {
        // Intentar recibir mensaje
        match self.channel.recv() {
            Ok(msg) => {
                if let Some(comando) = Comando::from_message(&msg) {
                    Ok(Some(comando))
                } else {
                    // Mensaje que no es comando (evento recibido de vuelta, etc.)
                    Ok(None)
                }
            }
            Err(e) => {
                // WouldBlock significa no hay datos
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Recibe todos los comandos pendientes (non-blocking)
    pub fn recibir_comandos_todos(&mut self) -> io::Result<Vec<Comando>> {
        let mut comandos = Vec::new();

        while let Some(cmd) = self.recibir_comando()? {
            comandos.push(cmd);
        }

        Ok(comandos)
    }

    /// Envía respuesta a un comando (ack)
    pub fn enviar_respuesta(&mut self, seq: u64, ok: bool, mensaje: &str) -> io::Result<()> {
        let payload = if ok {
            format!("{{\"ok\":true,\"msg\":\"{}\"}}", mensaje)
        } else {
            format!("{{\"ok\":false,\"msg\":\"{}\"}}", mensaje)
        };

        let mut msg = Message::new(MessageType::Response, payload.into_bytes())?;
        msg.header.sequence = seq;
        self.channel.send(&msg)?;

        Ok(())
    }

    /// Configura non-blocking mode
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.channel.endpoint.set_nonblocking(nonblocking)
    }

    /// Obtiene la ruta del socket
    pub fn path(&self) -> &str {
        &self.channel.name
    }
}

/// Builder para crear EdenCommand con opciones
pub struct EdenCommandBuilder {
    path: String,
    is_server: bool,
    buffer_size: usize,
    nonblocking: bool,
}

impl EdenCommandBuilder {
    pub fn new() -> Self {
        Self {
            path: SOCKET_PATH.to_string(),
            is_server: false,
            buffer_size: 65536,
            nonblocking: true,
        }
    }

    /// Define la ruta del socket
    pub fn path<P: Into<String>>(mut self, path: P) -> Self {
        self.path = path.into();
        self
    }

    /// Configura como servidor (Rust escucha)
    pub fn server(mut self) -> Self {
        self.is_server = true;
        self
    }

    /// Configura como cliente (Rust conecta)
    pub fn client_mode(mut self) -> Self {
        self.is_server = false;
        self
    }

    /// Tamaño del buffer
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Non-blocking mode
    pub fn nonblocking(mut self, nb: bool) -> Self {
        self.nonblocking = nb;
        self
    }

    /// Build la conexión
    pub fn build(self) -> io::Result<EdenCommand> {
        let mut cmd = if self.is_server {
            EdenCommand::server(&self.path)?
        } else {
            EdenCommand::client(&self.path)?
        };

        cmd.set_nonblocking(self.nonblocking)?;
        Ok(cmd)
    }
}

impl Default for EdenCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_extract_f64() {
        let json = r#"{"x": 123.45, "y": 678.9}"#;
        assert_eq!(json_extract_f64(json, "x"), Some(123.45));
        assert_eq!(json_extract_f64(json, "y"), Some(678.9));
        assert_eq!(json_extract_f64(json, "z"), None);
    }

    #[test]
    fn test_json_extract_u64() {
        let json = r#"{"id": 42}"#;
        assert_eq!(json_extract_u64(json, "id"), Some(42));
    }

    #[test]
    fn test_json_extract_bool() {
        let json = r#"{"pausar": true}"#;
        assert_eq!(json_extract_bool(json, "pausar"), Some(true));

        let json2 = r#"{"pausar": false}"#;
        assert_eq!(json_extract_bool(json2, "pausar"), Some(false));
    }

    #[test]
    fn test_evento_to_message() {
        let evento = Evento::NacioAuton {
            id: 42,
            x: 10.0,
            y: 20.0,
        };
        let msg = evento.to_message().unwrap();
        assert!(msg.is_event());

        let evento2 = Evento::MurioAuton {
            id: 1,
            causa: "senescence".to_string(),
        };
        let msg2 = evento2.to_message().unwrap();
        assert!(msg2.is_event());
    }

    #[test]
    fn test_comando_parse() {
        let msg = Message::cmd_inyectar_energon(10.0, 20.0, 100.0).unwrap();
        let cmd = Comando::from_message(&msg);

        assert!(
            matches!(cmd, Some(Comando::InyectarEnergon { x, y, cantidad })
            if (x - 10.0).abs() < 0.001 && (y - 20.0).abs() < 0.001 && (cantidad - 100.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_builder() {
        let cmd = EdenCommandBuilder::new()
            .path("/tmp/test_eden.sock")
            .server()
            .nonblocking(true)
            .build();

        // El build puede fallar si hay errores de I/O, pero la construcción del builder no
        // En un test real, usaríamos tempdir
        let _ = cmd;
    }
}
