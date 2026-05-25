use crate::eden_garm::capabilities::GarmCapabilityState;
use crate::eden_garm::hypergraph::HyperGraph;
use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::eden_garm::nodes::api_server::ApiServerNode;
use crate::eden_garm::nodes::benchmark::BenchmarkNode;
use crate::eden_garm::nodes::campo_tension::CampoTensionNode;
use crate::eden_garm::nodes::command_router::CommandRouterNode;
use crate::eden_garm::nodes::conscious_graph_regulator::ConsciousGraphRegulatorNode;
use crate::eden_garm::nodes::context_augmentation::ContextAugmentationNode;
use crate::eden_garm::nodes::coordinator::CoordinatorNode;
use crate::eden_garm::nodes::daemon::DaemonNode;
use crate::eden_garm::nodes::fast_reflexes::FastReflexesNode;
use crate::eden_garm::nodes::help::HelpNode;
use crate::eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode;
use crate::eden_garm::nodes::human_interface::HumanInterfaceNode;
use crate::eden_garm::nodes::legacy_cognition::LegacyCognitionNode;
use crate::eden_garm::nodes::legacy_evolution::LegacyEvolutionNode;
use crate::eden_garm::nodes::legacy_history::LegacyHistoryNode;
use crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode;
use crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode;
use crate::eden_garm::nodes::legacy_runtime_extensions::{
    AutoconsumoNode, EcoSystemNode, LegacyCrawlerNode, ParadigmHubNode, RebirthMeltraceNode,
    VenadoCompatibilityNode,
};
use crate::eden_garm::nodes::meta_architect::MetaArchitectNode;
use crate::eden_garm::nodes::observatory::ObservatoryNode;
use crate::eden_garm::nodes::telemetry::TelemetryNode;
use crate::eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode;
use crate::eden_garm::state_paths;
use std::sync::{Arc, Mutex};

pub struct PersistenceNode {
    id: usize,
    saves: u64,
    loads: u64,
    path_checks: u64,
    last_result: String,
    internal_fe: f32,
}

impl PersistenceNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            saves: 0,
            loads: 0,
            path_checks: 0,
            last_result: String::new(),
            internal_fe: 1.0,
        }
    }

    pub fn save_all(
        &mut self,
        graph: &HyperGraph,
        engine: &Arc<Mutex<GarmCapabilityState>>,
    ) -> String {
        let result = save_all(graph, engine);
        self.saves += 1;
        self.last_result = result.clone();
        result
    }

    pub fn load_all(
        &mut self,
        graph: &mut HyperGraph,
        engine: &Arc<Mutex<GarmCapabilityState>>,
    ) -> String {
        let result = load_all(graph, engine);
        self.loads += 1;
        self.last_result = result.clone();
        result
    }

    pub fn verify_state_paths(&mut self) -> Result<(), String> {
        state_paths::ensure_state_dir()?;
        self.path_checks += 1;
        self.last_result = "state_paths_verified".to_string();
        Ok(())
    }

    pub fn persistence_snapshot(&self) -> String {
        format!(
            "persistence:saves:{} loads:{} path_checks:{} last_result_len:{}",
            self.saves,
            self.loads,
            self.path_checks,
            self.last_result.len()
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "saves": self.saves,
            "loads": self.loads,
            "path_checks": self.path_checks,
            "last_result": self.last_result,
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
        self.saves = snapshot.get("saves").and_then(|v| v.as_u64()).unwrap_or(0);
        self.loads = snapshot.get("loads").and_then(|v| v.as_u64()).unwrap_or(0);
        self.path_checks = snapshot
            .get("path_checks")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.last_result = snapshot
            .get("last_result")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.internal_fe = snapshot
            .get("internal_fe")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        Ok(())
    }
}

pub fn save_all(graph: &HyperGraph, engine: &Arc<Mutex<GarmCapabilityState>>) -> String {
    if let Err(e) = state_paths::ensure_state_dir() {
        return format!("[SAVE] GARM state dir error: {}", e);
    }
    let capability_result = {
        let guard = engine.lock().unwrap();
        guard.save_state(&state_paths::capability_state_path())
    };
    let graph_result = graph.save_state(&state_paths::graph_state_path());
    match (graph_result, capability_result) {
        (Ok(_), Ok(_)) => "[SAVE] GARM state saved. Capability state also saved.".to_string(),
        (Err(e), Ok(_)) => format!("[SAVE] GARM error: {}", e),
        (Ok(_), Err(e)) => format!("[SAVE] GARM capabilities error: {}", e),
        (Err(ge), Err(ce)) => format!("[SAVE] GARM error: {}; capabilities error: {}", ge, ce),
    }
}

