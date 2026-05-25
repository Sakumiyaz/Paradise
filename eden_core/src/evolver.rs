//! # Evolver - Evolución A-Life
//!
//! Sistema de evolución de patrones para organismos A-Life.
//! 100% Rust puro - sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::membrain::MemBrain;

const EVOLUTION_INTERVAL_SECS: u64 = 1800; // 30 minutes
const CURIOSITY_MAX_PER_SESSION: u32 = 100;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Simple hash function
fn hash_bytes(data: &[u8]) -> u64 {
    let mut hash: u64 = 0x9e3779b97f4a7c15;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.rotate_left(5)
            ^ (byte as u64)
                .wrapping_mul(0xbf58476d1ce4e5b9)
                .wrapping_add(i as u64);
    }
    hash ^ (hash >> 32)
}

/// Simple pseudo-random based on time
fn rand_simple() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    let hash = hasher.finish();
    (hash as f64) / (u64::MAX as f64)
}

pub fn start_evolver() {
    println!(
        "[EVOLVER] Evolución de Eden iniciada - ciclo cada {}s",
        EVOLUTION_INTERVAL_SECS
    );

    while RUNNING.load(Ordering::SeqCst) {
        if should_explore_with_curiosity() {
            println!("[EVOLVER] Condiciones de curiosidad detectadas - modo exploración");
            run_curiosity_session();
        }

        evolve();

        for _ in 0..EVOLUTION_INTERVAL_SECS {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    println!("[EVOLVER] Detenido");
}

pub fn stop_evolver() {
    RUNNING.store(false, Ordering::SeqCst);
}

fn evolve() {
    let mut membrain = match MemBrain::new("/home/ubuntu/eden_kg") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[EVOLVER] Error inicializando MemBrain: {}", e);
            return;
        }
    };

    let latest_eval = get_latest_evaluation(&mut membrain);
    let tasa = latest_eval.0;
    let timestamp = now_secs();

    if tasa < 0.4 {
        let duplicados = duplicate_strong_patterns(&mut membrain, timestamp);
        let eliminados = eliminate_weak_patterns(&mut membrain, timestamp);

        log_evolution(
            &mut membrain,
            "CRISIS_IMPROVEMENT",
            &format!(
                "tasa={:.2}% duplicados={} eliminados={}",
                tasa * 100.0,
                duplicados,
                eliminados
            ),
            timestamp,
        );

        println!(
            "[EVOLVER] Crisis - tasa={:.2}% duplicados={} eliminados={}",
            tasa * 100.0,
            duplicados,
            eliminados
        );
    } else if tasa > 0.8 {
        if let Some(variante_hash) = generate_variant(&mut membrain, &latest_eval.1, timestamp) {
            log_evolution(
                &mut membrain,
                "VARIANT_CREATION",
                &format!("patron_fuerte={} variante={}", latest_eval.1, variante_hash),
                timestamp,
            );

            println!(
                "[EVOLVER] Variante creada - patron_fuerte={} variante={}",
                latest_eval.1, variante_hash
            );
        }
    } else {
        log_evolution(
            &mut membrain,
            "STABLE",
            &format!("tasa={:.2}% - sin mutacion", tasa * 100.0),
            timestamp,
        );
        println!("[EVOLVER] Estable - tasa={:.2}%", tasa * 100.0);
    }
}

fn get_latest_evaluation(brain: &mut MemBrain) -> (f64, String, String, i64) {
    let results = brain.search(b"eval:");

    let mut best_eval: Option<(f64, String, String, i64)> = None;

    for data in results {
        if data.len() > 50 {
            // Parse evaluation data
            let tasa = read_f64(&data, 0);
            if tasa > 0.0 {
                if let Some(best) = &best_eval {
                    if tasa > best.0 {
                        best_eval = Some((tasa, "patron".to_string(), "debil".to_string(), 0));
                    }
                } else {
                    best_eval = Some((tasa, "patron".to_string(), "debil".to_string(), 0));
                }
            }
        }
    }

    best_eval.unwrap_or((0.5, "none".to_string(), "none".to_string(), 0))
}

