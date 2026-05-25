use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};
use std::collections::{HashMap, VecDeque};

const DEFAULT_TTL_TICKS: u64 = 32;
const MAX_CACHE_ENTRIES: usize = 128;
const MAX_LEARNED_SUMMARIES: usize = 64;
const MAX_ACTIONS: usize = 128;
const MAX_AUDIT_ENTRIES: usize = 128;
const MAX_ITEMS_PER_SOURCE: usize = 5;
const MAX_CONTEXT_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
pub struct ContextPack {
    pub query: String,
    pub created_tick: u64,
    pub ttl_ticks: u64,
    pub cache_hit: bool,
    pub memory_facts: Vec<String>,
    pub kg_hits: Vec<(String, f32)>,
    pub path_explanations: Vec<String>,
    pub history_fragments: Vec<String>,
    pub sources: Vec<String>,
    pub trace: Vec<String>,
    pub context_quality: f32,
    pub quality_label: String,
}

pub struct ContextAugmentationNode {
    id: usize,
    cache: HashMap<String, ContextPack>,
    order: VecDeque<String>,
    ttl_ticks: u64,
    hits: u64,
    misses: u64,
    feedback_positive: u64,
    feedback_negative: u64,
    learned_summaries: VecDeque<String>,
    weak_contexts: HashMap<String, u32>,
    actions: VecDeque<CagAction>,
    audit: VecDeque<CagAuditEntry>,
    next_action_id: u64,
    actions_executed: u64,
    actions_blocked: u64,
    autonomous_runs: u64,
    internal_fe: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CagAction {
    pub id: u64,
    pub query: String,
    pub kind: String,
    pub status: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CagAuditEntry {
    pub action_id: u64,
    pub query: String,
    pub kind: String,
    pub status: String,
    pub reason: String,
    pub mode: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContextAugmentationMetrics {
    pub cache_entries: u64,
    pub hits: u64,
    pub misses: u64,
    pub ttl_ticks: u64,
    pub feedback_positive: u64,
    pub feedback_negative: u64,
    pub learned_summaries: u64,
    pub weak_contexts: u64,
    pub pending_actions: u64,
    pub actions_executed: u64,
    pub actions_blocked: u64,
    pub autonomous_runs: u64,
}

impl ContextAugmentationNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            cache: HashMap::new(),
            order: VecDeque::new(),
            ttl_ticks: DEFAULT_TTL_TICKS,
            hits: 0,
            misses: 0,
            feedback_positive: 0,
            feedback_negative: 0,
            learned_summaries: VecDeque::new(),
            weak_contexts: HashMap::new(),
            actions: VecDeque::new(),
            audit: VecDeque::new(),
            next_action_id: 1,
            actions_executed: 0,
            actions_blocked: 0,
            autonomous_runs: 0,
            internal_fe: 1.0,
        }
    }

    pub fn build_context(
        &mut self,
        query: &str,
        tick: u64,
        memory_facts: &[String],
        kg_hits: &[(String, f32)],
        path_explanations: &[String],
        history_fragments: &[String],
    ) -> ContextPack {
        let key = normalize_query(query);
        self.expire(tick);
        if let Some(pack) = self.cache.get(&key) {
            if tick.saturating_sub(pack.created_tick) <= pack.ttl_ticks {
                self.hits += 1;
                let mut hit = pack.clone();
                hit.cache_hit = true;
                hit.trace.push(format!("cache:hit tick={}", tick));
                return hit;
            }
        }

        self.misses += 1;
        let mut pack = ContextPack {
            query: query.trim().to_string(),
            created_tick: tick,
            ttl_ticks: self.ttl_ticks,
            cache_hit: false,
            memory_facts: trim_strings(memory_facts),
            kg_hits: kg_hits.iter().take(MAX_ITEMS_PER_SOURCE).cloned().collect(),
            path_explanations: trim_strings(path_explanations),
            history_fragments: trim_strings(history_fragments),
            sources: Vec::new(),
            trace: vec![format!("cache:miss tick={}", tick)],
            context_quality: 0.0,
            quality_label: "low".to_string(),
        };
        if !pack.memory_facts.is_empty() {
            pack.sources.push("memory".to_string());
            pack.trace
                .push(format!("memory:facts={}", pack.memory_facts.len()));
        }
        if !pack.kg_hits.is_empty() || !pack.path_explanations.is_empty() {
            pack.sources.push("knowledge_graph".to_string());
            pack.trace.push(format!(
                "kg:hits={} paths={}",
                pack.kg_hits.len(),
                pack.path_explanations.len()
            ));
        }
        if !pack.history_fragments.is_empty() {
            pack.sources.push("history".to_string());
            pack.trace.push(format!(
                "history:fragments={}",
                pack.history_fragments.len()
            ));
        }
        let (context_quality, quality_label) = score_context(&pack, tick);
        pack.context_quality = context_quality;
        pack.quality_label = quality_label;
        pack.trace.push(format!(
            "quality:{:.2} label={}",
            pack.context_quality, pack.quality_label
        ));
        enforce_context_budget(&mut pack);

        self.cache.insert(key.clone(), pack.clone());
        self.order.push_back(key);
        self.enforce_capacity();
        pack
    }

