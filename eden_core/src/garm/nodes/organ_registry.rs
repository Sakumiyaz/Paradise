use crate::eden_garm::{state_paths, HyperGraph};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_ORGAN_ACTIONS: usize = 512;
const MAX_ORGAN_AUDIT: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrganProfile {
    pub name: &'static str,
    pub label: &'static str,
    pub role: &'static str,
    pub autonomy: &'static str,
    pub safe_action: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrganHealthFinding {
    pub name: &'static str,
    pub label: &'static str,
    pub free_energy: f32,
    pub severity: &'static str,
    pub recommendation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganAction {
    pub id: u64,
    pub organ: String,
    pub kind: String,
    pub status: String,
    pub reason: String,
    pub delta: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganAuditEntry {
    pub action_id: u64,
    pub organ: String,
    pub kind: String,
    pub status: String,
    pub reason: String,
    pub mode: String,
    pub delta: String,
}

#[derive(Clone, Debug, Default)]
struct OrganAutonomyState {
    organs: Vec<IndividualOrganAutonomy>,
}

#[derive(Clone, Debug)]
struct IndividualOrganAutonomy {
    name: String,
    actions: VecDeque<OrganAction>,
    audit: VecDeque<OrganAuditEntry>,
    next_action_id: u64,
    feedback_positive: u64,
    feedback_negative: u64,
    actions_executed: u64,
    actions_blocked: u64,
    autonomous_runs: u64,
}

impl IndividualOrganAutonomy {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            actions: VecDeque::new(),
            audit: VecDeque::new(),
            next_action_id: 1,
            feedback_positive: 0,
            feedback_negative: 0,
            actions_executed: 0,
            actions_blocked: 0,
            autonomous_runs: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrganAutonomyMetrics {
    pub pending_actions: u64,
    pub actions_executed: u64,
    pub actions_blocked: u64,
    pub autonomous_runs: u64,
    pub feedback_positive: u64,
    pub feedback_negative: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganRecoverySummary {
    pub schema: &'static str,
    pub organs: usize,
    pub actions: usize,
    pub audit_entries: usize,
    pub repaired_next_ids: usize,
}

pub struct OrganRunBatch {
    pub report: String,
    pub facts: Vec<String>,
    pub history: String,
}

struct OrganExecution {
    status: &'static str,
    reason: &'static str,
    effect: &'static str,
}

static ORGAN_STATE: OnceLock<Mutex<OrganAutonomyState>> = OnceLock::new();

fn organ_state() -> &'static Mutex<OrganAutonomyState> {
    ORGAN_STATE.get_or_init(|| Mutex::new(new_individual_state()))
}

pub const ORGAN_PROFILES: &[OrganProfile] = &[
    OrganProfile {
        name: "coordinator",
        label: "Coordinador",
        role: "sincroniza capacidades",
        autonomy: "supervisado",
        safe_action: "observar pulso",
    },
    OrganProfile {
        name: "human_interface",
        label: "Interfaz Humana",
        role: "puente humano-runtime",
        autonomy: "reactivo",
        safe_action: "mantener entrada local",
    },
    OrganProfile {
        name: "meta_architect",
        label: "Meta Arquitecto",
        role: "observa arquitectura",
        autonomy: "supervisado",
        safe_action: "recomendar sin mutar",
    },
    OrganProfile {
        name: "fast_reflexes",
        label: "Reflejos",
        role: "respuesta rapida",
        autonomy: "autonomo-acotado",
        safe_action: "emitir reflejo local",
    },
    OrganProfile {
        name: "benchmark",
        label: "Benchmark",
        role: "mide costo",
        autonomy: "reactivo",
        safe_action: "medir sin modificar",
    },
    OrganProfile {
        name: "command_router",
        label: "Router",
        role: "interpreta comandos",
        autonomy: "reactivo",
        safe_action: "clasificar entrada",
    },
    OrganProfile {
        name: "persistence",
        label: "Persistencia",
        role: "guarda/carga estado",
        autonomy: "manual",
        safe_action: "reportar rutas",
    },
    OrganProfile {
        name: "telemetry",
        label: "Telemetria",
        role: "mide runtime",
        autonomy: "autonomo-acotado",
        safe_action: "actualizar metricas",
    },
    OrganProfile {
        name: "api_server",
        label: "API Local",
        role: "expone API local",
        autonomy: "reactivo",
        safe_action: "servir localhost",
    },
    OrganProfile {
        name: "daemon",
        label: "Daemon",
        role: "supervision proceso",
        autonomy: "supervisado",
        safe_action: "vigilar sin red",
    },
    OrganProfile {
        name: "legacy_memory",
        label: "Memoria",
        role: "recuerda hechos",
        autonomy: "reactivo",
        safe_action: "buscar/recordar local",
    },
    OrganProfile {
        name: "legacy_reason",
        label: "Razon",
        role: "responde con memoria",
        autonomy: "reactivo",
        safe_action: "razonar con evidencia",
    },
    OrganProfile {
        name: "legacy_dialogue",
        label: "Dialogo",
        role: "voz conversacional",
        autonomy: "reactivo",
        safe_action: "responder local",
    },
    OrganProfile {
        name: "observatory",
        label: "Observatorio",
        role: "panorama global",
        autonomy: "reactivo",
        safe_action: "emitir reporte",
    },
    OrganProfile {
        name: "legacy_history",
        label: "Historial",
        role: "registra eventos",
        autonomy: "autonomo-acotado",
        safe_action: "anotar evento",
    },
    OrganProfile {
        name: "legacy_evolution",
        label: "Evolucion",
        role: "evolucion acotada",
        autonomy: "supervisado",
        safe_action: "proponer mejora",
    },
    OrganProfile {
        name: "legacy_cognition",
        label: "Cognicion",
        role: "estado cognitivo",
        autonomy: "autonomo-acotado",
        safe_action: "integrar senales",
    },
    OrganProfile {
        name: "campo_tension",
        label: "Campo Tension",
        role: "tension sistemica",
        autonomy: "autonomo-acotado",
        safe_action: "regular tension",
    },
    OrganProfile {
        name: "legacy_knowledge_graph",
        label: "Grafo Conocimiento",
        role: "RAG/KG local",
        autonomy: "reactivo",
        safe_action: "guardar aristas locales",
    },
    OrganProfile {
        name: "legacy_autoconsumo",
        label: "Autoconsumo",
        role: "lee arquitectura propia",
        autonomy: "supervisado",
        safe_action: "extraer contexto local",
    },
    OrganProfile {
        name: "legacy_venado",
        label: "Venado",
        role: "compatibilidad cristal",
        autonomy: "reactivo",
        safe_action: "roundtrip local",
    },
    OrganProfile {
        name: "legacy_paradigm_hub",
        label: "Paradigm Architecture Legacy",
        role: "tecnicas migradas a paradigm architecture eval",
        autonomy: "superseded",
        safe_action: "observacion sin ciclo",
    },
    OrganProfile {
        name: "legacy_ecosystem",
        label: "EcoSistema",
        role: "poblacion interna",
        autonomy: "autonomo-acotado",
        safe_action: "actualizar ecosistema",
    },
    OrganProfile {
        name: "legacy_rebirth_meltrace",
        label: "Rebirth Meltrace",
        role: "memoria/muerte/rebirth",
        autonomy: "supervisado",
        safe_action: "registrar ciclo",
    },
    OrganProfile {
        name: "legacy_crawler",
        label: "Crawler",
        role: "ingesta web/local",
        autonomy: "bloqueado-red",
        safe_action: "solo local/remoto gated",
    },
    OrganProfile {
        name: "help",
        label: "Ayuda",
        role: "documenta comandos",
        autonomy: "reactivo",
        safe_action: "mostrar ayuda",
    },
    OrganProfile {
        name: "readiness",
        label: "Readiness",
        role: "mide brechas de arquitectura",
        autonomy: "reactivo",
        safe_action: "diagnosticar readiness",
    },
    OrganProfile {
        name: "organic_lifecycle",
        label: "Lifecycle Organico",
        role: "Umbra/child-autons",
        autonomy: "autonomo-acotado",
        safe_action: "ritual interno",
    },
    OrganProfile {
        name: "conscious_graph_regulator",
        label: "Regulador Consciente",
        role: "regula escala/phi",
        autonomy: "autonomo-acotado",
        safe_action: "regular grafo",
    },
    OrganProfile {
        name: "context_augmentation",
        label: "CAG",
        role: "contexto/cache/acciones",
        autonomy: "autonomo-auditado",
        safe_action: "acciones locales seguras",
    },
    OrganProfile {
        name: "hrm_reasoner",
        label: "HRM",
        role: "razonamiento jerarquico local",
        autonomy: "autonomo-auditado",
        safe_action: "razonar con KG/CAG/historial",
    },
    OrganProfile {
        name: "voice_synthesizer",
        label: "Voz/TTS",
        role: "sintesis local opcional",
        autonomy: "autonomo-auditado",
        safe_action: "generar manifiesto local",
    },
];

pub fn organ_report(graph: &HyperGraph) -> String {
    let mut out = format!("[ORGANOS] total={}\n", ORGAN_PROFILES.len());
    for profile in ORGAN_PROFILES {
        let status = organ_status(graph, profile.name);
        out.push_str(&format!(
            "- {} ({}) status={} autonomy={} role={} safe_action={}\n",
            profile.label,
            profile.name,
            status,
            autonomy_mode(profile),
            profile.role,
            profile.safe_action
        ));
    }
    out
}

pub fn organ_audit(graph: &HyperGraph) -> String {
    let mut alive = 0usize;
    let mut missing = Vec::new();
    let mut high_fe = Vec::new();
    for profile in ORGAN_PROFILES {
        if let Some(node) = graph.nodes.iter().find(|node| node.name() == profile.name) {
            if node.is_alive() {
                alive += 1;
            }
            if node.free_energy() > 2.5 {
                high_fe.push(format!("{}:{:.2}", profile.name, node.free_energy()));
            }
        } else {
            missing.push(profile.name);
        }
    }
    format!(
        "[ORGANOS-AUDIT] total={} alive={} missing={} high_free_energy={} autonomous_ready={} blocked_remote=legacy_crawler\nmissing={}\nhigh_fe={}",
        ORGAN_PROFILES.len(),
        alive,
        missing.len(),
        high_fe.len(),
        ORGAN_PROFILES.len(),
        if missing.is_empty() { "none".to_string() } else { missing.join(",") },
        if high_fe.is_empty() { "none".to_string() } else { high_fe.join(",") },
    )
}

pub fn organ_plan(graph: &HyperGraph) -> String {
    plan_actions_for_all(graph);
    let mut out = String::from("[ORGANOS-PLAN]\n");
    for profile in ORGAN_PROFILES {
        let status = organ_status(graph, profile.name);
        let action = if status == "missing" {
            "repair_registration"
        } else if profile.autonomy == "bloqueado-red" {
            "keep_remote_gated"
        } else {
            domain_action_kind(profile)
        };
        out.push_str(&format!(
            "- organ={} status={} action={} guard={}\n",
            profile.name, status, action, profile.safe_action
        ));
    }
    out
}

pub fn organ_actions_report() -> String {
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    let metrics = metrics_from_state(&state);
    if state.organs.iter().all(|organ| organ.actions.is_empty()) {
        return format!(
            "[ORGANOS-ACTIONS] individual_organs={} pending={} executed={} blocked={} empty",
            state.organs.len(),
            metrics.pending_actions,
            metrics.actions_executed,
            metrics.actions_blocked
        );
    }
    let mut out = format!(
        "[ORGANOS-ACTIONS] individual_organs={} pending={} executed={} blocked={}\n",
        state.organs.len(),
        metrics.pending_actions,
        metrics.actions_executed,
        metrics.actions_blocked
    );
    for organ in &state.organs {
        let Some(action) = organ.actions.back() else {
            out.push_str(&format!("- organ={} actions=0\n", organ.name));
            continue;
        };
        out.push_str(&format!(
            "- organ={} actions={} last_id={} last_kind={} last_status={} last_reason={} last_delta={}\n",
            organ.name,
            organ.actions.len(),
            action.id,
            action.kind,
            action.status,
            action.reason,
            action.delta
        ));
    }
    out.trim_end().to_string()
}

pub fn organ_autonomy_audit_report() -> String {
    let state = organ_state().lock().unwrap();
    let metrics = metrics_from_state(&state);
    let mut out = format!(
        "[ORGANOS-AUTONOMY-AUDIT] organs={} pending={} executed={} blocked={} autonomous_runs={} feedback=+{}/-{}\n",
        ORGAN_PROFILES.len(),
        metrics.pending_actions,
        metrics.actions_executed,
        metrics.actions_blocked,
        metrics.autonomous_runs,
        metrics.feedback_positive,
        metrics.feedback_negative
    );
    for organ in &state.organs {
        let pending = organ
            .actions
            .iter()
            .filter(|action| action.status == "pending")
            .count();
        out.push_str(&format!(
            "- organ={} pending={} executed={} blocked={} autonomous_runs={} feedback=+{}/-{}\n",
            organ.name,
            pending,
            organ.actions_executed,
            organ.actions_blocked,
            organ.autonomous_runs,
            organ.feedback_positive,
            organ.feedback_negative
        ));
    }
    let mut recent: Vec<&OrganAuditEntry> = state
        .organs
        .iter()
        .flat_map(|organ| organ.audit.iter())
        .collect();
    recent.sort_by_key(|entry| entry.action_id);
    if recent.is_empty() {
        out.push_str("- audit=empty");
        return out;
    }
    for entry in recent
        .into_iter()
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        out.push_str(&format!(
            "- id={} mode={} organ={} kind={} status={} reason={} delta={}\n",
            entry.action_id,
            entry.mode,
            entry.organ,
            entry.kind,
            entry.status,
            entry.reason,
            entry.delta
        ));
    }
    out.trim_end().to_string()
}

pub fn organ_recovery_report() -> String {
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    let summary = recovery_summary(&mut state);
    format!(
        "[ORGANOS-RECOVERY] schema={} organs={} actions={} audit_entries={} repaired_next_ids={}",
        summary.schema,
        summary.organs,
        summary.actions,
        summary.audit_entries,
        summary.repaired_next_ids
    )
}

pub fn plan_actions_for_all(graph: &HyperGraph) -> usize {
    let findings = organ_health_findings(graph);
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    let mut planned = 0usize;
    for profile in ORGAN_PROFILES {
        let status = if graph.nodes.iter().any(|node| node.name() == profile.name) {
            "alive"
        } else {
            "missing"
        };
        let high_free_energy = findings.iter().any(|finding| finding.name == profile.name);
        let kind = if status == "missing" {
            "repair_registration"
        } else if profile.name == "legacy_crawler" {
            "keep_remote_gated"
        } else {
            domain_action_kind(profile)
        };
        let reason = if status == "missing" {
            "missing_node"
        } else if profile.name == "legacy_crawler" {
            "remote_network_blocked"
        } else if high_free_energy {
            "high_free_energy"
        } else {
            "scheduled_domain_autonomy"
        };
        let organ = individual_organ_mut(&mut state, profile.name);
        if push_action(organ, profile.name, kind, "pending", reason) {
            planned += 1;
        }
    }
    planned
}

pub fn run_pending_actions(graph: &mut HyperGraph, mode: &str, limit: usize) -> OrganRunBatch {
    plan_actions_for_all(graph);
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    let mut facts = Vec::new();
    let mut report = String::from("[ORGANOS-RUN]\n");
    let mut ran = 0usize;
    for organ in &mut state.organs {
        if ran >= limit {
            break;
        }
        let Some(index) = organ
            .actions
            .iter()
            .position(|action| action.status == "pending")
        else {
            continue;
        };
        let action = organ.actions[index].clone();
        let before = organ_observable(graph, &action.organ);
        let execution = execute_action(graph, &action);
        let after = organ_observable(graph, &action.organ);
        let delta = execution_delta(execution.status, execution.effect, &before, &after);
        organ.actions[index].status = execution.status.to_string();
        organ.actions[index].reason = execution.reason.to_string();
        organ.actions[index].delta = delta.clone();
        if execution.status == "executed" {
            organ.actions_executed += 1;
        } else if execution.status == "blocked" {
            organ.actions_blocked += 1;
        }
        organ.audit.push_back(OrganAuditEntry {
            action_id: action.id,
            organ: action.organ.clone(),
            kind: action.kind.clone(),
            status: execution.status.to_string(),
            reason: execution.reason.to_string(),
            mode: mode.to_string(),
            delta: delta.clone(),
        });
        while organ.audit.len() > MAX_ORGAN_AUDIT {
            organ.audit.pop_front();
        }
        if mode == "autonomous" {
            organ.autonomous_runs += 1;
        }
        facts.push(format!(
            "organ {} action {} status {} reason {} effect {} delta {}",
            action.organ, action.kind, execution.status, execution.reason, execution.effect, delta
        ));
        report.push_str(&format!(
            "- organ={} action={} status={} reason={} effect={} delta={}\n",
            action.organ, action.kind, execution.status, execution.reason, execution.effect, delta
        ));
        ran += 1;
    }
    OrganRunBatch {
        report: report.trim_end().to_string(),
        facts,
        history: format!(
            "[ORGANOS-AUTO] {} mode cycle executed {} organ actions; remote crawler remains gated",
            mode, ran
        ),
    }
}

pub fn record_feedback(useful: bool) {
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    for organ in &mut state.organs {
        if useful {
            organ.feedback_positive += 1;
        } else {
            organ.feedback_negative += 1;
        }
    }
}

pub fn metrics() -> OrganAutonomyMetrics {
    let state = organ_state().lock().unwrap();
    metrics_from_state(&state)
}

pub fn save_state() -> Result<(), String> {
    let mut state = organ_state().lock().unwrap();
    ensure_individual_organs(&mut state);
    let snapshot = serde_json::json!({
        "schema": "per-organ-v1",
        "organs": state.organs.iter().map(individual_organ_to_json).collect::<Vec<_>>(),
    });
    std::fs::write(
        state_paths::organ_autonomy_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write organ autonomy state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let data = std::fs::read_to_string(state_paths::organ_autonomy_state_path())
        .map_err(|e| format!("failed to read organ autonomy state: {}", e))?;
    let snapshot: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse organ autonomy JSON: {}", e))?;
    let mut state = organ_state().lock().unwrap();
    *state = if let Some(organs) = snapshot.get("organs").and_then(|v| v.as_array()) {
        let mut loaded = new_individual_state();
        for organ in organs.iter().filter_map(individual_organ_from_json) {
            replace_individual_organ(&mut loaded, organ);
        }
        loaded
    } else {
        load_legacy_global_state(&snapshot)
    };
    ensure_individual_organs(&mut state);
    recovery_summary(&mut state);
    Ok(())
}

pub fn organ_health_report(graph: &HyperGraph) -> String {
    let findings = organ_health_findings(graph);
    if findings.is_empty() {
        return format!(
            "[ORGANOS-HEALTH] total={} status=stable findings=0",
            ORGAN_PROFILES.len()
        );
    }
    let mut out = format!(
        "[ORGANOS-HEALTH] total={} status=needs_attention findings={}\n",
        ORGAN_PROFILES.len(),
        findings.len()
    );
    for finding in &findings {
        out.push_str(&format!(
            "- organ={} label='{}' severity={} free_energy={:.2} recommendation={}\n",
            finding.name,
            finding.label,
            finding.severity,
            finding.free_energy,
            finding.recommendation
        ));
    }
    out.trim_end().to_string()
}

pub fn organ_health_findings(graph: &HyperGraph) -> Vec<OrganHealthFinding> {
    let mut findings = Vec::new();
    for profile in ORGAN_PROFILES {
        if let Some(node) = graph.nodes.iter().find(|node| node.name() == profile.name) {
            let fe = node.free_energy();
            if fe > 2.5 {
                let severity = if fe > 100.0 {
                    "critical"
                } else if fe > 10.0 {
                    "high"
                } else {
                    "medium"
                };
                findings.push(OrganHealthFinding {
                    name: profile.name,
                    label: profile.label,
                    free_energy: fe,
                    severity,
                    recommendation: format!(
                        "safe_action={} autonomy={} no_code_mutation no_remote_network",
                        profile.safe_action,
                        autonomy_mode(profile)
                    ),
                });
            }
        } else {
            findings.push(OrganHealthFinding {
                name: profile.name,
                label: profile.label,
                free_energy: f32::INFINITY,
                severity: "critical",
                recommendation: "repair_registration_only no_spawn_without_explicit_design"
                    .to_string(),
            });
        }
    }
    findings.sort_by(|a, b| {
        b.free_energy
            .partial_cmp(&a.free_energy)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(b.name))
    });
    findings
}

pub fn autonomy_profiles() -> Vec<&'static OrganProfile> {
    ORGAN_PROFILES.iter().collect()
}

pub fn autonomy_mode(profile: &OrganProfile) -> &'static str {
    if profile.name == "legacy_crawler" {
        "autonomo-auditado-red-bloqueada"
    } else if profile.name == "context_augmentation" {
        "autonomo-auditado"
    } else {
        "autonomo-local-seguro"
    }
}

fn new_individual_state() -> OrganAutonomyState {
    OrganAutonomyState {
        organs: ORGAN_PROFILES
            .iter()
            .map(|profile| IndividualOrganAutonomy::new(profile.name))
            .collect(),
    }
}

fn ensure_individual_organs(state: &mut OrganAutonomyState) {
    for profile in ORGAN_PROFILES {
        if !state.organs.iter().any(|organ| organ.name == profile.name) {
            state
                .organs
                .push(IndividualOrganAutonomy::new(profile.name));
        }
    }
    state.organs.sort_by_key(|organ| {
        ORGAN_PROFILES
            .iter()
            .position(|profile| profile.name == organ.name)
            .unwrap_or(usize::MAX)
    });
}

fn recovery_summary(state: &mut OrganAutonomyState) -> OrganRecoverySummary {
    let mut repaired_next_ids = 0usize;
    for organ in &mut state.organs {
        let next_from_actions = organ
            .actions
            .iter()
            .map(|action| action.id)
            .max()
            .unwrap_or(0)
            + 1;
        let next_from_audit = organ
            .audit
            .iter()
            .map(|entry| entry.action_id)
            .max()
            .unwrap_or(0)
            + 1;
        let repaired = organ
            .next_action_id
            .max(next_from_actions)
            .max(next_from_audit)
            .max(1);
        if repaired != organ.next_action_id {
            organ.next_action_id = repaired;
            repaired_next_ids += 1;
        }
        while organ.actions.len() > MAX_ORGAN_ACTIONS {
            organ.actions.pop_front();
        }
        while organ.audit.len() > MAX_ORGAN_AUDIT {
            organ.audit.pop_front();
        }
    }
    OrganRecoverySummary {
        schema: "per-organ-v1",
        organs: state.organs.len(),
        actions: state.organs.iter().map(|organ| organ.actions.len()).sum(),
        audit_entries: state.organs.iter().map(|organ| organ.audit.len()).sum(),
        repaired_next_ids,
    }
}

fn individual_organ_mut<'a>(
    state: &'a mut OrganAutonomyState,
    name: &str,
) -> &'a mut IndividualOrganAutonomy {
    if let Some(index) = state.organs.iter().position(|organ| organ.name == name) {
        return &mut state.organs[index];
    }
    state.organs.push(IndividualOrganAutonomy::new(name));
    let index = state.organs.len() - 1;
    &mut state.organs[index]
}