fn read_f64(bytes: &[u8], offset: usize) -> f64 {
    if bytes.len() < offset + 8 {
        return 0.0;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[offset..offset + 8]);
    f64::from_le_bytes(arr)
}

fn duplicate_strong_patterns(brain: &mut MemBrain, timestamp: i64) -> i32 {
    let patterns = brain.search(b"weight:");

    let mut count = 0;
    for data in patterns {
        if data.len() > 16 {
            let peso = read_f64(&data, 8);
            if peso > 0.7 {
                let new_key = format!("weight:dup_{}_{}", timestamp, count);
                let mut new_data = data.clone();
                // Modify slightly
                new_data.extend_from_slice(&timestamp.to_le_bytes());
                brain.gaba(new_key.as_bytes(), new_data);
                count += 1;
            }
        }
    }
    count
}

fn eliminate_weak_patterns(_brain: &mut MemBrain, _timestamp: i64) -> i32 {
    // En MemBrain, los patrones débiles simplemente no se referencian
    // La próxima operación de limpieza los eliminará automáticamente
    0
}

fn generate_variant(brain: &mut MemBrain, patron_fuerte: &str, timestamp: i64) -> Option<String> {
    let key = format!("weight:{}", patron_fuerte);
    let results = brain.search(key.as_bytes());

    if let Some(data) = results.first() {
        let new_key = format!("variant:{}_{}", patron_fuerte, timestamp);
        let mut new_data = data.clone();

        // Modificar peso a 0.5
        let old_peso = read_f64(&new_data, 8);
        let new_peso = (old_peso * 0.9).max(0.3);

        let peso_bytes = new_peso.to_le_bytes();
        new_data.splice(8..16, peso_bytes);

        brain.dopa(new_key.as_bytes(), new_data);

        Some(new_key)
    } else {
        None
    }
}

fn log_evolution(brain: &mut MemBrain, mutation_type: &str, description: &str, timestamp: i64) {
    let key = format!("evolution:{}:{}", mutation_type, timestamp);
    let mut data = Vec::new();

    data.extend_from_slice(mutation_type.as_bytes());
    data.push(0);
    data.extend_from_slice(description.as_bytes());
    data.push(0);
    data.extend_from_slice(&timestamp.to_le_bytes());

    brain.dopa(key.as_bytes(), data);
}

// =============================================================================
// CURIOSITY MODULE
// =============================================================================

struct CuriosityTracker {
    low_cpu_since: Option<i64>,
    last_cpu_check: f32,
}

impl Default for CuriosityTracker {
    fn default() -> Self {
        Self {
            low_cpu_since: None,
            last_cpu_check: 50.0,
        }
    }
}

fn should_explore_with_curiosity() -> bool {
    let now = now_millis();
    let cpu = read_cpu_usage();
    let ram = read_ram_free_mb();
    let energy = get_cell_energy();

    static TRACKER: std::sync::LazyLock<
        std::sync::Mutex<CuriosityTracker>,
        fn() -> std::sync::Mutex<CuriosityTracker>,
    > = std::sync::LazyLock::new(|| std::sync::Mutex::new(CuriosityTracker::default()));

    let mut tracker = TRACKER.lock().unwrap();

    if cpu < 15.0 && energy > 70.0 && ram > 10 * 1024 {
        if tracker.low_cpu_since.is_none() {
            tracker.low_cpu_since = Some(now);
            println!("[CURIOSITY] CPU bajo detectado, iniciando conteo...");
            return false;
        }

        let elapsed = now - tracker.low_cpu_since.unwrap();
        let twenty_minutes_ms = 20 * 60 * 1000;

        if elapsed > twenty_minutes_ms {
            tracker.last_cpu_check = cpu as f32;
            return true;
        }
    } else {
        tracker.low_cpu_since = None;
    }

    tracker.last_cpu_check = cpu as f32;
    false
}

