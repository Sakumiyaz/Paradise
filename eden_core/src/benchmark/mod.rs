//! Benchmark utilities for EDEN Core
//! 
//! Usage: Cargo run --release --features benchmark -- [subcommand]
//! Subcommands: baseline, profile, compare
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::time::Instant;

// =============================================================================
// Hot path timing utilities
// =============================================================================

/// Wrap to measure elapsed time
pub struct Timer {
    start: Instant,
    name: &'static str,
}

impl Timer {
    pub fn new(name: &'static str) -> Self {
        Self { start: Instant::now(), name }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        eprintln!("[BENCH] {}: {:.3}ms", self.name, elapsed.as_secs_f64() * 1000.0);
    }
}

/// Record timing for later aggregation
thread_local! {
    static TIMINGS: std::cell::RefCell<Vec<(String, f64)>> = std::cell::RefCell::new(Vec::new());
}

pub fn record_time(name: &'static str, ms: f64) {
    TIMINGS.with(|t| t.borrow_mut().push((name.to_string(), ms)));
}

pub fn print_timings() {
    TIMINGS.with(|t| {
        let mut v = t.borrow_mut();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        eprintln!("\n=== TIMINGS (sorted by ms) ===");
        for (name, ms) in v.iter() {
            eprintln!("  {:.3}ms  {}", ms, name);
        }
        v.clear();
    });
}

/// Benchmark marker - only runs with "benchmark" feature
#[cfg(feature = "benchmark")]
pub use std::time::Instant;

#[cfg(not(feature = "benchmark"))]
pub struct NoOpTimer;
#[cfg(not(feature = "benchmark"))]
impl NoOpTimer {
    pub fn new(_: &'static str) -> Self { Self }
}
#[cfg(not(feature = "benchmark"))]
impl Drop for NoOpTimer {}

// =============================================================================
// Benchmark data collection
// =============================================================================

#[derive(Default)]
pub struct BenchStats {
    pub tick_times: Vec<f64>,
    pub cycle_vital_times: Vec<f64>,
    pub campo_step_times: Vec<f64>,
    pub mar_regen_times: Vec<f64>,
}

impl BenchStats {
    pub fn record_tick(&mut self, ms: f64) { self.tick_times.push(ms); }
    pub fn record_ciclo_vital(&mut self, ms: f64) { self.cycle_vital_times.push(ms); }
    pub fn record_campo_step(&mut self, ms: f64) { self.campo_step_times.push(ms); }
    pub fn record_mar_regen(&mut self, ms: f64) { self.mar_regen_times.push(ms); }

    pub fn summary(&self) -> String {
        let tick_p50 = percentile(&self.tick_times, 0.5);
        let tick_p95 = percentile(&self.tick_times, 0.95);
        let tick_p99 = percentile(&self.tick_times, 0.99);
        let tick_mean = mean(&self.tick_times);

        format!(
            "Tick Stats:\n  mean: {:.3}ms  p50: {:.3}ms  p95: {:.3}ms  p99: {:.3}ms",
            tick_mean, tick_p50, tick_p95, tick_p99
        )
    }
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { return 0.0; }
    v.iter().sum::<f64>() / v.len() as f64
}

fn percentile(v: &[f64], p: f64) -> f64 {
    if v.is_empty() { return 0.0; }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((s.len() - 1) as f64 * p).round() as usize;
    s[idx.min(s.len() - 1)]
}

// =============================================================================
// Main benchmark CLI
// =============================================================================

#[cfg(feature = "benchmark")]
pub fn run_benchmark(args: &[String]) {
    match args.get(0).map(|s| s.as_str()) {
        Some("baseline") => run_baseline(),
        Some("profile") => run_profile(),
        _ => {
            eprintln!("Usage: eden benchmark [baseline|profile]");
        }
    }
}

#[cfg(not(feature = "benchmark"))]
pub fn run_benchmark(_args: &[String]) {
    eprintln!("Rebuild with --features benchmark to enable benchmarking");
}

#[cfg(feature = "benchmark")]
fn run_baseline() {
    eprintln!("Running baseline benchmark...");
    // TODO: Initialize EDEN with standard config and run N ticks
}

#[cfg(feature = "benchmark")]
fn run_profile() {
    eprintln!("Running profiler...");
    // TODO: Use pprof or similar
}