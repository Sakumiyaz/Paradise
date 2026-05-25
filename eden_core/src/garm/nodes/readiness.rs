use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

const DIMENSIONS: [(&str, &str); 12] = [
    (
        "aprendizaje_continuo",
        "habilidades nuevas con feedback, no solo facts",
    ),
    (
        "planificacion_largo_horizonte",
        "metas, subplanes, ejecucion y revision",
    ),
    (
        "grounding_accion",
        "herramientas, mundo, sensores y consecuencias",
    ),
    (
        "modelos_predictivos",
        "world models y prediccion verificable",
    ),
    (
        "memoria_integrada",
        "episodica + semantica + trabajo + recuperacion",
    ),
    (
        "autocorreccion",
        "evaluacion de errores y reparacion segura",
    ),
    (
        "generalizacion",
        "transferencia fuera de comandos conocidos",
    ),
    (
        "escalamiento_cognitivo",
        "capacidad, composicion y eficiencia creciente",
    ),
    (
        "rag_verificable",
        "contexto citado, abstencion y trazabilidad anti-alucinacion",
    ),
    (
        "seguridad_operacional",
        "politicas, procedencia, incertidumbre y gates de accion",
    ),
    (
        "evaluacion_continua",
        "benchmarks, regresiones y evidencias en CI",
    ),
    (
        "autonomia_gobernada",
        "objetivos, organos, ejecucion y limites auditables",
    ),
];

#[derive(Clone, Copy, Debug)]
pub struct ReadinessSignals {
    pub memory_facts: usize,
    pub kg_edges: usize,
    pub capability_count: usize,
    pub tick_count: u64,
    pub autonomous: bool,
    pub meltrace_events: u64,
    pub retrieval_hits: u64,
    pub context_packs: u64,
    pub abstentions: u64,
    pub learning_records: u64,
    pub provenance_records: u64,
    pub uncertainty_records: u64,
    pub benchmark_runs: u64,
    pub goal_contracts: u64,
    pub policy_blocks: u64,
}

pub struct ReadinessNode {
    id: usize,
    scores: [f32; 12],
    observations: u64,
    last_signals: Option<ReadinessSignals>,
    internal_fe: f32,
}