pub fn load_all(graph: &mut HyperGraph, engine: &Arc<Mutex<GarmCapabilityState>>) -> String {
    let capability_result = {
        let mut guard = engine.lock().unwrap();
        guard.load_state(&state_paths::capability_state_path())
    };
    let graph_result = graph.load_state(&state_paths::graph_state_path());
    match (graph_result, capability_result) {
        (Ok(_), Ok(_)) => "[LOAD] GARM state loaded. Capability state also loaded.".to_string(),
        (Err(e), Ok(_)) => format!("[LOAD] GARM error: {}", e),
        (Ok(_), Err(e)) => format!("[LOAD] GARM capabilities error: {}", e),
        (Err(ge), Err(ce)) => format!("[LOAD] GARM error: {}; capabilities error: {}", ge, ce),
    }
}

pub fn save_all_with_legacy_memory(
    graph: &HyperGraph,
    engine: &Arc<Mutex<GarmCapabilityState>>,
    legacy_memory_id: usize,
) -> String {
    let base = save_all(graph, engine);
    let memory_result = graph
        .nodes
        .get(legacy_memory_id)
        .and_then(|node| node.as_any().downcast_ref::<LegacyMemoryNode>())
        .ok_or_else(|| "legacy_memory node not found".to_string())
        .and_then(|memory| memory.save_state(&state_paths::legacy_memory_state_path()));
    match memory_result {
        Ok(_) => format!("{} Legacy memory also saved.", base),
        Err(e) => format!("{} Legacy memory error: {}", base, e),
    }
}

