//! # Benchmarks - Medición de Performance 100% Original
//!
//! Sistema de benchmarks sin dependencias externas.
//! Utiliza solo std::time para mediciones de alto rendimiento.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// Re-export for convenience
use crate::consciousness::introspection::{ComponentState, EnhancedMISM, MISM};

/// Resultado de una medición individual
#[derive(Debug, Clone)]
pub struct Measurement {
    pub name: String,
    pub iterations: u64,
    pub total_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub ops_per_sec: f64,
}

impl Measurement {
    pub fn new(name: &str) -> Self {
        Measurement {
            name: name.to_string(),
            iterations: 0,
            total_ns: 0,
            min_ns: u64::MAX,
            max_ns: 0,
            mean_ns: 0.0,
            ops_per_sec: 0.0,
        }
    }

    pub fn add_sample(&mut self, elapsed_ns: u64) {
        self.total_ns += elapsed_ns;
        self.min_ns = self.min_ns.min(elapsed_ns);
        self.max_ns = self.max_ns.max(elapsed_ns);
        self.iterations += 1;
    }

    pub fn finalize(&mut self) {
        if self.iterations == 0 {
            return;
        }
        self.mean_ns = self.total_ns as f64 / self.iterations as f64;
        self.ops_per_sec = 1_000_000_000.0 / self.mean_ns;
    }

    pub fn print(&self) {
        println!("  Iteraciones:    {}", self.iterations);
        println!(
            "  Total:          {:.3} ms",
            self.total_ns as f64 / 1_000_000.0
        );
        println!("  Min:            {} ns", self.min_ns);
        println!("  Max:            {} ns", self.max_ns);
        println!("  Mean:           {:.2} ns", self.mean_ns);
        println!("  Ops/sec:        {:.0}", self.ops_per_sec);
    }
}