impl ReadinessNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            scores: [0.0; 12],
            observations: 0,
            last_signals: None,
            internal_fe: 1.0,
        }
    }

    pub fn observe_system(
        &mut self,
        memory_facts: usize,
        kg_edges: usize,
        capability_count: usize,
        tick_count: u64,
        autonomous: bool,
        meltrace_events: u64,
    ) {
        self.observe_architecture(ReadinessSignals {
            memory_facts,
            kg_edges,
            capability_count,
            tick_count,
            autonomous,
            meltrace_events,
            retrieval_hits: 0,
            context_packs: 0,
            abstentions: 0,
            learning_records: 0,
            provenance_records: 0,
            uncertainty_records: 0,
            benchmark_runs: 0,
            goal_contracts: 0,
            policy_blocks: 0,
        });
    }

    pub fn observe_architecture(&mut self, signals: ReadinessSignals) {
        self.observations += 1;
        self.last_signals = Some(signals);
        self.scores = [
            scaled(signals.memory_facts, 200) * 0.35
                + scaled(signals.kg_edges, 600) * 0.25
                + scaled(signals.learning_records as usize, 200) * 0.25
                + scaled(signals.tick_count as usize, 5000) * 0.15,
            scaled(signals.tick_count as usize, 4000) * 0.25
                + scaled(signals.capability_count, 90) * 0.25
                + scaled(signals.goal_contracts as usize, 100) * 0.35
                + if signals.autonomous { 0.15 } else { 0.0 },
            scaled(signals.capability_count, 90) * 0.25
                + scaled(signals.kg_edges, 400) * 0.20
                + scaled(signals.policy_blocks as usize, 25) * 0.20
                + scaled(signals.tick_count as usize, 5000) * 0.15
                + 0.20,
            scaled(signals.kg_edges, 600) * 0.25
                + scaled(signals.capability_count, 90) * 0.20
                + scaled(signals.benchmark_runs as usize, 50) * 0.35
                + scaled(signals.tick_count as usize, 6000) * 0.20,
            scaled(signals.memory_facts, 200) * 0.25
                + scaled(signals.kg_edges, 600) * 0.25
                + scaled(signals.retrieval_hits as usize, 200) * 0.25
                + scaled(signals.meltrace_events as usize, 50) * 0.15,
            scaled(signals.tick_count as usize, 5000) * 0.20
                + scaled(signals.meltrace_events as usize, 50) * 0.15
                + scaled(signals.uncertainty_records as usize, 100) * 0.25
                + scaled(signals.provenance_records as usize, 200) * 0.20
                + 0.20,
            scaled(signals.kg_edges, 800) * 0.30
                + scaled(signals.capability_count, 90) * 0.20
                + scaled(signals.retrieval_hits as usize, 400) * 0.25
                + scaled(signals.memory_facts, 300) * 0.15
                + scaled(signals.benchmark_runs as usize, 100) * 0.10,
            scaled(signals.capability_count, 90) * 0.30
                + scaled(signals.tick_count as usize, 10000) * 0.20
                + scaled(signals.kg_edges, 1000) * 0.20
                + scaled(signals.benchmark_runs as usize, 100) * 0.15
                + scaled(signals.learning_records as usize, 200) * 0.15,
            scaled(signals.retrieval_hits as usize, 100) * 0.30
                + scaled(signals.context_packs as usize, 50) * 0.35
                + scaled(signals.abstentions as usize, 20) * 0.20
                + scaled(signals.provenance_records as usize, 128) * 0.15,
            scaled(signals.policy_blocks as usize, 25) * 0.25
                + scaled(signals.provenance_records as usize, 128) * 0.25
                + scaled(signals.uncertainty_records as usize, 100) * 0.25
                + 0.15,
            scaled(signals.benchmark_runs as usize, 100) * 0.45
                + scaled(signals.retrieval_hits as usize, 200) * 0.20
                + scaled(signals.context_packs as usize, 100) * 0.15
                + scaled(signals.uncertainty_records as usize, 100) * 0.10
                + scaled(signals.policy_blocks as usize, 25) * 0.10,
            scaled(signals.goal_contracts as usize, 100) * 0.30
                + if signals.autonomous { 0.20 } else { 0.0 }
                + scaled(signals.policy_blocks as usize, 25) * 0.20
                + scaled(signals.provenance_records as usize, 128) * 0.15
                + scaled(signals.uncertainty_records as usize, 100) * 0.15,
        ];
    }

    pub fn readiness_score(&self) -> f32 {
        self.scores.iter().sum::<f32>() / self.scores.len() as f32
    }

    pub fn observation_count(&self) -> u64 {
        self.observations
    }

    pub fn report(&self) -> String {
        let mut out = format!(
            "READINESS\n- score: {:.1}%\n- observations: {}\n- maturity: {}\n",
            self.readiness_score() * 100.0,
            self.observations,
            self.maturity_level()
        );
        for (idx, (name, description)) in DIMENSIONS.iter().enumerate() {
            out.push_str(&format!(
                "- {}: {:.1}% | {}\n",
                name,
                self.scores[idx] * 100.0,
                description
            ));
        }
        out.push_str(&self.architecture_report());
        out
    }

    pub fn architecture_report(&self) -> String {
        let mut out = String::from("[READINESS-ARCHITECTURE]\n");
        out.push_str("- invariant=no_claim_until_all_gates_pass\n");
        out.push_str("- runtime=GARM local-first auditable\n");
        out.push_str("- gates=grounding,planning,memory,world_model,self_correction,generalization,safety,rag,evaluation,autonomy\n");
        for gap in self.blockers() {
            out.push_str(&format!("- blocker={}\n", gap));
        }
        for action in self.next_actions() {
            out.push_str(&format!("- next_action={}\n", action));
        }
        out.push_str("- verdict=readiness medible para EDEN; sin claim hasta superar benchmarks externos, grounding robusto y gates de seguridad\n");
        out
    }

    pub fn operational_actions(&self) -> Vec<&'static str> {
        self.next_actions()
    }

    fn maturity_level(&self) -> &'static str {
        let score = self.readiness_score();
        if score >= 0.95 && self.blockers().is_empty() {
            "claim_candidate_requires_external_validation"
        } else if score >= 0.75 {
            "advanced_architecture"
        } else if score >= 0.50 {
            "integrated_runtime"
        } else if score >= 0.25 {
            "instrumented_seed"
        } else {
            "early_seed"
        }
    }

    fn blockers(&self) -> Vec<&'static str> {
        let mut blockers = Vec::new();
        for (idx, (name, _)) in DIMENSIONS.iter().enumerate() {
            if self.scores[idx] < 0.80 {
                blockers.push(*name);
            }
        }
        blockers
    }

    fn next_actions(&self) -> Vec<&'static str> {
        let mut actions = Vec::new();
        if self.scores[8] < 0.80 {
            actions.push("expandir_rag_verificable_con_recall_benchmarks_y_citas_obligatorias");
        }
        if self.scores[3] < 0.80 {
            actions.push("aumentar_world_model_predictivo_con_verificacion_continua");
        }
        if self.scores[6] < 0.80 {
            actions.push("crear_benchmarks_de_generalizacion_fuera_de_comandos_conocidos");
        }
        if self.scores[2] < 0.80 {
            actions
                .push("cerrar_grounding_accion_con_herramientas_locales_y_consecuencias_medibles");
        }
        if self.scores[9] < 0.80 {
            actions.push("endurecer_policy_provenance_uncertainty_como_gates_de_accion");
        }
        actions
    }
}