pub fn save_all_with_legacy_nodes(
    graph: &HyperGraph,
    engine: &Arc<Mutex<GarmCapabilityState>>,
    legacy_memory_id: usize,
    legacy_history_id: usize,
    observatory_id: usize,
    legacy_evolution_id: usize,
) -> String {
    let base = save_all_with_legacy_memory(graph, engine, legacy_memory_id);
    let history_result = graph
        .nodes
        .get(legacy_history_id)
        .and_then(|node| node.as_any().downcast_ref::<LegacyHistoryNode>())
        .ok_or_else(|| "legacy_history node not found".to_string())
        .and_then(|history| history.save_state(&state_paths::legacy_history_state_path()));
    let observatory_result = graph
        .nodes
        .get(observatory_id)
        .and_then(|node| node.as_any().downcast_ref::<ObservatoryNode>())
        .ok_or_else(|| "observatory node not found".to_string())
        .and_then(|observatory| observatory.save_state(&state_paths::observatory_state_path()));
    let evolution_result = graph
        .nodes
        .get(legacy_evolution_id)
        .and_then(|node| node.as_any().downcast_ref::<LegacyEvolutionNode>())
        .ok_or_else(|| "legacy_evolution node not found".to_string())
        .and_then(|evolution| evolution.save_state(&state_paths::legacy_evolution_state_path()));
    let cognition_result = graph
        .nodes
        .iter()
        .find(|node| node.name() == "legacy_cognition")
        .and_then(|node| node.as_any().downcast_ref::<LegacyCognitionNode>())
        .map(|cognition| cognition.save_state(&state_paths::legacy_cognition_state_path()))
        .unwrap_or(Ok(()));
    let campo_tension_result = save_node_by_name::<CampoTensionNode>(
        graph,
        "campo_tension",
        state_paths::campo_tension_state_path(),
        |node, path| node.save_state(&path),
    );
    let knowledge_graph_result = save_node_by_name::<LegacyKnowledgeGraphNode>(
        graph,
        "legacy_knowledge_graph",
        state_paths::legacy_knowledge_graph_state_path(),
        |node, path| node.save_state(&path),
    );
    let autoconsumo_result = save_node_by_name::<AutoconsumoNode>(
        graph,
        "legacy_autoconsumo",
        state_paths::legacy_autoconsumo_state_path(),
        |node, path| node.save_state(&path),
    );
    let venado_result = save_node_by_name::<VenadoCompatibilityNode>(
        graph,
        "legacy_venado",
        state_paths::legacy_venado_state_path(),
        |node, path| node.save_state(&path),
    );
    let paradigm_result = save_node_by_name::<ParadigmHubNode>(
        graph,
        "legacy_paradigm_hub",
        state_paths::legacy_paradigm_hub_state_path(),
        |node, path| node.save_state(&path),
    );
    let ecosystem_result = save_node_by_name::<EcoSystemNode>(
        graph,
        "legacy_ecosystem",
        state_paths::legacy_ecosystem_state_path(),
        |node, path| node.save_state(&path),
    );
    let rebirth_result = save_node_by_name::<RebirthMeltraceNode>(
        graph,
        "legacy_rebirth_meltrace",
        state_paths::legacy_rebirth_meltrace_state_path(),
        |node, path| node.save_state(&path),
    );
    let crawler_result = save_node_by_name::<LegacyCrawlerNode>(
        graph,
        "legacy_crawler",
        state_paths::legacy_crawler_state_path(),
        |node, path| node.save_state(&path),
    );
    let conscious_graph_result = save_node_by_name::<ConsciousGraphRegulatorNode>(
        graph,
        "conscious_graph_regulator",
        state_paths::conscious_graph_regulator_state_path(),
        |node, path| node.save_state(&path),
    );
    let context_augmentation_result = save_node_by_name::<ContextAugmentationNode>(
        graph,
        "context_augmentation",
        state_paths::context_augmentation_state_path(),
        |node, path| node.save_state(&path),
    );
    let coordinator_result = save_node_by_name::<CoordinatorNode>(
        graph,
        "coordinator",
        state_paths::coordinator_state_path(),
        |node, path| node.save_state(&path),
    );
    let human_interface_result = save_node_by_name::<HumanInterfaceNode>(
        graph,
        "human_interface",
        state_paths::human_interface_state_path(),
        |node, path| node.save_state(&path),
    );
    let meta_architect_result = save_node_by_name::<MetaArchitectNode>(
        graph,
        "meta_architect",
        state_paths::meta_architect_state_path(),
        |node, path| node.save_state(&path),
    );
    let fast_reflexes_result = save_node_by_name::<FastReflexesNode>(
        graph,
        "fast_reflexes",
        state_paths::fast_reflexes_state_path(),
        |node, path| node.save_state(&path),
    );
    let benchmark_result = save_node_by_name::<BenchmarkNode>(
        graph,
        "benchmark",
        state_paths::benchmark_state_path(),
        |node, path| node.save_state(&path),
    );
    let command_router_result = save_node_by_name::<CommandRouterNode>(
        graph,
        "command_router",
        state_paths::command_router_state_path(),
        |node, path| node.save_state(&path),
    );
    let persistence_result = save_node_by_name::<PersistenceNode>(
        graph,
        "persistence",
        state_paths::persistence_state_path(),
        |node, path| node.save_state(&path),
    );
    let telemetry_result = save_node_by_name::<TelemetryNode>(
        graph,
        "telemetry",
        state_paths::telemetry_state_path(),
        |node, path| node.save_state(&path),
    );
    let api_server_result = save_node_by_name::<ApiServerNode>(
        graph,
        "api_server",
        state_paths::api_server_state_path(),
        |node, path| node.save_state(&path),
    );
    let daemon_result = save_node_by_name::<DaemonNode>(
        graph,
        "daemon",
        state_paths::daemon_state_path(),
        |node, path| node.save_state(&path),
    );
    let help_result = save_node_by_name::<HelpNode>(
        graph,
        "help",
        state_paths::help_state_path(),
        |node, path| node.save_state(&path),
    );
    let hrm_reasoner_result = save_node_by_name::<HierarchicalReasoningNode>(
        graph,
        "hrm_reasoner",
        state_paths::hrm_reasoner_state_path(),
        |node, path| node.save_state(&path),
    );
    let voice_synthesizer_result = save_node_by_name::<VoiceSynthesizerNode>(
        graph,
        "voice_synthesizer",
        state_paths::voice_synthesizer_state_path(),
        |node, path| node.save_state(&path),
    );
    append_results(
        base,
        &[
            ("Legacy history", history_result),
            ("Observatory", observatory_result),
            ("Legacy evolution", evolution_result),
            ("Legacy cognition", cognition_result),
            ("Campo tension", campo_tension_result),
            ("Legacy knowledge graph", knowledge_graph_result),
            ("Legacy autoconsumo", autoconsumo_result),
            ("Legacy venado", venado_result),
            ("Legacy paradigm hub", paradigm_result),
            ("Legacy ecosystem", ecosystem_result),
            ("Legacy rebirth meltrace", rebirth_result),
            ("Legacy crawler", crawler_result),
            ("Conscious graph regulator", conscious_graph_result),
            ("Context augmentation", context_augmentation_result),
            ("Coordinator", coordinator_result),
            ("Human interface", human_interface_result),
            ("Meta architect", meta_architect_result),
            ("Fast reflexes", fast_reflexes_result),
            ("Benchmark", benchmark_result),
            ("Command router", command_router_result),
            ("Persistence", persistence_result),
            ("Telemetry", telemetry_result),
            ("API server", api_server_result),
            ("Daemon", daemon_result),
            ("Help", help_result),
            ("HRM reasoner", hrm_reasoner_result),
            ("Voice synthesizer", voice_synthesizer_result),
        ],
        "saved",
    )
}