    pub fn report(&self) -> String {
        let last_pack = self.order.back().and_then(|key| self.cache.get(key));
        let last_query = last_pack.map(|pack| pack.query.as_str()).unwrap_or("none");
        let last_sources = last_pack
            .map(|pack| {
                if pack.sources.is_empty() {
                    "none".to_string()
                } else {
                    pack.sources.join(",")
                }
            })
            .unwrap_or_else(|| "none".to_string());
        format!(
            "[CAG] cache_entries={} hits={} misses={} ttl_ticks={} feedback=+{}/-{} learned={} weak={} actions={} last_query='{}' sources={}",
            self.cache.len(),
            self.hits,
            self.misses,
            self.ttl_ticks,
            self.feedback_positive,
            self.feedback_negative,
            self.learned_summaries.len(),
            self.weak_contexts.len(),
            self.actions.len(),
            last_query,
            last_sources
        )
    }

    pub fn metrics(&self) -> ContextAugmentationMetrics {
        ContextAugmentationMetrics {
            cache_entries: self.cache.len() as u64,
            hits: self.hits,
            misses: self.misses,
            ttl_ticks: self.ttl_ticks,
            feedback_positive: self.feedback_positive,
            feedback_negative: self.feedback_negative,
            learned_summaries: self.learned_summaries.len() as u64,
            weak_contexts: self.weak_contexts.len() as u64,
            pending_actions: self
                .actions
                .iter()
                .filter(|action| action.status == "pending")
                .count() as u64,
            actions_executed: self.actions_executed,
            actions_blocked: self.actions_blocked,
            autonomous_runs: self.autonomous_runs,
        }
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn record_feedback(&mut self, query: &str, useful: bool) {
        if useful {
            self.feedback_positive += 1;
            self.ttl_ticks = (self.ttl_ticks + 1).min(128);
        } else {
            self.feedback_negative += 1;
            self.ttl_ticks = self.ttl_ticks.saturating_sub(1).max(8);
        }
        let key = normalize_query(query);
        if let Some(pack) = self.cache.get_mut(&key) {
            pack.trace.push(format!(
                "feedback:{} ttl_ticks={}",
                if useful { "positive" } else { "negative" },
                self.ttl_ticks
            ));
            if useful && pack.quality_label == "high" {
                let summary = learned_summary(pack);
                if !self.learned_summaries.iter().any(|item| item == &summary) {
                    self.learned_summaries.push_back(summary);
                    while self.learned_summaries.len() > MAX_LEARNED_SUMMARIES {
                        self.learned_summaries.pop_front();
                    }
                }
            }
            if !useful || pack.quality_label == "low" {
                *self.weak_contexts.entry(key).or_insert(0) += 1;
                pack.trace.push("decay:weak_context".to_string());
            }
        }
    }

    pub fn learned_summaries(&self) -> Vec<String> {
        self.learned_summaries.iter().cloned().collect()
    }

    pub fn explain(&self, query: &str) -> String {
        let key = normalize_query(query);
        let Some(pack) = self.cache.get(&key) else {
            return format!(
                "[CAG-EXPLAIN] query='{}' cache=miss diagnostic=no_cached_context",
                query.trim()
            );
        };
        format!(
            "[CAG-EXPLAIN] query='{}' quality={:.2} label={} sources={} memory={} kg_hits={} paths={} history={} trace={}",
            pack.query,
            pack.context_quality,
            pack.quality_label,
            if pack.sources.is_empty() { "none".to_string() } else { pack.sources.join(",") },
            pack.memory_facts.len(),
            pack.kg_hits.len(),
            pack.path_explanations.len(),
            pack.history_fragments.len(),
            pack.trace.join("; ")
        )
    }

    pub fn gaps(&self, query: &str) -> String {
        let key = normalize_query(query);
        let Some(pack) = self.cache.get(&key) else {
            return format!(
                "[CAG-GAPS] query='{}' gaps=no_cached_context recommendations=recuerda|conceptnet|crawl-gated",
                query.trim()
            );
        };
        let mut gaps = Vec::new();
        let mut recommendations = Vec::new();
        if pack.kg_hits.is_empty() && pack.path_explanations.is_empty() {
            gaps.push("sin_kg");
            recommendations.push("conceptnet");
        }
        if pack.memory_facts.is_empty() {
            gaps.push("sin_memoria");
            recommendations.push("recuerda");
        }
        if pack.history_fragments.is_empty() {
            gaps.push("sin_historial");
        }
        if pack.sources.len() < 2 {
            gaps.push("baja_diversidad");
            recommendations.push("crawl-gated");
        }
        if self.weak_contexts.get(&key).copied().unwrap_or(0) > 0 {
            gaps.push("contexto_debil_recurrente");
            recommendations.push("validar_con_juez");
        }
        if gaps.is_empty() {
            gaps.push("sin_brechas_criticas");
            recommendations.push("reutilizar_contexto");
        }
        gaps.sort_unstable();
        gaps.dedup();
        recommendations.sort_unstable();
        recommendations.dedup();
        format!(
            "[CAG-GAPS] query='{}' quality={:.2} label={} gaps={} recommendations={}",
            pack.query,
            pack.context_quality,
            pack.quality_label,
            gaps.join(","),
            recommendations.join("|")
        )
    }

    pub fn plan_actions(&mut self, query: &str) -> String {
        let key = normalize_query(query);
        let Some(pack) = self.cache.get(&key).cloned() else {
            self.push_action(query, "prompt_remember", "blocked", "no_cached_context");
            return self.actions_report();
        };
        if pack.memory_facts.is_empty() {
            self.push_action(&pack.query, "prompt_remember", "pending", "sin_memoria");
        }
        if pack.kg_hits.is_empty() && pack.path_explanations.is_empty() {
            self.push_action(
                &pack.query,
                "prioritize_local_conceptnet",
                "pending",
                "sin_kg",
            );
        }
        if pack.sources.len() < 2 {
            self.push_action(
                &pack.query,
                "crawl_gated",
                "blocked",
                "baja_diversidad_requires_remote_flag",
            );
        }
        if self.weak_contexts.get(&key).copied().unwrap_or(0) > 0 {
            self.push_action(
                &pack.query,
                "validate_with_juez",
                "pending",
                "contexto_debil_recurrente",
            );
        }
        self.actions_report()
    }

    pub fn plan_weak_contexts(&mut self, limit: usize) {
        let queries = self.prioritized_weak_queries(limit);
        for query in queries {
            self.plan_actions(&query);
        }
    }

    pub fn prioritized_weak_queries(&self, limit: usize) -> Vec<String> {
        let newest_tick = self
            .cache
            .values()
            .map(|pack| pack.created_tick)
            .max()
            .unwrap_or(0);
        let mut scored = self
            .weak_contexts
            .iter()
            .map(|(query, count)| {
                let pack = self.cache.get(query);
                let missing_kg = pack
                    .map(|pack| pack.kg_hits.is_empty() && pack.path_explanations.is_empty())
                    .unwrap_or(true) as u64;
                let missing_memory = pack
                    .map(|pack| pack.memory_facts.is_empty())
                    .unwrap_or(true) as u64;
                let low_quality =
                    pack.map(|pack| pack.quality_label == "low").unwrap_or(true) as u64;
                let age = pack
                    .map(|pack| newest_tick.saturating_sub(pack.created_tick))
                    .unwrap_or(newest_tick);
                let score = (*count as u64 * 100)
                    + missing_kg * 30
                    + missing_memory * 25
                    + low_quality * 20
                    + age.min(50);
                (score, query.clone())
            })
            .collect::<Vec<_>>();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, query)| query)
            .collect()
    }

    pub fn actions_report(&self) -> String {
        if self.actions.is_empty() {
            return "[CAG-ACTIONS] empty".to_string();
        }
        let mut out = String::from("[CAG-ACTIONS]\n");
        for action in self.actions.iter().rev().take(12).rev() {
            out.push_str(&format!(
                "- id={} query='{}' kind={} status={} reason={}\n",
                action.id, action.query, action.kind, action.status, action.reason
            ));
        }
        out.trim_end().to_string()
    }

    pub fn take_runnable_actions(&mut self, query: &str) -> Vec<CagAction> {
        let key = normalize_query(query);
        let mut runnable = Vec::new();
        for action in &mut self.actions {
            if normalize_query(&action.query) == key
                && action.status == "pending"
                && is_local_safe_action(&action.kind)
            {
                action.status = "running".to_string();
                runnable.push(action.clone());
            }
        }
        runnable
    }

    pub fn take_autonomous_safe_actions(&mut self, limit: usize) -> Vec<CagAction> {
        let mut runnable = Vec::new();
        for action in &mut self.actions {
            if runnable.len() >= limit {
                break;
            }
            if action.status == "pending" && is_local_safe_action(&action.kind) {
                action.status = "running".to_string();
                runnable.push(action.clone());
            }
        }
        if !runnable.is_empty() {
            self.autonomous_runs += 1;
        }
        runnable
    }

    pub fn complete_action(&mut self, id: u64, status: &str, reason: &str) {
        self.complete_action_with_mode(id, status, reason, "manual");
    }

    pub fn complete_action_with_mode(&mut self, id: u64, status: &str, reason: &str, mode: &str) {
        if let Some(action) = self.actions.iter_mut().find(|action| action.id == id) {
            action.status = status.to_string();
            action.reason = reason.to_string();
            if status == "executed" {
                self.actions_executed += 1;
            } else if status == "blocked" {
                self.actions_blocked += 1;
            }
            self.audit.push_back(CagAuditEntry {
                action_id: action.id,
                query: action.query.clone(),
                kind: action.kind.clone(),
                status: action.status.clone(),
                reason: action.reason.clone(),
                mode: mode.to_string(),
            });
            while self.audit.len() > MAX_AUDIT_ENTRIES {
                self.audit.pop_front();
            }
        }
    }

    pub fn audit_report(&self) -> String {
        if self.audit.is_empty() {
            return format!(
                "[CAG-AUDIT] empty executed={} blocked={} autonomous_runs={}",
                self.actions_executed, self.actions_blocked, self.autonomous_runs
            );
        }
        let mut out = format!(
            "[CAG-AUDIT] executed={} blocked={} autonomous_runs={}\n",
            self.actions_executed, self.actions_blocked, self.autonomous_runs
        );
        for entry in self.audit.iter().rev().take(12).rev() {
            out.push_str(&format!(
                "- id={} mode={} query='{}' kind={} status={} reason={}\n",
                entry.action_id, entry.mode, entry.query, entry.kind, entry.status, entry.reason
            ));
        }
        out.trim_end().to_string()
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let entries: Vec<_> = self
            .order
            .iter()
            .filter_map(|key| self.cache.get(key).map(|pack| (key, pack)))
            .map(|(key, pack)| {
                serde_json::json!({
                    "key": key,
                    "pack": context_pack_to_json(pack),
                })
            })
            .collect();
        let snapshot = serde_json::json!({
            "ttl_ticks": self.ttl_ticks,
            "hits": self.hits,
            "misses": self.misses,
            "feedback_positive": self.feedback_positive,
            "feedback_negative": self.feedback_negative,
            "learned_summaries": self.learned_summaries.iter().cloned().collect::<Vec<_>>(),
            "weak_contexts": self.weak_contexts.iter().map(|(query, count)| serde_json::json!({"query": query, "count": count})).collect::<Vec<_>>(),
            "next_action_id": self.next_action_id,
            "actions_executed": self.actions_executed,
            "actions_blocked": self.actions_blocked,
            "autonomous_runs": self.autonomous_runs,
            "actions": self.actions.iter().map(action_to_json).collect::<Vec<_>>(),
            "audit": self.audit.iter().map(audit_to_json).collect::<Vec<_>>(),
            "cache_entries": self.cache.len(),
            "entries": entries,
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.ttl_ticks = snapshot
            .get("ttl_ticks")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_TTL_TICKS);
        self.hits = snapshot.get("hits").and_then(|v| v.as_u64()).unwrap_or(0);
        self.misses = snapshot.get("misses").and_then(|v| v.as_u64()).unwrap_or(0);
        self.feedback_positive = snapshot
            .get("feedback_positive")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.feedback_negative = snapshot
            .get("feedback_negative")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.learned_summaries = snapshot
            .get("learned_summaries")
            .map(json_string_array)
            .unwrap_or_default()
            .into_iter()
            .take(MAX_LEARNED_SUMMARIES)
            .collect();
        self.weak_contexts.clear();
        if let Some(items) = snapshot.get("weak_contexts").and_then(|v| v.as_array()) {
            for item in items {
                if let (Some(query), Some(count)) = (
                    item.get("query").and_then(|v| v.as_str()),
                    item.get("count").and_then(|v| v.as_u64()),
                ) {
                    self.weak_contexts.insert(query.to_string(), count as u32);
                }
            }
        }
        self.next_action_id = snapshot
            .get("next_action_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        self.actions_executed = snapshot
            .get("actions_executed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.actions_blocked = snapshot
            .get("actions_blocked")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.autonomous_runs = snapshot
            .get("autonomous_runs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.actions.clear();
        if let Some(actions) = snapshot.get("actions").and_then(|v| v.as_array()) {
            for action in actions
                .iter()
                .filter_map(action_from_json)
                .take(MAX_ACTIONS)
            {
                self.actions.push_back(action);
            }
        }
        self.audit.clear();
        if let Some(audit) = snapshot.get("audit").and_then(|v| v.as_array()) {
            for entry in audit
                .iter()
                .filter_map(audit_from_json)
                .take(MAX_AUDIT_ENTRIES)
            {
                self.audit.push_back(entry);
            }
        }
        self.cache.clear();
        self.order.clear();
        if let Some(entries) = snapshot.get("entries").and_then(|v| v.as_array()) {
            for entry in entries.iter().take(MAX_CACHE_ENTRIES) {
                let Some(key) = entry.get("key").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(pack) = entry.get("pack").and_then(context_pack_from_json) else {
                    continue;
                };
                self.cache.insert(key.to_string(), pack);
                self.order.push_back(key.to_string());
            }
        }
        Ok(())
    }

    fn expire(&mut self, tick: u64) {
        self.cache
            .retain(|_, pack| tick.saturating_sub(pack.created_tick) <= pack.ttl_ticks);
        self.order.retain(|key| self.cache.contains_key(key));
    }

    fn enforce_capacity(&mut self) {
        while self.cache.len() > MAX_CACHE_ENTRIES {
            if let Some(oldest) = self.order.pop_front() {
                self.cache.remove(&oldest);
            } else {
                break;
            }
        }
    }

    fn push_action(&mut self, query: &str, kind: &str, status: &str, reason: &str) {
        let normalized = normalize_query(query);
        if self.actions.iter().any(|action| {
            normalize_query(&action.query) == normalized
                && action.kind == kind
                && action.status == status
        }) {
            return;
        }
        let action = CagAction {
            id: self.next_action_id,
            query: query.trim().to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
        };
        self.next_action_id += 1;
        self.actions.push_back(action);
        while self.actions.len() > MAX_ACTIONS {
            self.actions.pop_front();
        }
    }
}

fn action_to_json(action: &CagAction) -> serde_json::Value {
    serde_json::json!({
        "id": action.id,
        "query": action.query,
        "kind": action.kind,
        "status": action.status,
        "reason": action.reason,
    })
}

fn action_from_json(value: &serde_json::Value) -> Option<CagAction> {
    Some(CagAction {
        id: value.get("id")?.as_u64()?,
        query: value.get("query")?.as_str()?.to_string(),
        kind: value.get("kind")?.as_str()?.to_string(),
        status: value.get("status")?.as_str()?.to_string(),
        reason: value.get("reason")?.as_str()?.to_string(),
    })
}

fn audit_to_json(entry: &CagAuditEntry) -> serde_json::Value {
    serde_json::json!({
        "action_id": entry.action_id,
        "query": entry.query,
        "kind": entry.kind,
        "status": entry.status,
        "reason": entry.reason,
        "mode": entry.mode,
    })
}

fn audit_from_json(value: &serde_json::Value) -> Option<CagAuditEntry> {
    Some(CagAuditEntry {
        action_id: value.get("action_id")?.as_u64()?,
        query: value.get("query")?.as_str()?.to_string(),
        kind: value.get("kind")?.as_str()?.to_string(),
        status: value.get("status")?.as_str()?.to_string(),
        reason: value.get("reason")?.as_str()?.to_string(),
        mode: value.get("mode")?.as_str()?.to_string(),
    })
}

fn learned_summary(pack: &ContextPack) -> String {
    format!(
        "CAG good context: query='{}' quality={:.2} sources={}",
        pack.query,
        pack.context_quality,
        if pack.sources.is_empty() {
            "none".to_string()
        } else {
            pack.sources.join(",")
        }
    )
}

fn context_pack_to_json(pack: &ContextPack) -> serde_json::Value {
    serde_json::json!({
        "query": &pack.query,
        "created_tick": pack.created_tick,
        "ttl_ticks": pack.ttl_ticks,
        "cache_hit": pack.cache_hit,
        "memory_facts": &pack.memory_facts,
        "kg_hits": pack.kg_hits.iter().map(|(name, score)| serde_json::json!({"name": name, "score": score})).collect::<Vec<_>>(),
        "path_explanations": &pack.path_explanations,
        "history_fragments": &pack.history_fragments,
        "sources": &pack.sources,
        "trace": &pack.trace,
        "context_quality": pack.context_quality,
        "quality_label": &pack.quality_label,
    })
}

fn context_pack_from_json(value: &serde_json::Value) -> Option<ContextPack> {
    Some(ContextPack {
        query: value.get("query")?.as_str()?.to_string(),
        created_tick: value.get("created_tick")?.as_u64()?,
        ttl_ticks: value.get("ttl_ticks")?.as_u64()?,
        cache_hit: value
            .get("cache_hit")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        memory_facts: json_string_array(value.get("memory_facts")?),
        kg_hits: value
            .get("kg_hits")?
            .as_array()?
            .iter()
            .filter_map(|item| {
                Some((
                    item.get("name")?.as_str()?.to_string(),
                    item.get("score")?.as_f64()? as f32,
                ))
            })
            .take(MAX_ITEMS_PER_SOURCE)
            .collect(),
        path_explanations: json_string_array(value.get("path_explanations")?),
        history_fragments: json_string_array(value.get("history_fragments")?),
        sources: json_string_array(value.get("sources")?),
        trace: json_string_array(value.get("trace")?),
        context_quality: value
            .get("context_quality")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        quality_label: value
            .get("quality_label")
            .and_then(|v| v.as_str())
            .unwrap_or("low")
            .to_string(),
    })
}

fn json_string_array(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn score_context(pack: &ContextPack, tick: u64) -> (f32, String) {
    let kg_score = ((pack.kg_hits.len() + pack.path_explanations.len()) as f32 / 6.0).min(1.0);
    let memory_score = (pack.memory_facts.len() as f32 / 3.0).min(1.0);
    let source_score = (pack.sources.len() as f32 / 3.0).min(1.0);
    let freshness = 1.0
        - (tick.saturating_sub(pack.created_tick) as f32 / pack.ttl_ticks.max(1) as f32).min(1.0);
    let quality = (kg_score * 0.4 + memory_score * 0.25 + source_score * 0.2 + freshness * 0.15)
        .clamp(0.0, 1.0);
    let label = if quality >= 0.7 {
        "high"
    } else if quality >= 0.35 {
        "medium"
    } else {
        "low"
    };
    (quality, label.to_string())
}

fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn is_local_safe_action(kind: &str) -> bool {
    matches!(
        kind,
        "prompt_remember" | "prioritize_local_conceptnet" | "validate_with_juez"
    )
}

fn trim_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            let clean = value.trim();
            if clean.is_empty() {
                None
            } else {
                Some(clean.to_string())
            }
        })
        .take(MAX_ITEMS_PER_SOURCE)
        .collect()
}

fn enforce_context_budget(pack: &mut ContextPack) {
    while context_size(pack) > MAX_CONTEXT_BYTES {
        if pack.history_fragments.pop().is_some()
            || pack.path_explanations.pop().is_some()
            || pack.memory_facts.pop().is_some()
            || pack.kg_hits.pop().is_some()
        {
            continue;
        }
        break;
    }
}

fn context_size(pack: &ContextPack) -> usize {
    pack.query.len()
        + pack.memory_facts.iter().map(String::len).sum::<usize>()
        + pack
            .kg_hits
            .iter()
            .map(|(name, _)| name.len())
            .sum::<usize>()
        + pack
            .path_explanations
            .iter()
            .map(String::len)
            .sum::<usize>()
        + pack
            .history_fragments
            .iter()
            .map(String::len)
            .sum::<usize>()
        + pack.sources.iter().map(String::len).sum::<usize>()
        + pack.trace.iter().map(String::len).sum::<usize>()
}

impl GARMNode for ContextAugmentationNode {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        "context_augmentation"
    }

    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }

    fn free_energy(&self) -> f32 {
        self.internal_fe + (self.cache.len() as f32).ln_1p() * 0.01
    }

    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.cache.len() as f32,
            self.hits as f32,
            self.misses as f32,
        ]
    }

    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.03).min(5.0);
        }
        NodeAction::Output(vec![
            self.cache.len() as f32,
            self.hits as f32,
            self.misses as f32,
        ])
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
    use super::ContextAugmentationNode;

    #[test]
    fn build_context_records_cache_miss_then_hit() {
        let mut node = ContextAugmentationNode::new(1);
        let memory = vec!["eden is local".to_string()];
        let kg_hits = vec![("eden".to_string(), 1.0)];
        let paths = vec!["eden -is-> local".to_string()];
        let history = vec!["cmd: que es eden".to_string()];

        let miss = node.build_context("eden", 1, &memory, &kg_hits, &paths, &history);
        let hit = node.build_context(" eden ", 2, &[], &[], &[], &[]);

        assert!(!miss.cache_hit);
        assert!(hit.cache_hit);
        assert!(hit.trace.iter().any(|line| line.contains("cache:hit")));
    }

    #[test]
    fn build_context_expires_after_ttl() {
        let mut node = ContextAugmentationNode::new(1);
        let memory = vec!["eden is local".to_string()];

        let first = node.build_context("eden", 1, &memory, &[], &[], &[]);
        let expired = node.build_context("eden", 40, &[], &[], &[], &[]);

        assert!(!first.cache_hit);
        assert!(!expired.cache_hit);
    }

    #[test]
    fn build_context_traces_all_sources() {
        let mut node = ContextAugmentationNode::new(1);
        let memory = vec!["eden is local".to_string()];
        let kg_hits = vec![("eden".to_string(), 1.0)];
        let paths = vec!["eden -is-> local".to_string()];
        let history = vec!["cmd: lengua eden".to_string()];

        let pack = node.build_context("eden", 1, &memory, &kg_hits, &paths, &history);

        assert_eq!(pack.sources, vec!["memory", "knowledge_graph", "history"]);
        assert!(pack
            .trace
            .iter()
            .any(|line| line.contains("kg:hits=1 paths=1")));
        assert!(pack.context_quality >= 0.35);
        assert_eq!(pack.quality_label, "medium");
    }

    #[test]
    fn saves_loads_cache_and_metrics() {
        let path =
            std::env::temp_dir().join(format!("eden_garm_cag_test_{}.json", std::process::id()));
        let path_str = path.to_string_lossy().to_string();
        let mut source = ContextAugmentationNode::new(1);
        let memory = vec!["eden is local".to_string()];
        source.build_context("eden", 1, &memory, &[], &[], &[]);
        source.build_context("eden", 2, &[], &[], &[], &[]);
        source.build_context("unknown", 3, &[], &[], &[], &[]);
        source.record_feedback("unknown", false);
        source.plan_actions("unknown");
        let action = source.take_autonomous_safe_actions(1).remove(0);
        source.complete_action_with_mode(action.id, "executed", "test_execution", "autonomous");

        source.save_state(&path_str).unwrap();
        let mut restored = ContextAugmentationNode::new(2);
        restored.load_state(&path_str).unwrap();
        let metrics = restored.metrics();

        assert_eq!(metrics.cache_entries, 2);
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 2);
        assert_eq!(metrics.actions_executed, 1);
        assert_eq!(metrics.autonomous_runs, 1);
        assert!(restored.audit_report().contains("mode=autonomous"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn feedback_learns_good_context_and_marks_gaps() {
        let mut node = ContextAugmentationNode::new(1);
        let memory = vec![
            "eden is local".to_string(),
            "eden has memory".to_string(),
            "eden has graph".to_string(),
        ];
        let kg_hits = vec![
            ("eden".to_string(), 1.0),
            ("local".to_string(), 0.8),
            ("graph".to_string(), 0.7),
        ];
        let paths = vec![
            "eden -is-> local".to_string(),
            "eden -has-> graph".to_string(),
            "eden -has-> memory".to_string(),
        ];
        let pack = node.build_context("eden", 1, &memory, &kg_hits, &paths, &[]);
        node.record_feedback("eden", true);

        assert_eq!(pack.quality_label, "high");
        assert_eq!(node.learned_summaries().len(), 1);

        node.build_context("unknown", 2, &[], &[], &[], &[]);
        node.record_feedback("unknown", false);
        let gaps = node.gaps("unknown");

        assert!(gaps.contains("sin_kg"));
        assert!(gaps.contains("sin_memoria"));
        assert!(gaps.contains("contexto_debil_recurrente"));
    }

    #[test]
    fn plans_and_tracks_safe_actions() {
        let mut node = ContextAugmentationNode::new(1);
        node.build_context("unknown", 1, &[], &[], &[], &[]);
        node.record_feedback("unknown", false);

        let plan = node.plan_actions("unknown");
        let runnable = node.take_runnable_actions("unknown");
        let autonomous = node.take_autonomous_safe_actions(4);

        assert!(plan.contains("prompt_remember"));
        assert!(plan.contains("crawl_gated"));
        assert!(runnable
            .iter()
            .any(|action| action.kind == "prompt_remember"));
        assert!(runnable
            .iter()
            .any(|action| action.kind == "prioritize_local_conceptnet"));
        assert!(!runnable.iter().any(|action| action.kind == "crawl_gated"));
        assert!(autonomous.is_empty());
    }

    #[test]
    fn autonomous_safe_actions_never_take_blocked_crawler() {
        let mut node = ContextAugmentationNode::new(1);
        node.build_context("unknown", 1, &[], &[], &[], &[]);
        node.record_feedback("unknown", false);
        node.plan_actions("unknown");

        let runnable = node.take_autonomous_safe_actions(4);

        assert!(runnable
            .iter()
            .any(|action| action.kind == "prompt_remember"));
        assert!(runnable
            .iter()
            .any(|action| action.kind == "prioritize_local_conceptnet"));
        assert!(runnable
            .iter()
            .any(|action| action.kind == "validate_with_juez"));
        assert!(!runnable.iter().any(|action| action.kind == "crawl_gated"));
    }

    #[test]
    fn prioritizes_recurrent_old_weak_contexts() {
        let mut node = ContextAugmentationNode::new(1);
        node.build_context("old_missing", 1, &[], &[], &[], &[]);
        node.record_feedback("old_missing", false);
        node.record_feedback("old_missing", false);
        node.build_context(
            "newer_with_memory",
            20,
            &["newer has memory".to_string()],
            &[],
            &[],
            &[],
        );
        node.record_feedback("newer_with_memory", false);

        let prioritized = node.prioritized_weak_queries(2);

        assert_eq!(prioritized.first().map(String::as_str), Some("old_missing"));
    }
}
