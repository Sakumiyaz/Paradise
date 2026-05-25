use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct CampoTensionNode {
    id: usize,
    tension: f32,
    tension_conocimiento: f32,
    tension_identidad: f32,
    tension_mision: f32,
    tension_emocional: f32,
    tension_memoria: f32,
    umbral: f32,
    historial_disparos: Vec<u64>,
    ciclos_sin_disparo: u32,
    last_fired: bool,
}

impl CampoTensionNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            tension: 0.0,
            tension_conocimiento: 0.0,
            tension_identidad: 0.0,
            tension_mision: 0.0,
            tension_emocional: 0.0,
            tension_memoria: 0.0,
            umbral: 1.0,
            historial_disparos: Vec::new(),
            ciclos_sin_disparo: 0,
            last_fired: false,
        }
    }

    pub fn tension(&self) -> f32 {
        self.tension
    }

    pub fn umbral(&self) -> f32 {
        self.umbral
    }

    pub fn disparos(&self) -> usize {
        self.historial_disparos.len()
    }

    pub fn calcular(
        &mut self,
        gaps: usize,
        facts: usize,
        evolution_level: u32,
        capabilities: usize,
        has_mission: bool,
        mission_progress: f32,
        valence: f32,
        episodes: usize,
    ) {
        let valence = if valence.is_finite() {
            valence.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let mission_progress = if mission_progress.is_finite() {
            mission_progress.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let delta_conocimiento = if gaps > 0 {
            (gaps as f32 / (facts as f32 + 5.0)) * 0.02
        } else {
            0.0
        };
        self.tension_conocimiento += delta_conocimiento;

        let nivel_real = evolution_level as f32;
        let capacidades_modelo = capabilities as f32;
        let discrepancia = (nivel_real * 0.05 - capacidades_modelo * 0.1).abs();
        if discrepancia > 1.0 {
            self.tension_identidad += 0.015;
        }

        if has_mission && mission_progress < 0.1 {
            self.tension_mision += 0.025;
        }

        if valence < 0.3 {
            self.tension_emocional += (0.3 - valence) * 0.03;
        }

        if episodes > 10 {
            self.tension_memoria += (episodes as f32 - 10.0).min(100.0) * 0.02;
        }

        let decaimiento = 0.005;
        self.tension_conocimiento = (self.tension_conocimiento - decaimiento).max(0.0);
        self.tension_identidad = (self.tension_identidad - decaimiento).max(0.0);
        self.tension_mision = (self.tension_mision - decaimiento * 2.0).max(0.0);
        self.tension_emocional = (self.tension_emocional - decaimiento * 1.5).max(0.0);
        self.tension_memoria = (self.tension_memoria - decaimiento).max(0.0);

        self.tension = self.tension_conocimiento
            + self.tension_identidad
            + self.tension_mision
            + self.tension_emocional
            + self.tension_memoria;
        self.ciclos_sin_disparo += 1;
        self.last_fired = false;
    }

    pub fn debe_evolucionar(&self) -> bool {
        self.tension >= self.umbral
    }

    pub fn disparar(&mut self, ciclo: u64) {
        self.historial_disparos.push(ciclo);
        if self.historial_disparos.len() > 50 {
            self.historial_disparos.remove(0);
        }

        self.tension = 0.0;
        self.tension_conocimiento = 0.0;
        self.tension_identidad = 0.0;
        self.tension_mision = 0.0;
        self.tension_emocional = 0.0;
        self.tension_memoria = 0.0;
        self.ciclos_sin_disparo = 0;
        self.last_fired = true;

        if self.historial_disparos.len() >= 2 {
            let ultimo = self.historial_disparos[self.historial_disparos.len() - 1];
            let penultimo = self.historial_disparos[self.historial_disparos.len() - 2];
            let ciclos_entre = ultimo.saturating_sub(penultimo) as f32;
            if ciclos_entre < 20.0 {
                self.umbral = (self.umbral * 1.05).min(3.0);
            } else if ciclos_entre > 100.0 {
                self.umbral = (self.umbral * 0.95).max(0.5);
            }
        }
    }

    pub fn informe(&self) -> String {
        format!(
            "[CAMPO-TENSION] Total: {:.2} / Umbral: {:.2}\n- Conocimiento: {:.2} | Identidad: {:.2} | Mision: {:.2}\n- Emocional: {:.2} | Memoria: {:.2} | Disparos: {} | Relaj: {} ciclos",
            self.tension,
            self.umbral,
            self.tension_conocimiento,
            self.tension_identidad,
            self.tension_mision,
            self.tension_emocional,
            self.tension_memoria,
            self.historial_disparos.len(),
            self.ciclos_sin_disparo,
        )
    }

    pub fn regulate_once(&mut self, tick: u64) -> String {
        self.calcular(1, 1, 1, 1, true, 0.5, 0.7, 1);
        if self.debe_evolucionar() {
            self.disparar(tick);
        }
        self.informe()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "tension": self.tension,
            "tension_conocimiento": self.tension_conocimiento,
            "tension_identidad": self.tension_identidad,
            "tension_mision": self.tension_mision,
            "tension_emocional": self.tension_emocional,
            "tension_memoria": self.tension_memoria,
            "umbral": self.umbral,
            "historial_disparos": self.historial_disparos,
            "ciclos_sin_disparo": self.ciclos_sin_disparo,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.tension = json_f32(&snapshot, "tension", 0.0);
        self.tension_conocimiento = json_f32(&snapshot, "tension_conocimiento", 0.0);
        self.tension_identidad = json_f32(&snapshot, "tension_identidad", 0.0);
        self.tension_mision = json_f32(&snapshot, "tension_mision", 0.0);
        self.tension_emocional = json_f32(&snapshot, "tension_emocional", 0.0);
        self.tension_memoria = json_f32(&snapshot, "tension_memoria", 0.0);
        self.umbral = json_f32(&snapshot, "umbral", 1.0);
        self.historial_disparos = snapshot
            .get("historial_disparos")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default();
        self.ciclos_sin_disparo = snapshot
            .get("ciclos_sin_disparo")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        Ok(())
    }
}

fn json_f32(snapshot: &serde_json::Value, key: &str, default: f32) -> f32 {
    snapshot
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default as f64) as f32
}