fn replace_individual_organ(state: &mut OrganAutonomyState, organ: IndividualOrganAutonomy) {
    if let Some(slot) = state
        .organs
        .iter_mut()
        .find(|existing| existing.name == organ.name)
    {
        *slot = organ;
    } else {
        state.organs.push(organ);
    }
}

fn push_action(
    state: &mut IndividualOrganAutonomy,
    organ: &str,
    kind: &str,
    status: &str,
    reason: &str,
) -> bool {
    if state.actions.iter().any(|action| {
        action.organ == organ
            && action.kind == kind
            && matches!(action.status.as_str(), "pending" | "running")
    }) {
        return false;
    }
    let action = OrganAction {
        id: state.next_action_id.max(1),
        organ: organ.to_string(),
        kind: kind.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        delta: "pending".to_string(),
    };
    state.next_action_id = action.id + 1;
    state.actions.push_back(action);
    while state.actions.len() > MAX_ORGAN_ACTIONS {
        state.actions.pop_front();
    }
    true
}

fn domain_action_kind(profile: &OrganProfile) -> &'static str {
    match profile.name {
        "coordinator" => "coordinate_capability_pressure",
        "human_interface" => "maintain_local_dialogue_bridge",
        "meta_architect" => "review_architecture_without_mutation",
        "fast_reflexes" => "emit_local_reflex_probe",
        "benchmark" => "sample_runtime_cost",
        "command_router" => "validate_command_surface",
        "persistence" => "verify_state_paths",
        "telemetry" => "refresh_runtime_metrics",
        "api_server" => "check_local_api_readiness",
        "daemon" => "inspect_daemon_liveness",
        "legacy_memory" => "consolidate_local_memory",
        "legacy_reason" => "ground_reasoning_evidence",
        "legacy_dialogue" => "prepare_local_response_patterns",
        "observatory" => "emit_system_snapshot",
        "legacy_history" => "append_autonomy_trace",
        "legacy_evolution" => "propose_bounded_improvement",
        "legacy_cognition" => "integrate_cognitive_signals",
        "campo_tension" => "regulate_tension_field",
        "legacy_knowledge_graph" => "strengthen_local_kg_edges",
        "legacy_autoconsumo" => "extract_local_architecture_context",
        "legacy_venado" => "validate_crystal_roundtrip",
        "legacy_paradigm_hub" => "cycle_paradigm_budget",
        "legacy_ecosystem" => "update_internal_ecosystem",
        "legacy_rebirth_meltrace" => "record_rebirth_trace",
        "legacy_crawler" => "keep_remote_gated",
        "help" => "refresh_command_help",
        "readiness" => "measure_readiness_gaps",
        "organic_lifecycle" => "run_internal_lifecycle_ritual",
        "conscious_graph_regulator" => "regulate_graph_consciousness",
        "context_augmentation" => "coordinate_context_pack_actions",
        "hrm_reasoner" => "run_hierarchical_reasoning_cycle",
        "voice_synthesizer" => "synthesize_voice_manifest",
        _ => "unknown_domain_action",
    }
}

