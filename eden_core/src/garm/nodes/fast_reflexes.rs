//! GARM Node: FastReflexes — Capa de reflejos (ms)
//!
//! Gate de veto, seguridad constitucional, paradas de emergencia.
//! Corre a la maxima frecuencia. No razona, reacciona.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct FastReflexesNode {
    id: usize,
    threat_level: f32,
    veto_active: bool,
    last_sensor: Vec<f32>,
    probe_count: u64,
}

impl FastReflexesNode {
    pub fn new(id: usize) -> Self {
        FastReflexesNode {
            id,
            threat_level: 0.0,
            veto_active: false,
            last_sensor: Vec::new(),
            probe_count: 0,
        }
    }

    pub fn is_veto_active(&self) -> bool {
        self.veto_active
    }
    pub fn threat_level(&self) -> f32 {
        self.threat_level
    }

    pub fn local_reflex_probe(&mut self, global_fe: f32) -> String {
        self.probe_count += 1;
        self.veto_active = global_fe > 100.0 || self.threat_level > 0.9;
        if !self.veto_active {
            self.threat_level *= 0.9;
        }
        format!(
            "[REFLEX-AUTO] veto={} threat={:.2} global_fe={:.2} probes={}",
            self.veto_active, self.threat_level, global_fe, self.probe_count
        )
    }

    pub fn reflex_snapshot(&self) -> String {
        format!(
            "reflex:veto:{} threat:{:.3} sensors:{} probes:{}",
            self.veto_active,
            self.threat_level,
            self.last_sensor.len(),
            self.probe_count
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "threat_level": self.threat_level,
            "veto_active": self.veto_active,
            "last_sensor": self.last_sensor,
            "probe_count": self.probe_count,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.threat_level = snapshot
            .get("threat_level")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.veto_active = snapshot
            .get("veto_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.last_sensor = snapshot
            .get("last_sensor")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_f64().map(|n| n as f32))
                    .collect()
            })
            .unwrap_or_default();
        self.probe_count = snapshot
            .get("probe_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(())
    }
}

impl GARMNode for FastReflexesNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "fast_reflexes"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }

    fn free_energy(&self) -> f32 {
        // Los reflejos SIEMPRE tienen free energy alta — necesitan estar alerta
        5.0 + self.threat_level * 10.0
    }

    fn predict(&mut self, ctx: &NodeContext) -> Vec<f32> {
        // Predice: habra amenaza?
        let sensor_norm = ctx.sensor_input.iter().map(|x| x.abs()).sum::<f32>()
            / ctx.sensor_input.len().max(1) as f32;
        vec![sensor_norm, self.threat_level]
    }

    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        self.last_sensor = ctx.sensor_input.clone();
        self.veto_active = false;

        // Regla 1: si la energia libre global es > 100, hay caos → veto parcial
        if ctx.global_free_energy > 100.0 {
            self.veto_active = true;
            self.threat_level = 0.5;
            return NodeAction::Output(vec![-1.0, self.threat_level]); // senial de veto
        }

        // Regla 2: si algun sensor excede umbral, amenaza directa
        if ctx.sensor_input.iter().any(|&x| x.abs() > 0.9) {
            self.threat_level = 1.0;
            self.veto_active = true;
            return NodeAction::Output(vec![-1.0, self.threat_level]);
        }

        // Regla 3: disipar amenaza si todo esta tranquilo
        self.threat_level *= 0.9;
        if self.threat_level < 0.1 {
            self.threat_level = 0.0;
        }

        NodeAction::Output(vec![0.0, self.threat_level])
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost = 1.0; // reflejos son baratos
        if energy_in < cost {
            // Sin energia, los reflejos fallan — maxima amenaza
            self.threat_level = 1.0;
        }
        cost
    }

    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        20.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