fn run_curiosity_session() {
    println!("[CURIOSITY] Iniciando sesión de exploración autónoma...");

    let mut membrain = match MemBrain::new("/home/ubuntu/eden_kg") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[CURIOSITY] Error inicializando MemBrain: {}", e);
            return;
        }
    };

    let mut explorations = 0;

    while explorations < CURIOSITY_MAX_PER_SESSION {
        let cpu = read_cpu_usage();
        if cpu > 30.0 {
            println!("[CURIOSITY] CPU subió a {:.1}%, pausando exploración", cpu);
            break;
        }

        let adopted = explore_one_pattern(&mut membrain);

        if adopted {
            println!(
                "[CURIOSITY] Exploración {}: variante adoptada!",
                explorations + 1
            );
        } else {
            println!(
                "[CURIOSITY] Exploración {}: variante descartada",
                explorations + 1
            );
        }

        explorations += 1;
        thread::sleep(Duration::from_millis(100));
    }

    println!(
        "[CURIOSITY] Sesión completada: {} exploraciones",
        explorations
    );
}

fn explore_one_pattern(brain: &mut MemBrain) -> bool {
    let patterns = brain.search(b"weight:");

    if patterns.is_empty() {
        println!("[CURIOSITY] No hay patrones para explorar");
        return false;
    }

    // Seleccionar un patrón pseudo-aleatorio
    let seed = now_millis() as usize;
    let pattern_index = seed % patterns.len();

    let origin_data = &patterns[pattern_index];
    let origin_peso = read_f64(origin_data, 8);

    // Generar variante
    let noise = (rand_simple() - 0.5) * 0.2;
    let variant_peso = (origin_peso + noise).clamp(0.0, 1.0);

    let variant_key = format!("variant:cur_{}_{}", now_millis(), exploration_count());

    let mut variant_data = origin_data.clone();
    let peso_bytes = variant_peso.to_le_bytes();
    variant_data.splice(8..16, peso_bytes);

    brain.dopa(variant_key.as_bytes(), variant_data);

    // Log curiosity
    log_curiosity(brain, "origin", &variant_key, variant_peso, noise > 0.0);

    noise > 0.02
}

fn exploration_count() -> u64 {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed)
}

fn log_curiosity(
    brain: &mut MemBrain,
    origin: &str,
    variant: &str,
    variant_peso: f64,
    adopted: bool,
) {
    let key = format!("curiosity:{}", now_millis());
    let mut data = Vec::new();

    data.extend_from_slice(origin.as_bytes());
    data.push(0);
    data.extend_from_slice(variant.as_bytes());
    data.push(0);
    data.extend_from_slice(&variant_peso.to_le_bytes());
    data.push(if adopted { 1 } else { 0 });

    brain.dopa(key.as_bytes(), data);
}

fn read_cpu_usage() -> f64 {
    if let Ok(contents) = fs::read_to_string("/proc/stat") {
        if let Some(first_line) = contents.lines().next() {
            if first_line.starts_with("cpu ") {
                let parts: Vec<u64> = first_line
                    .split_whitespace()
                    .skip(1)
                    .take(8)
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if parts.len() >= 4 {
                    let total: u64 = parts.iter().sum();
                    let idle = parts[3];
                    if total > 0 {
                        return 100.0 * (1.0 - idle as f64 / total as f64);
                    }
                }
            }
        }
    }
    50.0
}

fn read_ram_free_mb() -> u64 {
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if line.starts_with("MemFree:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb / 1024;
                    }
                }
            }
        }
    }
    8192
}

fn get_cell_energy() -> f64 {
    let brain = match MemBrain::new("/home/ubuntu/eden_kg") {
        Ok(m) => m,
        Err(_) => return 50.0,
    };

    let results = brain.search(b"cell:");
    if let Some(data) = results.first() {
        read_f64(&data, 40) // energy offset en CellState
    } else {
        50.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"test";
        let h1 = hash_bytes(data);
        let h2 = hash_bytes(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_rand_simple() {
        let r1 = rand_simple();
        assert!(r1 >= 0.0 && r1 <= 1.0);
    }
}