fn execute_action(graph: &mut HyperGraph, action: &OrganAction) -> OrganExecution {
    match action.kind.as_str() {
        "keep_remote_gated" => OrganExecution {
            status: "blocked",
            reason: "remote_network_requires_explicit_user_flag",
            effect: "remote_network_remains_disabled",
        },
        "repair_registration" => OrganExecution {
            status: "blocked",
            reason: "registration_repair_requires_design_change",
            effect: "no_runtime_mutation_performed",
        },
        "coordinate_capability_pressure" => execute_coordinator_action(graph),
        "maintain_local_dialogue_bridge" => execute_human_interface_action(graph),
        "review_architecture_without_mutation" => execute_meta_architect_action(graph),
        "emit_local_reflex_probe" => execute_fast_reflex_action(graph),
        "sample_runtime_cost" => execute_benchmark_action(graph),
        "validate_command_surface" => execute_command_router_action(graph),
        "verify_state_paths" => match execute_persistence_action(graph) {
            Ok(()) => real_execution("real_api:state_paths_verified"),
            Err(_) => OrganExecution {
                status: "blocked",
                reason: "state_dir_unavailable",
                effect: "real_api:state_paths_failed",
            },
        },
        "consolidate_local_memory" => execute_legacy_memory_action(graph),
        "ground_reasoning_evidence" => execute_reason_action(graph),
        "prepare_local_response_patterns" => execute_dialogue_action(graph),
        "emit_system_snapshot" => execute_observatory_action(graph),
        "propose_bounded_improvement" => execute_evolution_action(graph),
        "regulate_tension_field" => execute_campo_tension_action(graph),
        "strengthen_local_kg_edges" => execute_knowledge_graph_action(graph),
        "extract_local_architecture_context" => execute_autoconsumo_action(graph),
        "validate_crystal_roundtrip" => execute_venado_action(graph),
        "cycle_paradigm_budget" => execute_paradigm_hub_action(graph),
        "update_internal_ecosystem" => execute_ecosystem_action(graph),
        "record_rebirth_trace" => execute_rebirth_action(graph),
        "integrate_cognitive_signals" => execute_cognition_action(graph),
        "append_autonomy_trace" => execute_history_action(graph),
        "inspect_daemon_liveness" => execute_daemon_action(graph),
        "refresh_runtime_metrics" => execute_telemetry_action(graph),
        "check_local_api_readiness" => execute_api_server_action(graph),
        "measure_readiness_gaps" => execute_readiness_action(graph),
        "run_internal_lifecycle_ritual" => execute_organic_lifecycle_action(graph),
        "regulate_graph_consciousness" => execute_conscious_regulator_action(graph),
        "coordinate_context_pack_actions" => execute_context_augmentation_action(graph),
        "run_hierarchical_reasoning_cycle" => execute_hrm_reasoner_action(graph),
        "synthesize_voice_manifest" => execute_voice_synthesizer_action(graph),
        "refresh_command_help" => execute_help_action(graph),
        kind => match domain_action_effect(kind) {
            Some(effect) => OrganExecution {
                status: "executed",
                reason: "observed_only_no_safe_api",
                effect,
            },
            None => OrganExecution {
                status: "blocked",
                reason: "unknown_action_kind",
                effect: "observed_only:no_effect_recorded",
            },
        },
    }
}

