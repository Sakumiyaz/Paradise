use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use crate::paradigms::{autograd, models, parser};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::net::{IpAddr, ToSocketAddrs};
use std::time::Duration;

pub struct AutoconsumoNode {
    id: usize,
    route: String,
    structures: usize,
    functions: usize,
    modules: usize,
    last_hash: u64,
    fragments: Vec<String>,
    parses: u64,
    internal_fe: f32,
}

impl AutoconsumoNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            route: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/bin/eden_garm.rs")
                .to_string_lossy()
                .to_string(),
            structures: 0,
            functions: 0,
            modules: 0,
            last_hash: 0,
            fragments: Vec::new(),
            parses: 0,
            internal_fe: 1.0,
        }
    }

    pub fn nutrirse(&mut self) -> Vec<String> {
        let Ok(content) = std::fs::read_to_string(&self.route) else {
            return Vec::new();
        };
        let hash = content.bytes().fold(0xcbf29ce484222325u64, |mut h, b| {
            h ^= b as u64;
            h.wrapping_mul(0x100000001b3)
        });
        if hash == self.last_hash && !self.fragments.is_empty() {
            return self.fragments.iter().take(3).cloned().collect();
        }
        self.last_hash = hash;
        self.parses += 1;
        self.structures = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("struct ")
                    || line.trim_start().starts_with("pub struct ")
            })
            .count();
        self.functions = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("fn ") || line.trim_start().starts_with("pub fn ")
            })
            .count();
        self.modules = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("mod ") || line.trim_start().starts_with("pub mod ")
            })
            .count();
        self.fragments = vec![
            format!("[AUTO-ESTRUCTURA] {} estructuras", self.structures),
            format!("[AUTO-FUNCION] {} funciones", self.functions),
            format!("[AUTO-MODULO] {} modulos", self.modules),
        ];
        self.fragments.clone()
    }

    pub fn informe(&self) -> String {
        format!(
            "[AUTOCONSUMO] parses={} estructuras={} funciones={} modulos={} fragmentos={}",
            self.parses,
            self.structures,
            self.functions,
            self.modules,
            self.fragments.len()
        )
    }

    pub fn parse_count(&self) -> u64 {
        self.parses
    }

    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        write_json(
            path,
            serde_json::json!({
                "route": self.route,
                "structures": self.structures,
                "functions": self.functions,
                "modules": self.modules,
                "last_hash": self.last_hash,
                "fragments": self.fragments,
                "parses": self.parses,
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.route = json_string(&snapshot, "route", &self.route);
        self.structures = json_usize(&snapshot, "structures", 0);
        self.functions = json_usize(&snapshot, "functions", 0);
        self.modules = json_usize(&snapshot, "modules", 0);
        self.last_hash = snapshot
            .get("last_hash")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.fragments = json_string_array(snapshot.get("fragments"));
        self.parses = snapshot.get("parses").and_then(|v| v.as_u64()).unwrap_or(0);
        Ok(())
    }
}

impl GARMNode for AutoconsumoNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_autoconsumo"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.structures as f32,
            self.functions as f32,
            self.modules as f32,
        ]
    }
    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        if ctx.tick % 25 == 0 {
            let out = self.nutrirse();
            self.internal_fe = (self.internal_fe * 0.95).max(0.2);
            return NodeAction::Output(vec![out.len() as f32, self.parses as f32]);
        }
        NodeAction::None
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.4
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

pub struct VenadoCompatibilityNode {
    id: usize,
    root: String,
    writes: u64,
    reads: u64,
    internal_fe: f32,
}

impl VenadoCompatibilityNode {
    pub fn new(id: usize) -> Self {
        let root = crate::eden_garm::state_paths::state_dir()
            .join("venas")
            .to_string_lossy()
            .to_string();
        let _ = std::fs::create_dir_all(&root);
        Self {
            id,
            root,
            writes: 0,
            reads: 0,
            internal_fe: 1.0,
        }
    }

    pub fn cristalizar(&mut self, name: &str, fields: &[(String, String)]) -> Result<(), String> {
        self.cristalizar_con_bloques(name, fields, &[])
    }

    pub fn cristalizar_con_bloques(
        &mut self,
        name: &str,
        fields: &[(String, String)],
        blocks: &[(String, Vec<String>)],
    ) -> Result<(), String> {
        std::fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let path = self.path(name);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut data = format!("~CRISTAL|v1|{}|{}~\n", Self::safe(name), timestamp);
        for (k, v) in fields {
            data.push_str(&format!(">{}:{}\n", Self::escape(k), Self::escape(v)));
        }
        for (block_name, lines) in blocks {
            data.push_str(&format!("<{}_inicio\n", Self::escape(block_name)));
            for line in lines {
                data.push_str(&format!("{}\n", Self::escape(line)));
            }
            data.push_str(&format!("{}_fin>\n", Self::escape(block_name)));
        }
        data.push_str("~END~\n");
        std::fs::write(path, data).map_err(|e| e.to_string())?;
        self.writes += 1;
        Ok(())
    }

    pub fn descristalizar(&mut self, name: &str) -> Result<Vec<(String, String)>, String> {
        self.descristalizar_completo(name).map(|(fields, _)| fields)
    }

    pub fn descristalizar_completo(
        &mut self,
        name: &str,
    ) -> Result<(Vec<(String, String)>, Vec<(String, Vec<String>)>), String> {
        let text = std::fs::read_to_string(self.path(name)).map_err(|e| e.to_string())?;
        let mut fields = Vec::new();
        let mut blocks = Vec::new();
        let mut current_block: Option<(String, Vec<String>)> = None;
        for line in text.lines() {
            if line.starts_with('>') && !line.starts_with(">>") {
                if let Some((k, v)) = line[1..].split_once(':') {
                    fields.push((Self::unescape(k), Self::unescape(v)));
                }
            } else if line.starts_with('<') && line.ends_with("_inicio") {
                current_block = Some((Self::unescape(&line[1..line.len() - 7]), Vec::new()));
            } else if line.ends_with("_fin>") {
                if let Some((name, lines)) = current_block.take() {
                    let end_name = Self::unescape(&line[..line.len() - 5]);
                    if end_name == name {
                        blocks.push((name, lines));
                    }
                }
            } else if let Some((_, lines)) = current_block.as_mut() {
                lines.push(Self::unescape(line));
            }
        }
        self.reads += 1;
        Ok((fields, blocks))
    }

