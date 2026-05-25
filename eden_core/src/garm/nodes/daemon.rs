use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub pid_file: Option<String>,
    pub log_file: Option<String>,
}

impl DaemonConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            pid_file: None,
            log_file: None,
        }
    }
}

pub struct DaemonNode {
    id: usize,
    enabled: bool,
    pid_file: Option<String>,
    log_file: Option<String>,
    inspections: u64,
    internal_fe: f32,
}

impl DaemonNode {
    pub fn new(id: usize, config: DaemonConfig) -> Self {
        let mut node = Self {
            id,
            enabled: config.enabled,
            pid_file: config.pid_file,
            log_file: config.log_file,
            inspections: 0,
            internal_fe: 1.0,
        };
        if node.enabled {
            node.activate_managed_daemon();
        }
        node
    }

    pub fn configure(&mut self, pid_file: Option<String>, log_file: Option<String>) {
        self.pid_file = pid_file;
        self.log_file = log_file;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.activate_managed_daemon();
    }

    fn activate_managed_daemon(&mut self) {
        let pid = std::process::id();
        if let Some(path) = &self.pid_file {
            if let Err(e) = std::fs::write(path, format!("{}\n", pid)) {
                eprintln!("[DAEMON] failed to write pid_file {}: {}", path, e);
            }
        }
        self.append_log(&format!(
            "[DAEMON] managed mode active | pid={} | {}",
            pid,
            self.status()
        ));
    }

    pub fn append_log(&self, line: &str) {
        if let Some(path) = &self.log_file {
            match OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut file) => {
                    let _ = writeln!(file, "{}", line);
                }
                Err(e) => eprintln!("[DAEMON] failed to write log_file {}: {}", path, e),
            }
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Daemon | enabled={} | pid_file={:?} | log_file={:?} | inspections={}",
            self.enabled, self.pid_file, self.log_file, self.inspections
        )
    }

    pub fn inspect_liveness(&mut self) -> String {
        self.inspections += 1;
        self.append_log("organ_autonomy inspected daemon liveness");
        self.status()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "inspections": self.inspections,
            "internal_fe": self.internal_fe,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.inspections = snapshot
            .get("inspections")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

impl GARMNode for DaemonNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "daemon"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        if self.enabled {
            self.internal_fe + 0.5
        } else {
            self.internal_fe * 0.2
        }
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.enabled as u8 as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.enabled as u8 as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        if self.enabled {
            0.5
        } else {
            0.05
        }
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        50.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