fn domain_action_effect(_kind: &str) -> Option<&'static str> {
    None
}

fn has_real_api_executor(kind: &str) -> bool {
    matches!(
        kind,
        "coordinate_capability_pressure"
            | "maintain_local_dialogue_bridge"
            | "review_architecture_without_mutation"
            | "emit_local_reflex_probe"
            | "sample_runtime_cost"
            | "validate_command_surface"
            | "verify_state_paths"
            | "refresh_runtime_metrics"
            | "check_local_api_readiness"
            | "inspect_daemon_liveness"
            | "consolidate_local_memory"
            | "ground_reasoning_evidence"
            | "prepare_local_response_patterns"
            | "emit_system_snapshot"
            | "append_autonomy_trace"
            | "propose_bounded_improvement"
            | "integrate_cognitive_signals"
            | "regulate_tension_field"
            | "strengthen_local_kg_edges"
            | "extract_local_architecture_context"
            | "validate_crystal_roundtrip"
            | "cycle_paradigm_budget"
            | "update_internal_ecosystem"
            | "record_rebirth_trace"
            | "refresh_command_help"
            | "measure_readiness_gaps"
            | "run_internal_lifecycle_ritual"
            | "regulate_graph_consciousness"
            | "coordinate_context_pack_actions"
            | "run_hierarchical_reasoning_cycle"
            | "synthesize_voice_manifest"
    )
}

fn execute_coordinator_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(coordinator) =
        node_mut::<crate::eden_garm::nodes::coordinator::CoordinatorNode>(graph, "coordinator")
    else {
        return missing_real_api("coordinator");
    };
    let _ = coordinator.observe_capability_pressure();
    real_execution("real_api:capability_pressure_observed")
}

fn execute_human_interface_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(human) = node_mut::<crate::eden_garm::nodes::human_interface::HumanInterfaceNode>(
        graph,
        "human_interface",
    ) else {
        return missing_real_api("human_interface");
    };
    let _ = human.maintain_local_bridge();
    real_execution("real_api:local_dialogue_bridge_checked")
}

fn execute_meta_architect_action(graph: &mut HyperGraph) -> OrganExecution {
    let global_fe = graph_free_energy(graph);
    let Some(architect) = node_mut::<crate::eden_garm::nodes::meta_architect::MetaArchitectNode>(
        graph,
        "meta_architect",
    ) else {
        return missing_real_api("meta_architect");
    };
    let _ = architect.review_without_mutation(global_fe);
    real_execution("real_api:architecture_review_recorded")
}

fn execute_fast_reflex_action(graph: &mut HyperGraph) -> OrganExecution {
    let global_fe = graph_free_energy(graph);
    let Some(reflexes) = node_mut::<crate::eden_garm::nodes::fast_reflexes::FastReflexesNode>(
        graph,
        "fast_reflexes",
    ) else {
        return missing_real_api("fast_reflexes");
    };
    let _ = reflexes.local_reflex_probe(global_fe);
    real_execution("real_api:reflex_probe_recorded")
}

fn execute_benchmark_action(graph: &mut HyperGraph) -> OrganExecution {
    let global_fe = graph_free_energy(graph);
    let Some(benchmark) =
        node_mut::<crate::eden_garm::nodes::benchmark::BenchmarkNode>(graph, "benchmark")
    else {
        return missing_real_api("benchmark");
    };
    let _ = benchmark.sample_runtime_cost(global_fe);
    real_execution("real_api:runtime_cost_sampled")
}

fn execute_command_router_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(router) = node_mut::<crate::eden_garm::nodes::command_router::CommandRouterNode>(
        graph,
        "command_router",
    ) else {
        return missing_real_api("command_router");
    };
    let _ = router.validate_surface();
    real_execution("real_api:command_surface_validated")
}

fn execute_legacy_memory_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(memory) = node_mut::<crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode>(
        graph,
        "legacy_memory",
    ) else {
        return missing_real_api("legacy_memory");
    };
    let before = memory.fact_count();
    let _ = memory.remember("organ autonomy: legacy_memory consolidated local memory");
    if memory.fact_count() >= before {
        OrganExecution {
            status: "executed",
            reason: "real_api_executed",
            effect: "real_api:local_memory_consolidated",
        }
    } else {
        missing_real_api("legacy_memory")
    }
}

fn execute_knowledge_graph_action(graph: &mut HyperGraph) -> OrganExecution {
    let tick = graph.global_tick;
    let Some(kg) = node_mut::<
        crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode,
    >(graph, "legacy_knowledge_graph") else {
        return missing_real_api("legacy_knowledge_graph");
    };
    let _ = kg.add_fact_from(
        "organ_autonomy strengthens legacy_knowledge_graph",
        "organ_autonomy",
    );
    let _ = kg.regulate_capacity(tick, 100_000, 150_000, 0.20);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:local_kg_edges_strengthened",
    }
}

fn execute_persistence_action(graph: &mut HyperGraph) -> Result<(), String> {
    let Some(persistence) =
        node_mut::<crate::eden_garm::nodes::persistence::PersistenceNode>(graph, "persistence")
    else {
        return state_paths::ensure_state_dir();
    };
    persistence.verify_state_paths()
}

fn execute_reason_action(graph: &mut HyperGraph) -> OrganExecution {
    let facts = memory_facts(graph);
    let Some(reason) = node_mut::<crate::eden_garm::nodes::legacy_reason::LegacyReasonNode>(
        graph,
        "legacy_reason",
    ) else {
        return missing_real_api("legacy_reason");
    };
    let _ = reason.ground_evidence(&facts);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:reasoning_evidence_grounded",
    }
}

fn execute_dialogue_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(dialogue) = node_mut::<crate::eden_garm::nodes::legacy_dialogue::LegacyDialogueNode>(
        graph,
        "legacy_dialogue",
    ) else {
        return missing_real_api("legacy_dialogue");
    };
    let _ = dialogue.prepare_response_patterns();
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:local_response_patterns_prepared",
    }
}

fn execute_observatory_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let alive = graph.alive_node_count();
    let edge_count = graph.adjacency.iter().map(Vec::len).sum();
    let Some(observatory) =
        node_mut::<crate::eden_garm::nodes::observatory::ObservatoryNode>(graph, "observatory")
    else {
        return missing_real_api("observatory");
    };
    let _ = observatory.autonomy_snapshot(alive, edge_count, stats.memory_facts, stats.tick);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:system_snapshot_emitted",
    }
}

