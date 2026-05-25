#![allow(dead_code)]
//! EDEN GARM — Entry point
//!
//! Grafo Autopoietico Reflectivo Multi-escala.
//! Sin tick() maestro. Sin shell aparte.

#[path = "paradigms/mod.rs"]
pub mod paradigms;

#[path = "eden_v12/mod.rs"]
mod eden_v12;

#[path = "eden_garm/mod.rs"]
mod eden_garm;

use eden_garm::node::GARMNode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  EDEN GARM — Grafo Autopoietico Reflectivo Multi-escala        ║");
    println!("║                                                              ║");
    println!("║  Arquitectura: Nodos autopoieticos + FEP + 3 escalas        ║");
    println!("║  Sin tick() maestro. Sin shell aparte.                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let mut graph = eden_garm::HyperGraph::new();

    // Motor V12Engine compartido entre TODOS los nodos GARM (unificacion total)
    let shared_engine: Arc<Mutex<eden_v12::V12Engine>> = Arc::new(Mutex::new(eden_v12::V12Engine::new_fast()));

    // Helper: register all capabilities as individual nodes
    let mut next_id = 0usize;
    let mut cap_ids: Vec<(usize, eden_garm::nodes::capability::CapabilityId)> = Vec::new();

    let caps = vec![
        eden_garm::nodes::capability::CapabilityId::RecurrentState,
        eden_garm::nodes::capability::CapabilityId::Homeostasis,
        eden_garm::nodes::capability::CapabilityId::CorpusProcessing,
        eden_garm::nodes::capability::CapabilityId::BigTransformerTrain,
        eden_garm::nodes::capability::CapabilityId::BigTransformerGenerate,
        eden_garm::nodes::capability::CapabilityId::SemanticsObserve,
        eden_garm::nodes::capability::CapabilityId::SyntaxParse,
        eden_garm::nodes::capability::CapabilityId::SceneParser,
        eden_garm::nodes::capability::CapabilityId::Morphogenesis,
        eden_garm::nodes::capability::CapabilityId::Causality,
        eden_garm::nodes::capability::CapabilityId::Grounding,
        eden_garm::nodes::capability::CapabilityId::Physics,
        eden_garm::nodes::capability::CapabilityId::Hippocampus,
        eden_garm::nodes::capability::CapabilityId::Mood,
        eden_garm::nodes::capability::CapabilityId::Motivation,
        eden_garm::nodes::capability::CapabilityId::GoalStack,
        eden_garm::nodes::capability::CapabilityId::Planner,
        eden_garm::nodes::capability::CapabilityId::Security,
        eden_garm::nodes::capability::CapabilityId::Neural,
        eden_garm::nodes::capability::CapabilityId::TransformerSmall,
        eden_garm::nodes::capability::CapabilityId::BusPredictor,
        eden_garm::nodes::capability::CapabilityId::WorldModelNN,
        eden_garm::nodes::capability::CapabilityId::MoE,
        eden_garm::nodes::capability::CapabilityId::HierarchicalAttention,
        eden_garm::nodes::capability::CapabilityId::ContinualLearning,
        eden_garm::nodes::capability::CapabilityId::MetaLearning,
        eden_garm::nodes::capability::CapabilityId::EWC,
        eden_garm::nodes::capability::CapabilityId::MDLPruner,
        eden_garm::nodes::capability::CapabilityId::EmotionalModulation,
        eden_garm::nodes::capability::CapabilityId::DNC,
        eden_garm::nodes::capability::CapabilityId::ActiveInference,
        eden_garm::nodes::capability::CapabilityId::Body,
        eden_garm::nodes::capability::CapabilityId::TemporalHierarchy,
        eden_garm::nodes::capability::CapabilityId::SelfModification,
        eden_garm::nodes::capability::CapabilityId::LogicReasoning,
        eden_garm::nodes::capability::CapabilityId::ConstitutionalSafety,
        eden_garm::nodes::capability::CapabilityId::Phenomenology,
        eden_garm::nodes::capability::CapabilityId::EconomicAgent,
        eden_garm::nodes::capability::CapabilityId::RewardOracle,
        eden_garm::nodes::capability::CapabilityId::BPTT,
        eden_garm::nodes::capability::CapabilityId::CorpusMassive,
        eden_garm::nodes::capability::CapabilityId::GenController,
        eden_garm::nodes::capability::CapabilityId::SocialComplex,
        eden_garm::nodes::capability::CapabilityId::MultiAgent,
        eden_garm::nodes::capability::CapabilityId::Swarm,
        eden_garm::nodes::capability::CapabilityId::Metacognition,
        eden_garm::nodes::capability::CapabilityId::SelfAwareness,
        eden_garm::nodes::capability::CapabilityId::IntentionHierarchy,
        eden_garm::nodes::capability::CapabilityId::Exploration,
        eden_garm::nodes::capability::CapabilityId::Gate,
        eden_garm::nodes::capability::CapabilityId::Evidence,
        eden_garm::nodes::capability::CapabilityId::Surprise,
        eden_garm::nodes::capability::CapabilityId::Epistemic,
        eden_garm::nodes::capability::CapabilityId::Circadian,
        eden_garm::nodes::capability::CapabilityId::Critic,
        eden_garm::nodes::capability::CapabilityId::WorkingMemory,
        eden_garm::nodes::capability::CapabilityId::ProgramInduction,
        eden_garm::nodes::capability::CapabilityId::Counterfactual,
        eden_garm::nodes::capability::CapabilityId::Analogy,
        eden_garm::nodes::capability::CapabilityId::Composition,
        eden_garm::nodes::capability::CapabilityId::Autonomy,
        eden_garm::nodes::capability::CapabilityId::GoalExecutor,
        eden_garm::nodes::capability::CapabilityId::LanguageGen,
        eden_garm::nodes::capability::CapabilityId::SyntheticVision,
        eden_garm::nodes::capability::CapabilityId::PredictiveLoop,
        eden_garm::nodes::capability::CapabilityId::Curriculum,
        eden_garm::nodes::capability::CapabilityId::MemoryClustering,
        eden_garm::nodes::capability::CapabilityId::Gridworld,
        eden_garm::nodes::capability::CapabilityId::AgentMesh,
        eden_garm::nodes::capability::CapabilityId::Compositional,
        eden_garm::nodes::capability::CapabilityId::NeuralExtractors,
        eden_garm::nodes::capability::CapabilityId::World3D,
        eden_garm::nodes::capability::CapabilityId::PluginSystem,
        eden_garm::nodes::capability::CapabilityId::UnifiedPerception,
        eden_garm::nodes::capability::CapabilityId::UnifiedBus,
        eden_garm::nodes::capability::CapabilityId::ArchitectureModel,
        eden_garm::nodes::capability::CapabilityId::AutoDebug,
        eden_garm::nodes::capability::CapabilityId::OpenEndedness,
        eden_garm::nodes::capability::CapabilityId::Evolution,
        eden_garm::nodes::capability::CapabilityId::SelfModel,
        eden_garm::nodes::capability::CapabilityId::Temporal,
        eden_garm::nodes::capability::CapabilityId::TheoryOfMind,
        eden_garm::nodes::capability::CapabilityId::InternalLanguage,
        eden_garm::nodes::capability::CapabilityId::Perception,
        eden_garm::nodes::capability::CapabilityId::Sandbox,
        eden_garm::nodes::capability::CapabilityId::ComputerUse,
        eden_garm::nodes::capability::CapabilityId::ToolCalling,
        eden_garm::nodes::capability::CapabilityId::McpClient,
        eden_garm::nodes::capability::CapabilityId::Voice,
        eden_garm::nodes::capability::CapabilityId::Vision,
        eden_garm::nodes::capability::CapabilityId::NaturalLanguage,
    ];

    for cap in &caps {
        let id = next_id;
        next_id += 1;
        graph.add_node(Box::new(eden_garm::nodes::capability::CapabilityNode::new(id, *cap, Arc::clone(&shared_engine))));
        cap_ids.push((id, *cap));
    }

    // Coordinator: minimal clock. All capability logic lives in CapabilityNode.
    let coord_id = next_id; next_id += 1;
    graph.add_node(Box::new(eden_garm::nodes::coordinator::CoordinatorNode::new(coord_id, Arc::clone(&shared_engine))));

    // Specialized GARM nodes (not V12 capabilities)
    let human_id = next_id; next_id += 1;
    graph.add_node(Box::new(eden_garm::nodes::human_interface::HumanInterfaceNode::new(human_id, true)));

    let meta_id = next_id; next_id += 1;
    graph.add_node(Box::new(eden_garm::nodes::meta_architect::MetaArchitectNode::new(meta_id)));

    let fast_id = next_id; next_id += 1;
    graph.add_node(Box::new(eden_garm::nodes::fast_reflexes::FastReflexesNode::new(fast_id)));

    let bench_id = next_id;
    graph.add_node(Box::new(eden_garm::nodes::benchmark::BenchmarkNode::new(bench_id)));

    // Conexiones: cada capability conectada a Benchmark + MetaArchitect
    let n_caps = cap_ids.len();
    for &(cid, _) in &cap_ids {
        graph.add_edge(cid, bench_id, 0.05); // benchmark observa
        graph.add_edge(meta_id, cid, 0.1); // arquitecto puede modificar
    }
    // HumanInterface conectado a todos los deliberativos
    for &(cid, cap) in &cap_ids {
        if !matches!(cap,
            eden_garm::nodes::capability::CapabilityId::Security |
            eden_garm::nodes::capability::CapabilityId::Gate |
            eden_garm::nodes::capability::CapabilityId::Surprise
        ) {
            graph.add_edge(human_id, cid, 0.1);
        }
    }
    // FastReflexes conectado a Security + Gate
    for &(cid, cap) in &cap_ids {
        if matches!(cap,
            eden_garm::nodes::capability::CapabilityId::Security |
            eden_garm::nodes::capability::CapabilityId::Gate |
            eden_garm::nodes::capability::CapabilityId::Surprise
        ) {
            graph.add_edge(fast_id, cid, 0.3);
        }
    }

    // Coordinator connected to all capabilities as pulse clock.
    for &(cid, _) in &cap_ids {
        graph.add_edge(cid, coord_id, 0.1);
        graph.add_edge(coord_id, cid, 0.05);
    }

    println!("[GARM] HyperGraph initialized | nodes={} | edges={} | capabilities={}",
        graph.alive_node_count(),
        cap_ids.len() * 4 + n_caps,
        n_caps
    );
    println!("[GARM] Commands: tick | estado | auto N | save | load | quit");
    println!();

    // Thread para stdin no-bloqueante
    let stdin_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let stdin_q_clone = Arc::clone(&stdin_queue);
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lines() {
            if let Ok(l) = line {
                if let Ok(mut q) = stdin_q_clone.lock() {
                    q.push(l);
                }
            }
        }
    });

    let start = Instant::now();
    let mut last_report = Instant::now();
    let mut running = true;

    while running {
        let dt = 0.1;
        let elapsed = start.elapsed().as_secs_f64();

        // Procesar comandos humanos
        let commands: Vec<String> = {
            if let Ok(mut q) = stdin_queue.lock() {
                std::mem::take(&mut *q)
            } else {
                Vec::new()
            }
        };

        for cmd in commands {
            let trimmed = cmd.trim();
            match trimmed {
                "quit" | "exit" => {
                    println!("[GARM] Shutdown requested.");
                    running = false;
                    break;
                }
                "tick" => {
                    // Forzar pulso inmediato con sensor de comando
                    graph.inject_sensor(vec![1.0; 8]);
                    graph.pulse(dt);
                }
                "estado" | "status" => {
                    let guard = shared_engine.lock().unwrap();
                    println!("{}", guard.status_summary());
                    drop(guard);
                }
                "save" => {
                    let mut saved = false;
                    {
                        let guard = shared_engine.lock().unwrap();
                        if let Err(e) = guard.save_state("/tmp/eden_garm_engine_state.json") {
                            println!("[SAVE] GARM capabilities error: {}", e);
                        } else {
                            saved = true;
                        }
                        drop(guard);
                    }
                    match graph.save_state("/tmp/eden_garm_state.json") {
                            Ok(_) => println!("[SAVE] GARM state saved.{}", if saved { " Capability state also saved." } else { "" }),
                        Err(e) => println!("[SAVE] GARM error: {}", e),
                    }
                }
                "load" => {
                    let mut loaded_capabilities = false;
                    {
                        let mut guard = shared_engine.lock().unwrap();
                        match guard.load_state("/tmp/eden_garm_engine_state.json") {
                            Ok(_) => { loaded_capabilities = true; }
                            Err(e) => println!("[LOAD] GARM capabilities error: {}", e),
                        }
                        drop(guard);
                    }
                    match graph.load_state("/tmp/eden_garm_state.json") {
                        Ok(_) => println!("[LOAD] GARM state loaded.{}", if loaded_capabilities { " Capability state also loaded." } else { "" }),
                        Err(e) => println!("[LOAD] GARM error: {}", e),
                    }
                }
                _ => {
                    if trimmed.starts_with("auto ") {
                        if let Some(n_str) = trimmed.split_whitespace().nth(1) {
                            if let Ok(n) = n_str.parse::<usize>() {
                                println!("[AUTO] Running {} pulses...", n);
                                let auto_start = Instant::now();
                                for i in 0..n {
                                    graph.inject_sensor(vec![0.0; 8]);
                                    graph.pulse(dt);
                                    if (i + 1) % 50 == 0 {
                                        let guard = shared_engine.lock().unwrap();
                                        println!(
                                            "  pulse {:4} | v12_ticks={} | parse={:.3} | reward_ema={:.3} | energy={:.1}",
                                            i + 1, guard.state.tick_count,
                                            guard.gen_metrics.parse_rate(),
                                            guard.gen_metrics.reward_ema,
                                            guard.metabolism.energy
                                        );
                                        drop(guard);
                                    }
                                }
                                let auto_elapsed = auto_start.elapsed().as_secs_f64();
                                println!("[AUTO] Done | {} pulses in {:.1}s | pps={:.2}", n, auto_elapsed, n as f64 / auto_elapsed);
                            }
                        }
                    } else {
                        println!("Comando no reconocido: '{}'", trimmed);
                        println!("Usa: tick | estado | auto N | save | load | quit");
                    }
                }
            }
        }

        if !running { break; }

        // Pulso autonomo del grafo (si no hay comando humano bloqueando)
        graph.inject_sensor(vec![0.0; 8]);
        graph.pulse(dt);

        // Log periodico cada 5 segundos
        if last_report.elapsed().as_secs() >= 5 {
            {
                let guard = shared_engine.lock().unwrap();
                println!(
                    "[GARM] t={:.0}s | alive={} | v12_ticks={} | parse={:.3} | reward={:.3} | energy={:.1}",
                    elapsed, graph.alive_node_count(), guard.state.tick_count,
                    guard.gen_metrics.parse_rate(), guard.gen_metrics.reward_ema,
                    guard.metabolism.energy
                );
                drop(guard);
            }
            if let Some(meta_node) = graph.nodes.get(meta_id) {
                if let Some(meta) = meta_node.as_any().downcast_ref::<eden_garm::nodes::meta_architect::MetaArchitectNode>() {
                    println!(
                        "[META] action='{}' | proposals={}/{} | fe={:.2}",
                        meta.last_action(), meta.proposals_applied(), meta.proposals_generated(), meta.free_energy()
                    );
                }
            }
            last_report = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    println!("[GARM] Session ended.");
}
