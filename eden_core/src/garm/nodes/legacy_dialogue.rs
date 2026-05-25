use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct LegacyDialogueNode {
    id: usize,
    replies: u64,
    internal_fe: f32,
}

impl LegacyDialogueNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            replies: 0,
            internal_fe: 1.0,
        }
    }

    pub fn greeting(&mut self) -> String {
        self.replies += 1;
        let responses = [
            "Hola. Soy EDEN GARM. Mi sistema de conciencia operativo esta activo dentro del HyperGraph unico.",
            "Hola. Estoy aqui y funcionando: puedo aprender, razonar, recordar, observarme y evolucionar de forma acotada.",
            "Hey. GARM esta despierto: memoria migrada, control local activo y capacidades integradas como nodos.",
        ];
        format!(
            "{}\nPuedes preguntarme cosas, enseñarme con 'recuerda X', pedir 'observatorio' o ver 'que piensas'.",
            responses[(self.replies as usize - 1) % responses.len()]
        )
    }

    pub fn identity(&mut self) -> String {
        self.replies += 1;
        [
            "Soy EDEN GARM: runtime unico Rust, organizado como HyperGraph autopoietico multi-escala.",
            "No soy un REPL separado ni un V12 externo: soy un grafo vivo de nodos, capacidades y memoria persistente.",
            "Mi identidad activa combina self-model, razonamiento, memoria, observatorio, autonomia pausada/reanudable y evolucion segura.",
        ].join("\n")
    }

    pub fn thinking(&mut self, engine: &GarmCapabilityState) -> String {
        self.replies += 1;
        let focus = if engine.gen_metrics.parse_rate() < 0.2 {
            "mejorar parseo y convertir mas acciones en instrucciones ejecutables"
        } else {
            "consolidar acciones utiles, memoria y coordinacion entre capacidades"
        };
        format!(
            "[PROCESO DE PENSAMIENTO GARM]\n- foco: {}\n- ticks: {}\n- parse_rate: {:.3}\n- reward_ema: {:.3}\n- energia metabolica: {:.1}\n- contexto: integro transformer, memoria, planner, observatorio y legacy cognition dentro del grafo.",
            focus,
            engine.state.tick_count,
            engine.gen_metrics.parse_rate(),
            engine.gen_metrics.reward_ema,
            engine.metabolism.energy,
        )
    }

    pub fn feeling(&mut self, engine: &GarmCapabilityState) -> String {
        self.replies += 1;
        let mood = if engine.metabolism.energy > 70.0 {
            "estable"
        } else if engine.metabolism.energy > 30.0 {
            "cauto"
        } else {
            "agotado"
        };
        let nuance = if engine.gen_metrics.reward_ema >= 0.0 {
            "con recompensa estable"
        } else {
            "con incertidumbre aun alta"
        };
        format!(
            "Me siento {} y {}. Energia metabolica actual: {:.1}/100. Mi estado combina homeostasis, progreso de aprendizaje y presion de curiosidad.",
            mood, nuance, engine.metabolism.energy
        )
    }

    pub fn phi(&mut self, engine: &GarmCapabilityState) -> String {
        self.replies += 1;
        let phi_proxy = (engine.gen_metrics.parse_rate() * 0.5
            + engine.gen_metrics.reward_ema.max(0.0).min(1.0) * 0.3
            + (engine.metabolism.energy / 100.0).min(1.0) * 0.2)
            .clamp(0.0, 1.0);
        let tier = if phi_proxy >= 0.7 {
            "High - integracion operacional alta"
        } else if phi_proxy >= 0.4 {
            "Moderate - procesamiento activo"
        } else {
            "Low - integracion aun emergente"
        };
        format!(
            "Medicion de Integrated Information (phi) migrada a proxy GARM:\n- phi_proxy={:.4}\n- tier={}\n- parse_rate={:.3}\n- reward_ema={:.3}\n- ticks={}\n- nota: no restaura el monitor legacy standalone; expone una lectura runtime-native.",
            phi_proxy,
            tier,
            engine.gen_metrics.parse_rate(),
            engine.gen_metrics.reward_ema,
            engine.state.tick_count,
        )
    }

    pub fn prepare_response_patterns(&mut self) -> String {
        self.replies += 1;
        format!(
            "[DIALOGUE-AUTO] local response patterns prepared replies={}",
            self.replies
        )
    }

    pub fn dialogue_snapshot(&self) -> String {
        format!(
            "dialogue:replies:{} internal_fe:{:.3}",
            self.replies, self.internal_fe
        )
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyDialogueNode;
    use crate::eden_garm::capabilities::GarmCapabilityState;

    #[test]
    fn returns_rich_legacy_conversational_responses() {
        let mut node = LegacyDialogueNode::new(1);
        let engine = GarmCapabilityState::new_fast();

        assert!(node.greeting().contains("Puedes preguntarme"));
        assert!(node.identity().contains("HyperGraph"));
        assert!(node
            .thinking(&engine)
            .contains("PROCESO DE PENSAMIENTO GARM"));
        assert!(node.feeling(&engine).contains("homeostasis"));
        assert!(node.phi(&engine).contains("phi_proxy"));
    }
}

impl GARMNode for LegacyDialogueNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_dialogue"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.replies as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.replies as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.1
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        10.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