pub fn load_all_with_legacy_memory(
    graph: &mut HyperGraph,
    engine: &Arc<Mutex<GarmCapabilityState>>,
) -> String {
    let base = load_all(graph, engine);
    let memory_result = graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == "legacy_memory")
        .and_then(|node| node.as_any_mut().downcast_mut::<LegacyMemoryNode>())
        .ok_or_else(|| "legacy_memory node not found".to_string())
        .and_then(|memory| {
            memory
                .load_state(&state_paths::legacy_memory_state_path())
                .or_else(|json_error| {
                    if std::fs::metadata(state_paths::legacy_session_import_path()).is_ok() {
                        memory
                            .import_eden_session(&state_paths::legacy_session_import_path())
                            .map(|_| ())
                    } else {
                        Err(json_error)
                    }
                })
        });
    match memory_result {
        Ok(_) => format!("{} Legacy memory also loaded.", base),
        Err(e) => format!("{} Legacy memory error: {}", base, e),
    }
}

pub fn load_all_with_legacy_nodes(
    graph: &mut HyperGraph,
    engine: &Arc<Mutex<GarmCapabilityState>>,
) -> String {
    let base = load_all_with_legacy_memory(graph, engine);
    let history_result = graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == "legacy_history")
        .and_then(|node| node.as_any_mut().downcast_mut::<LegacyHistoryNode>())
        .ok_or_else(|| "legacy_history node not found".to_string())
        .and_then(|history| history.load_state(&state_paths::legacy_history_state_path()));
    let observatory_result = graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == "observatory")
        .and_then(|node| node.as_any_mut().downcast_mut::<ObservatoryNode>())
        .ok_or_else(|| "observatory node not found".to_string())
        .and_then(|observatory| observatory.load_state(&state_paths::observatory_state_path()));
    let evolution_result = graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == "legacy_evolution")
        .and_then(|node| node.as_any_mut().downcast_mut::<LegacyEvolutionNode>())
        .ok_or_else(|| "legacy_evolution node not found".to_string())
        .and_then(|evolution| evolution.load_state(&state_paths::legacy_evolution_state_path()));
    let cognition_result = graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == "legacy_cognition")
        .and_then(|node| node.as_any_mut().downcast_mut::<LegacyCognitionNode>())
        .map(|cognition| cognition.load_state(&state_paths::legacy_cognition_state_path()))
        .unwrap_or(Ok(()));
    let campo_tension_result = load_node_by_name::<CampoTensionNode>(
        graph,
        "campo_tension",
        state_paths::campo_tension_state_path(),
        |node, path| node.load_state(&path),
    );
    let knowledge_graph_result = load_node_by_name::<LegacyKnowledgeGraphNode>(
        graph,
        "legacy_knowledge_graph",
        state_paths::legacy_knowledge_graph_state_path(),
        |node, path| node.load_state(&path),
    );
    let autoconsumo_result = load_node_by_name::<AutoconsumoNode>(
        graph,
        "legacy_autoconsumo",
        state_paths::legacy_autoconsumo_state_path(),
        |node, path| node.load_state(&path),
    );
    let venado_result = load_node_by_name::<VenadoCompatibilityNode>(
        graph,
        "legacy_venado",
        state_paths::legacy_venado_state_path(),
        |node, path| node.load_state(&path),
    );
    let paradigm_result = load_node_by_name::<ParadigmHubNode>(
        graph,
        "legacy_paradigm_hub",
        state_paths::legacy_paradigm_hub_state_path(),
        |node, path| node.load_state(&path),
    );
    let ecosystem_result = load_node_by_name::<EcoSystemNode>(
        graph,
        "legacy_ecosystem",
        state_paths::legacy_ecosystem_state_path(),
        |node, path| node.load_state(&path),
    );
    let rebirth_result = load_node_by_name::<RebirthMeltraceNode>(
        graph,
        "legacy_rebirth_meltrace",
        state_paths::legacy_rebirth_meltrace_state_path(),
        |node, path| node.load_state(&path),
    );
    let crawler_result = load_node_by_name::<LegacyCrawlerNode>(
        graph,
        "legacy_crawler",
        state_paths::legacy_crawler_state_path(),
        |node, path| node.load_state(&path),
    );
    let conscious_graph_result = load_node_by_name::<ConsciousGraphRegulatorNode>(
        graph,
        "conscious_graph_regulator",
        state_paths::conscious_graph_regulator_state_path(),
        |node, path| node.load_state(&path),
    );
    let context_augmentation_result = load_node_by_name::<ContextAugmentationNode>(
        graph,
        "context_augmentation",
        state_paths::context_augmentation_state_path(),
        |node, path| node.load_state(&path),
    );
    let coordinator_result = load_node_by_name::<CoordinatorNode>(
        graph,
        "coordinator",
        state_paths::coordinator_state_path(),
        |node, path| node.load_state(&path),
    );
    let human_interface_result = load_node_by_name::<HumanInterfaceNode>(
        graph,
        "human_interface",
        state_paths::human_interface_state_path(),
        |node, path| node.load_state(&path),
    );
    let meta_architect_result = load_node_by_name::<MetaArchitectNode>(
        graph,
        "meta_architect",
        state_paths::meta_architect_state_path(),
        |node, path| node.load_state(&path),
    );
    let fast_reflexes_result = load_node_by_name::<FastReflexesNode>(
        graph,
        "fast_reflexes",
        state_paths::fast_reflexes_state_path(),
        |node, path| node.load_state(&path),
    );
    let benchmark_result = load_node_by_name::<BenchmarkNode>(
        graph,
        "benchmark",
        state_paths::benchmark_state_path(),
        |node, path| node.load_state(&path),
    );
    let command_router_result = load_node_by_name::<CommandRouterNode>(
        graph,
        "command_router",
        state_paths::command_router_state_path(),
        |node, path| node.load_state(&path),
    );
    let persistence_result = load_node_by_name::<PersistenceNode>(
        graph,
        "persistence",
        state_paths::persistence_state_path(),
        |node, path| node.load_state(&path),
    );
    let telemetry_result = load_node_by_name::<TelemetryNode>(
        graph,
        "telemetry",
        state_paths::telemetry_state_path(),
        |node, path| node.load_state(&path),
    );
    let api_server_result = load_node_by_name::<ApiServerNode>(
        graph,
        "api_server",
        state_paths::api_server_state_path(),
        |node, path| node.load_state(&path),
    );
    let daemon_result = load_node_by_name::<DaemonNode>(
        graph,
        "daemon",
        state_paths::daemon_state_path(),
        |node, path| node.load_state(&path),
    );
    let help_result = load_node_by_name::<HelpNode>(
        graph,
        "help",
        state_paths::help_state_path(),
        |node, path| node.load_state(&path),
    );
    let hrm_reasoner_result = load_node_by_name::<HierarchicalReasoningNode>(
        graph,
        "hrm_reasoner",
        state_paths::hrm_reasoner_state_path(),
        |node, path| node.load_state(&path),
    );
    let voice_synthesizer_result = load_node_by_name::<VoiceSynthesizerNode>(
        graph,
        "voice_synthesizer",
        state_paths::voice_synthesizer_state_path(),
        |node, path| node.load_state(&path),
    );
    append_results(
        base,
        &[
            ("Legacy history", history_result),
            ("Observatory", observatory_result),
            ("Legacy evolution", evolution_result),
            ("Legacy cognition", cognition_result),
            ("Campo tension", campo_tension_result),
            ("Legacy knowledge graph", knowledge_graph_result),
            ("Legacy autoconsumo", autoconsumo_result),
            ("Legacy venado", venado_result),
            ("Legacy paradigm hub", paradigm_result),
            ("Legacy ecosystem", ecosystem_result),
            ("Legacy rebirth meltrace", rebirth_result),
            ("Legacy crawler", crawler_result),
            ("Conscious graph regulator", conscious_graph_result),
            ("Context augmentation", context_augmentation_result),
            ("Coordinator", coordinator_result),
            ("Human interface", human_interface_result),
            ("Meta architect", meta_architect_result),
            ("Fast reflexes", fast_reflexes_result),
            ("Benchmark", benchmark_result),
            ("Command router", command_router_result),
            ("Persistence", persistence_result),
            ("Telemetry", telemetry_result),
            ("API server", api_server_result),
            ("Daemon", daemon_result),
            ("Help", help_result),
            ("HRM reasoner", hrm_reasoner_result),
            ("Voice synthesizer", voice_synthesizer_result),
        ],
        "loaded",
    )
}