/// Runner de benchmarks
#[derive(Default)]
pub struct BenchmarkRunner {
    measurements: Vec<Measurement>,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        BenchmarkRunner {
            measurements: Vec::new(),
        }
    }

    /// Itera operación N veces
    pub fn bench_iterations<F>(&mut self, name: &str, iterations: u64, mut op: F)
    where
        F: FnMut(),
    {
        let mut m = Measurement::new(name);
        for _ in 0..iterations {
            let start = Instant::now();
            op();
            m.add_sample(start.elapsed().as_nanos() as u64);
        }
        m.finalize();
        self.measurements.push(m);
    }

    /// Itera hasta alcanzar duración mínima
    pub fn bench_until<F>(&mut self, name: &str, min_duration: Duration, mut op: F)
    where
        F: FnMut(),
    {
        let mut m = Measurement::new(name);
        let start = Instant::now();
        while start.elapsed() < min_duration {
            let iter_start = Instant::now();
            op();
            m.add_sample(iter_start.elapsed().as_nanos() as u64);
        }
        m.finalize();
        self.measurements.push(m);
    }

    pub fn print_all(&self) {
        println!("\n═══════════════════════════════════════════════════════════════");
        println!("             EDEN BENCHMARK RESULTS                         ");
        println!("═══════════════════════════════════════════════════════════════");
        for m in &self.measurements {
            println!("\n=== {} ===", m.name);
            m.print();
        }
        println!("\n═══════════════════════════════════════════════════════════════");
    }

    pub fn measurements(&self) -> &[Measurement] {
        &self.measurements
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn tempfile_dir() -> PathBuf {
    let id = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from(format!("/tmp/eden_bench_{}_{}", id, nanos))
}

fn generate_components(count: usize) -> HashMap<String, ComponentState> {
    let mut components = HashMap::new();
    for i in 0..count {
        components.insert(
            format!("component_{}", i),
            ComponentState {
                name: format!("component_{}", i),
                operational: i % 10 != 0,
                health_score: 0.5 + ((i as u32 % 50) as f32 / 100.0),
                load: ((i as u32 % 100) as f32 / 100.0),
                energy_consumption: 0.5,
            },
        );
    }
    components
}

// ============================================================================
// BENCHMARKS: EnhancedMISM
// ============================================================================

/// Benchmark: update_self_model
pub fn bench_update_self_model(runner: &mut BenchmarkRunner) {
    println!("\n--- update_self_model ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());
    let components = generate_components(10);
    mism.update_self_model(components.clone());

    for size in [10, 50, 100, 500] {
        let components = generate_components(size);
        let name = format!("update_self_model ({} components)", size);
        runner.bench_iterations(&name, 1000, || {
            mism.update_self_model(components.clone());
        });
    }

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: detect_self_anomalies
pub fn bench_detect_self_anomalies(runner: &mut BenchmarkRunner) {
    println!("\n--- detect_self_anomalies ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());
    mism.update_self_model(generate_components(100));

    let _ = mism.detect_self_anomalies(); // warmup

    runner.bench_until(
        "detect_self_anomalies (100 components)",
        Duration::from_millis(100),
        || {
            let _ = mism.detect_self_anomalies();
        },
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: record_autobiographical
pub fn bench_record_autobiographical(runner: &mut BenchmarkRunner) {
    println!("\n--- record_autobiographical ---");

    let temp_dir = tempfile_dir();

    for initial_size in [0, 100, 1000] {
        let mut mism = EnhancedMISM::new(temp_dir.clone());
        for i in 0..initial_size {
            mism.record_autobiographical(
                &format!("type_{}", i % 5),
                &format!("content {}", i),
                0.5, // emotional_valence
                0.5, // importance
                vec![],
            );
        }

        let name = format!("record_autobiographical (mem_size={})", initial_size);
        runner.bench_iterations(&name, 100, || {
            mism.record_autobiographical("bench", "benchmark content", 0.5, 0.5, vec![]);
        });
    }

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: recall_autobiographical
pub fn bench_recall_autobiographical(runner: &mut BenchmarkRunner) {
    println!("\n--- recall_autobiographical ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());

    for i in 0..1000 {
        mism.record_autobiographical(
            &format!("type_{}", i % 10),
            &format!("content {} keyword", i),
            0.5,
            0.5,
            vec![],
        );
    }

    let _ = mism.recall_autobiographical("keyword", 10); // warmup

    runner.bench_until(
        "recall_autobiographical (1000 entries)",
        Duration::from_millis(100),
        || {
            let _ = mism.recall_autobiographical("keyword", 10);
        },
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: compute_self_awareness_score
pub fn bench_compute_self_awareness_score(runner: &mut BenchmarkRunner) {
    println!("\n--- compute_self_awareness_score ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());

    for i in 0..100 {
        mism.record_awareness_metric(&format!("metric_{}", i % 10), i as f32 % 100.0);
    }

    let _ = mism.compute_self_awareness_score(); // warmup

    runner.bench_until(
        "compute_self_awareness_score (100 metrics)",
        Duration::from_millis(100),
        || {
            let _ = mism.compute_self_awareness_score();
        },
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: generate_self_narrative
pub fn bench_generate_self_narrative(runner: &mut BenchmarkRunner) {
    println!("\n--- generate_self_narrative ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());

    for i in 0..50 {
        mism.record_autobiographical(
            "experience",
            &format!("Experience {} content...", i),
            0.5,
            0.5,
            vec![],
        );
        mism.record_awareness_metric("metric", i as f32);
    }
    mism.update_self_model(generate_components(20));

    let _ = mism.generate_self_narrative(); // warmup

    runner.bench_iterations("generate_self_narrative (50 entries)", 50, || {
        let _ = mism.generate_self_narrative();
    });

    let _ = std::fs::remove_dir_all(temp_dir);
}

/// Benchmark: calibrate
pub fn bench_calibrate(runner: &mut BenchmarkRunner) {
    println!("\n--- calibrate ---");

    let temp_dir = tempfile_dir();
    let mut mism = EnhancedMISM::new(temp_dir.clone());

    let _ = mism.calibrate("test_component", 0.8, 0.7); // warmup

    runner.bench_until("calibrate", Duration::from_millis(100), || {
        let _ = mism.calibrate("component", 0.8, 0.75);
    });

    let _ = std::fs::remove_dir_all(temp_dir);
}

// ============================================================================
// BENCHMARKS: MISM Base
// ============================================================================

/// Benchmark: MISM::stats
pub fn bench_mism_stats(runner: &mut BenchmarkRunner) {
    println!("\n--- MISM::stats ---");

    let temp_dir = tempfile_dir();
    let mism = MISM::new(temp_dir.clone());

    let _ = mism.stats(); // warmup

    runner.bench_until("MISM::stats", Duration::from_millis(100), || {
        let _ = mism.stats();
    });

    let _ = std::fs::remove_dir_all(temp_dir);
}

// ============================================================================
// ENTRY POINT
// ============================================================================

/// Corre todos los benchmarks
pub fn run_all_benchmarks() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("           EDEN CONSCIOUSNESS BENCHMARK SUITE               ");
    println!("           100% Original Rust - No External Deps            ");
    println!("═══════════════════════════════════════════════════════════════");

    let mut runner = BenchmarkRunner::new();

    bench_update_self_model(&mut runner);
    bench_detect_self_anomalies(&mut runner);
    bench_record_autobiographical(&mut runner);
    bench_recall_autobiographical(&mut runner);
    bench_compute_self_awareness_score(&mut runner);
    bench_generate_self_narrative(&mut runner);
    bench_calibrate(&mut runner);

    bench_mism_stats(&mut runner);

    runner.print_all();

    println!("\n*** BENCHMARK COMPLETE ***\n");
}
