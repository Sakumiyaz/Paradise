//! Cell - Binario standalone para Eden
//! Se lanza desde genesis como proceso hijo con restart automático

use eden_core::cell::CellConfig;
use eden_core::server::CellServer;

fn main() {
    println!("[CELL] Iniciando CellServer como proceso independiente...");

    let config = CellConfig {
        db_path: "/home/ubuntu/eden_kg.db",
        energy_threshold_low: 20,
        energy_threshold_high: 80,
        energy_cost_input: 5,
        energy_gain_pattern: 10,
        energy_gain_learn: 8,
    };

    match CellServer::new(config) {
        Ok(mut server) => {
            println!("[CELL] CellServer iniciado, socket en /tmp/eden_cell.sock");
            if let Err(e) = server.run() {
                eprintln!("[CELL] CellServer error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("[CELL] Error iniciando CellServer: {}", e);
            std::process::exit(1);
        }
    }
}
