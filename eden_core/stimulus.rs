//! Stimulus - Generador de estímulos autónomos para Eden
//! Sin estímulos no hay aprendizaje. Sin aprendizaje no hay vida.

use chrono::Utc;
use rand::Rng;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::thread;

fn main() {
    println!("[STIMULUS] Iniciando motor de estímulos...");
    match StimulusEngine::new() {
        Ok(mut engine) => {
            println!("[STIMULUS] Motor listo. Enviando estímulos cada 30s...");
            engine.run();
        }
        Err(e) => {
            eprintln!("[STIMULUS] Error inicializando: {}", e);
            std::process::exit(1);
        }
    }
}

const CELL_SOCK: &str = "/tmp/eden_cell.sock";
const DB_PATH: &str = "/home/ubuntu/eden_kg.db";

const INPUTS: [&str; 20] = [
    "depurar error memoria",
    "optimizar consulta sql",
    "refactorizar función recursiva",
    "analizar complejidad algoritmo",
    "resolver conflicto git",
    "diseñar esquema base datos",
    "revisar seguridad codigo",
    "comprimir datos binarios",
    "sincronizar procesos paralelos",
    "validar entrada usuario",
    "detectar memory leak",
    "optimizar loop anidado",
    "parsear formato json",
    "manejar excepcion timeout",
    "calcular hash consistente",
    "balancear carga procesos",
    "limpiar cache sináptica",
    "evaluar patron emergente",
    "fusionar ramas divergentes",
    "restaurar estado anterior",
];

#[derive(Serialize, Deserialize)]
struct CellRequest {
    input: Option<String>,
    contexto: Option<String>,
    action: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CellResponse {
    respuesta: String,
    confianza: f32,
    energia: i32,
    ciclos: u64,
}

#[derive(Debug)]
struct StimulusLog {
    timestamp: String,
    input_text: String,
    respuesta: String,
    energia_post: i32,
    exitos: bool,
}

pub struct StimulusEngine {
    conn: Connection,
}

impl StimulusEngine {
    pub fn new() -> Result<Self, String> {
        let conn = Connection::open(DB_PATH)
            .map_err(|e| format!("Error abriendo DB: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS stimulus_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                input_text TEXT NOT NULL,
                respuesta TEXT NOT NULL,
                energia_post INTEGER NOT NULL,
                exitos INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .map_err(|e| format!("Error creando tabla stimulus_log: {}", e))?;

        Ok(StimulusEngine { conn })
    }

    pub fn run(&mut self) {
        println!("[STIMULUS] Motor de estímulos activo - loop cada 30s");

        loop {
            let stim = self.generate_and_send_stimulus();

            if let Err(e) = self.log_stimulus(&stim) {
                eprintln!("[STIMULUS] Error logging: {}", e);
            } else {
                println!(
                    "[STIMULUS] Estímulo \"{}\" | energia: {} | éxito: {}",
                    stim.input_text, stim.energia_post, stim.exitos
                );
            }

            thread::sleep(Duration::from_secs(30));
        }
    }

    fn generate_and_send_stimulus(&self) -> StimulusLog {
        let mut rng = rand::thread_rng();

        let input_text = INPUTS[rng.gen_range(0..INPUTS.len())];
        let timestamp = Utc::now().to_rfc3339();

        let (respuesta, energia_post, exitos) = self.send_to_cell(input_text);

        StimulusLog {
            timestamp,
            input_text: input_text.to_string(),
            respuesta,
            energia_post,
            exitos,
        }
    }

    fn send_to_cell(&self, input: &str) -> (String, i32, bool) {
        let request = CellRequest {
            input: Some(input.to_string()),
            contexto: Some("stimulus".to_string()),
            action: None,
        };

        if let Ok(json) = serde_json::to_string(&request) {
            if let Ok(mut stream) = UnixStream::connect(CELL_SOCK) {
                if stream.write_all(json.as_bytes()).is_ok() {
                    let mut buf = [0u8; 4096];
                    if let Ok(n) = stream.read(&mut buf) {
                        if let Ok(response) = serde_json::from_str::<CellResponse>(&String::from_utf8_lossy(&buf[..n])) {
                            let exitos = response.confianza > 0.5;
                            return (response.respuesta, response.energia, exitos);
                        }
                    }
                }
            }
        }

        ("sin respuesta".to_string(), 0, false)
    }

    fn log_stimulus(&self, stim: &StimulusLog) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO stimulus_log (timestamp, input_text, respuesta, energia_post, exitos)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                stim.timestamp,
                stim.input_text,
                stim.respuesta,
                stim.energia_post,
                stim.exitos as i32,
            ],
        )
        .map_err(|e| format!("Error insertando log: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_stimulus_generation() {
        // Tests would go here
    }
}