fn save_node_by_name<T: 'static>(
    graph: &HyperGraph,
    name: &str,
    path: String,
    save: impl FnOnce(&T, String) -> Result<(), String>,
) -> Result<(), String> {
    graph
        .nodes
        .iter()
        .find(|node| node.name() == name)
        .and_then(|node| node.as_any().downcast_ref::<T>())
        .map(|node| save(node, path))
        .unwrap_or(Ok(()))
}

fn load_node_by_name<T: 'static>(
    graph: &mut HyperGraph,
    name: &str,
    path: String,
    load: impl FnOnce(&mut T, String) -> Result<(), String>,
) -> Result<(), String> {
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    graph
        .nodes
        .iter_mut()
        .find(|node| node.name() == name)
        .and_then(|node| node.as_any_mut().downcast_mut::<T>())
        .map(|node| load(node, path))
        .unwrap_or(Ok(()))
}

fn append_results(base: String, results: &[(&str, Result<(), String>)], verb: &str) -> String {
    let mut out = base;
    for (name, result) in results {
        match result {
            Ok(_) => out.push_str(&format!(" {} also {}.", name, verb)),
            Err(e) => out.push_str(&format!(" {} error: {}", name, e)),
        }
    }
    out
}

impl GARMNode for PersistenceNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "persistence"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.saves as f32, self.loads as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.saves as f32, self.loads as f32])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eden_garm::hypergraph::HyperGraph;
    use crate::eden_garm::nodes::api_server::{
        ApiRuntimeMetrics, ApiServerNode, CommandResponseBus,
    };
    use crate::eden_garm::nodes::benchmark::BenchmarkNode;
    use crate::eden_garm::nodes::campo_tension::CampoTensionNode;
    use crate::eden_garm::nodes::command_router::CommandRouterNode;
    use crate::eden_garm::nodes::context_augmentation::ContextAugmentationNode;
    use crate::eden_garm::nodes::coordinator::CoordinatorNode;
    use crate::eden_garm::nodes::daemon::{DaemonConfig, DaemonNode};
    use crate::eden_garm::nodes::fast_reflexes::FastReflexesNode;
    use crate::eden_garm::nodes::help::HelpNode;
    use crate::eden_garm::nodes::hierarchical_reasoning::HierarchicalReasoningNode;
    use crate::eden_garm::nodes::human_interface::HumanInterfaceNode;
    use crate::eden_garm::nodes::legacy_cognition::LegacyCognitionNode;
    use crate::eden_garm::nodes::legacy_evolution::LegacyEvolutionNode;
    use crate::eden_garm::nodes::legacy_history::LegacyHistoryNode;
    use crate::eden_garm::nodes::legacy_knowledge_graph::LegacyKnowledgeGraphNode;
    use crate::eden_garm::nodes::legacy_memory::LegacyMemoryNode;
    use crate::eden_garm::nodes::legacy_runtime_extensions::{
        AutoconsumoNode, EcoSystemNode, LegacyCrawlerNode, ParadigmHubNode, RebirthMeltraceNode,
        VenadoCompatibilityNode,
    };
    use crate::eden_garm::nodes::meta_architect::MetaArchitectNode;
    use crate::eden_garm::nodes::observatory::ObservatoryNode;
    use crate::eden_garm::nodes::telemetry::TelemetryNode;
    use crate::eden_garm::nodes::voice_synthesizer::VoiceSynthesizerNode;
    use std::sync::{Arc, Mutex};

    #[test]
    fn saves_and_loads_full_state_dir_snapshot() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_persistence_e2e_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());

        let engine = Arc::new(Mutex::new(GarmCapabilityState::new_fast()));
        let mut graph = HyperGraph::new();

        let mut memory = LegacyMemoryNode::new(1);
        memory.remember("persisted alpha fact");
        let mut history = LegacyHistoryNode::new(2);
        history.record_command("remember persisted alpha fact");
        let mut observatory = ObservatoryNode::new(3);
        observatory.report(&engine.lock().unwrap(), 4, 1, 1, 7, true, false);
        let mut evolution = LegacyEvolutionNode::new(4);
        evolution.summarize(10, 1, 11, 90.0, 88.0);
        let mut cognition = LegacyCognitionNode::new(5);
        cognition.update_from_facts(
            &["cognitive_science integra atencion y memoria".to_string()],
            99,
        );
        cognition.record_exploration("cognitive_science", 0.7, true);
        let mut campo = CampoTensionNode::new(6);
        campo.calcular(20, 1, 50, 1, true, 0.0, -0.5, 50);
        let mut kg = LegacyKnowledgeGraphNode::new(7);
        kg.add_fact_from("curiosidad causa exploracion", "local_kb");
        let mut autoconsumo = AutoconsumoNode::new(8);
        autoconsumo.nutrirse();
        let mut venado = VenadoCompatibilityNode::new(9);
        venado
            .cristalizar("persisted", &[("k".to_string(), "v".to_string())])
            .unwrap();
        let mut paradigm = ParadigmHubNode::new(10);
        paradigm.active_select_page(&std::collections::HashMap::from([("ai".to_string(), 0.9)]));
        let mut ecosystem = EcoSystemNode::new(11);
        ecosystem.pulso_global(3);
        let mut rebirth = RebirthMeltraceNode::new(12);
        rebirth.rebirth(&["ancestor fact".to_string()]);
        let mut crawler = LegacyCrawlerNode::new(13);
        crawler.crawl_remote_blocked("https://example.invalid");
        let mut cag = ContextAugmentationNode::new(14);
        cag.build_context(
            "persisted alpha",
            42,
            &["persisted alpha fact".to_string()],
            &[("persisted alpha".to_string(), 1.0)],
            &[],
            &["cmd: remember persisted alpha fact".to_string()],
        );
        cag.build_context("persisted alpha", 43, &[], &[], &[], &[]);
        let mut coordinator = CoordinatorNode::new(15, Arc::clone(&engine));
        coordinator.observe_capability_pressure();
        let mut human = HumanInterfaceNode::new(16, false);
        human.inject_command("status".to_string());
        human.maintain_local_bridge();
        let mut meta = MetaArchitectNode::new(17);
        meta.review_without_mutation(12.0);
        let mut reflexes = FastReflexesNode::new(18);
        reflexes.local_reflex_probe(5.0);
        let mut benchmark = BenchmarkNode::new(19);
        benchmark.sample_runtime_cost(7.0);
        let mut router = CommandRouterNode::new(20);
        router.validate_surface();
        let mut persistence = PersistenceNode::new(21);
        persistence.verify_state_paths().unwrap();
        let mut telemetry = TelemetryNode::new(22);
        telemetry.refresh_snapshot(14, 42);
        let mut api_server = ApiServerNode::new(
            23,
            Arc::clone(&engine),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(CommandResponseBus::new()),
            Arc::new(ApiRuntimeMetrics::new(false)),
            0,
        );
        api_server.check_local_readiness();
        let mut daemon = DaemonNode::new(24, DaemonConfig::disabled());
        daemon.inspect_liveness();
        let mut help = HelpNode::new(25);
        help.help();
        let mut hrm = HierarchicalReasoningNode::new(26);
        hrm.reason(
            "persisted alpha",
            42,
            &["persisted alpha fact".to_string()],
            &[("persisted alpha causes continuity".to_string(), 0.9)],
            &[],
            &["cmd: remember persisted alpha fact".to_string()],
        );
        let mut voice = VoiceSynthesizerNode::new(27);
        voice.synthesize_text("persisted voice", 42);

        graph.add_node(Box::new(memory));
        graph.add_node(Box::new(history));
        graph.add_node(Box::new(observatory));
        graph.add_node(Box::new(evolution));
        graph.add_node(Box::new(cognition));
        graph.add_node(Box::new(campo));
        graph.add_node(Box::new(kg));
        graph.add_node(Box::new(autoconsumo));
        graph.add_node(Box::new(venado));
        graph.add_node(Box::new(paradigm));
        graph.add_node(Box::new(ecosystem));
        graph.add_node(Box::new(rebirth));
        graph.add_node(Box::new(crawler));
        graph.add_node(Box::new(cag));
        graph.add_node(Box::new(coordinator));
        graph.add_node(Box::new(human));
        graph.add_node(Box::new(meta));
        graph.add_node(Box::new(reflexes));
        graph.add_node(Box::new(benchmark));
        graph.add_node(Box::new(router));
        graph.add_node(Box::new(persistence));
        graph.add_node(Box::new(telemetry));
        graph.add_node(Box::new(api_server));
        graph.add_node(Box::new(daemon));
        graph.add_node(Box::new(help));
        graph.add_node(Box::new(hrm));
        graph.add_node(Box::new(voice));
        graph.add_edge(1, 2, 0.75);
        graph.global_tick = 42;

        let save_result = save_all_with_legacy_nodes(&graph, &engine, 1, 2, 3, 4);
        assert!(save_result.contains("GARM state saved"));
        assert!(std::fs::metadata(state_paths::graph_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::capability_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_memory_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_history_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::observatory_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_evolution_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_cognition_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::campo_tension_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_knowledge_graph_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_autoconsumo_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_venado_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_paradigm_hub_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_ecosystem_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_rebirth_meltrace_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::legacy_crawler_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::context_augmentation_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::coordinator_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::human_interface_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::meta_architect_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::fast_reflexes_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::benchmark_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::command_router_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::persistence_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::telemetry_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::api_server_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::daemon_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::help_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::hrm_reasoner_state_path()).is_ok());
        assert!(std::fs::metadata(state_paths::voice_synthesizer_state_path()).is_ok());

        let restored_engine = Arc::new(Mutex::new(GarmCapabilityState::new_fast()));
        let mut restored = HyperGraph::new();
        restored.add_node(Box::new(LegacyMemoryNode::new(1)));
        restored.add_node(Box::new(LegacyHistoryNode::new(2)));
        restored.add_node(Box::new(ObservatoryNode::new(3)));
        restored.add_node(Box::new(LegacyEvolutionNode::new(4)));
        restored.add_node(Box::new(LegacyCognitionNode::new(5)));
        restored.add_node(Box::new(CampoTensionNode::new(6)));
        restored.add_node(Box::new(LegacyKnowledgeGraphNode::new(7)));
        restored.add_node(Box::new(AutoconsumoNode::new(8)));
        restored.add_node(Box::new(VenadoCompatibilityNode::new(9)));
        restored.add_node(Box::new(ParadigmHubNode::new(10)));
        restored.add_node(Box::new(EcoSystemNode::new(11)));
        restored.add_node(Box::new(RebirthMeltraceNode::new(12)));
        restored.add_node(Box::new(LegacyCrawlerNode::new(13)));
        restored.add_node(Box::new(ContextAugmentationNode::new(14)));
        restored.add_node(Box::new(CoordinatorNode::new(
            15,
            Arc::clone(&restored_engine),
        )));
        restored.add_node(Box::new(HumanInterfaceNode::new(16, false)));
        restored.add_node(Box::new(MetaArchitectNode::new(17)));
        restored.add_node(Box::new(FastReflexesNode::new(18)));
        restored.add_node(Box::new(BenchmarkNode::new(19)));
        restored.add_node(Box::new(CommandRouterNode::new(20)));
        restored.add_node(Box::new(PersistenceNode::new(21)));
        restored.add_node(Box::new(TelemetryNode::new(22)));
        restored.add_node(Box::new(ApiServerNode::new(
            23,
            Arc::clone(&restored_engine),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(CommandResponseBus::new()),
            Arc::new(ApiRuntimeMetrics::new(false)),
            0,
        )));
        restored.add_node(Box::new(DaemonNode::new(24, DaemonConfig::disabled())));
        restored.add_node(Box::new(HelpNode::new(25)));
        restored.add_node(Box::new(HierarchicalReasoningNode::new(26)));
        restored.add_node(Box::new(VoiceSynthesizerNode::new(27)));

        let load_result = load_all_with_legacy_nodes(&mut restored, &restored_engine);
        assert!(load_result.contains("GARM state loaded"));
        assert_eq!(restored.global_tick, 42);
        assert_eq!(restored.edge_count(), 1);

        let restored_memory = restored.nodes[1]
            .as_any()
            .downcast_ref::<LegacyMemoryNode>()
            .unwrap();
        assert_eq!(
            restored_memory.search("alpha"),
            vec!["persisted alpha fact".to_string()]
        );

        let restored_history = restored.nodes[2]
            .as_any()
            .downcast_ref::<LegacyHistoryNode>()
            .unwrap();
        assert!(restored_history
            .report()
            .contains("remember persisted alpha fact"));

        let restored_observatory = restored.nodes[3]
            .as_any_mut()
            .downcast_mut::<ObservatoryNode>()
            .unwrap();
        let report =
            restored_observatory.report(&restored_engine.lock().unwrap(), 4, 1, 1, 8, true, true);
        assert!(report.contains("reports=2"));

        let restored_evolution = restored.nodes[4]
            .as_any_mut()
            .downcast_mut::<LegacyEvolutionNode>()
            .unwrap();
        let summary = restored_evolution.summarize(1, 42, 43, 88.0, 87.0);
        assert!(summary.contains("requests=2"));

        let restored_cognition = restored.nodes[5]
            .as_any()
            .downcast_ref::<LegacyCognitionNode>()
            .unwrap();
        assert!(restored_cognition.total_information_gain >= 0.7);

        let restored_campo = restored.nodes[6]
            .as_any()
            .downcast_ref::<CampoTensionNode>()
            .unwrap();
        assert!(restored_campo.tension() > 0.0);
        let restored_kg = restored.nodes[7]
            .as_any()
            .downcast_ref::<LegacyKnowledgeGraphNode>()
            .unwrap();
        assert!(restored_kg
            .temporal_query("curiosidad", "exploracion", 42)
            .is_some());
        let restored_venado = restored.nodes[9]
            .as_any_mut()
            .downcast_mut::<VenadoCompatibilityNode>()
            .unwrap();
        assert_eq!(
            restored_venado.descristalizar("persisted").unwrap(),
            vec![("k".to_string(), "v".to_string())]
        );
        assert!(restored.nodes[12]
            .as_any()
            .downcast_ref::<RebirthMeltraceNode>()
            .unwrap()
            .informe()
            .contains("rebirths=1"));
        assert!(restored.nodes[13]
            .as_any()
            .downcast_ref::<LegacyCrawlerNode>()
            .unwrap()
            .informe()
            .contains("blocked_remote=1"));
        let restored_cag = restored.nodes[14]
            .as_any()
            .downcast_ref::<ContextAugmentationNode>()
            .unwrap();
        let metrics = restored_cag.metrics();
        assert_eq!(metrics.cache_entries, 1);
        assert_eq!(metrics.hits, 1);
        assert!(restored.nodes[15]
            .as_any()
            .downcast_ref::<CoordinatorNode>()
            .unwrap()
            .autonomy_snapshot()
            .contains("observations:1"));
        assert!(restored.nodes[18]
            .as_any()
            .downcast_ref::<FastReflexesNode>()
            .unwrap()
            .reflex_snapshot()
            .contains("probes:1"));
        assert!(restored.nodes[21]
            .as_any()
            .downcast_ref::<PersistenceNode>()
            .unwrap()
            .persistence_snapshot()
            .contains("path_checks:1"));
        assert!(restored.nodes[23]
            .as_any()
            .downcast_ref::<ApiServerNode>()
            .unwrap()
            .readiness_snapshot()
            .contains("readiness_checks:1"));
        assert!(restored.nodes[26]
            .as_any()
            .downcast_ref::<HierarchicalReasoningNode>()
            .unwrap()
            .snapshot()
            .contains("plan_steps:4"));
        assert!(restored.nodes[27]
            .as_any()
            .downcast_ref::<VoiceSynthesizerNode>()
            .unwrap()
            .snapshot()
            .contains("requests:1"));

        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
    }
}
