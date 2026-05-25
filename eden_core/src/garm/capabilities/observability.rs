// EDEN GARM — Observability Engine
// Metrics + Tracing + Histograms (zero external service)

use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct MetricPoint {
    pub timestamp_ms: u64,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct TraceSpan {
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub parent: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Histogram {
    pub bins: Vec<u64>,
    pub bin_edges: Vec<f64>,
    pub total_count: u64,
    pub sum: f64,
}

impl Histogram {
    pub fn new(edges: &[f64]) -> Self {
        Histogram {
            bins: vec![0; edges.len() + 1],
            bin_edges: edges.to_vec(),
            total_count: 0,
            sum: 0.0,
        }
    }

    pub fn observe(&mut self, value: f64) {
        let idx = self
            .bin_edges
            .iter()
            .position(|&e| value < e)
            .unwrap_or(self.bins.len() - 1);
        self.bins[idx] += 1;
        self.total_count += 1;
        self.sum += value;
    }

    pub fn mean(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.sum / self.total_count as f64
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        let target = (self.total_count as f64 * p / 100.0) as u64;
        let mut cum = 0u64;
        for (i, &count) in self.bins.iter().enumerate() {
            cum += count;
            if cum >= target {
                return self
                    .bin_edges
                    .get(i)
                    .copied()
                    .unwrap_or(self.bin_edges.last().copied().unwrap_or(0.0));
            }
        }
        self.bin_edges.last().copied().unwrap_or(0.0)
    }
}

pub struct ObservabilityEngine {
    pub metrics: HashMap<String, Vec<MetricPoint>>,
    pub traces: Vec<TraceSpan>,
    pub histograms: HashMap<String, Histogram>,
    pub active_spans: HashMap<String, (u64, Instant)>, // name -> (start_ms, instant)
    pub uptime_start: Instant,
}

impl ObservabilityEngine {
    pub fn new() -> Self {
        ObservabilityEngine {
            metrics: HashMap::new(),
            traces: Vec::new(),
            histograms: HashMap::new(),
            active_spans: HashMap::new(),
            uptime_start: Instant::now(),
        }
    }

    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn record_metric(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        let point = MetricPoint {
            timestamp_ms: self.now_ms(),
            value,
            labels,
        };
        self.metrics
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(point);
        // cap per metric
        if let Some(v) = self.metrics.get_mut(name) {
            if v.len() > 10000 {
                v.drain(0..v.len() - 10000);
            }
        }
    }

    pub fn start_span(&mut self, name: &str, _parent: Option<&str>) {
        self.active_spans
            .insert(name.to_string(), (self.now_ms(), Instant::now()));
    }

    pub fn end_span(&mut self, name: &str, attributes: HashMap<String, String>) {
        if let Some((start_ms, _inst)) = self.active_spans.remove(name) {
            self.traces.push(TraceSpan {
                name: name.to_string(),
                start_ms,
                end_ms: self.now_ms(),
                parent: None,
                attributes,
            });
            if self.traces.len() > 10000 {
                self.traces.remove(0);
            }
        }
    }

    pub fn record_histogram(&mut self, name: &str, value: f64, edges: &[f64]) {
        let hist = self
            .histograms
            .entry(name.to_string())
            .or_insert_with(|| Histogram::new(edges));
        hist.observe(value);
    }

    pub fn get_metric_avg(&self, name: &str, window: usize) -> f64 {
        if let Some(points) = self.metrics.get(name) {
            let recent: Vec<f64> = points.iter().rev().take(window).map(|p| p.value).collect();
            if !recent.is_empty() {
                recent.iter().sum::<f64>() / recent.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn get_trace_summary(&self, name: &str) -> (u64, f64) {
        let spans: Vec<&TraceSpan> = self.traces.iter().filter(|t| t.name == name).collect();
        let count = spans.len() as u64;
        let avg_dur = if !spans.is_empty() {
            spans
                .iter()
                .map(|s| (s.end_ms - s.start_ms) as f64)
                .sum::<f64>()
                / spans.len() as f64
        } else {
            0.0
        };
        (count, avg_dur)
    }

    pub fn status(&self) -> String {
        let uptime = self.uptime_start.elapsed().as_secs();
        format!(
            "Observability | metrics: {} | traces: {} | histograms: {} | uptime: {}s",
            self.metrics.len(),
            self.traces.len(),
            self.histograms.len(),
            uptime
        )
    }
}
