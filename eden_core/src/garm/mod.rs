//! EDEN GARM — Grafo Autopoietico Reflectivo Multi-escala
//!
//! Arquitectura unificada de capacidades + interfaz humana.
//! - Ningun tick() maestro.
//! - Cada capability es un nodo autopoietico con metabolismo propio.
//! - El grafo se reconfigura solo via nodo MetaArchitect.
//! - HumanInterface es un nodo mas, no un shell aparte.

pub mod action_evidence;
pub mod architecture_advantage;
pub mod artifact_api;
pub mod capabilities;
pub mod capability_reality_eval;
pub mod capability_registry;
pub mod client_sdk;
pub mod cognitive_architecture;
pub mod eden_locus_layer;
pub mod eden_operator_forge;
pub mod embodied_grounding;
pub mod external_ecosystem;
pub mod external_validation;
pub mod fep;
pub mod frontier_architecture_layers;
pub mod gewc_operational_benchmark;
pub mod global_executive_workspace;
pub mod graph_builder;
pub mod hypergraph;
pub mod integration_governance;
pub mod legacy_migration;
pub mod memory_eval;
pub mod model_runtime;
pub mod neural_architecture;
pub mod node;
pub mod nodes;
pub mod operational_api;
pub mod operational_runtime;
pub mod paradigm_architecture;
pub mod paradise_worldcell;
pub mod praxis_nexus;
pub mod reproducible_package;
pub mod runtime;
pub mod runtime_spine;
pub mod runtime_state_api;
pub mod schema_registry;
pub mod self_improvement_architecture;
pub mod sovereign_cognition;
pub mod state_paths;
pub mod symbolic_architecture;
pub mod training_evidence;
pub mod world_eval;

// Full historical REPL source is physically kept inside GARM as migration
// material. It is not compiled as a standalone shell; GARM is the only runtime.
#[cfg(any())]
pub mod legacy_repl;

pub use hypergraph::HyperGraph;

pub fn main_entry() {
    if !std::env::args().any(|arg| arg == "--mcp") {
        print_startup_banner();
    }

    runtime::GarmRuntime::from_args().run();
}

pub fn print_startup_banner() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  EDEN GARM — Grafo Autopoietico Reflectivo Multi-escala        ║");
    println!("║                                                              ║");
    println!("║  Arquitectura: Nodos autopoieticos + FEP + 3 escalas        ║");
    println!("║  Sin tick() maestro. Sin shell aparte.                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
