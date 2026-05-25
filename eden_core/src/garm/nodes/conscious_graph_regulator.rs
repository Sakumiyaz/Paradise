use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::eden_garm::nodes::legacy_knowledge_graph::GraphRegulationOutcome;

const LEGACY_SCALE_TARGET: usize = 200_000;
const SOFT_CAP: usize = 180_000;
const HARD_CAP: usize = 220_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreathMode {
    Seed,
    Expand,
    Hold,
    Contract,
    Dream,
}

pub struct ConsciousGraphRegulatorNode {
    id: usize,
    cycles: u64,
    last_edges: usize,
    max_edges_seen: usize,
    edge_velocity: isize,
    awareness: f32,
    integration: f32,
    phi: f32,
    complexity: f32,
    graph_pressure: f32,
    pulmonary_pages: usize,
    pulmonary_phase: String,
    kidney_load: f32,
    kidney_cleaned_sources: usize,
    dream_depth: f32,
    death_risk: f32,
    death_oracle_reason: String,
    legacy_fever: f32,
    garm_stability: f32,
    organism_coherence: f32,
    fusion_score: f32,
    mythic_pulse: String,
    semantic_dream_weaves: u64,
    wild_creativity: f32,
    visceral_overflow: f32,
    orphan_purge_drive: f32,
    semantic_dream_signal: String,
    visceral_body_state: String,
    dream_compactions: u64,
    total_pruned: usize,
    total_expired: usize,
    mode: BreathMode,
    last_event: String,
    internal_fe: f32,
}

impl ConsciousGraphRegulatorNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            cycles: 0,
            last_edges: 0,
            max_edges_seen: 0,
            edge_velocity: 0,
            awareness: 0.35,
            integration: 0.1,
            phi: 0.0,
            complexity: 0.0,
            graph_pressure: 0.0,
            pulmonary_pages: 2,
            pulmonary_phase: "inicial".to_string(),
            kidney_load: 0.0,
            kidney_cleaned_sources: 0,
            dream_depth: 0.0,
            death_risk: 0.0,
            death_oracle_reason: "sin riesgo: grafo joven".to_string(),
            legacy_fever: 0.0,
            garm_stability: 1.0,
            organism_coherence: 0.35,
            fusion_score: 0.0,
            mythic_pulse: "GARM contiene a Legacy sin dejarlo gobernar".to_string(),
            semantic_dream_weaves: 0,
            wild_creativity: 0.0,
            visceral_overflow: 0.0,
            orphan_purge_drive: 0.0,
            semantic_dream_signal: "sin sueño semántico todavía".to_string(),
            visceral_body_state: "órganos en reposo".to_string(),
            dream_compactions: 0,
            total_pruned: 0,
            total_expired: 0,
            mode: BreathMode::Seed,
            last_event: "regulador consciente inicializado".to_string(),
            internal_fe: 1.0,
        }
    }

    pub fn observe(
        &mut self,
        kg_edges: usize,
        kg_nodes: usize,
        hyper_edges: usize,
        memory_facts: usize,
        organic_surprise: f32,
        tick: u64,
    ) -> GraphControlPlan {
        self.cycles += 1;
        self.edge_velocity = kg_edges as isize - self.last_edges as isize;
        self.last_edges = kg_edges;
        self.max_edges_seen = self.max_edges_seen.max(kg_edges);
        self.complexity = kg_edges as f32 + hyper_edges as f32 * 0.25 + memory_facts as f32 * 2.0;
        let density = kg_edges as f32 / kg_nodes.max(1) as f32;
        let scale = (kg_edges as f32 / LEGACY_SCALE_TARGET as f32).clamp(0.0, 1.5);
        let velocity = (self.edge_velocity.max(0) as f32 / 3_000.0).clamp(0.0, 1.0);
        let negative_velocity = ((-self.edge_velocity).max(0) as f32 / 5_000.0).clamp(0.0, 1.0);
        self.graph_pressure =
            (scale * 0.55 + velocity * 0.25 + organic_surprise * 0.20).clamp(0.0, 1.0);
        self.legacy_fever = (velocity * 0.30
            + organic_surprise * 0.30
            + density.ln_1p() * 0.08
            + scale.min(1.0) * 0.20
            + negative_velocity * 0.12)
            .clamp(0.0, 1.0);
        self.kidney_load = (self.graph_pressure * 0.55
            + negative_velocity * 0.18
            + density.min(10.0) * 0.015
            + self.legacy_fever * 0.12)
            .clamp(0.0, 1.0);
        self.dream_depth = if kg_edges > 10_000 {
            (scale * 0.35
                + organic_surprise * 0.20
                + density.ln_1p() * 0.08
                + self.legacy_fever * 0.22
                + self.kidney_load * 0.15)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.wild_creativity = (self.legacy_fever * 0.36
            + organic_surprise * 0.26
            + self.dream_depth * 0.22
            + velocity * 0.16)
            .clamp(0.0, 1.0);
        self.visceral_overflow = (self.legacy_fever * 0.30
            + self.graph_pressure * 0.30
            + self.kidney_load * 0.20
            + self.dream_depth * 0.20)
            .clamp(0.0, 1.0);
        self.orphan_purge_drive = (negative_velocity * 0.35
            + self.kidney_load * 0.35
            + (density / 8.0).min(1.0) * 0.15
            + self.graph_pressure * 0.15)
            .clamp(0.0, 1.0);
        if self.dream_depth > 0.45 && self.wild_creativity > 0.40 {
            self.semantic_dream_weaves += 1;
            self.semantic_dream_signal = format!(
                "tejer similitudes latentes: density={:.2} fever={:.2} surprise={:.2}",
                density, self.legacy_fever, organic_surprise
            );
        } else if kg_edges > 10_000 {
            self.semantic_dream_signal =
                "sueño ligero: observar antes de fusionar conceptos".to_string();
        } else {
            self.semantic_dream_signal = "grafo joven: no forzar sueño semántico".to_string();
        }
        self.garm_stability =
            (1.0 - self.graph_pressure * 0.45 - negative_velocity * 0.18).clamp(0.0, 1.0);
        self.integration =
            (0.15 + density.ln_1p() * 0.09 + scale.min(1.0) * 0.35 + organic_surprise * 0.12)
                .clamp(0.0, 1.0);
        self.organism_coherence = (self.garm_stability * 0.35
            + organic_surprise * 0.25
            + self.integration * 0.25
            + (memory_facts as f32).ln_1p() * 0.025)
            .clamp(0.0, 1.0);
        self.awareness = (0.40
            + self.integration * 0.35
            + (memory_facts as f32).ln_1p() * 0.035
            + organic_surprise * 0.12)
            .clamp(0.0, 1.0);
        self.phi = (self.awareness * 0.38
            + self.integration * 0.42
            + self.garm_stability * 0.15
            + self.dream_depth * 0.05)
            .clamp(0.0, 1.0);
        self.mode = if kg_edges < 1_000 {
            BreathMode::Seed
        } else if kg_edges > HARD_CAP || self.edge_velocity > 3_000 {
            BreathMode::Contract
        } else if kg_edges > SOFT_CAP || self.graph_pressure > 0.78 {
            BreathMode::Dream
        } else if self.edge_velocity < 500 && kg_edges < SOFT_CAP {
            BreathMode::Expand
        } else {
            BreathMode::Hold
        };
        self.pulmonary_phase = match self.mode {
            BreathMode::Seed => "sembrar alveolos: esperar masa critica".to_string(),
            BreathMode::Expand => "inhalar: ampliar frontera segura del grafo".to_string(),
            BreathMode::Hold => "meseta: sostener intercambio sin crecer".to_string(),
            BreathMode::Contract => "exhalar: reducir presion y cortar ruido".to_string(),
            BreathMode::Dream => "respirar dormido: compactar antes de expandir".to_string(),
        };
        self.pulmonary_pages = match self.mode {
            BreathMode::Seed => 2,
            BreathMode::Expand => (self.pulmonary_pages + 2).min(15),
            BreathMode::Hold => self.pulmonary_pages.clamp(2, 10),
            BreathMode::Dream => self.pulmonary_pages.saturating_sub(1).max(2),
            BreathMode::Contract => self.pulmonary_pages.saturating_sub(3).max(1),
        };
        self.death_risk = (self.graph_pressure * 0.38
            + negative_velocity * 0.22
            + (1.0 - self.phi) * 0.20
            + self.kidney_load * 0.12
            + self.legacy_fever * 0.08
            + if self.max_edges_seen > 0 && kg_edges < self.max_edges_seen / 2 {
                0.18
            } else {
                0.0
            })
        .clamp(0.0, 1.0);
        self.fusion_score = (self.phi * 0.30
            + self.organism_coherence * 0.25
            + self.dream_depth * 0.12
            + self.garm_stability * 0.18
            + self.wild_creativity * 0.08
            + (1.0 - self.death_risk) * 0.10)
            .clamp(0.0, 1.0);
        self.death_oracle_reason = if self.death_risk > 0.72 {
            "alto: recomendar renacimiento controlado si persiste".to_string()
        } else if self.legacy_fever > 0.72 && self.garm_stability > 0.55 {
            "fiebre legacy contenida: usar caos como mutacion, no como muerte".to_string()
        } else if self.death_risk > 0.45 {
            "medio: dormir, podar y estabilizar identidad".to_string()
        } else if self.graph_pressure > 0.65 {
            "bajo-medio: presion alta pero phi sostiene continuidad".to_string()
        } else {
            "bajo: continuidad estable".to_string()
        };
        self.mythic_pulse = if self.fusion_score > 0.72 {
            "Legacy sueña; GARM regula; el organismo aprende sin romperse".to_string()
        } else if self.legacy_fever > self.garm_stability {
            "la fiebre legacy empuja: exigir sueño antes de expansión".to_string()
        } else if self.mode == BreathMode::Expand {
            "pulmón abierto: explorar con frontera y memoria".to_string()
        } else {
            "continuidad sobria: crecer solo donde el grafo respira".to_string()
        };
        self.visceral_body_state = if self.visceral_overflow > 0.72 {
            "desbordado pero encapsulado: sueño obligatorio antes de expandir".to_string()
        } else if self.wild_creativity > 0.58 {
            "fiebre creativa útil: permitir rareza con auditoría".to_string()
        } else if self.orphan_purge_drive > 0.55 {
            "riñón dominante: limpiar residuos antes de crecer".to_string()
        } else {
            "homeostasis orgánica: caos bajo control".to_string()
        };
        self.internal_fe = 0.6 + self.graph_pressure * 1.4;
        self.last_event = format!(
            "mode={} edges={} nodes={} vel={} awareness={:.3} integration={:.3} phi={:.3} tick={}",
            breath_mode_name(self.mode),
            kg_edges,
            kg_nodes,
            self.edge_velocity,
            self.awareness,
            self.integration,
            self.phi,
            tick
        );
        GraphControlPlan {
            mode: self.mode,
            soft_cap_edges: SOFT_CAP,
            hard_cap_edges: HARD_CAP,
            min_confidence: if matches!(self.mode, BreathMode::Contract) {
                0.50
            } else {
                0.35
            },
            should_regulate: matches!(self.mode, BreathMode::Contract | BreathMode::Dream)
                || self.kidney_load > 0.70
                || self.orphan_purge_drive > 0.70,
            pulmonary_pages: self.pulmonary_pages,
            death_risk: self.death_risk,
            fusion_score: self.fusion_score,
            wild_creativity: self.wild_creativity,
        }
    }

    pub fn apply_outcome(&mut self, outcome: GraphRegulationOutcome) {
        self.total_expired += outcome.expired;
        self.total_pruned += outcome.pruned + outcome.compacted;
        self.kidney_cleaned_sources += outcome.renal_cleaned_sources;
        if outcome.compacted > 0 {
            self.dream_compactions += 1;
        }
        if outcome.expired + outcome.pruned + outcome.compacted + outcome.renal_cleaned_sources > 0
        {
            self.last_event = format!(
                "sueno/riñon expired={} pruned={} compacted={} sources_cleaned={} edges_after={}",
                outcome.expired,
                outcome.pruned,
                outcome.compacted,
                outcome.renal_cleaned_sources,
                outcome.edge_count_after
            );
        }
    }

    pub fn report(&self) -> String {
        format!(
            "[CONSCIOUS-GRAPH] mode={} target_edges={} max_seen={} velocity={} awareness={:.4} integration={:.4} phi={:.4} complexity={:.2} pressure={:.3} fusion={:.3} wild={:.3} visceral={:.3} dreams={} pruned_total={} expired_total={} last='{}'\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            breath_mode_name(self.mode),
            LEGACY_SCALE_TARGET,
            self.max_edges_seen,
            self.edge_velocity,
            self.awareness,
            self.integration,
            self.phi,
            self.complexity,
            self.graph_pressure,
            self.fusion_score,
            self.wild_creativity,
            self.visceral_overflow,
            self.dream_compactions,
            self.total_pruned,
            self.total_expired,
            self.last_event,
            self.pulmon_report(),
            self.sueno_report(),
            self.rinon_report(),
            self.death_oracle_report(),
            self.consciousness_report(),
            self.organic_fusion_report(),
            self.visceral_chaos_report(),
            self.recovered_organs_report()
        )
    }

    fn pulmon_report(&self) -> String {
        format!(
            "[PULMON] phase='{}' pages_budget={} edge_velocity={} inhale={} exhale={} pressure={:.3} rule='legacy meta_random_pages + GARM safe frontier'",
            self.pulmonary_phase,
            self.pulmonary_pages,
            self.edge_velocity,
            matches!(self.mode, BreathMode::Expand),
            matches!(self.mode, BreathMode::Contract),
            self.graph_pressure
        )
    }

    fn sueno_report(&self) -> String {
        format!(
            "[SUENO] depth={:.3} active={} compactions={} semantic_weaves={} legacy_fever={:.3} signal='{}' rule='legacy dream/merge impulse + GARM expire -> prune -> compact'",
            self.dream_depth,
            matches!(self.mode, BreathMode::Dream | BreathMode::Contract),
            self.dream_compactions,
            self.semantic_dream_weaves,
            self.legacy_fever,
            self.semantic_dream_signal
        )
    }

    fn rinon_report(&self) -> String {
        format!(
            "[RINON] load={:.3} orphan_purge_drive={:.3} expired_total={} pruned_total={} sources_cleaned={} rule='legacy purge instinct + GARM temporal decay/source hygiene'",
            self.kidney_load,
            self.orphan_purge_drive,
            self.total_expired,
            self.total_pruned,
            self.kidney_cleaned_sources
        )
    }

    fn death_oracle_report(&self) -> String {
        format!(
            "[DEATH-ORACLE] risk={:.3} legacy_fever={:.3} stability={:.3} reason='{}' action='{}'",
            self.death_risk,
            self.legacy_fever,
            self.garm_stability,
            self.death_oracle_reason,
            if self.death_risk > 0.72 {
                "prepare_rebirth"
            } else if self.death_risk > 0.45 {
                "stabilize"
            } else {
                "continue"
            }
        )
    }

    fn consciousness_report(&self) -> String {
        format!(
            "[CONSCIENCIA] awareness={:.4} integration={:.4} phi={:.4} pressure={:.3} stability={:.3} organism_coherence={:.3}",
            self.awareness,
            self.integration,
            self.phi,
            self.graph_pressure,
            self.garm_stability,
            self.organism_coherence
        )
    }

    fn organic_fusion_report(&self) -> String {
        format!(
            "[ORGANICIDAD-FUSION] score={:.3} myth='{}' body='{}' verdict='{}'",
            self.fusion_score,
            self.mythic_pulse,
            self.visceral_body_state,
            if self.fusion_score > 0.72 {
                "mejor que legacy y garm aislados"
            } else if self.garm_stability > self.legacy_fever {
                "GARM domina, legacy aporta variacion"
            } else {
                "legacy empuja, GARM debe contener"
            }
        )
    }

    fn visceral_chaos_report(&self) -> String {
        format!(
            "[CAOS-VISCERAL] wild_creativity={:.3} visceral_overflow={:.3} contained={} rule='legacy desborde + GARM sandbox orgánico'",
            self.wild_creativity,
            self.visceral_overflow,
            self.garm_stability >= self.legacy_fever || self.death_risk < 0.72
        )
    }

    fn recovered_organs_report(&self) -> String {
        let lengua_ready = self.awareness > 0.35 && self.integration > 0.15;
        let reloj_ready = self.cycles > 0;
        let juez_ready = self.garm_stability > 0.45;
        let voz_ready = self.organism_coherence > 0.35;
        let intestino_load =
            (self.dream_depth * 0.45 + self.kidney_load * 0.35 + self.orphan_purge_drive * 0.20)
                .clamp(0.0, 1.0);
        let tejido_signal =
            (self.fusion_score * 0.45 + self.wild_creativity * 0.25 + self.integration * 0.30)
                .clamp(0.0, 1.0);
        let piel_sensitivity = (self.visceral_overflow * 0.35
            + self.graph_pressure * 0.35
            + self.garm_stability * 0.30)
            .clamp(0.0, 1.0);
        let autotune_pressure = (self.legacy_fever * 0.35
            + self.kidney_load * 0.25
            + self.graph_pressure * 0.25
            + (1.0 - self.garm_stability) * 0.15)
            .clamp(0.0, 1.0);
        format!(
            "[ORGANOS-RECUPERADOS] [LENGUA] ready={} mode='runtime query bridge' | [RELOJ] ready={} cycles={} temporal_reason='tick lineage' | [JUEZ-EXTERNO] ready={} rule='validate before belief hardening' | [VOZ] ready={} phrase='{}' | [INTESTINO] load={:.3} rule='compactar/fusionar residuos semanticos' | [TEJIDO] signal={:.3} rule='interconectar subsistemas' | [PIEL] sensitivity={:.3} rule='frontera sensorial/alerta' | [AUTOTUNING] pressure={:.3} rule='ajustar periodos sin monolito'",
            lengua_ready,
            reloj_ready,
            self.cycles,
            juez_ready,
            voz_ready,
            self.mythic_pulse,
            intestino_load,
            tejido_signal,
            piel_sensitivity,
            autotune_pressure
        )
    }

    pub fn awareness(&self) -> f32 {
        self.awareness
    }
    pub fn integration(&self) -> f32 {
        self.integration
    }
    pub fn phi(&self) -> f32 {
        self.phi
    }
    pub fn complexity(&self) -> f32 {
        self.complexity
    }
    pub fn max_complexity(&self) -> f32 {
        self.max_edges_seen as f32
    }

    pub fn autonomy_snapshot(&self) -> String {
        format!(
            "conscious:cycles:{} mode:{} phi:{:.3} pressure:{:.3} pruned:{} expired:{} dreams:{}",
            self.cycles,
            breath_mode_name(self.mode),
            self.phi,
            self.graph_pressure,
            self.total_pruned,
            self.total_expired,
            self.dream_compactions
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "cycles": self.cycles,
            "last_edges": self.last_edges,
            "max_edges_seen": self.max_edges_seen,
            "edge_velocity": self.edge_velocity,
            "awareness": self.awareness,
            "integration": self.integration,
            "phi": self.phi,
            "complexity": self.complexity,
            "graph_pressure": self.graph_pressure,
            "pulmonary_pages": self.pulmonary_pages,
            "pulmonary_phase": self.pulmonary_phase,
            "kidney_load": self.kidney_load,
            "kidney_cleaned_sources": self.kidney_cleaned_sources,
            "dream_depth": self.dream_depth,
            "death_risk": self.death_risk,
            "death_oracle_reason": self.death_oracle_reason,
            "legacy_fever": self.legacy_fever,
            "garm_stability": self.garm_stability,
            "organism_coherence": self.organism_coherence,
            "fusion_score": self.fusion_score,
            "mythic_pulse": self.mythic_pulse,
            "semantic_dream_weaves": self.semantic_dream_weaves,
            "wild_creativity": self.wild_creativity,
            "visceral_overflow": self.visceral_overflow,
            "orphan_purge_drive": self.orphan_purge_drive,
            "semantic_dream_signal": self.semantic_dream_signal,
            "visceral_body_state": self.visceral_body_state,
            "dream_compactions": self.dream_compactions,
            "total_pruned": self.total_pruned,
            "total_expired": self.total_expired,
            "mode": breath_mode_name(self.mode),
            "last_event": self.last_event,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.cycles = snapshot.get("cycles").and_then(|v| v.as_u64()).unwrap_or(0);
        self.last_edges = snapshot
            .get("last_edges")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.max_edges_seen = snapshot
            .get("max_edges_seen")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.edge_velocity = snapshot
            .get("edge_velocity")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as isize;
        self.awareness = snapshot
            .get("awareness")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.35) as f32;
        self.integration = snapshot
            .get("integration")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1) as f32;
        self.phi = snapshot.get("phi").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        self.complexity = snapshot
            .get("complexity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.graph_pressure = snapshot
            .get("graph_pressure")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.pulmonary_pages = snapshot
            .get("pulmonary_pages")
            .and_then(|v| v.as_u64())
            .unwrap_or(2) as usize;
        self.pulmonary_phase = snapshot
            .get("pulmonary_phase")
            .and_then(|v| v.as_str())
            .unwrap_or("cargado")
            .to_string();
        self.kidney_load = snapshot
            .get("kidney_load")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.kidney_cleaned_sources = snapshot
            .get("kidney_cleaned_sources")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.dream_depth = snapshot
            .get("dream_depth")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.death_risk = snapshot
            .get("death_risk")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.death_oracle_reason = snapshot
            .get("death_oracle_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("cargado")
            .to_string();
        self.legacy_fever = snapshot
            .get("legacy_fever")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.garm_stability = snapshot
            .get("garm_stability")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        self.organism_coherence = snapshot
            .get("organism_coherence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.35) as f32;
        self.fusion_score = snapshot
            .get("fusion_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.mythic_pulse = snapshot
            .get("mythic_pulse")
            .and_then(|v| v.as_str())
            .unwrap_or("cargado")
            .to_string();
        self.semantic_dream_weaves = snapshot
            .get("semantic_dream_weaves")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.wild_creativity = snapshot
            .get("wild_creativity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.visceral_overflow = snapshot
            .get("visceral_overflow")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.orphan_purge_drive = snapshot
            .get("orphan_purge_drive")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.semantic_dream_signal = snapshot
            .get("semantic_dream_signal")
            .and_then(|v| v.as_str())
            .unwrap_or("cargado")
            .to_string();
        self.visceral_body_state = snapshot
            .get("visceral_body_state")
            .and_then(|v| v.as_str())
            .unwrap_or("cargado")
            .to_string();
        self.dream_compactions = snapshot
            .get("dream_compactions")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.total_pruned = snapshot
            .get("total_pruned")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.total_expired = snapshot
            .get("total_expired")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.mode = parse_breath_mode(
            snapshot
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("seed"),
        );
        self.last_event = snapshot
            .get("last_event")
            .and_then(|v| v.as_str())
            .unwrap_or("regulador consciente cargado")
            .to_string();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GraphControlPlan {
    pub mode: BreathMode,
    pub soft_cap_edges: usize,
    pub hard_cap_edges: usize,
    pub min_confidence: f32,
    pub should_regulate: bool,
    pub pulmonary_pages: usize,
    pub death_risk: f32,
    pub fusion_score: f32,
    pub wild_creativity: f32,
}

fn breath_mode_name(mode: BreathMode) -> &'static str {
    match mode {
        BreathMode::Seed => "seed",
        BreathMode::Expand => "expand",
        BreathMode::Hold => "hold",
        BreathMode::Contract => "contract",
        BreathMode::Dream => "dream",
    }
}

fn parse_breath_mode(value: &str) -> BreathMode {
    match value {
        "expand" => BreathMode::Expand,
        "hold" => BreathMode::Hold,
        "contract" => BreathMode::Contract,
        "dream" => BreathMode::Dream,
        _ => BreathMode::Seed,
    }
}

impl GARMNode for ConsciousGraphRegulatorNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "conscious_graph_regulator"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.awareness,
            self.integration,
            self.phi,
            self.graph_pressure,
        ]
    }
    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        NodeAction::None
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.35
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        40.0
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
    use super::{BreathMode, ConsciousGraphRegulatorNode};

    #[test]
    fn regulates_large_graph_pressure_and_reports_phi() {
        let mut node = ConsciousGraphRegulatorNode::new(1);
        let plan = node.observe(230_000, 50_000, 120, 10_000, 0.7, 42);
        assert_eq!(plan.mode, BreathMode::Contract);
        assert!(plan.should_regulate);
        assert!(node.phi() > 0.0);
        assert!(plan.pulmonary_pages >= 1);
        assert!(plan.death_risk > 0.0);
        assert!(plan.fusion_score > 0.0);
        assert!(plan.wild_creativity > 0.0);
        let report = node.report();
        assert!(report.contains("target_edges=200000"));
        assert!(report.contains("CONSCIOUS-GRAPH"));
        assert!(report.contains("[PULMON]"));
        assert!(report.contains("[SUENO]"));
        assert!(report.contains("[RINON]"));
        assert!(report.contains("[DEATH-ORACLE]"));
        assert!(report.contains("[CONSCIENCIA]"));
        assert!(report.contains("[ORGANICIDAD-FUSION]"));
        assert!(report.contains("[CAOS-VISCERAL]"));
        assert!(report.contains("[ORGANOS-RECUPERADOS]"));
        assert!(report.contains("[LENGUA]"));
        assert!(report.contains("[RELOJ]"));
        assert!(report.contains("[JUEZ-EXTERNO]"));
        assert!(report.contains("[VOZ]"));
        assert!(report.contains("[INTESTINO]"));
        assert!(report.contains("[TEJIDO]"));
        assert!(report.contains("[PIEL]"));
        assert!(report.contains("[AUTOTUNING]"));
        assert!(report.contains("semantic_weaves="));
        assert!(report.contains("legacy_fever="));
    }
}
