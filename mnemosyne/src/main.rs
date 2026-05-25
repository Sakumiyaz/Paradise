//! Mnemosyne - Pure Rust knowledge graph server
//! Rewritten without external dependencies (no tokio, dashmap, rusqlite, rayon, flate2, uuid, chrono)
//!
//! Functionality:
//! - Tiered storage (Hot/Warm/Cold memory layers)
//! - Events from purgatory (evento_vital)
//! - Evolution record tracking
//! - Vector similarity search

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

mod layers;
mod protocol;

use layers::{EvolutionRecord, KnowledgeGraph, Layer, VitalEvent};
use protocol::{Request, Response};

// Get socket path in protected directory (~/.eden/)
fn get_socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ubuntu".to_string());
    PathBuf::from(&home)
        .join(".eden")
        .join("eden_mnemosyne.sock")
}

const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB

fn log_op(op: &str, layer: &'static str, latency_ms: u128) {
    println!(
        "[MNEMOSYNE] op={} layer={} latency={}ms",
        op, layer, latency_ms
    );
}

fn handle_client(
    stream: &mut UnixStream,
    kg: &KnowledgeGraph,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accumulated = Vec::with_capacity(8192);
    let mut temp_buf = [0u8; 8192];

    loop {
        match stream.read(&mut temp_buf) {
            Ok(0) => break,
            Ok(n) => {
                // Check request size
                if accumulated.len() + n > MAX_REQUEST_SIZE {
                    let resp = Response::error("Request too large".to_string(), 413);
                    let resp_json = serde_json::to_string(&resp)?;
                    stream.write_all(resp_json.as_bytes())?;
                    break;
                }
                accumulated.extend_from_slice(&temp_buf[..n]);
            }
            Err(e) => {
                eprintln!("[MNEMOSYNE] read_error={}", e);
                break;
            }
        }

        // Try to parse JSON - check if we have complete JSON object
        let json_str = String::from_utf8_lossy(&accumulated);

        // Check if it looks like complete JSON (ends with })
        if json_str.ends_with('}') {
            if let Ok(request) = serde_json::from_str::<Request>(&json_str) {
                // Success - we have complete request
                let start = Instant::now();
                let resp: Response;

                match request {
                    Request::CreateNode {
                        label,
                        properties,
                        ttl,
                        embeddings,
                    } => {
                        resp = match kg.create_node(label, properties, ttl, embeddings) {
                            Ok(node) => Response::ok(node),
                            Err(e) => Response::error(e.to_string(), 500),
                        };
                        log_op(
                            "create_node",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::DeleteNode { id } => {
                        resp = match kg.delete_node(&id) {
                            Ok(_) => Response::ok(serde_json::json!({ "deleted": id })),
                            Err(_) => Response::error("Node not found".to_string(), 404),
                        };
                        log_op(
                            "delete_node",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::CreateEdge {
                        source,
                        target,
                        relation,
                    } => {
                        resp = match kg.create_edge(&source, &target, &relation) {
                            Ok(_) => Response::ok(serde_json::json!({ "edge_created": true })),
                            Err(e) => Response::error(e.to_string(), 500),
                        };
                        log_op(
                            "create_edge",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::GetNode { id } => {
                        resp = match kg.get_node(&id) {
                            Ok(node) => Response::ok(node),
                            Err(_) => Response::error("Node not found".to_string(), 404),
                        };
                        log_op("get_node", Layer::Hot.name(), start.elapsed().as_millis());
                    }
                    Request::GetStaleNodes { days } => {
                        resp = match kg.get_stale_nodes(days) {
                            Ok(nodes) => Response::ok(nodes),
                            Err(e) => Response::error(e.to_string(), 500),
                        };
                        log_op(
                            "get_stale_nodes",
                            Layer::Warm.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::SearchSimilar { embedding, top_k } => {
                        resp = match kg.search_similar(&embedding, top_k) {
                            Ok(results) => Response::ok(results),
                            Err(e) => Response::error(e.to_string(), 500),
                        };
                        log_op(
                            "search_similar",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::StoreEvolution {
                        generacion,
                        mutacion_id,
                        patron_origen,
                        resultado,
                        peso_final,
                        sobrevivio,
                        timestamp,
                    } => {
                        let record = EvolutionRecord {
                            generacion,
                            mutacion_id,
                            patron_origen,
                            resultado,
                            peso_final,
                            sobrevivio,
                            timestamp,
                            layer: Layer::Hot,
                        };
                        resp = match kg.store_evolution_record(record) {
                            Ok(_) => Response::ok(serde_json::json!({ "stored": true })),
                            Err(e) => Response::error(e.to_string(), 500),
                        };
                        log_op(
                            "store_evolution",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::GetEvolution { mutacion_id } => {
                        resp = match kg.get_evolution_record(&mutacion_id) {
                            Some(record) => Response::ok(record),
                            None => Response::error("Evolution record not found".to_string(), 404),
                        };
                        log_op(
                            "get_evolution",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::GetAllEvolution {} => {
                        let records = kg.get_all_evolution_records();
                        resp = Response::ok(records);
                        log_op(
                            "get_all_evolution",
                            Layer::Hot.name(),
                            start.elapsed().as_millis(),
                        );
                    }
                    Request::EventoVital {
                        tipo,
                        accion,
                        ram_libre,
                        cpu_pct,
                        timestamp,
                    } => {
                        let event = VitalEvent {
                            tipo,
                            accion,
                            ram_libre,
                            cpu_pct,
                            timestamp,
                        };
                        kg.log_vital_event(event);
                        resp = Response::ok(serde_json::json!({ "logged": true }));
                        log_op("evento_vital", "VITAL", start.elapsed().as_millis());
                    }
                    Request::Ping {} => {
                        resp = Response::ok(serde_json::json!({
                            "status": "alive",
                            "version": "0.2.0-pure",
                        }));
                        log_op("ping", "SYSTEM", start.elapsed().as_millis());
                    }
                }

                let resp_json = serde_json::to_string(&resp)?;
                stream.write_all(resp_json.as_bytes())?;

                // Clear accumulated buffer after successful processing
                accumulated.clear();
            }
        }
        // If JSON is incomplete, continue reading
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();

    // Ensure ~/.eden directory exists
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Error creating {}: {}", parent.display(), e))?;
        }
    }

    // Remove old socket if exists
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    // Create knowledge graph
    let kg = Arc::new(KnowledgeGraph::new()?);
    println!("[MNEMOSYNE] initialized");

    // Create Unix socket listener
    let listener =
        UnixListener::bind(&socket_path).map_err(|e| format!("Failed to bind socket: {}", e))?;

    // Set socket permissions to 0o600 (owner only)
    use std::os::unix::fs::PermissionsExt;
    if let Ok(mut perms) = fs::metadata(&socket_path).map(|m| m.permissions()) {
        perms.set_mode(0o600);
        let _ = fs::set_permissions(&socket_path, perms);
    }

    println!("[MNEMOSYNE] listening socket={}", socket_path.display());

    // Accept connections in loop
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {
                let kg = kg.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(&mut stream, &kg) {
                        eprintln!("[MNEMOSYNE] client_error={}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[MNEMOSYNE] accept_error={}", e);
            }
        }
    }

    // Graceful shutdown on exit
    println!("[MNEMOSYNE] shutting down...");
    kg.graceful_shutdown()?;
    println!("[MNEMOSYNE] shutdown complete");

    Ok(())
}