    pub fn lista_venas(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if let Some(name) = entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.strip_suffix(".vena"))
                {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    pub fn existe(&self, name: &str) -> bool {
        std::fs::metadata(self.path(name)).is_ok()
    }

    pub fn informe(&self) -> String {
        format!(
            "[VENADO] root={} writes={} reads={}",
            self.root, self.writes, self.reads
        )
    }

    pub fn write_count(&self) -> u64 {
        self.writes
    }

    pub fn read_count(&self) -> u64 {
        self.reads
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        write_json(
            path,
            serde_json::json!({
                "root": self.root,
                "writes": self.writes,
                "reads": self.reads,
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.root = json_string(&snapshot, "root", &self.root);
        self.writes = snapshot.get("writes").and_then(|v| v.as_u64()).unwrap_or(0);
        self.reads = snapshot.get("reads").and_then(|v| v.as_u64()).unwrap_or(0);
        let _ = std::fs::create_dir_all(&self.root);
        Ok(())
    }

    fn path(&self, name: &str) -> String {
        format!("{}/{}.vena", self.root, Self::safe(name))
    }
    fn safe(name: &str) -> String {
        name.replace(['/', '\\', '\0'], "_").replace("..", "_")
    }
    fn escape(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace(':', "\\:")
            .replace('~', "\\~")
    }
    fn unescape(value: &str) -> String {
        let mut out = String::new();
        let mut escaped = false;
        for ch in value.chars() {
            if escaped {
                out.push(if ch == 'n' { '\n' } else { ch });
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else {
                out.push(ch);
            }
        }
        out
    }
}

impl GARMNode for VenadoCompatibilityNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_venado"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.writes as f32, self.reads as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        NodeAction::Output(vec![self.writes as f32, self.reads as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        15.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct ParadigmHubNode {
    id: usize,
    active: HashSet<&'static str>,
    curriculum_level: usize,
    info_gain_history: Vec<f32>,
    cycle_count: u64,
    last_activations: Vec<&'static str>,
    inferred_facts: Vec<String>,
    autograd_bank: AutogradModelBank,
    internal_fe: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ParadigmCycleOutcome {
    pub activations: Vec<&'static str>,
    pub inferred_facts: Vec<String>,
    pub info_gain: f32,
    pub autograd_models_trained: usize,
}

struct AutogradModelBank {
    neural_parser: parser::NeuralParser,
    edge_scorer: models::EdgeScorer,
    emotion_model: models::EmotionModel,
    sleep_trigger: models::SleepTrigger,
    death_oracle: models::DeathOracle,
    crawl_picker: models::CrawlPicker,
    warden_detector: models::WardenDetector,
    predictor_temporal: autograd::Linear,
    trained_steps: u64,
    last_loss: f32,
    last_signal: f32,
}

impl AutogradModelBank {
    fn new() -> Self {
        Self {
            neural_parser: parser::NeuralParser::new(),
            edge_scorer: models::EdgeScorer::new(),
            emotion_model: models::EmotionModel::new(),
            sleep_trigger: models::SleepTrigger::new(),
            death_oracle: models::DeathOracle::new(),
            crawl_picker: models::CrawlPicker::new(),
            warden_detector: models::WardenDetector::new(),
            predictor_temporal: autograd::Linear::new(4, 4),
            trained_steps: 0,
            last_loss: 0.0,
            last_signal: 0.0,
        }
    }

    fn train_cycle(
        &mut self,
        kg_edges: usize,
        memory_facts: usize,
        tension: f32,
        info_gain: f32,
        tick: u64,
    ) -> (usize, f32) {
        let edge_scale = (kg_edges as f32 / 200_000.0).clamp(0.0, 1.0);
        let memory_scale = (memory_facts as f32 / 10_000.0).clamp(0.0, 1.0);
        let danger = tension.clamp(0.0, 1.0);
        let stability = (1.0 - danger).clamp(0.0, 1.0);
        let novelty = info_gain.clamp(0.0, 1.0);

        let mut loss = 0.0;
        let text = format!(
            "paradigm_tick_{} causa autograd_signal_{}",
            tick,
            (novelty * 100.0) as u32
        );
        self.neural_parser.expand_vocab(&text);
        loss += self.neural_parser.train(&text, 1, 0.003);

        loss += self.edge_scorer.train(
            &[novelty, memory_scale, edge_scale],
            (novelty + stability) * 0.5,
            0.003,
        );
        loss += self.emotion_model.train(
            &[novelty, danger, memory_scale, edge_scale, stability],
            &[stability, danger, novelty],
            0.003,
        );
        loss += self.sleep_trigger.train(
            &[edge_scale, danger, novelty],
            danger.max(edge_scale * 0.5),
            0.002,
        );
        loss += self.death_oracle.train(
            &[danger, edge_scale, novelty, stability],
            if danger > 0.85 { 1.0 } else { 0.0 },
            0.002,
        );
        loss += self.crawl_picker.train(
            &[novelty, memory_scale, edge_scale],
            novelty.max(memory_scale),
            0.003,
        );
        loss += self
            .warden_detector
            .train(&[danger, edge_scale, stability], stability, 0.003);

        let temporal_features = [danger, novelty, memory_scale, edge_scale];
        let temporal_pred = self.predictor_temporal.forward(&temporal_features);
        loss += self.predictor_temporal.backward(
            &temporal_features,
            &temporal_pred,
            &[danger, novelty, memory_scale, edge_scale],
            0.003,
        );

        self.trained_steps += 1;
        self.last_loss = loss / 8.0;
        self.last_signal = novelty;
        (8, self.last_loss)
    }

    fn restore_counters(&mut self, trained_steps: u64, last_loss: f32, last_signal: f32) {
        self.trained_steps = trained_steps;
        self.last_loss = last_loss;
        self.last_signal = last_signal;
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "neural_parser": {
                "model": linear_to_json(&self.neural_parser.model),
                "vocab": self.neural_parser.vocab,
            },
            "edge_scorer": linear_to_json(&self.edge_scorer.m),
            "emotion_model": linear_to_json(&self.emotion_model.m),
            "sleep_trigger": linear_to_json(&self.sleep_trigger.m),
            "death_oracle": linear_to_json(&self.death_oracle.m),
            "crawl_picker": linear_to_json(&self.crawl_picker.m),
            "warden_detector": linear_to_json(&self.warden_detector.m),
            "predictor_temporal": linear_to_json(&self.predictor_temporal),
            "trained_steps": self.trained_steps,
            "last_loss": self.last_loss,
            "last_signal": self.last_signal,
        })
    }

    fn load_json(&mut self, value: &serde_json::Value) {
        if let Some(parser_state) = value.get("neural_parser") {
            if let Some(model_state) = parser_state.get("model") {
                load_linear(&mut self.neural_parser.model, model_state);
            }
            self.neural_parser.vocab = json_string_array(parser_state.get("vocab"));
        }
        if let Some(v) = value.get("edge_scorer") {
            load_linear(&mut self.edge_scorer.m, v);
        }
        if let Some(v) = value.get("emotion_model") {
            load_linear(&mut self.emotion_model.m, v);
        }
        if let Some(v) = value.get("sleep_trigger") {
            load_linear(&mut self.sleep_trigger.m, v);
        }
        if let Some(v) = value.get("death_oracle") {
            load_linear(&mut self.death_oracle.m, v);
        }
        if let Some(v) = value.get("crawl_picker") {
            load_linear(&mut self.crawl_picker.m, v);
        }
        if let Some(v) = value.get("warden_detector") {
            load_linear(&mut self.warden_detector.m, v);
        }
        if let Some(v) = value.get("predictor_temporal") {
            load_linear(&mut self.predictor_temporal, v);
        }
        self.restore_counters(
            value
                .get("trained_steps")
                .and_then(|v| v.as_u64())
                .unwrap_or(self.trained_steps),
            value
                .get("last_loss")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.last_loss as f64) as f32,
            value
                .get("last_signal")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.last_signal as f64) as f32,
        );
    }

    fn report(&self) -> String {
        format!(
            "8/8 steps={} last_loss={:.4} signal={:.3}",
            self.trained_steps, self.last_loss, self.last_signal
        )
    }
}

impl ParadigmHubNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            active: ALL_PARADIGMS.iter().copied().collect(),
            curriculum_level: 1,
            info_gain_history: Vec::new(),
            cycle_count: 0,
            last_activations: Vec::new(),
            inferred_facts: Vec::new(),
            autograd_bank: AutogradModelBank::new(),
            internal_fe: 1.0,
        }
    }

    pub fn active_select_page(&mut self, errors: &HashMap<String, f32>) -> Option<String> {
        let selected = errors
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| {
                self.info_gain_history.push(*v);
                k.clone()
            });
        if self.info_gain_history.len() > 100 {
            self.info_gain_history.remove(0);
        }
        selected
    }

    pub fn curriculum_topic(&self, evolution_level: u32) -> &'static str {
        match (evolution_level as usize / 30)
            .max(self.curriculum_level)
            .min(2)
        {
            0 => "Physics/Chemistry/Biology",
            1 => "Quantum mechanics/Relativity/Genetics",
            _ => "Quantum field theory/String theory/CRISPR",
        }
    }

    pub fn paradigm_count(&self) -> usize {
        self.active.len()
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    pub fn inferred_fact_count(&self) -> usize {
        self.inferred_facts.len()
    }

    pub fn run_paradigm_cycle(
        &mut self,
        kg_edges: usize,
        memory_facts: usize,
        tension: f32,
        tick: u64,
    ) -> ParadigmCycleOutcome {
        self.cycle_count += 1;
        let budget = 7usize;
        let start = (self.cycle_count as usize + self.curriculum_level) % ALL_PARADIGMS.len();
        let mut activations = Vec::new();
        for offset in 0..budget {
            let name = ALL_PARADIGMS[(start + offset * 6) % ALL_PARADIGMS.len()];
            if self.active.contains(name) {
                activations.push(name);
            }
        }
        let scale = (kg_edges as f32 / 200_000.0).clamp(0.0, 1.0);
        let memory = (memory_facts as f32).ln_1p() / 12.0;
        let info_gain =
            (scale * 0.35 + memory * 0.35 + tension.clamp(0.0, 1.0) * 0.30).clamp(0.0, 1.0);
        let mut inferred_facts = Vec::new();
        if activations
            .iter()
            .any(|p| matches!(*p, "causal" | "bayesian" | "logic" | "neuro_symbolic"))
        {
            inferred_facts.push(format!(
                "paradigm_reasoning_tick_{} causes hypothesis_pressure",
                tick
            ));
        }
        if activations
            .iter()
            .any(|p| matches!(*p, "gnn" | "graph_v8" | "contrastive" | "rag"))
        {
            inferred_facts.push(format!("paradigm_graph_tick_{} is retrieval_signal", tick));
        }
        if activations.iter().any(|p| {
            matches!(
                *p,
                "death_oracle" | "sleep_trigger" | "emotion_model" | "warden_detector"
            )
        }) {
            inferred_facts.push(format!(
                "paradigm_regulator_tick_{} causes safety_signal",
                tick
            ));
        }
        if inferred_facts.is_empty() {
            inferred_facts.push(format!("paradigm_cycle_{} is active", tick));
        }
        let (autograd_models_trained, autograd_loss) =
            self.autograd_bank
                .train_cycle(kg_edges, memory_facts, tension, info_gain, tick);
        inferred_facts.push(format!(
            "autograd_bank_tick_{} trains {} legacy_models loss_{:.4}",
            tick, autograd_models_trained, autograd_loss
        ));
        self.last_activations = activations.clone();
        self.inferred_facts.extend(inferred_facts.iter().cloned());
        if self.inferred_facts.len() > 100 {
            self.inferred_facts
                .drain(0..self.inferred_facts.len() - 100);
        }
        self.info_gain_history.push(info_gain);
        if self.info_gain_history.len() > 100 {
            self.info_gain_history.remove(0);
        }
        ParadigmCycleOutcome {
            activations,
            inferred_facts,
            info_gain,
            autograd_models_trained,
        }
    }

    pub fn informe(&self) -> String {
        format!(
            "[PARADIGM-ARCHITECTURE-LEGACY] legacy_source=ParadigmHub status=superseded_by_paradigm_architecture_eval migrated_techniques=43 active_snapshot={}/43 curriculum_level={} cycles={} samples={} last='{}' inferred={} autograd={}",
            self.active.len(),
            self.curriculum_level,
            self.cycle_count,
            self.info_gain_history.len(),
            if self.last_activations.is_empty() { "none".to_string() } else { self.last_activations.join(",") },
            self.inferred_facts.len(),
            self.autograd_bank.report(),
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        write_json(
            path,
            serde_json::json!({
                "active": self.active.iter().copied().collect::<Vec<_>>(),
                "curriculum_level": self.curriculum_level,
                "info_gain_history": self.info_gain_history,
                "cycle_count": self.cycle_count,
                "last_activations": self.last_activations,
                "inferred_facts": self.inferred_facts,
                "autograd_trained_steps": self.autograd_bank.trained_steps,
                "autograd_last_loss": self.autograd_bank.last_loss,
                "autograd_last_signal": self.autograd_bank.last_signal,
                "autograd": self.autograd_bank.to_json(),
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.active = snapshot
            .get("active")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| parse_static_paradigm(v.as_str()?))
                    .collect()
            })
            .unwrap_or_else(|| self.active.clone());
        self.curriculum_level = json_usize(&snapshot, "curriculum_level", 1);
        self.info_gain_history = snapshot
            .get("info_gain_history")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|n| n as f32))
                    .collect()
            })
            .unwrap_or_default();
        self.cycle_count = snapshot
            .get("cycle_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.last_activations = snapshot
            .get("last_activations")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| parse_static_paradigm(v.as_str()?))
                    .collect()
            })
            .unwrap_or_default();
        self.inferred_facts = json_string_array(snapshot.get("inferred_facts"));
        self.autograd_bank.restore_counters(
            snapshot
                .get("autograd_trained_steps")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            snapshot
                .get("autograd_last_loss")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            snapshot
                .get("autograd_last_signal")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
        );
        if let Some(value) = snapshot.get("autograd") {
            self.autograd_bank.load_json(value);
        }
        Ok(())
    }
}