fn execute_evolution_action(graph: &mut HyperGraph) -> OrganExecution {
    let tick = graph.global_tick;
    let Some(evolution) = node_mut::<crate::eden_garm::nodes::legacy_evolution::LegacyEvolutionNode>(
        graph,
        "legacy_evolution",
    ) else {
        return missing_real_api("legacy_evolution");
    };
    let _ = evolution.propose_bounded_improvement(tick);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:bounded_improvement_proposed",
    }
}

fn execute_campo_tension_action(graph: &mut HyperGraph) -> OrganExecution {
    let tick = graph.global_tick;
    let Some(campo) = node_mut::<crate::eden_garm::nodes::campo_tension::CampoTensionNode>(
        graph,
        "campo_tension",
    ) else {
        return missing_real_api("campo_tension");
    };
    let _ = campo.regulate_once(tick);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:tension_field_regulated",
    }
}

fn execute_autoconsumo_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(autoconsumo) = node_mut::<
        crate::eden_garm::nodes::legacy_runtime_extensions::AutoconsumoNode,
    >(graph, "legacy_autoconsumo") else {
        return missing_real_api("legacy_autoconsumo");
    };
    let _ = autoconsumo.nutrirse();
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:local_architecture_context_extracted",
    }
}

fn execute_venado_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(venado) = node_mut::<
        crate::eden_garm::nodes::legacy_runtime_extensions::VenadoCompatibilityNode,
    >(graph, "legacy_venado") else {
        return missing_real_api("legacy_venado");
    };
    let fields = vec![("source".to_string(), "organ_autonomy".to_string())];
    if venado.cristalizar("organ_autonomy_probe", &fields).is_ok()
        && venado.descristalizar("organ_autonomy_probe").is_ok()
    {
        OrganExecution {
            status: "executed",
            reason: "real_api_executed",
            effect: "real_api:crystal_roundtrip_validated",
        }
    } else {
        missing_real_api("legacy_venado")
    }
}

fn execute_paradigm_hub_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(_hub) = node_ref::<crate::eden_garm::nodes::legacy_runtime_extensions::ParadigmHubNode>(
        graph,
        "legacy_paradigm_hub",
    ) else {
        return missing_real_api("legacy_paradigm_hub");
    };
    OrganExecution {
        status: "blocked",
        reason: "superseded_by_paradigm_architecture_eval",
        effect: "observed_only:legacy_paradigm_hub_not_cycled",
    }
}

fn execute_ecosystem_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let Some(ecosystem) = node_mut::<
        crate::eden_garm::nodes::legacy_runtime_extensions::EcoSystemNode,
    >(graph, "legacy_ecosystem") else {
        return missing_real_api("legacy_ecosystem");
    };
    let _ = ecosystem.pulso_global(stats.memory_facts);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:internal_ecosystem_updated",
    }
}

fn execute_rebirth_action(graph: &mut HyperGraph) -> OrganExecution {
    let facts = memory_facts(graph);
    let Some(rebirth) = node_mut::<
        crate::eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode,
    >(graph, "legacy_rebirth_meltrace") else {
        return missing_real_api("legacy_rebirth_meltrace");
    };
    let _ = rebirth.rebirth(&facts);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:rebirth_trace_recorded",
    }
}

fn execute_cognition_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(cognition) = node_mut::<crate::eden_garm::nodes::legacy_cognition::LegacyCognitionNode>(
        graph,
        "legacy_cognition",
    ) else {
        return missing_real_api("legacy_cognition");
    };
    cognition.record_exploration("organ_autonomy", 0.05, true);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:cognitive_signals_integrated",
    }
}

fn execute_history_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(history) = node_mut::<crate::eden_garm::nodes::legacy_history::LegacyHistoryNode>(
        graph,
        "legacy_history",
    ) else {
        return missing_real_api("legacy_history");
    };
    history.record_command("[ORGANOS] append_autonomy_trace real_api");
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:autonomy_trace_appended",
    }
}

fn execute_daemon_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(daemon) = node_mut::<crate::eden_garm::nodes::daemon::DaemonNode>(graph, "daemon")
    else {
        return missing_real_api("daemon");
    };
    let _ = daemon.inspect_liveness();
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:daemon_liveness_inspected",
    }
}

fn execute_api_server_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(api) =
        node_mut::<crate::eden_garm::nodes::api_server::ApiServerNode>(graph, "api_server")
    else {
        return missing_real_api("api_server");
    };
    let _ = api.check_local_readiness();
    real_execution("real_api:local_api_readiness_checked")
}

fn execute_help_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(help) = node_mut::<crate::eden_garm::nodes::help::HelpNode>(graph, "help") else {
        return missing_real_api("help");
    };
    let _ = help.help();
    real_execution("real_api:command_help_refreshed")
}

fn execute_telemetry_action(graph: &mut HyperGraph) -> OrganExecution {
    let tick = graph.global_tick;
    let alive = graph.alive_node_count();
    let Some(telemetry) =
        node_mut::<crate::eden_garm::nodes::telemetry::TelemetryNode>(graph, "telemetry")
    else {
        return missing_real_api("telemetry");
    };
    let _ = telemetry.refresh_snapshot(alive, tick);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:runtime_metrics_refreshed",
    }
}

fn execute_readiness_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let Some(readiness) =
        node_mut::<crate::eden_garm::nodes::readiness::ReadinessNode>(graph, "readiness")
    else {
        return missing_real_api("readiness");
    };
    readiness.observe_system(
        stats.memory_facts,
        stats.kg_edges,
        stats.capability_count,
        stats.tick,
        true,
        stats.history_events,
    );
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:readiness_gaps_measured",
    }
}

fn execute_organic_lifecycle_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let facts = memory_facts(graph);
    let tension = node_ref::<crate::eden_garm::nodes::campo_tension::CampoTensionNode>(
        graph,
        "campo_tension",
    )
    .map(|campo| campo.tension())
    .unwrap_or(0.0);
    let Some(lifecycle) = node_mut::<
        crate::eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode,
    >(graph, "organic_lifecycle") else {
        return missing_real_api("organic_lifecycle");
    };
    let _ = lifecycle.ritual(&facts, stats.kg_edges, tension, stats.tick);
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:internal_lifecycle_ritual_run",
    }
}

fn execute_conscious_regulator_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let hyper_edges = graph.adjacency.iter().map(Vec::len).sum();
    let Some(regulator) = node_mut::<
        crate::eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode,
    >(graph, "conscious_graph_regulator") else {
        return missing_real_api("conscious_graph_regulator");
    };
    let _ = regulator.observe(
        stats.kg_edges,
        stats.kg_nodes,
        hyper_edges,
        stats.memory_facts,
        0.0,
        stats.tick,
    );
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:graph_consciousness_regulated",
    }
}

fn execute_context_augmentation_action(graph: &mut HyperGraph) -> OrganExecution {
    let Some(cag) = node_mut::<
        crate::eden_garm::nodes::context_augmentation::ContextAugmentationNode,
    >(graph, "context_augmentation") else {
        return missing_real_api("context_augmentation");
    };
    let _ = cag.plan_actions("organ autonomy");
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:context_pack_actions_coordinated",
    }
}

fn execute_hrm_reasoner_action(graph: &mut HyperGraph) -> OrganExecution {
    let stats = graph_stats(graph);
    let query = format!(
        "organ autonomy continuity memory={} kg_edges={} history={}",
        stats.memory_facts, stats.kg_edges, stats.history_events
    );
    let Some(hrm) = node_mut::<
        crate::eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode,
    >(graph, "hrm_reasoner") else {
        return missing_real_api("hrm_reasoner");
    };
    let _ = hrm.autonomy_cycle(
        stats.tick,
        stats.memory_facts,
        stats.kg_edges,
        stats.history_events,
    );
    if let Some(kg) = node_mut::<
        crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode,
    >(graph, "legacy_knowledge_graph")
    {
        let _ = kg.add_fact_from(&format!("{} is hrm_cycle", query), "hrm_reasoner");
    }
    if let Some(history) = node_mut::<crate::eden_garm::nodes::legacy_history::LegacyHistoryNode>(
        graph,
        "legacy_history",
    ) {
        history.record_command("[HRM] autonomous hierarchical reasoning cycle");
    }
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:hierarchical_reasoning_cycle_run",
    }
}

fn execute_voice_synthesizer_action(graph: &mut HyperGraph) -> OrganExecution {
    let tick = graph.global_tick;
    let Some(voice) = node_mut::<crate::eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode>(
        graph,
        "voice_synthesizer",
    ) else {
        return missing_real_api("voice_synthesizer");
    };
    let _ = voice.synthesize_text("organ autonomy voice manifest", tick);
    if let Some(history) = node_mut::<crate::eden_garm::nodes::legacy_history::LegacyHistoryNode>(
        graph,
        "legacy_history",
    ) {
        history.record_command("[VOZ-TTS] autonomous local manifest generated");
    }
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect: "real_api:voice_manifest_generated",
    }
}

struct OrganGraphStats {
    memory_facts: usize,
    kg_edges: usize,
    kg_nodes: usize,
    capability_count: usize,
    history_events: u64,
    tick: u64,
}