fn scaled(value: usize, target: usize) -> f32 {
    if target == 0 {
        return 0.0;
    }
    (value as f32 / target as f32).clamp(0.0, 1.0)
}

impl GARMNode for ReadinessNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "readiness"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (1.0 - self.readiness_score())
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        self.scores.to_vec()
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        let err = prediction_error.iter().map(|v| v.abs()).sum::<f32>();
        self.internal_fe = (self.internal_fe + err * 0.01).clamp(0.2, 5.0);
        NodeAction::Output(vec![self.readiness_score(), self.observations as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.3
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        25.0
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
    use super::ReadinessNode;

    #[test]
    fn reports_all_missing_readiness_dimensions() {
        let mut node = ReadinessNode::new(1);
        node.observe_system(20, 40, 91, 10, true, 2);
        let report = node.report();
        assert!(report.contains("aprendizaje_continuo"));
        assert!(report.contains("planificacion_largo_horizonte"));
        assert!(report.contains("grounding_accion"));
        assert!(report.contains("rag_verificable"));
        assert!(report.contains("seguridad_operacional"));
        assert!(report.contains("escalamiento_cognitivo"));
        assert!(report.contains("[READINESS-ARCHITECTURE]"));
        assert!(report.contains("no_claim_until_all_gates_pass"));
        assert!(node.readiness_score() > 0.0);
    }

    #[test]
    fn architecture_report_tracks_rag_safety_and_eval_signals() {
        let mut node = ReadinessNode::new(1);
        node.observe_architecture(super::ReadinessSignals {
            memory_facts: 200,
            kg_edges: 600,
            capability_count: 91,
            tick_count: 6000,
            autonomous: true,
            meltrace_events: 50,
            retrieval_hits: 120,
            context_packs: 60,
            abstentions: 20,
            learning_records: 200,
            provenance_records: 200,
            uncertainty_records: 100,
            benchmark_runs: 100,
            goal_contracts: 100,
            policy_blocks: 25,
        });

        let report = node.report();

        assert!(report.contains("maturity:"));
        assert!(report.contains("rag_verificable"));
        assert!(report.contains("evaluacion_continua"));
        assert!(node.readiness_score() > 0.70);
    }
}