pub const ALL_PARADIGMS: [&str; 43] = [
    "active",
    "curriculum",
    "causal",
    "contrastive",
    "ensemble",
    "xai",
    "gnn",
    "transformer",
    "rl",
    "mcts",
    "bayesian",
    "logic",
    "neuro_symbolic",
    "program_synthesis",
    "rag",
    "diffusion",
    "adversarial",
    "spike",
    "quantum",
    "neuromorphic",
    "neural_ode",
    "hypernet",
    "active_learning",
    "self_supervised",
    "transfer",
    "enactive",
    "meta_learning",
    "continual",
    "zero_shot",
    "few_shot",
    "compression",
    "federated",
    "distillation",
    "cascade",
    "automl",
    "neural_parser",
    "edge_scorer",
    "emotion_model",
    "sleep_trigger",
    "death_oracle",
    "crawl_picker",
    "warden_detector",
    "graph_v8",
];

impl GARMNode for ParadigmHubNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_paradigm_hub"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.active.len() as f32, self.curriculum_level as f32]
    }
    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        self.curriculum_level = (ctx.global_free_energy as usize / 30).min(2);
        NodeAction::Output(vec![self.active.len() as f32, self.curriculum_level as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.3
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        20.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
enum EcoPhase {
    Germination,
    Expansion,
    Resonance,
    Decoherence,
}

struct Eco {
    phase: EcoPhase,
    coherence: f32,
    energy: f32,
    age: u64,
    temperament: f32,
    knowledge: usize,
}

pub struct EcoSystemNode {
    id: usize,
    ecos: Vec<Eco>,
    pool: f32,
    births: u64,
    deaths: u64,
    pulses: u64,
    internal_fe: f32,
}

impl EcoSystemNode {
    pub fn new(id: usize) -> Self {
        let mut node = Self {
            id,
            ecos: Vec::new(),
            pool: 100.0,
            births: 0,
            deaths: 0,
            pulses: 0,
            internal_fe: 1.0,
        };
        node.germinar(0.2);
        node.germinar(0.7);
        node.germinar(0.5);
        node
    }

    pub fn germinar(&mut self, temperament: f32) {
        self.ecos.push(Eco {
            phase: EcoPhase::Germination,
            coherence: 1.0,
            energy: 20.0,
            age: 0,
            temperament,
            knowledge: 0,
        });
        self.births += 1;
    }

    pub fn pulso_global(&mut self, external_knowledge: usize) -> Vec<String> {
        self.pulses += 1;
        let mut events = Vec::new();
        for eco in &mut self.ecos {
            eco.age += 1;
            let take = (1.0 + eco.temperament * 2.0).min(self.pool);
            self.pool -= take;
            eco.energy += take;
            eco.knowledge += external_knowledge.min(3);
            match eco.phase {
                EcoPhase::Germination if eco.age > 5 => eco.phase = EcoPhase::Expansion,
                EcoPhase::Expansion if eco.age > 15 => eco.phase = EcoPhase::Resonance,
                EcoPhase::Resonance if eco.age > 30 || eco.energy < 5.0 => {
                    eco.phase = EcoPhase::Decoherence
                }
                EcoPhase::Decoherence => eco.coherence -= 0.02 + eco.temperament * 0.01,
                _ => {}
            }
        }
        let before = self.ecos.len();
        self.ecos.retain(|eco| eco.coherence > 0.0);
        let died = before - self.ecos.len();
        if died > 0 {
            self.deaths += died as u64;
            events.push(format!("[ECO-MUERTE] {} ecos disueltos", died));
        }
        self.pool = (self.pool + 0.5).min(200.0);
        if self.ecos.len() < 3 && self.pool > 40.0 {
            self.germinar((self.births % 100) as f32 / 100.0);
            self.pool -= 25.0;
            events.push("[ECO-NACIMIENTO] nuevo eco auto-generado".to_string());
        }
        events
    }

    pub fn informe(&self) -> String {
        format!(
            "[ECO-SISTEMA] vivos={} muertes={} nacimientos={} pool={:.1}",
            self.ecos.len(),
            self.deaths,
            self.births,
            self.pool
        )
    }

    pub fn live_count(&self) -> usize {
        self.ecos.len()
    }

    pub fn birth_count(&self) -> u64 {
        self.births
    }

    pub fn death_count(&self) -> u64 {
        self.deaths
    }

    pub fn pulse_count(&self) -> u64 {
        self.pulses
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let ecos: Vec<_> = self
            .ecos
            .iter()
            .map(|eco| {
                serde_json::json!({
                    "phase": eco_phase_name(eco.phase),
                    "coherence": eco.coherence,
                    "energy": eco.energy,
                    "age": eco.age,
                    "temperament": eco.temperament,
                    "knowledge": eco.knowledge,
                })
            })
            .collect();
        write_json(
            path,
            serde_json::json!({
                "ecos": ecos,
                "pool": self.pool,
                "births": self.births,
                "deaths": self.deaths,
                "pulses": self.pulses,
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.ecos = snapshot
            .get("ecos")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| Eco {
                        phase: parse_eco_phase(
                            item.get("phase")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Germination"),
                        ),
                        coherence: json_f32(item, "coherence", 1.0),
                        energy: json_f32(item, "energy", 20.0),
                        age: item.get("age").and_then(|v| v.as_u64()).unwrap_or(0),
                        temperament: json_f32(item, "temperament", 0.5),
                        knowledge: json_usize(item, "knowledge", 0),
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.pool = json_f32(&snapshot, "pool", 100.0);
        self.births = snapshot
            .get("births")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.ecos.len() as u64);
        self.deaths = snapshot.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0);
        self.pulses = snapshot.get("pulses").and_then(|v| v.as_u64()).unwrap_or(0);
        Ok(())
    }
}

impl GARMNode for EcoSystemNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_ecosystem"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (3usize.saturating_sub(self.ecos.len()) as f32 * 0.2)
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.ecos.len() as f32, self.pool, self.deaths as f32]
    }
    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        let events = self.pulso_global(ctx.neighbor_outputs.len() + ctx.sensor_input.len());
        NodeAction::Output(vec![self.ecos.len() as f32, self.pool, events.len() as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.5
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

pub struct RebirthMeltraceNode {
    id: usize,
    lineage_age: u64,
    rebirths: u32,
    deaths: u32,
    inherited_facts: Vec<String>,
    meltrace_events: Vec<String>,
    internal_fe: f32,
}

impl RebirthMeltraceNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            lineage_age: 0,
            rebirths: 0,
            deaths: 0,
            inherited_facts: Vec::new(),
            meltrace_events: Vec::new(),
            internal_fe: 1.0,
        }
    }

    pub fn rebirth(&mut self, facts: &[String]) -> String {
        self.deaths += 1;
        let death_event = format!(
            "[MELTRACE-DEATH] vida={} lifespan={} facts={}",
            self.rebirths + 1,
            self.lineage_age,
            facts.len()
        );
        self.meltrace_events.push(death_event);
        self.rebirths += 1;
        self.lineage_age = 0;
        self.inherited_facts
            .extend(select_inherited_facts(facts, 32));
        if self.inherited_facts.len() > 100 {
            self.inherited_facts
                .drain(0..self.inherited_facts.len() - 100);
        }
        let event = format!(
            "[REBIRTH] ciclo={} deaths={} inherited={} preserved={}",
            self.rebirths,
            self.deaths,
            self.inherited_facts.len(),
            facts.len().min(32)
        );
        self.meltrace_events.push(event.clone());
        if self.meltrace_events.len() > 100 {
            self.meltrace_events.remove(0);
        }
        event
    }

    pub fn informe(&self) -> String {
        format!(
            "[MELTRACE] rebirths={} deaths={} lineage_age={} inherited={} events={}",
            self.rebirths,
            self.deaths,
            self.lineage_age,
            self.inherited_facts.len(),
            self.meltrace_events.len()
        )
    }

    pub fn rebirth_count(&self) -> u32 {
        self.rebirths
    }

    pub fn death_count(&self) -> u32 {
        self.deaths
    }

    pub fn event_count(&self) -> usize {
        self.meltrace_events.len()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        write_json(
            path,
            serde_json::json!({
                "lineage_age": self.lineage_age,
                "rebirths": self.rebirths,
                "deaths": self.deaths,
                "inherited_facts": self.inherited_facts,
                "meltrace_events": self.meltrace_events,
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.lineage_age = snapshot
            .get("lineage_age")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.rebirths = snapshot
            .get("rebirths")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        self.deaths = snapshot
            .get("deaths")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.rebirths as u64) as u32;
        self.inherited_facts = json_string_array(snapshot.get("inherited_facts"));
        self.meltrace_events = json_string_array(snapshot.get("meltrace_events"));
        Ok(())
    }
}

impl GARMNode for RebirthMeltraceNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_rebirth_meltrace"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.rebirths as f32, self.lineage_age as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        self.lineage_age += 1;
        NodeAction::Output(vec![self.rebirths as f32, self.lineage_age as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        20.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct LegacyCrawlerNode {
    id: usize,
    loaded_files: u64,
    loaded_facts: Vec<String>,
    blocked_remote_requests: u64,
    internal_fe: f32,
}

const MAX_REMOTE_BYTES: usize = 512 * 1024;
const MAX_EXTRACTED_FACTS: usize = 32;

impl LegacyCrawlerNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            loaded_files: 0,
            loaded_facts: Vec::new(),
            blocked_remote_requests: 0,
            internal_fe: 1.0,
        }
    }

    pub fn load_local_kb(&mut self, path: &str) -> usize {
        let Ok(entries) = std::fs::read_dir(path) else {
            return 0;
        };
        let mut loaded = 0;
        for entry in entries.flatten().take(50) {
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("txt") {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                self.loaded_files += 1;
                for line in text
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.len() > 5)
                    .take(20)
                {
                    self.loaded_facts.push(line.to_string());
                    loaded += 1;
                }
            }
        }
        if self.loaded_facts.len() > 1000 {
            self.loaded_facts.drain(0..self.loaded_facts.len() - 1000);
        }
        loaded
    }

    pub fn load_conceptnet(&mut self, path: &str) -> Result<Vec<String>, String> {
        let metadata =
            std::fs::metadata(path).map_err(|e| format!("cannot read {}: {}", path, e))?;
        let mut facts = Vec::new();
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path)
                .map_err(|e| e.to_string())?
                .flatten()
                .take(20)
            {
                if entry.path().is_file() {
                    facts.extend(self.load_conceptnet_file(&entry.path().to_string_lossy())?);
                    if facts.len() >= 500 {
                        break;
                    }
                }
            }
        } else {
            facts = self.load_conceptnet_file(path)?;
        }
        facts.truncate(500);
        self.loaded_facts.extend(facts.iter().cloned());
        if self.loaded_facts.len() > 1000 {
            self.loaded_facts.drain(0..self.loaded_facts.len() - 1000);
        }
        Ok(facts)
    }

    fn load_conceptnet_file(&mut self, path: &str) -> Result<Vec<String>, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))?;
        self.loaded_files += 1;
        let mut facts = Vec::new();
        for line in text.lines().take(20_000) {
            if let Some(fact) = conceptnet_line_to_fact(line) {
                if !facts.iter().any(|existing| existing == &fact) {
                    facts.push(fact);
                    if facts.len() >= 500 {
                        break;
                    }
                }
            }
        }
        Ok(facts)
    }

    pub fn crawl_remote_blocked(&mut self, _url: &str) -> String {
        self.blocked_remote_requests += 1;
        "[CRAWLER] remote crawl blocked; use ToolCalling/McpClient/Sandbox capability gates"
            .to_string()
    }

    pub fn crawl_remote_gated(
        &mut self,
        url: &str,
        allow_remote: bool,
    ) -> Result<Vec<String>, String> {
        if !allow_remote {
            self.blocked_remote_requests += 1;
            return Err("remote crawl blocked; start with --allow-remote-crawl".to_string());
        }
        validate_remote_url(url)?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("eden-garm-safe-crawler/1.0")
            .build()
            .map_err(|e| format!("crawler client error: {}", e))?;
        let mut current_url = url.to_string();
        let mut response = None;
        for redirect_count in 0..=3 {
            validate_remote_url(&current_url)?;
            let candidate = client
                .get(&current_url)
                .send()
                .map_err(|e| format!("crawler fetch error: {}", e))?;
            if candidate.status().is_redirection() {
                if redirect_count == 3 {
                    return Err("crawler redirect limit exceeded".to_string());
                }
                let location = candidate
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| "crawler redirect missing Location".to_string())?;
                current_url = resolve_redirect_url(&current_url, location)?;
                continue;
            }
            response = Some(candidate);
            break;
        }
        let Some(response) = response else {
            return Err("crawler redirect resolution failed".to_string());
        };
        if !response.status().is_success() {
            return Err(format!("crawler HTTP status: {}", response.status()));
        }
        if let Some(content_type) = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
        {
            let lower = content_type.to_lowercase();
            if !(lower.contains("text/")
                || lower.contains("html")
                || lower.contains("xml")
                || lower.contains("json"))
            {
                return Err(format!("crawler content-type blocked: {}", content_type));
            }
        }
        let mut limited = response.take((MAX_REMOTE_BYTES + 1) as u64);
        let mut bytes = Vec::new();
        limited
            .read_to_end(&mut bytes)
            .map_err(|e| format!("crawler read error: {}", e))?;
        if bytes.len() > MAX_REMOTE_BYTES {
            return Err(format!("crawler response too large: {} bytes", bytes.len()));
        }
        let text = String::from_utf8_lossy(&bytes);
        let facts = extract_facts_from_text(&strip_markup(&text));
        self.loaded_facts.extend(facts.iter().cloned());
        if self.loaded_facts.len() > 1000 {
            self.loaded_facts.drain(0..self.loaded_facts.len() - 1000);
        }
        Ok(facts)
    }

    pub fn informe(&self) -> String {
        format!(
            "[CRAWLER] files={} facts={} blocked_remote={}",
            self.loaded_files,
            self.loaded_facts.len(),
            self.blocked_remote_requests
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        write_json(
            path,
            serde_json::json!({
                "loaded_files": self.loaded_files,
                "loaded_facts": self.loaded_facts,
                "blocked_remote_requests": self.blocked_remote_requests,
            }),
        )
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let snapshot = read_json(path)?;
        self.loaded_files = snapshot
            .get("loaded_files")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.loaded_facts = json_string_array(snapshot.get("loaded_facts"));
        self.blocked_remote_requests = snapshot
            .get("blocked_remote_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(())
    }
}

pub fn validate_remote_url(url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("invalid URL: {}", e))?;
    match parsed.scheme() {
        "http" | "https" => {}
        other => return Err(format!("blocked URL scheme: {}", other)),
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "missing URL host".to_string())?;
    let lower = host.to_lowercase();
    if lower == "localhost" || lower.ends_with(".localhost") {
        return Err("blocked localhost host".to_string());
    }
    let port = parsed.port_or_known_default().unwrap_or(80);
    if let Ok(ip) = lower.parse::<IpAddr>() {
        if is_private_or_local_ip(ip) {
            return Err(format!("blocked private/local IP: {}", ip));
        }
        return Ok(());
    }
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed: {}", e))?;
    for addr in addrs {
        if is_private_or_local_ip(addr.ip()) {
            return Err(format!("blocked private/local resolved IP: {}", addr.ip()));
        }
    }
    Ok(())
}