fn graph_stats(graph: &HyperGraph) -> OrganGraphStats {
    let memory_facts = node_ref::<crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode>(
        graph,
        "legacy_memory",
    )
    .map(|memory| memory.fact_count())
    .unwrap_or(0);
    let (kg_edges, kg_nodes) = node_ref::<
        crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode,
    >(graph, "legacy_knowledge_graph")
    .map(|kg| (kg.edge_count(), kg.node_count()))
    .unwrap_or((0, 0));
    let history_events = node_ref::<crate::eden_garm::nodes::legacy_history::LegacyHistoryNode>(
        graph,
        "legacy_history",
    )
    .map(|history| history.report().lines().count() as u64)
    .unwrap_or(0);
    OrganGraphStats {
        memory_facts,
        kg_edges,
        kg_nodes,
        capability_count: graph.nodes.len(),
        history_events,
        tick: graph.global_tick,
    }
}

fn graph_free_energy(graph: &HyperGraph) -> f32 {
    graph.nodes.iter().map(|node| node.free_energy()).sum()
}

fn memory_facts(graph: &HyperGraph) -> Vec<String> {
    node_ref::<crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode>(graph, "legacy_memory")
        .map(|memory| memory.facts().iter().take(16).cloned().collect())
        .unwrap_or_else(|| vec!["organ autonomy is active".to_string()])
}

fn organ_observable(graph: &HyperGraph, organ: &str) -> String {
    match organ {
        "meta_architect" => {
            node_ref::<crate::eden_garm::nodes::meta_architect::MetaArchitectNode>(graph, organ)
                .map(|node| node.architecture_snapshot())
                .unwrap_or_else(|| "meta:missing".to_string())
        }
        "coordinator" => {
            node_ref::<crate::eden_garm::nodes::coordinator::CoordinatorNode>(graph, organ)
                .map(|node| node.autonomy_snapshot())
                .unwrap_or_else(|| "coordinator:missing".to_string())
        }
        "human_interface" => {
            node_ref::<crate::eden_garm::nodes::human_interface::HumanInterfaceNode>(graph, organ)
                .map(|node| node.bridge_snapshot())
                .unwrap_or_else(|| "human:missing".to_string())
        }
        "fast_reflexes" => {
            node_ref::<crate::eden_garm::nodes::fast_reflexes::FastReflexesNode>(graph, organ)
                .map(|node| node.reflex_snapshot())
                .unwrap_or_else(|| "reflex:missing".to_string())
        }
        "benchmark" => node_ref::<crate::eden_garm::nodes::benchmark::BenchmarkNode>(graph, organ)
            .map(|node| node.benchmark_snapshot())
            .unwrap_or_else(|| "benchmark:missing".to_string()),
        "command_router" => {
            node_ref::<crate::eden_garm::nodes::command_router::CommandRouterNode>(graph, organ)
                .map(|node| node.router_snapshot())
                .unwrap_or_else(|| "router:missing".to_string())
        }
        "persistence" => {
            node_ref::<crate::eden_garm::nodes::persistence::PersistenceNode>(graph, organ)
                .map(|node| node.persistence_snapshot())
                .unwrap_or_else(|| "persistence:missing".to_string())
        }
        "telemetry" => node_ref::<crate::eden_garm::nodes::telemetry::TelemetryNode>(graph, organ)
            .map(|node| node.telemetry_snapshot())
            .unwrap_or_else(|| "telemetry:missing".to_string()),
        "api_server" => {
            node_ref::<crate::eden_garm::nodes::api_server::ApiServerNode>(graph, organ)
                .map(|node| node.readiness_snapshot())
                .unwrap_or_else(|| "api_server:missing".to_string())
        }
        "daemon" => node_ref::<crate::eden_garm::nodes::daemon::DaemonNode>(graph, organ)
            .map(|node| node.status())
            .unwrap_or_else(|| "daemon:missing".to_string()),
        "legacy_dialogue" => {
            node_ref::<crate::eden_garm::nodes::legacy_dialogue::LegacyDialogueNode>(graph, organ)
                .map(|node| node.dialogue_snapshot())
                .unwrap_or_else(|| "dialogue:missing".to_string())
        }
        "observatory" => {
            node_ref::<crate::eden_garm::nodes::observatory::ObservatoryNode>(graph, organ)
                .map(|node| node.observatory_snapshot())
                .unwrap_or_else(|| "observatory:missing".to_string())
        }
        "help" => node_ref::<crate::eden_garm::nodes::help::HelpNode>(graph, organ)
            .map(|node| node.help_snapshot())
            .unwrap_or_else(|| "help:missing".to_string()),
        "legacy_memory" => {
            node_ref::<crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode>(graph, organ)
                .map(|node| format!("memory_facts:{}", node.fact_count()))
                .unwrap_or_else(|| "memory_facts:missing".to_string())
        }
        "legacy_reason" => {
            node_ref::<crate::eden_garm::nodes::legacy_reason::LegacyReasonNode>(graph, organ)
                .map(|node| node.reason_snapshot())
                .unwrap_or_else(|| "reason:missing".to_string())
        }
        "legacy_history" => {
            node_ref::<crate::eden_garm::nodes::legacy_history::LegacyHistoryNode>(graph, organ)
                .map(|node| node.history_snapshot())
                .unwrap_or_else(|| "history:missing".to_string())
        }
        "legacy_evolution" => {
            node_ref::<crate::eden_garm::nodes::legacy_evolution::LegacyEvolutionNode>(graph, organ)
                .map(|node| node.evolution_snapshot())
                .unwrap_or_else(|| "evolution:missing".to_string())
        }
        "legacy_cognition" => {
            node_ref::<crate::eden_garm::nodes::legacy_cognition::LegacyCognitionNode>(graph, organ)
                .map(|node| node.cognition_snapshot())
                .unwrap_or_else(|| "cognition:missing".to_string())
        }
        "legacy_knowledge_graph" => node_ref::<
            crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode,
        >(graph, organ)
        .map(|node| node.autonomy_snapshot())
        .unwrap_or_else(|| "kg_edges:missing".to_string()),
        "campo_tension" => {
            node_ref::<crate::eden_garm::nodes::campo_tension::CampoTensionNode>(graph, organ)
                .map(|node| format!("tension:{:.3}", node.tension()))
                .unwrap_or_else(|| "tension:missing".to_string())
        }
        "legacy_autoconsumo" => node_ref::<
            crate::eden_garm::nodes::legacy_runtime_extensions::AutoconsumoNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "autoconsumo:parses:{} fragments:{}",
                node.parse_count(),
                node.fragment_count()
            )
        })
        .unwrap_or_else(|| "autoconsumo:missing".to_string()),
        "legacy_venado" => node_ref::<
            crate::eden_garm::nodes::legacy_runtime_extensions::VenadoCompatibilityNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "venado:writes:{} reads:{}",
                node.write_count(),
                node.read_count()
            )
        })
        .unwrap_or_else(|| "venado:missing".to_string()),
        "legacy_paradigm_hub" => node_ref::<
            crate::eden_garm::nodes::legacy_runtime_extensions::ParadigmHubNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "paradigm_architecture_legacy:superseded active_snapshot:{} cycles:{} inferred:{}",
                node.paradigm_count(),
                node.cycle_count(),
                node.inferred_fact_count()
            )
        })
        .unwrap_or_else(|| "paradigm:missing".to_string()),
        "legacy_ecosystem" => node_ref::<
            crate::eden_garm::nodes::legacy_runtime_extensions::EcoSystemNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "ecosystem:live:{} births:{} deaths:{} pulses:{}",
                node.live_count(),
                node.birth_count(),
                node.death_count(),
                node.pulse_count()
            )
        })
        .unwrap_or_else(|| "ecosystem:missing".to_string()),
        "legacy_rebirth_meltrace" => node_ref::<
            crate::eden_garm::nodes::legacy_runtime_extensions::RebirthMeltraceNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "rebirth:rebirths:{} deaths:{} events:{}",
                node.rebirth_count(),
                node.death_count(),
                node.event_count()
            )
        })
        .unwrap_or_else(|| "rebirth:missing".to_string()),
        "legacy_crawler" => "remote_network:gated".to_string(),
        "readiness" => node_ref::<crate::eden_garm::nodes::readiness::ReadinessNode>(graph, organ)
            .map(|node| {
                format!(
                    "readiness:observations:{} score:{:.3}",
                    node.observation_count(),
                    node.readiness_score()
                )
            })
            .unwrap_or_else(|| "readiness:missing".to_string()),
        "organic_lifecycle" => node_ref::<
            crate::eden_garm::nodes::organic_lifecycle::OrganicLifecycleNode,
        >(graph, organ)
        .map(|node| {
            format!(
                "lifecycle:children:{} thoughts:{}",
                node.child_count(),
                node.autonomous_thought_count()
            )
        })
        .unwrap_or_else(|| "lifecycle:missing".to_string()),
        "conscious_graph_regulator" => node_ref::<
            crate::eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode,
        >(graph, organ)
        .map(|node| node.autonomy_snapshot())
        .unwrap_or_else(|| "conscious:missing".to_string()),
        "context_augmentation" => node_ref::<
            crate::eden_garm::nodes::context_augmentation::ContextAugmentationNode,
        >(graph, organ)
        .map(|node| {
            let metrics = node.metrics();
            format!(
                "cag:cache:{} hits:{} misses:{} actions:{} pending:{} executed:{} blocked:{}",
                metrics.cache_entries,
                metrics.hits,
                metrics.misses,
                node.action_count(),
                metrics.pending_actions,
                metrics.actions_executed,
                metrics.actions_blocked
            )
        })
        .unwrap_or_else(|| "cag:missing".to_string()),
        "hrm_reasoner" => node_ref::<
            crate::eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode,
        >(graph, organ)
        .map(|node| node.snapshot())
        .unwrap_or_else(|| "hrm:missing".to_string()),
        "voice_synthesizer" => node_ref::<
            crate::eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode,
        >(graph, organ)
        .map(|node| node.snapshot())
        .unwrap_or_else(|| "voice:missing".to_string()),
        _ => graph
            .nodes
            .iter()
            .find(|node| node.name() == organ)
            .map(|node| format!("free_energy:{:.3}", node.free_energy()))
            .unwrap_or_else(|| "node:missing".to_string()),
    }
}

