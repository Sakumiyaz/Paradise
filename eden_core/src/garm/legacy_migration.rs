pub struct LegacyMigrationItem {
    pub legacy_area: &'static str,
    pub garm_target: &'static str,
    pub status: &'static str,
}

pub fn migration_items() -> Vec<LegacyMigrationItem> {
    vec![
        LegacyMigrationItem {
            legacy_area: "interactive command routing",
            garm_target: "CommandRouterNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "save/load",
            garm_target: "PersistenceNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "status/metrics",
            garm_target: "TelemetryNode + ApiRuntimeMetrics",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "local HTTP API",
            garm_target: "ApiServerNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "daemon pid/log",
            garm_target: "DaemonNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "eidetic remember/recall",
            garm_target: "LegacyMemoryNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "basic memory reasoning",
            garm_target: "LegacyReasonNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "what-is/why/tell-me reasoning",
            garm_target: "LegacyReasonNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "legacy conversational intents",
            garm_target: "LegacyDialogueNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "observatory/dashboard",
            garm_target: "ObservatoryNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "history/log",
            garm_target: "LegacyHistoryNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "start/stop autonomy",
            garm_target: "GarmRuntime autonomous pulse gate",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "evolve/improve",
            garm_target: "LegacyEvolutionNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "help text",
            garm_target: "HelpNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "full historical source",
            garm_target: "legacy_sources + legacy_repl",
            status: "preserved",
        },
        LegacyMigrationItem {
            legacy_area: "final non-loss audit",
            garm_target: "LEGACY_FINAL_AUDIT.md",
            status: "documented",
        },
        LegacyMigrationItem {
            legacy_area: "curiosity, missions, self-model, dream, shared knowledge",
            garm_target: "LegacyCognitionNode",
            status: "migrated",
        },
        LegacyMigrationItem {
            legacy_area: "complexity tracker and old metrics formulas",
            garm_target: "TelemetryNode + BenchmarkNode + ApiRuntimeMetrics",
            status: "replaced/partial",
        },
        LegacyMigrationItem {
            legacy_area: "organic timers and tension field",
            garm_target: "temporal scales + CampoTensionNode + FEPEngine + Surprise/Homeostasis/ActiveInference capabilities",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "custom venado .vena persistence",
            garm_target: "VenadoCompatibilityNode + state_paths + JSON snapshots + legacy text memory export",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "self-source autoconsumo and architecture map",
            garm_target: "AutoconsumoNode + ArchitectureModel + SelfModification + SelfAwareness + MetaArchitectNode",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "rich KnowledgeGraph, TTL, source trust, hybrid RAG",
            garm_target: "LegacyKnowledgeGraphNode + HyperGraph + Semantics + Causality + Evidence + Epistemic + LogicReasoning",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "crawler, HTTP tools, local KB and ConceptNet loaders",
            garm_target: "LegacyCrawlerNode + conceptnet command + ToolCalling + McpClient + ComputerUse + Sandbox + CorpusReader",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "ParadigmHub",
            garm_target: "paradigm architecture eval + paradigm_architecture_technique_map.json + compatibility snapshot",
            status: "superseded",
        },
        LegacyMigrationItem {
            legacy_area: "narrative experimental entities and bounded eco/rebirth mechanics",
            garm_target: "EcoSystemNode + RebirthMeltraceNode + rebirth command + LegacyCognitionNode + Phenomenology + SelfAwareness",
            status: "migrated/replaced",
        },
        LegacyMigrationItem {
            legacy_area: "Readiness gaps: learning, planning, grounding, prediction, memory, self-correction, generalization, scaling",
            garm_target: "ReadinessNode + readiness command + observatory report",
            status: "active roadmap",
        },
        LegacyMigrationItem {
            legacy_area: "organic narrative, Umbra, child-autons, theatrical death/rebirth, experimental heuristics",
            garm_target: "OrganicLifecycleNode + ritual/umbra command + observatory report",
            status: "migrated/better",
        },
    ]
}

pub fn migration_report() -> String {
    let mut out = String::from("GARM legacy migration map:\n");
    for item in migration_items() {
        out.push_str(&format!(
            "- {} -> {} [{}]\n",
            item.legacy_area, item.garm_target, item.status
        ));
    }
    out.push_str(&legacy_source_archive_report());
    out
}

pub fn legacy_source_archive_report() -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/garm");
    let files = [
        root.join("legacy_repl.rs"),
        root.join("legacy_sources/eden_repl.rs"),
        root.join("legacy_sources/eden_repl.rs.debug"),
        root.join("legacy_sources/eden_repl.rs.bak"),
    ];
    let mut out = String::from("legacy source archive:\n");
    for file in files {
        match std::fs::read(&file) {
            Ok(bytes) => out.push_str(&format!(
                "- {} bytes={} fnv64={:016x}\n",
                file.display(),
                bytes.len(),
                fnv64(&bytes)
            )),
            Err(e) => out.push_str(&format!("- {} error={}\n", file.display(), e)),
        }
    }
    out
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
        hash ^= *byte as u64;
        hash.wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::{legacy_source_archive_report, migration_report};

    #[test]
    fn report_includes_key_migrated_nodes() {
        let report = migration_report();
        assert!(report.contains("CommandRouterNode"));
        assert!(report.contains("LegacyMemoryNode"));
        assert!(report.contains("LegacyDialogueNode"));
        assert!(report.contains("ObservatoryNode"));
        assert!(report.contains("LegacyHistoryNode"));
        assert!(report.contains("LegacyEvolutionNode"));
        assert!(report.contains("ApiServerNode"));
        assert!(report.contains("LegacyCognitionNode"));
        assert!(report.contains("ArchitectureModel"));
        assert!(report.contains("CampoTensionNode"));
        assert!(report.contains("LegacyKnowledgeGraphNode"));
        assert!(report.contains("AutoconsumoNode"));
        assert!(report.contains("VenadoCompatibilityNode"));
        assert!(report.contains("paradigm architecture eval"));
        assert!(report.contains("EcoSystemNode"));
        assert!(report.contains("RebirthMeltraceNode"));
        assert!(report.contains("LegacyCrawlerNode"));
        assert!(report.contains("ReadinessNode"));
        assert!(report.contains("OrganicLifecycleNode"));
        assert!(report.contains("HyperGraph"));
        assert!(report.contains("ParadigmHub"));
        assert!(report.contains("LEGACY_FINAL_AUDIT.md"));
        assert!(report.contains("legacy source archive"));
        assert!(report.contains("[migrated]"));
        assert!(report.contains("[replaced/partial]"));
    }

    #[test]
    fn legacy_source_archive_report_hashes_preserved_files() {
        let report = legacy_source_archive_report();
        assert!(report.contains("legacy_repl.rs"));
        assert!(report.contains("fnv64="));
    }
}