fn resolve_redirect_url(base: &str, location: &str) -> Result<String, String> {
    let base = reqwest::Url::parse(base).map_err(|e| format!("invalid redirect base: {}", e))?;
    let next = base
        .join(location)
        .map_err(|e| format!("invalid redirect Location: {}", e))?;
    let next = next.to_string();
    validate_remote_url(&next)?;
    Ok(next)
}

fn is_private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
        }
    }
}

pub fn strip_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut tag_buf = String::new();
    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
            tag_buf.clear();
            continue;
        }
        if in_tag {
            if ch == '>' {
                let tag = tag_buf.trim().to_lowercase();
                if tag.starts_with("script") || tag.starts_with("style") {
                    in_script = true;
                }
                if tag.starts_with("/script") || tag.starts_with("/style") {
                    in_script = false;
                }
                in_tag = false;
                out.push(' ');
            } else {
                tag_buf.push(ch);
            }
            continue;
        }
        if !in_script {
            out.push(ch);
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

pub fn extract_facts_from_text(text: &str) -> Vec<String> {
    let mut facts = Vec::new();
    for sentence in text.split(['.', '\n', ';']) {
        let clean = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
        if clean.len() < 20 || clean.len() > 240 {
            continue;
        }
        let lower = clean.to_lowercase();
        let informative = lower.contains(" es ")
            || lower.contains(" causa ")
            || lower.contains(" provoca ")
            || lower.contains(" tiene ")
            || lower.contains(" is ")
            || lower.contains(" are ")
            || lower.contains(" causes ")
            || lower.contains(" has ");
        if informative && !facts.iter().any(|fact| fact == &clean) {
            facts.push(clean);
            if facts.len() >= MAX_EXTRACTED_FACTS {
                break;
            }
        }
    }
    facts
}

fn conceptnet_line_to_fact(line: &str) -> Option<String> {
    if conceptnet_weight(line).is_some_and(|w| w <= 0.0) {
        return None;
    }
    let relation = extract_conceptnet_part(line, "/r/")?;
    let concepts = extract_concepts(line);
    if concepts.len() < 2 {
        return None;
    }
    let fact = match relation.as_str() {
        "IsA" | "InstanceOf" => format!("{} es {}", concepts[0], concepts[1]),
        "Causes" | "HasPrerequisite" | "MotivatedByGoal" => {
            format!("{} causa {}", concepts[0], concepts[1])
        }
        "HasProperty" | "HasA" | "PartOf" | "MadeOf" => {
            format!("{} tiene {}", concepts[0], concepts[1])
        }
        "UsedFor" => format!("{} used for {}", concepts[0], concepts[1]),
        "CapableOf" => format!("{} can {}", concepts[0], concepts[1]),
        "LocatedNear" | "AtLocation" | "RelatedTo" | "Synonym" | "SimilarTo" => {
            format!("{} tiene {}", concepts[0], concepts[1])
        }
        "NotIsA" | "Antonym" | "DistinctFrom" => format!("{} no es {}", concepts[0], concepts[1]),
        _ => return None,
    };
    Some(fact)
}

fn extract_concepts(line: &str) -> Vec<String> {
    let mut concepts = Vec::new();
    let mut rest = line;
    while let Some(pos) = rest.find("/c/") {
        rest = &rest[pos + 3..];
        let mut parts = rest.split('/');
        let lang = parts.next().unwrap_or("");
        if lang != "es" && lang != "en" {
            continue;
        }
        let concept = parts
            .next()
            .unwrap_or("")
            .split(['\t', ',', ']', ' '])
            .next()
            .unwrap_or("")
            .replace('_', " ");
        if concept.len() >= 2 && !concepts.iter().any(|c| c == &concept) {
            concepts.push(concept);
        }
        if concepts.len() >= 2 {
            break;
        }
    }
    concepts
}

fn conceptnet_weight(line: &str) -> Option<f32> {
    let key = "\"weight\":";
    let pos = line.find(key)? + key.len();
    let tail = &line[pos..];
    let raw = tail
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .find(|s| !s.is_empty())?;
    raw.parse().ok()
}

fn select_inherited_facts(facts: &[String], limit: usize) -> Vec<String> {
    let mut ranked = facts.to_vec();
    ranked.sort_by_key(|fact| std::cmp::Reverse((fact.len().min(240), relation_weight(fact))));
    ranked.into_iter().take(limit).collect()
}

fn relation_weight(fact: &str) -> usize {
    let lower = fact.to_lowercase();
    usize::from(lower.contains(" causa ") || lower.contains(" causes ")) * 3
        + usize::from(lower.contains(" es ") || lower.contains(" is ")) * 2
        + usize::from(lower.contains(" tiene ") || lower.contains(" has "))
}

fn extract_conceptnet_part(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)? + prefix.len();
    let tail = &line[start..];
    let end = tail.find(['/', '\t', ',', ']', ' ']).unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

fn write_json(path: &str, value: serde_json::Value) -> Result<(), String> {
    std::fs::write(path, value.to_string()).map_err(|e| format!("failed to write {}: {}", path, e))
}

fn read_json(path: &str) -> Result<serde_json::Value, String> {
    let data =
        std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
    serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))
}

