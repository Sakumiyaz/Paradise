use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

pub struct HierarchicalReasoningNode {
    id: usize,
    runs: u64,
    last_query: String,
    last_mode: String,
    last_summary: String,
    last_evidence_count: usize,
    last_plan_steps: Vec<String>,
    plan_executions: u64,
    plan_successes: u64,
    plan_failures: u64,
    last_execution_status: String,
    last_execution_trace: String,
    internal_fe: f32,
}

impl HierarchicalReasoningNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            runs: 0,
            last_query: String::new(),
            last_mode: "init".to_string(),
            last_summary: String::new(),
            last_evidence_count: 0,
            last_plan_steps: Vec::new(),
            plan_executions: 0,
            plan_successes: 0,
            plan_failures: 0,
            last_execution_status: "not_run".to_string(),
            last_execution_trace: String::new(),
            internal_fe: 1.0,
        }
    }

    pub fn reason(
        &mut self,
        query: &str,
        tick: u64,
        memory_facts: &[String],
        kg_hits: &[(String, f32)],
        path_explanations: &[String],
        history_fragments: &[String],
    ) -> String {
        self.runs += 1;
        self.last_query = query.trim().to_string();
        self.last_evidence_count =
            memory_facts.len() + kg_hits.len() + path_explanations.len() + history_fragments.len();
        self.last_mode = if !path_explanations.is_empty() || kg_hits.len() >= 2 {
            "strategic"
        } else if !history_fragments.is_empty() {
            "episodic"
        } else if !memory_facts.is_empty() {
            "semantic"
        } else {
            "exploratory"
        }
        .to_string();
        let stance = if self.last_evidence_count >= 4 {
            "sostenido"
        } else if self.last_evidence_count >= 1 {
            "plausible-no-endurecer"
        } else {
            "sin-evidencia"
        };
        self.last_summary = format!(
            "mode={} stance={} evidence={} tick={}",
            self.last_mode, stance, self.last_evidence_count, tick
        );
        self.last_plan_steps = self.plan_layers(
            memory_facts,
            kg_hits,
            path_explanations,
            history_fragments,
            stance,
        );

        let mut out = format!(
            "[HRM] query='{}' mode={} stance={} evidence={} runs={}\n",
            self.last_query, self.last_mode, stance, self.last_evidence_count, self.runs
        );
        out.push_str("[HRM-LAYERS]\n");
        out.push_str(&format!(
            "- strategic: kg_hits={} paths={}\n",
            kg_hits.len(),
            path_explanations.len()
        ));
        out.push_str(&format!(
            "- episodic: history_fragments={}\n",
            history_fragments.len()
        ));
        out.push_str(&format!(
            "- semantic: memory_facts={}\n",
            memory_facts.len()
        ));
        for path in path_explanations.iter().take(3) {
            out.push_str(&format!("- path: {}\n", path));
        }
        for (hit, score) in kg_hits.iter().take(3) {
            out.push_str(&format!("- kg: {} ({:.2})\n", hit, score));
        }
        for fact in memory_facts.iter().take(3) {
            out.push_str(&format!("- memory: {}\n", fact));
        }
        if self.last_evidence_count == 0 {
            out.push_str("- action: ask CAG/KG/history for local evidence before asserting.\n");
        }
        out.push_str("[HRM-PLAN]\n");
        for step in &self.last_plan_steps {
            out.push_str(&format!("- {}\n", step));
        }
        out.push_str(&format!(
            "[HRM-METRICS] confidence={:.2} executable_steps={} executions={} successes={} failures={} last_execution={}\n",
            self.confidence_score(),
            self.executable_step_count(),
            self.plan_executions,
            self.plan_successes,
            self.plan_failures,
            self.last_execution_status
        ));
        out.push_str(&format!("[HRM-SUMMARY] {}\n", self.last_summary));
        out
    }

    pub fn record_execution_result(&mut self, executed: bool, trace: &str) -> String {
        self.plan_executions += 1;
        if executed {
            self.plan_successes += 1;
            self.last_execution_status = "executed".to_string();
        } else {
            self.plan_failures += 1;
            self.last_execution_status = "no_safe_action".to_string();
        }
        self.last_execution_trace = trace.trim().chars().take(512).collect();
        format!(
            "[HRM-RUN-METRICS] executions={} successes={} failures={} status={} trace_len={}\n",
            self.plan_executions,
            self.plan_successes,
            self.plan_failures,
            self.last_execution_status,
            self.last_execution_trace.len()
        )
    }

    fn confidence_score(&self) -> f32 {
        let evidence = (self.last_evidence_count as f32 / 6.0).min(1.0);
        let execution = if self.plan_executions == 0 {
            0.0
        } else {
            self.plan_successes as f32 / self.plan_executions as f32
        };
        (evidence * 0.75 + execution * 0.25).min(1.0)
    }

    fn executable_step_count(&self) -> usize {
        self.last_plan_steps
            .iter()
            .filter(|step| step.starts_with("action:"))
            .count()
    }

    fn plan_layers(
        &self,
        memory_facts: &[String],
        kg_hits: &[(String, f32)],
        path_explanations: &[String],
        history_fragments: &[String],
        stance: &str,
    ) -> Vec<String> {
        let strategic = if !path_explanations.is_empty() || kg_hits.len() >= 2 {
            format!(
                "strategic:prioritize_kg_paths paths={} hits={}",
                path_explanations.len(),
                kg_hits.len()
            )
        } else {
            "strategic:request_more_kg_context".to_string()
        };
        let episodic = if history_fragments.is_empty() {
            "episodic:record_trace_then_requery".to_string()
        } else {
            format!(
                "episodic:anchor_recent_history count={}",
                history_fragments.len()
            )
        };
        let semantic = if memory_facts.is_empty() {
            "semantic:avoid_claim_until_memory_fact_exists".to_string()
        } else {
            format!("semantic:use_memory_facts count={}", memory_facts.len())
        };
        let action = match stance {
            "sostenido" => "action:answer_with_citations_and_update_kg",
            "plausible-no-endurecer" => "action:answer_as_hypothesis_and_mark_uncertainty",
            _ => "action:defer_assertion_and_request_local_evidence",
        }
        .to_string();
        vec![strategic, episodic, semantic, action]
    }

    pub fn autonomy_cycle(
        &mut self,
        tick: u64,
        memory_facts: usize,
        kg_edges: usize,
        history_events: u64,
    ) -> String {
        let query = format!(
            "runtime continuity memory={} kg_edges={} history={}",
            memory_facts, kg_edges, history_events
        );
        self.reason(&query, tick, &[], &[], &[], &[])
    }

    pub fn snapshot(&self) -> String {
        format!(
            "hrm:runs:{} mode:{} evidence:{} plan_steps:{} executions:{} successes:{} failures:{} last_execution:{} last_query_len:{} summary_len:{} internal_fe:{:.3}",
            self.runs,
            self.last_mode,
            self.last_evidence_count,
            self.last_plan_steps.len(),
            self.plan_executions,
            self.plan_successes,
            self.plan_failures,
            self.last_execution_status,
            self.last_query.len(),
            self.last_summary.len(),
            self.internal_fe
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let snapshot = serde_json::json!({
            "runs": self.runs,
            "last_query": self.last_query,
            "last_mode": self.last_mode,
            "last_summary": self.last_summary,
            "last_evidence_count": self.last_evidence_count,
            "last_plan_steps": self.last_plan_steps,
            "plan_executions": self.plan_executions,
            "plan_successes": self.plan_successes,
            "plan_failures": self.plan_failures,
            "last_execution_status": self.last_execution_status,
            "last_execution_trace": self.last_execution_trace,
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
        self.runs = snapshot.get("runs").and_then(|v| v.as_u64()).unwrap_or(0);
        self.last_query = snapshot
            .get("last_query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.last_mode = snapshot
            .get("last_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("init")
            .to_string();
        self.last_summary = snapshot
            .get("last_summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        self.last_evidence_count = snapshot
            .get("last_evidence_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        self.last_plan_steps = snapshot
            .get("last_plan_steps")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();
        self.plan_executions = snapshot
            .get("plan_executions")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.plan_successes = snapshot
            .get("plan_successes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.plan_failures = snapshot
            .get("plan_failures")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.last_execution_status = snapshot
            .get("last_execution_status")
            .and_then(|v| v.as_str())
            .unwrap_or("not_run")
            .to_string();
        self.last_execution_trace = snapshot
            .get("last_execution_trace")
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

impl GARMNode for HierarchicalReasoningNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "hrm_reasoner"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe
            + if self.last_evidence_count == 0 {
                0.5
            } else {
                0.0
            }
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![self.runs as f32, self.last_evidence_count as f32]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![self.runs as f32, self.last_evidence_count as f32])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
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
    use super::*;

    #[test]
    fn reasons_with_hierarchical_evidence_without_llm() {
        let mut node = HierarchicalReasoningNode::new(1);
        let out = node.reason(
            "energia",
            7,
            &["energia regula metabolismo".to_string()],
            &[("energia causes accion".to_string(), 0.8)],
            &["energia -> metabolismo".to_string()],
            &["cmd: energia".to_string()],
        );

        assert!(out.contains("[HRM] query='energia'"));
        assert!(out.contains("mode=strategic"));
        assert!(out.contains("[HRM-PLAN]"));
        assert!(out.contains("[HRM-METRICS] confidence="));
        assert!(out.contains("action:answer_with_citations_and_update_kg"));
        assert!(node.snapshot().contains("runs:1"));
        assert!(node.snapshot().contains("plan_steps:4"));
    }

    #[test]
    fn plans_defer_when_evidence_is_missing() {
        let mut node = HierarchicalReasoningNode::new(1);
        let out = node.reason("desconocido", 8, &[], &[], &[], &[]);

        assert!(out.contains("stance=sin-evidencia"));
        assert!(out.contains("strategic:request_more_kg_context"));
        assert!(out.contains("action:defer_assertion_and_request_local_evidence"));
    }

    #[test]
    fn records_plan_execution_metrics() {
        let mut node = HierarchicalReasoningNode::new(1);
        node.reason("energia", 8, &[], &[], &[], &[]);

        let metrics = node.record_execution_result(true, "cag=ok organs=ok");

        assert!(metrics.contains("[HRM-RUN-METRICS]"));
        assert!(metrics.contains("successes=1"));
        assert!(node.snapshot().contains("executions:1"));
        assert!(node.snapshot().contains("last_execution:executed"));
    }
}
