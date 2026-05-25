//! GARM Node: HumanInterface — Sensor/Actuador hacia el mundo exterior
//!
//! No es un shell aparte. Es un nodo como cualquier otro.
//! Su "sensor" es stdin. Su "actuador" es stdout.
//! Minimiza su energia libre prediciendo que input va a recibir el humano.

use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::VecDeque;

pub struct HumanInterfaceNode {
    id: usize,
    command_queue: VecDeque<String>,
    last_human_input: String,
    prediction_error: f32,
    is_interactive: bool,
    pending_output: Vec<String>,
}

impl HumanInterfaceNode {
    pub fn new(id: usize, interactive: bool) -> Self {
        HumanInterfaceNode {
            id,
            command_queue: VecDeque::new(),
            last_human_input: String::new(),
            prediction_error: 1.0,
            is_interactive: interactive,
            pending_output: Vec::new(),
        }
    }

    /// Inyecta un comando desde fuera (llamado por el entry point o por mensajes de otros nodos).
    pub fn inject_command(&mut self, cmd: String) {
        self.command_queue.push_back(cmd);
    }

    pub fn take_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_output)
    }

    pub fn is_interactive(&self) -> bool {
        self.is_interactive
    }

    pub fn maintain_local_bridge(&mut self) -> String {
        self.pending_output
            .push("[HUMAN-AUTO] local dialogue bridge checked".to_string());
        format!(
            "[HUMAN-AUTO] interactive={} queued={} pending_output={}",
            self.is_interactive,
            self.command_queue.len(),
            self.pending_output.len()
        )
    }

    pub fn bridge_snapshot(&self) -> String {
        format!(
            "human:interactive:{} queued:{} pending_output:{} last_input_len:{}",
            self.is_interactive,
            self.command_queue.len(),
            self.pending_output.len(),
            self.last_human_input.len()
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "command_queue": self.command_queue,
            "last_human_input": self.last_human_input,
            "prediction_error": self.prediction_error,
            "pending_output": self.pending_output,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.command_queue = snapshot
            .get("command_queue")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();
        self.last_human_input = snapshot
            .get("last_human_input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.prediction_error = snapshot
            .get("prediction_error")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        self.pending_output = snapshot
            .get("pending_output")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Ok(())
    }
}

impl GARMNode for HumanInterfaceNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "human_interface"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        // Alta FE cuando hay comandos sin procesar (sorpresa pendiente)
        let queue_surprise = self.command_queue.len() as f32 * 0.5;
        let prediction_surprise = self.prediction_error;
        queue_surprise + prediction_surprise
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        // Predice si va a haber input humano en el proximo ciclo
        let will_have_input = if self.command_queue.is_empty() {
            0.0
        } else {
            1.0
        };
        vec![will_have_input, self.prediction_error]
    }

    fn act(&mut self, ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.prediction_error = (self.prediction_error * 0.9 + err.abs() * 0.1).min(5.0);
        }

        // Procesar comandos pendientes
        if let Some(cmd) = self.command_queue.pop_front() {
            self.last_human_input = cmd.clone();
            self.pending_output
                .push(format!("[HUMAN] received: {}", cmd));

            // Comandos de control del grafo se traducen a acciones
            match cmd.trim() {
                "quit" | "exit" => {
                    self.pending_output
                        .push("[HUMAN] shutdown requested".to_string());
                    return NodeAction::Output(vec![-1.0]); // senial especial
                }
                "estado" | "status" => {
                    return NodeAction::SendMessage(0, vec![1.0, 0.0]); // pedir status al nodo 0
                }
                _ => {}
            }
        }

        // Emitir output resumido del contexto
        let out = vec![
            ctx.global_free_energy,
            ctx.tick as f32,
            self.command_queue.len() as f32,
        ];
        NodeAction::Output(out)
    }

    fn update(&mut self, _dt: f32, energy_in: f32) -> f32 {
        let cost = 2.0;
        if energy_in < cost {
            self.prediction_error += 0.1; // estres por falta de energia
        }
        cost
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