fn json_string(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn json_usize(value: &serde_json::Value, key: &str, default: usize) -> usize {
    value
        .get(key)
        .and_then(|v| v.as_u64())
        .unwrap_or(default as u64) as usize
}

fn json_f32(value: &serde_json::Value, key: &str, default: f32) -> f32 {
    value
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default as f64) as f32
}

fn linear_to_json(linear: &autograd::Linear) -> serde_json::Value {
    serde_json::json!({
        "w": linear.w.data,
        "b": linear.b.data,
    })
}

fn load_linear(linear: &mut autograd::Linear, value: &serde_json::Value) {
    if let Some(weights) = value.get("w").and_then(|v| v.as_array()) {
        let restored: Vec<f32> = weights
            .iter()
            .filter_map(|v| v.as_f64().map(|n| n as f32))
            .collect();
        if restored.len() == linear.w.data.len() {
            linear.w.data = restored;
            linear.w.grad = vec![0.0; linear.w.data.len()];
        }
    }
    if let Some(biases) = value.get("b").and_then(|v| v.as_array()) {
        let restored: Vec<f32> = biases
            .iter()
            .filter_map(|v| v.as_f64().map(|n| n as f32))
            .collect();
        if restored.len() == linear.b.data.len() {
            linear.b.data = restored;
            linear.b.grad = vec![0.0; linear.b.data.len()];
        }
    }
}

