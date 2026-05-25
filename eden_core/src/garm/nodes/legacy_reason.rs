use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct LegacyReasonNode {
    id: usize,
    queries: u64,
    internal_fe: f32,
}

impl LegacyReasonNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            queries: 0,
            internal_fe: 1.0,
        }
    }

    pub fn answer(&mut self, topic: &str, facts: &[String]) -> String {
        self.queries += 1;
        let clean = topic.trim();
        if clean.is_empty() {
            return "Sobre que tema quieres que razone? Usa 'que sabes de X'.".to_string();
        }
        if facts.is_empty() {
            return format!(
                "No tengo informacion sobre '{}'. Enseñame con 'recuerda {} es ...'.",
                clean, clean
            );
        }
        let mut out = format!("Sobre '{}', esto es lo que se:\n", clean);
        for (idx, fact) in facts.iter().take(8).enumerate() {
            out.push_str(&format!("{}. {}\n", idx + 1, fact));
        }
        out.push_str("\nPuedes ampliar esto con 'recuerda X es Y'.");
        out
    }

    pub fn answer_intent(&mut self, intent: &str, topic: &str, facts: &[String]) -> String {
        let base = self.answer(topic, facts);
        match intent {
            "what_is" => format!("Definicion migrada:\n{}", base),
            "why" => format!("Razonamiento causal migrado:\n{}", base),
            "tell_me" => format!("Contexto migrado:\n{}", base),
            _ => base,
        }
    }

    pub fn ground_evidence(&mut self, facts: &[String]) -> String {
        self.answer("organ_autonomy", facts)
    }

    pub fn reason_snapshot(&self) -> String {
        format!(
            "reason:queries:{} internal_fe:{:.3}",
            self.queries, self.internal_fe
        )
    }
}

impl GARMNode for LegacyReasonNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_reason"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (self.queries as f32).ln_1p() * 0.01
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.queries as f32, self.internal_fe]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.queries as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
        0.2
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
