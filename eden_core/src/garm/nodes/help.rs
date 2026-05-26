use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct HelpNode {
    id: usize,
    requests: u64,
    internal_fe: f32,
}

impl HelpNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            requests: 0,
            internal_fe: 1.0,
        }
    }

    pub fn help(&mut self) -> String {
        self.requests += 1;
        [
            "Comandos GARM:",
            "  tick                      - fuerza un pulso",
            "  hola | quien eres         - dialogo legacy migrado",
            "  que tal | buenos          - saludo legacy migrado",
            "  que piensas | explicale   - introspeccion legacy rica",
            "  phi | grafo consciente   - conciencia runtime + regulador de grafo 200k",
            "  como te sientes           - estado emocional/metabolico",
            "  observatorio | dashboard  - panorama global migrado",
            "  readiness                - mide readiness de arquitectura EDEN",
            "  readiness bench/probe    - bench mide; probe genera evidencia local",
            "  readiness external       - genera manifiesto Phase 6 para validacion externa sin claim AGI",
            "  readiness external run   - ejecuta harness held-out local sin claim",
            "  readiness package        - empaqueta artifacts reproducibles con hashes",
            "  readiness plan/run       - convierte brechas readiness en goals/contracts locales",
            "  action evidence          - audita intent->policy->execution->consequence",
            "  memory eval | world eval | cognitive eval | embodied eval | neural eval | symbolic eval | self improvement eval - evalua memoria, mundo, cognicion, grounding, neural, simbolico y mejora propia",
            "  frontier architecture eval - evalua safety-control, foundation, multimodal, LLM-agent, probabilistic, HRL, robotics, VLA, sim-to-real, evolution, developmental, whole-brain y neuromorphic",
            "  paradigm architecture eval - mapea paradigmas existentes, evita duplicados y evalua huecos neuro-symbolic, formal, active-inference, systemic, programmatic, affective, HITL y emergence",
            "  integration governance eval - evalua integracion ejecutiva, aprendizaje seguro, grounding, causal world model, metacognicion, objetivos, evaluacion, alineacion y limites",
            "  global executive workspace eval - evalua el nucleo operativo HEC/GEWC y su traza runtime unificada",
            "  gewc operational benchmark - mide GEWC en generalidad, autonomia gobernada, seguridad y estabilidad",
            "  capability reality eval - separa capacidades ejecutables, arquitectura, entrenamiento pendiente y bloqueos de seguridad",
            "  architecture advantage eval - genera trace spec, matrix v2, suite cognitiva, SDK, adapters y demos reproducibles",
            "  paradise worldcell eval - crea el artefacto publico Paradise: worldcell runtime gobernado para agentes autonomos",
            "  paradise intent/plan/approve/execute/sessions - corre el loop operativo Paradise: intencion, dry-run, permiso, ejecucion y evidencia",
            "  runtime spine eval/audit/verify/enforce/risk/breakers/replay - genera, protege, analiza y reconstruye el spine runtime",
            "  praxis nexus eval - formaliza el sustrato praxico: 7 primitivos, 5 bloques, reglas, razonador y bench",
            "  locus eval/ingest/context/audit - Capa Locus: autoridad, contexto personal, evidencia, permisos y privacidad",
            "  operator forge eval/synth/verify/audit - Forja Operativa: primitivas formales, grafos tipados y verificacion",
            "  external ecosystem eval - formaliza ecosistema externo EDEN: contratos, interop, certificacion, onboarding, gobernanza y benchmark exchange",
            "  sovereign cognition eval - define 11 victorias arquitectonicas EDEN sobre Hyperon sin copiar AtomSpace/MeTTa",
            "  training evidence eval - admite capability_report.json como evidencia GEWC sin elevar claims",
            "  megatron 7b evidence eval - admite evidencia formal del piloto 7B EDEN-only sin admitir checkpoints",
            "  eden capable eval - genera los 7 artefactos que convierten checkpoint, datos, inferencia estructurada, registry y ELCP/SFT en capacidad gobernada",
            "  model runtime eval - genera adapter runtime, checkpoint manifest, training harness y gobernanza de modelos sin entrenar pesos",
            "  first model prepare/readiness - prepara el primer modelo EDEN como 4A formal sin ejecutar entrenamiento",
            "  elcp prepare/objective/admission/hardening/readiness - prepara, endurece y mide ELCP sin entrenar ni admitir pesos",
            "  model register/load/evaluate/unload X | model audit - ciclo de vida de modelos subordinado a GEWC",
            "  capabilities audit       - registra capacidades validadas localmente",
            "  eval | eval run/audit      - evaluation loop local para medir arquitectura",
            "  learning | learning record X/consolidate/audit - ledger de aprendizaje verificable",
            "  world | world observe/predict/verify/audit - world model local verificable",
            "  bench | bench run/audit  - benchmark local de competencia GARM",
            "  exec | exec plan X/run/audit - ejecutor local con scoring y rollback",
            "  attention | attention X/clear/audit - working memory y foco operativo",
            "  uncertainty | uncertainty X/resolve/audit - ledger de riesgos e incertidumbre",
            "  experiment | experiment plan X/run/audit - experimentos locales reproducibles",
            "  provenance | provenance X/verify/audit - procedencia local de evidencia",
            "  policy | policy eval X/audit - guard local de restricciones operativas",
            "  maturity | maturity assess X/audit - madurez local de capacidades",
            "  ritual | umbra            - narrativa interna, Umbra y child-autons",
            "  lengua X | responde X    - órgano Lengua: respuesta KG+memoria",
            "  reloj X | cuando X       - órgano Reloj: cadena temporal ligera",
            "  juez X | validar X       - órgano Juez: valida evidencia local/KG",
            "  voz | autodocumenta      - órgano Voz: escribe estado al historial",
            "  voz texto X | tts X      - Voz/TTS local opcional: manifiesto si no hay audio backend",
            "  hybrid voice plan/synth X/audit - TTS hibrido: transformer apilado + bucle GARM",
            "  hrm text corpus/ingest/search/objective/plan/run/audit - seam local de pretraining textual HRM",
            "  intestino | compacta     - órgano Intestino: compactación KG acotada",
            "  piel | frontera          - órgano Piel: presión frontera/API/grafo",
            "  autotuning | autoajuste  - órgano Autotuning: recomienda ajuste seguro",
            "  cag | contexto | cache    - muestra cache CAG y metricas de contexto",
            "  cag explain X            - explica fuentes/traza del contexto CAG",
            "  cag gaps X               - diagnostica brechas y recomendaciones CAG",
            "  cag plan X | actions     - planifica/lista acciones CAG seguras",
            "  cag run X                - ejecuta solo acciones CAG locales seguras",
            "  cag audit                - audita acciones CAG manuales/autonomas",
            "  hrm X | razona jerarquico X - HRM local: razona por capas KG/CAG/historial",
            "  hrm run X                - ejecuta plan HRM via CAG/organos auditables",
            "  garm audit/report/report history/export/import/verify export/artifacts/backup/restore/compact",
            "  goals | goals plan X/run/audit - scheduler de objetivos y contratos de accion",
            "  organos | organs         - lista los organos principales",
            "  organos audit/plan/run   - los organos ejecutan ciclo autonomo seguro",
            "  organos health/repair    - diagnostica y registra reparacion segura",
            "  organos actions/feedback - cola, auditoria y feedback por organo",
            "  historial | log           - ultimos comandos migrados",
            "  start | stop              - reanuda/pausa autonomia",
            "  evolve | improve          - evolucion acotada segura",
            "  rebirth | renacimiento    - ciclo Meltrace/rebirth acotado",
            "  estado | status | estas   - imprime estado completo",
            "  auto N                    - ejecuta N pulsos",
            "  save/load | guarda/carga  - persistencia GARM",
            "  recuerda X | aprende X    - guarda memoria legacy migrada",
            "  aprendizaje X | recorder X - aliases legacy de aprendizaje",
            "  remember X | learn X      - guarda memoria legacy migrada",
            "  crawl URL | web URL       - crawler remoto seguro (--allow-remote-crawl)",
            "  conceptnet PATH           - importa ConceptNet/local KG estructurado",
            "  memoria | que sabes       - lista recuerdos",
            "  busca X | search X        - busca recuerdos por tema",
            "  que sabes de X            - razona sobre recuerdos por tema",
            "  que es/why/tell me X      - razonamiento legacy migrado",
            "  cual es la razon X        - causalidad legacy migrada",
            "  migration | legacy        - mapa REPL legacy -> nodos GARM",
            "  help | ayuda | que puedes - muestra esta ayuda",
            "  quit | adios | bye        - apaga el runtime",
            "API local: /status /metrics /report /export /artifacts /command?cmd=...",
        ]
        .join("\n")
    }

    pub fn help_snapshot(&self) -> String {
        format!(
            "help:requests:{} internal_fe:{:.3}",
            self.requests, self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "requests": self.requests,
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
        self.requests = snapshot
            .get("requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

impl GARMNode for HelpNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "help"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Fast
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.requests as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.requests as f32])
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