fn json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_static_paradigm(value: &str) -> Option<&'static str> {
    match value {
        "active" => Some("active"),
        "curriculum" => Some("curriculum"),
        "causal" => Some("causal"),
        "contrastive" => Some("contrastive"),
        "ensemble" => Some("ensemble"),
        "xai" => Some("xai"),
        "gnn" => Some("gnn"),
        "transformer" => Some("transformer"),
        "rl" => Some("rl"),
        "mcts" => Some("mcts"),
        "bayesian" => Some("bayesian"),
        "logic" => Some("logic"),
        "neuro_symbolic" => Some("neuro_symbolic"),
        "program_synthesis" => Some("program_synthesis"),
        "rag" => Some("rag"),
        "diffusion" => Some("diffusion"),
        "adversarial" => Some("adversarial"),
        "spike" => Some("spike"),
        "quantum" => Some("quantum"),
        "neuromorphic" => Some("neuromorphic"),
        "neural_ode" => Some("neural_ode"),
        "hypernet" => Some("hypernet"),
        "active_learning" => Some("active_learning"),
        "self_supervised" => Some("self_supervised"),
        "transfer" => Some("transfer"),
        "enactive" => Some("enactive"),
        "meta_learning" => Some("meta_learning"),
        "continual" => Some("continual"),
        "zero_shot" => Some("zero_shot"),
        "few_shot" => Some("few_shot"),
        "compression" => Some("compression"),
        "federated" => Some("federated"),
        "distillation" => Some("distillation"),
        "cascade" => Some("cascade"),
        "automl" => Some("automl"),
        "neural_parser" => Some("neural_parser"),
        "edge_scorer" => Some("edge_scorer"),
        "emotion_model" => Some("emotion_model"),
        "sleep_trigger" => Some("sleep_trigger"),
        "death_oracle" => Some("death_oracle"),
        "crawl_picker" => Some("crawl_picker"),
        "warden_detector" => Some("warden_detector"),
        "graph_v8" => Some("graph_v8"),
        _ => None,
    }
}