fn execution_delta(status: &str, effect: &str, before: &str, after: &str) -> String {
    if status == "blocked" {
        return "not_executed".to_string();
    }
    if before == after {
        format!("{};observable_unchanged:{}", effect, after)
    } else {
        format!("{}->{}", before, after)
    }
}

fn node_mut<'a, T: 'static>(graph: &'a mut HyperGraph, name: &str) -> Option<&'a mut T> {
    graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == name)
        .and_then(|node| node.as_any_mut().downcast_mut::<T>())
}

fn node_ref<'a, T: 'static>(graph: &'a HyperGraph, name: &str) -> Option<&'a T> {
    graph
        .nodes
        .iter()
        .find(|node| node.name() == name)
        .and_then(|node| node.as_any().downcast_ref::<T>())
}

fn missing_real_api(organ: &'static str) -> OrganExecution {
    let effect = match organ {
        "legacy_memory" => "observed_only:legacy_memory_api_unavailable",
        "coordinator" => "observed_only:coordinator_api_unavailable",
        "human_interface" => "observed_only:human_interface_api_unavailable",
        "meta_architect" => "observed_only:meta_architect_api_unavailable",
        "fast_reflexes" => "observed_only:fast_reflexes_api_unavailable",
        "benchmark" => "observed_only:benchmark_api_unavailable",
        "command_router" => "observed_only:command_router_api_unavailable",
        "api_server" => "observed_only:api_server_api_unavailable",
        "help" => "observed_only:help_api_unavailable",
        "legacy_reason" => "observed_only:legacy_reason_api_unavailable",
        "legacy_dialogue" => "observed_only:legacy_dialogue_api_unavailable",
        "observatory" => "observed_only:observatory_api_unavailable",
        "legacy_evolution" => "observed_only:legacy_evolution_api_unavailable",
        "campo_tension" => "observed_only:campo_tension_api_unavailable",
        "legacy_knowledge_graph" => "observed_only:legacy_knowledge_graph_api_unavailable",
        "legacy_autoconsumo" => "observed_only:legacy_autoconsumo_api_unavailable",
        "legacy_venado" => "observed_only:legacy_venado_api_unavailable",
        "legacy_paradigm_hub" => "observed_only:legacy_paradigm_hub_api_unavailable",
        "legacy_ecosystem" => "observed_only:legacy_ecosystem_api_unavailable",
        "legacy_rebirth_meltrace" => "observed_only:legacy_rebirth_meltrace_api_unavailable",
        "legacy_cognition" => "observed_only:legacy_cognition_api_unavailable",
        "legacy_history" => "observed_only:legacy_history_api_unavailable",
        "daemon" => "observed_only:daemon_api_unavailable",
        "telemetry" => "observed_only:telemetry_api_unavailable",
        "readiness" => "observed_only:readiness_api_unavailable",
        "organic_lifecycle" => "observed_only:organic_lifecycle_api_unavailable",
        "conscious_graph_regulator" => "observed_only:conscious_regulator_api_unavailable",
        "context_augmentation" => "observed_only:context_augmentation_api_unavailable",
        "hrm_reasoner" => "observed_only:hrm_reasoner_api_unavailable",
        "voice_synthesizer" => "observed_only:voice_synthesizer_api_unavailable",
        _ => "observed_only:real_api_unavailable",
    };
    OrganExecution {
        status: "executed",
        reason: "observed_only_missing_safe_api",
        effect,
    }
}

fn real_execution(effect: &'static str) -> OrganExecution {
    OrganExecution {
        status: "executed",
        reason: "real_api_executed",
        effect,
    }
}

fn metrics_from_state(state: &OrganAutonomyState) -> OrganAutonomyMetrics {
    OrganAutonomyMetrics {
        pending_actions: state
            .organs
            .iter()
            .map(|organ| {
                organ
                    .actions
                    .iter()
                    .filter(|action| action.status == "pending")
                    .count() as u64
            })
            .sum(),
        actions_executed: state
            .organs
            .iter()
            .map(|organ| organ.actions_executed)
            .sum(),
        actions_blocked: state.organs.iter().map(|organ| organ.actions_blocked).sum(),
        autonomous_runs: state.organs.iter().map(|organ| organ.autonomous_runs).sum(),
        feedback_positive: state
            .organs
            .iter()
            .map(|organ| organ.feedback_positive)
            .sum(),
        feedback_negative: state
            .organs
            .iter()
            .map(|organ| organ.feedback_negative)
            .sum(),
    }
}

fn individual_organ_to_json(organ: &IndividualOrganAutonomy) -> serde_json::Value {
    serde_json::json!({
        "name": organ.name,
        "next_action_id": organ.next_action_id,
        "feedback_positive": organ.feedback_positive,
        "feedback_negative": organ.feedback_negative,
        "actions_executed": organ.actions_executed,
        "actions_blocked": organ.actions_blocked,
        "autonomous_runs": organ.autonomous_runs,
        "actions": organ.actions.iter().map(action_to_json).collect::<Vec<_>>(),
        "audit": organ.audit.iter().map(audit_to_json).collect::<Vec<_>>(),
    })
}

fn individual_organ_from_json(value: &serde_json::Value) -> Option<IndividualOrganAutonomy> {
    let name = value.get("name")?.as_str()?.to_string();
    let mut organ = IndividualOrganAutonomy::new(&name);
    organ.next_action_id = value
        .get("next_action_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        .max(1);
    organ.feedback_positive = value
        .get("feedback_positive")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    organ.feedback_negative = value
        .get("feedback_negative")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    organ.actions_executed = value
        .get("actions_executed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    organ.actions_blocked = value
        .get("actions_blocked")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    organ.autonomous_runs = value
        .get("autonomous_runs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(actions) = value.get("actions").and_then(|v| v.as_array()) {
        for action in actions
            .iter()
            .filter_map(action_from_json)
            .take(MAX_ORGAN_ACTIONS)
        {
            organ.actions.push_back(action);
        }
    }
    if let Some(audit) = value.get("audit").and_then(|v| v.as_array()) {
        for entry in audit
            .iter()
            .filter_map(audit_from_json)
            .take(MAX_ORGAN_AUDIT)
        {
            organ.audit.push_back(entry);
        }
    }
    Some(organ)
}

fn load_legacy_global_state(snapshot: &serde_json::Value) -> OrganAutonomyState {
    let mut state = new_individual_state();
    let feedback_positive = snapshot
        .get("feedback_positive")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let feedback_negative = snapshot
        .get("feedback_negative")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let actions_executed = snapshot
        .get("actions_executed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let actions_blocked = snapshot
        .get("actions_blocked")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let autonomous_runs = snapshot
        .get("autonomous_runs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if let Some(actions) = snapshot.get("actions").and_then(|v| v.as_array()) {
        for action in actions.iter().filter_map(action_from_json) {
            let organ = individual_organ_mut(&mut state, &action.organ);
            organ.actions.push_back(action);
            while organ.actions.len() > MAX_ORGAN_ACTIONS {
                organ.actions.pop_front();
            }
        }
    }
    if let Some(audit) = snapshot.get("audit").and_then(|v| v.as_array()) {
        for entry in audit.iter().filter_map(audit_from_json) {
            let organ = individual_organ_mut(&mut state, &entry.organ);
            organ.audit.push_back(entry);
            while organ.audit.len() > MAX_ORGAN_AUDIT {
                organ.audit.pop_front();
            }
        }
    }
    for organ in &mut state.organs {
        organ.feedback_positive = feedback_positive;
        organ.feedback_negative = feedback_negative;
        organ.actions_executed = organ
            .actions
            .iter()
            .filter(|action| action.status == "executed")
            .count() as u64;
        organ.actions_blocked = organ
            .actions
            .iter()
            .filter(|action| action.status == "blocked")
            .count() as u64;
        organ.autonomous_runs = autonomous_runs;
        organ.next_action_id = organ
            .actions
            .iter()
            .map(|action| action.id)
            .max()
            .unwrap_or(0)
            + 1;
    }
    if state.organs.iter().all(|organ| organ.actions.is_empty()) {
        let per_organ_executed = actions_executed / ORGAN_PROFILES.len() as u64;
        let per_organ_blocked = actions_blocked / ORGAN_PROFILES.len() as u64;
        for organ in &mut state.organs {
            organ.actions_executed = per_organ_executed;
            organ.actions_blocked = per_organ_blocked;
        }
    }
    state
}

fn action_to_json(action: &OrganAction) -> serde_json::Value {
    serde_json::json!({
        "id": action.id,
        "organ": action.organ,
        "kind": action.kind,
        "status": action.status,
        "reason": action.reason,
        "delta": action.delta,
    })
}

fn action_from_json(value: &serde_json::Value) -> Option<OrganAction> {
    Some(OrganAction {
        id: value.get("id")?.as_u64()?,
        organ: value.get("organ")?.as_str()?.to_string(),
        kind: value.get("kind")?.as_str()?.to_string(),
        status: value.get("status")?.as_str()?.to_string(),
        reason: value.get("reason")?.as_str()?.to_string(),
        delta: value
            .get("delta")
            .and_then(|v| v.as_str())
            .unwrap_or("legacy:no_delta")
            .to_string(),
    })
}

fn audit_to_json(entry: &OrganAuditEntry) -> serde_json::Value {
    serde_json::json!({
        "action_id": entry.action_id,
        "organ": entry.organ,
        "kind": entry.kind,
        "status": entry.status,
        "reason": entry.reason,
        "mode": entry.mode,
        "delta": entry.delta,
    })
}

fn audit_from_json(value: &serde_json::Value) -> Option<OrganAuditEntry> {
    Some(OrganAuditEntry {
        action_id: value.get("action_id")?.as_u64()?,
        organ: value.get("organ")?.as_str()?.to_string(),
        kind: value.get("kind")?.as_str()?.to_string(),
        status: value.get("status")?.as_str()?.to_string(),
        reason: value.get("reason")?.as_str()?.to_string(),
        mode: value.get("mode")?.as_str()?.to_string(),
        delta: value
            .get("delta")
            .and_then(|v| v.as_str())
            .unwrap_or("legacy:no_delta")
            .to_string(),
    })
}

fn organ_status(graph: &HyperGraph, name: &str) -> String {
    graph
        .nodes
        .iter()
        .find(|node| node.name() == name)
        .map(|node| {
            format!(
                "{} scale={:?} fe={:.2}",
                if node.is_alive() { "alive" } else { "dead" },
                node.scale(),
                node.free_energy()
            )
        })
        .unwrap_or_else(|| "missing".to_string())
}

#[cfg(test)]
mod tests {
    use super::ORGAN_PROFILES;

    #[test]
    fn registry_tracks_primary_organs_with_hrm() {
        assert_eq!(ORGAN_PROFILES.len(), 32);
        assert!(ORGAN_PROFILES
            .iter()
            .any(|organ| organ.name == "context_augmentation"));
        assert!(ORGAN_PROFILES
            .iter()
            .any(|organ| organ.name == "legacy_crawler"));
        assert!(ORGAN_PROFILES
            .iter()
            .any(|organ| organ.name == "hrm_reasoner"));
        assert!(ORGAN_PROFILES
            .iter()
            .any(|organ| organ.name == "voice_synthesizer"));
        assert_eq!(super::autonomy_profiles().len(), 32);
        assert!(ORGAN_PROFILES
            .iter()
            .all(|organ| super::autonomy_mode(organ).contains("autonomo")));
    }

    #[test]
    fn health_report_is_well_formed_without_graph_nodes() {
        let graph = crate::eden_garm::HyperGraph::new();
        let report = super::organ_health_report(&graph);

        assert!(report.contains("[ORGANOS-HEALTH]"));
        assert!(report.contains("findings=32"));
        assert_eq!(super::organ_health_findings(&graph).len(), 32);
    }

    #[test]
    fn autonomy_state_is_individual_per_organ() {
        let mut state = super::new_individual_state();

        assert_eq!(state.organs.len(), 32);
        assert!(super::push_action(
            super::individual_organ_mut(&mut state, "legacy_memory"),
            "legacy_memory",
            super::domain_action_kind(
                ORGAN_PROFILES
                    .iter()
                    .find(|organ| organ.name == "legacy_memory")
                    .unwrap()
            ),
            "pending",
            "scheduled_domain_autonomy",
        ));
        assert_eq!(super::metrics_from_state(&state).pending_actions, 1);
        assert_eq!(
            state
                .organs
                .iter()
                .filter(|organ| !organ.actions.is_empty())
                .count(),
            1
        );
        assert!(state
            .organs
            .iter()
            .any(|organ| organ.name == "legacy_memory" && organ.next_action_id == 2));
    }

    #[test]
    fn planner_assigns_distinct_domain_actions_to_all_organs() {
        let mut kinds = std::collections::HashSet::new();

        for profile in ORGAN_PROFILES {
            let kind = super::domain_action_kind(profile);
            assert_ne!(kind, "safe_local_cycle");
            assert_ne!(kind, "repair_high_free_energy");
            assert!(kinds.insert(kind), "duplicate action kind: {}", kind);
        }
        assert_eq!(kinds.len(), 32);
    }

    #[test]
    fn all_non_crawler_organs_have_real_api_executors() {
        let mut real_api = 0usize;
        let mut blocked = 0usize;

        for profile in ORGAN_PROFILES {
            let kind = super::domain_action_kind(profile);
            if profile.name == "legacy_crawler" {
                assert_eq!(kind, "keep_remote_gated");
                blocked += 1;
            } else {
                assert!(
                    super::has_real_api_executor(kind),
                    "missing real_api executor for {} -> {}",
                    profile.name,
                    kind
                );
                assert!(super::domain_action_effect(kind).is_none());
                real_api += 1;
            }
        }

        assert_eq!(real_api, 31);
        assert_eq!(blocked, 1);
    }

    #[test]
    fn domain_actions_execute_with_specific_effects() {
        let profile = ORGAN_PROFILES
            .iter()
            .find(|organ| organ.name == "legacy_knowledge_graph")
            .unwrap();
        let action = super::OrganAction {
            id: 1,
            organ: profile.name.to_string(),
            kind: super::domain_action_kind(profile).to_string(),
            status: "pending".to_string(),
            reason: "scheduled_domain_autonomy".to_string(),
            delta: "pending".to_string(),
        };

        let mut graph = crate::eden_garm::HyperGraph::new();
        graph.add_node(Box::new(
            crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode::new(0),
        ));

        let execution = super::execute_action(&mut graph, &action);

        assert_eq!(execution.status, "executed");
        assert_eq!(execution.reason, "real_api_executed");
        assert_eq!(execution.effect, "real_api:local_kg_edges_strengthened");
    }

    #[test]
    fn real_api_delta_is_recorded_and_blocked_has_no_execution_delta() {
        let changed = super::execution_delta(
            "executed",
            "real_api:local_memory_consolidated",
            "memory_facts:1",
            "memory_facts:2",
        );
        let unchanged = super::execution_delta(
            "executed",
            "real_api:capability_pressure_observed",
            "free_energy:1.000",
            "free_energy:1.000",
        );
        let blocked = super::execution_delta(
            "blocked",
            "remote_network_remains_disabled",
            "remote_network:gated",
            "remote_network:gated",
        );

        assert_eq!(changed, "memory_facts:1->memory_facts:2");
        assert_eq!(
            unchanged,
            "real_api:capability_pressure_observed;observable_unchanged:free_energy:1.000"
        );
        assert!(!unchanged.contains("api_call:pending->executed"));
        assert_eq!(blocked, "not_executed");
    }

    #[test]
    fn stable_real_api_delta_names_effect_and_observable() {
        let delta = super::execution_delta(
            "executed",
            "real_api:state_paths_verified",
            "persistence:saves:0 loads:0 last_result_len:0",
            "persistence:saves:0 loads:0 last_result_len:0",
        );

        assert_eq!(
            delta,
            "real_api:state_paths_verified;observable_unchanged:persistence:saves:0 loads:0 last_result_len:0"
        );
        assert!(!delta.contains("api_call:pending->executed"));
        assert!(!delta.contains("free_energy"));
    }

    #[test]
    fn recovery_repairs_next_action_ids_after_load() {
        let mut state = super::new_individual_state();
        let organ = super::individual_organ_mut(&mut state, "legacy_memory");
        organ.next_action_id = 1;
        organ.actions.push_back(super::OrganAction {
            id: 41,
            organ: "legacy_memory".to_string(),
            kind: "consolidate_local_memory".to_string(),
            status: "executed".to_string(),
            reason: "real_api_executed".to_string(),
            delta: "memory_facts:1->memory_facts:2".to_string(),
        });

        let summary = super::recovery_summary(&mut state);

        assert_eq!(summary.repaired_next_ids, 1);
        assert_eq!(
            super::individual_organ_mut(&mut state, "legacy_memory").next_action_id,
            42
        );
    }
}