impl GARMNode for CampoTensionNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "campo_tension"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        0.4 + self.tension
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.tension, self.umbral, self.ciclos_sin_disparo as f32]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        let avg_neighbor_signal = if ctx.neighbor_outputs.is_empty() {
            0.0
        } else {
            let sum: f32 = ctx
                .neighbor_outputs
                .iter()
                .flat_map(|(_, out)| out.iter())
                .copied()
                .sum();
            let count: usize = ctx.neighbor_outputs.iter().map(|(_, out)| out.len()).sum();
            sum / count.max(1) as f32
        };
        let surprise = prediction_error.iter().map(|v| v.abs()).sum::<f32>()
            + ctx.global_free_energy.max(0.0) * 0.001;
        let gaps = (surprise * 10.0).round().max(1.0) as usize;
        let facts = (avg_neighbor_signal.abs() * 10.0).round() as usize;
        let valence = (1.0 - ctx.global_free_energy * 0.01).clamp(-1.0, 1.0);
        let episodes = ctx.sensor_input.len() + ctx.neighbor_outputs.len();

        self.calcular(gaps, facts, 1, facts, true, 0.0, valence, episodes);
        if self.debe_evolucionar() {
            self.disparar(ctx.tick);
        }
        NodeAction::Output(vec![
            self.tension,
            self.umbral,
            if self.last_fired { 1.0 } else { 0.0 },
        ])
    }

    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.4
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

#[cfg(test)]
mod tests {
    use super::CampoTensionNode;

    #[test]
    fn accumulates_and_discharges_legacy_tension() {
        let mut node = CampoTensionNode::new(7);
        for _ in 0..4 {
            node.calcular(20, 1, 50, 1, true, 0.0, -0.5, 50);
        }
        assert!(node.tension() >= node.umbral());
        node.disparar(42);
        assert_eq!(node.disparos(), 1);
        assert_eq!(node.tension(), 0.0);
        assert!(node.informe().contains("CAMPO-TENSION"));
    }
}