fn eco_phase_name(phase: EcoPhase) -> &'static str {
    match phase {
        EcoPhase::Germination => "Germination",
        EcoPhase::Expansion => "Expansion",
        EcoPhase::Resonance => "Resonance",
        EcoPhase::Decoherence => "Decoherence",
    }
}

fn parse_eco_phase(value: &str) -> EcoPhase {
    match value {
        "Expansion" => EcoPhase::Expansion,
        "Resonance" => EcoPhase::Resonance,
        "Decoherence" => EcoPhase::Decoherence,
        _ => EcoPhase::Germination,
    }
}

impl GARMNode for LegacyCrawlerNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_crawler"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.loaded_files as f32, self.loaded_facts.len() as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        NodeAction::Output(vec![
            self.loaded_files as f32,
            self.loaded_facts.len() as f32,
        ])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.2
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        20.0
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

    #[test]
    fn autoconsumo_extracts_architecture_fragments() {
        let mut node = AutoconsumoNode::new(1);
        let fragments = node.nutrirse();
        assert!(!fragments.is_empty());
        assert!(node.informe().contains("AUTOCONSUMO"));
    }

    #[test]
    fn venado_roundtrips_cristal_fields() {
        let _state_guard = crate::eden_garm::state_paths::test_state_guard();
        crate::eden_garm::state_paths::set_state_dir(
            std::env::temp_dir().join(format!("eden_garm_venado_test_{}", std::process::id())),
        );
        let mut node = VenadoCompatibilityNode::new(2);
        node.cristalizar_con_bloques(
            "test",
            &[("clave".to_string(), "valor".to_string())],
            &[(
                "bloque".to_string(),
                vec!["linea uno".to_string(), "linea dos".to_string()],
            )],
        )
        .unwrap();
        let fields = node.descristalizar("test").unwrap();
        assert_eq!(fields, vec![("clave".to_string(), "valor".to_string())]);
        let (_, blocks) = node.descristalizar_completo("test").unwrap();
        assert_eq!(
            blocks,
            vec![(
                "bloque".to_string(),
                vec!["linea uno".to_string(), "linea dos".to_string()]
            )]
        );
        assert!(node.existe("test"));
        assert!(node.lista_venas().contains(&"test".to_string()));
    }

    #[test]
    fn paradigm_ecosystem_rebirth_and_crawler_are_active() {
        let mut hub = ParadigmHubNode::new(3);
        assert_eq!(hub.paradigm_count(), 43);
        let selected = hub.active_select_page(&HashMap::from([
            ("ai".to_string(), 0.9),
            ("bio".to_string(), 0.2),
        ]));
        assert_eq!(selected.as_deref(), Some("ai"));
        assert!(hub.curriculum_topic(60).contains("Quantum"));
        let outcome = hub.run_paradigm_cycle(2_000, 100, 0.4, 17);
        assert!(!outcome.activations.is_empty());
        assert!(!outcome.inferred_facts.is_empty());
        assert_eq!(outcome.autograd_models_trained, 8);
        assert!(hub.informe().contains("active_snapshot=43/43"));
        assert!(hub.informe().contains("autograd=8/8"));

        let mut eco = EcoSystemNode::new(4);
        assert!(!eco.pulso_global(2).iter().any(|event| event.is_empty()));
        assert!(eco.informe().contains("ECO-SISTEMA"));

        let mut rebirth = RebirthMeltraceNode::new(5);
        assert!(rebirth.rebirth(&["fact".to_string()]).contains("REBIRTH"));
        assert_eq!(rebirth.rebirth_count(), 1);
        assert_eq!(rebirth.death_count(), 1);
        assert_eq!(rebirth.event_count(), 2);

        let mut crawler = LegacyCrawlerNode::new(6);
        assert!(crawler
            .crawl_remote_blocked("https://example.invalid")
            .contains("blocked"));
    }

    #[test]
    fn remote_crawler_safety_and_extraction_work() {
        assert!(validate_remote_url("file:///etc/passwd").is_err());
        assert!(validate_remote_url("http://127.0.0.1:8080").is_err());
        assert!(validate_remote_url("http://localhost:8080").is_err());
        assert!(resolve_redirect_url("https://93.184.216.34/a", "http://127.0.0.1/").is_err());
        assert!(resolve_redirect_url("https://93.184.216.34/a", "/safe").is_ok());
        let text = strip_markup("<html><script>bad()</script><body>Rust is a systems language. EDEN tiene memoria persistente.</body></html>");
        let facts = extract_facts_from_text(&text);
        assert!(facts.iter().any(|fact| fact.contains("Rust is")));
        assert!(facts.iter().any(|fact| fact.contains("EDEN tiene")));
    }

    #[test]
    fn conceptnet_loader_extracts_structured_relations() {
        let path =
            std::env::temp_dir().join(format!("eden_garm_conceptnet_{}.tsv", std::process::id()));
        std::fs::write(
            &path,
            "/a/[/r/IsA/,/c/es/perro/,/c/es/animal/]\t/r/IsA\t/c/es/perro\t/c/es/animal\t{\"weight\":2.0}\n/a/[/r/CapableOf/,/c/en/bird/,/c/en/fly/]\t/r/CapableOf\t/c/en/bird\t/c/en/fly\t{\"weight\":1.0}\n/a/[/r/Causes/,/c/es/fuego/,/c/es/calor/]\n",
        ).unwrap();
        let mut crawler = LegacyCrawlerNode::new(7);
        let facts = crawler.load_conceptnet(&path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(path);
        assert!(facts.contains(&"perro es animal".to_string()));
        assert!(facts.contains(&"bird can fly".to_string()));
        assert!(facts.contains(&"fuego causa calor".to_string()));
    }
}
