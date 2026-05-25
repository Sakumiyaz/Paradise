//! Eden Reason - Binario standalone
//! Motor de razonamiento propio
#![allow(dead_code)]
#![allow(non_snake_case)]

use eden_core::reason::ReasonEngine;

fn main() {
    println!("[REASON] Iniciando cerebro propio de Eden...");

    match ReasonEngine::new() {
        Ok(engine) => {
            println!("[REASON] Motor de razonamiento iniciado");
            if let Err(e) = engine.iniciar_servidor() {
                eprintln!("[REASON] Error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[REASON] Error inicializando: {}", e);
        }
    }
}
