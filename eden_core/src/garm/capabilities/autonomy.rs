// EDEN GARM Autonomy — Self-directed learning. EDEN inspecciona su propio estado,
// detecta gaps de conocimiento, y genera goals para llenarlos.
// 100% Rust puro, 0 LLM, 0 red.
//
// Closes the metacognitive loop:
//   knowledge introspection -> gap detection -> goal generation -> execution
//
// Sin esto, EDEN solo puede aprender lo que el usuario le da. Con esto,
// el sistema decide autonomamente que aprender en base a lo que NO sabe.

use crate::eden_garm::capabilities::grounding::GroundingEngine;
use crate::eden_garm::capabilities::intention_hierarchy::GoalStack;
use crate::eden_garm::capabilities::morphogenesis::ConceptSpace;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum GapType {
    /// Concept exists with high count but no incoming "causes" edges
    UnexplainedFrequent,
    /// Concept marked as physical but no grounding facts associated
    PhysicalNoFacts,
    /// Concept has an abstract pattern keyword but no relations at all
    PatternIsolated,
    /// Concept has out_degree but zero in_degree (a "root cause" leaf - might need upstream)
    NoUpstream,
    /// Concept has in_degree but zero out_degree (a "leaf effect" - might need downstream)
    NoDownstream,
}

#[derive(Clone, Debug)]
pub struct KnowledgeGap {
    pub kind: GapType,
    pub concept_id: u64,
    pub concept_label: String,
    pub severity: f32, // 0..1, higher = more important to fill
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct AutonomyEngine {
    pub gaps_detected: Vec<KnowledgeGap>,
    pub n_introspections: u64,
    pub n_goals_generated: u64,
    pub last_introspection_tick: u64,
    pub min_concept_count_for_gap: u32,
    pub max_gaps_per_introspect: usize,
    pub max_goals_per_run: usize,
}

impl AutonomyEngine {
    pub fn new() -> Self {
        AutonomyEngine {
            gaps_detected: Vec::new(),
            n_introspections: 0,
            n_goals_generated: 0,
            last_introspection_tick: 0,
            min_concept_count_for_gap: 1,
            max_gaps_per_introspect: 50,
            max_goals_per_run: 8,
        }
    }

    /// Introspect the morphogenesis + grounding state, detect knowledge gaps.
    pub fn introspect(
        &mut self,
        space: &ConceptSpace,
        grounding: &GroundingEngine,
        tick: u64,
    ) -> Vec<KnowledgeGap> {
        self.last_introspection_tick = tick;
        self.n_introspections += 1;
        self.gaps_detected.clear();

        // Compute in-degrees
        let mut in_deg: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
        for c in space.concepts.values() {
            for targets in c.relations.values() {
                for &t in targets {
                    *in_deg.entry(t).or_insert(0) += 1;
                }
            }
        }

        for c in space.concepts.values() {
            let out_deg: usize = c.relations.values().map(|v| v.len()).sum();
            let in_d = *in_deg.get(&c.id).unwrap_or(&0);
            let total_deg = in_d + out_deg;

            // Gap 1: high count, no causes (unexplained frequent concept)
            if c.count >= self.min_concept_count_for_gap.max(2) && in_d == 0 && c.count >= 2 {
                self.gaps_detected.push(KnowledgeGap {
                    kind: GapType::UnexplainedFrequent,
                    concept_id: c.id,
                    concept_label: c.label.clone(),
                    severity: (c.count as f32 / 5.0).min(1.0),
                    note: format!("seen {} times but no causes attributed", c.count),
                });
            }

            // Gap 2: physical concept with no grounding facts
            if grounding.physical_concepts.contains(&c.id) {
                let has_grounding_fact = grounding.facts.iter().any(|f| {
                    let label_low = c.label.to_lowercase();
                    match &f.claim {
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::Attracts {
                            source,
                            target,
                        } => label_low.contains(source) || label_low.contains(target),
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::Falls {
                            entity,
                        } => label_low.contains(entity),
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::HasMass {
                            entity,
                            ..
                        } => label_low.contains(entity),
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::Temperature {
                            entity,
                            ..
                        } => label_low.contains(entity),
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::Contains {
                            container,
                            contents,
                        } => label_low.contains(container) || label_low.contains(contents),
                        crate::eden_garm::capabilities::grounding::PhysicalClaim::SupportedBy {
                            entity,
                            support,
                        } => label_low.contains(entity) || label_low.contains(support),
                        _ => false,
                    }
                });
                if !has_grounding_fact {
                    self.gaps_detected.push(KnowledgeGap {
                        kind: GapType::PhysicalNoFacts,
                        concept_id: c.id,
                        concept_label: c.label.clone(),
                        severity: 0.6,
                        note: format!("physical concept but no grounding facts learned"),
                    });
                }
            }

            // Gap 3: completely isolated concept (no relations at all) but seen multiple times
            if total_deg == 0 && c.count >= 2 {
                self.gaps_detected.push(KnowledgeGap {
                    kind: GapType::PatternIsolated,
                    concept_id: c.id,
                    concept_label: c.label.clone(),
                    severity: 0.4,
                    note: format!("seen {} times, zero relations", c.count),
                });
            }

            // Gap 4: out-only concept (root cause without upstream)
            if out_deg > 0 && in_d == 0 && c.count >= 1 {
                self.gaps_detected.push(KnowledgeGap {
                    kind: GapType::NoUpstream,
                    concept_id: c.id,
                    concept_label: c.label.clone(),
                    severity: 0.5,
                    note: format!("out-degree={}, no upstream causes", out_deg),
                });
            }

            // Gap 5: in-only concept (leaf effect without downstream)
            if in_d > 0 && out_deg == 0 && c.count >= 1 {
                self.gaps_detected.push(KnowledgeGap {
                    kind: GapType::NoDownstream,
                    concept_id: c.id,
                    concept_label: c.label.clone(),
                    severity: 0.3,
                    note: format!("in-degree={}, no downstream effects", in_d),
                });
            }
        }

        // Sort by severity desc and truncate
        self.gaps_detected.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.gaps_detected.truncate(self.max_gaps_per_introspect);
        self.gaps_detected.clone()
    }

    /// Generate goal labels from detected gaps. Each gap becomes a goal aimed at its capability.
    pub fn generate_goals(
        &mut self,
        gaps: &[KnowledgeGap],
        goal_stack: &mut GoalStack,
        tick: u64,
    ) -> usize {
        let mut generated = 0usize;
        let limit = self.max_goals_per_run;
        // Track what's already in the stack to avoid duplicates
        let existing: HashSet<String> =
            goal_stack.goals.values().map(|g| g.label.clone()).collect();
        for gap in gaps.iter().take(limit) {
            let goal_label = match gap.kind {
                GapType::UnexplainedFrequent => {
                    format!("derivar inferencia causa de {}", gap.concept_label)
                }
                GapType::PhysicalNoFacts => {
                    format!("calienta absorbe energia analizar {}", gap.concept_label)
                }
                GapType::PatternIsolated => {
                    format!("explorar memoria conectar {}", gap.concept_label)
                }
                GapType::NoUpstream => {
                    format!("derivar inferencia upstream de {}", gap.concept_label)
                }
                GapType::NoDownstream => {
                    format!("derivar inferencia downstream de {}", gap.concept_label)
                }
            };
            // Truncate to keep labels short (helps keyword matching)
            let goal_label = if goal_label.len() > 80 {
                goal_label[..80].to_string()
            } else {
                goal_label
            };
            if existing.contains(&goal_label) {
                continue;
            }
            let priority = (gap.severity * 0.9).min(1.0);
            goal_stack.push(&goal_label, priority, Some(tick + 50), None);
            generated += 1;
            self.n_goals_generated += 1;
        }
        generated
    }

    pub fn report_gaps(&self, max: usize) -> String {
        if self.gaps_detected.is_empty() {
            return "No gaps detected".to_string();
        }
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for g in &self.gaps_detected {
            let k = match g.kind {
                GapType::UnexplainedFrequent => "UnexplainedFrequent",
                GapType::PhysicalNoFacts => "PhysicalNoFacts",
                GapType::PatternIsolated => "PatternIsolated",
                GapType::NoUpstream => "NoUpstream",
                GapType::NoDownstream => "NoDownstream",
            };
            *counts.entry(k).or_insert(0) += 1;
        }
        let mut summary: Vec<(String, usize)> = counts
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        summary.sort_by(|a, b| b.1.cmp(&a.1));
        let mut out = format!("Gaps detectados: {} totales\n", self.gaps_detected.len());
        out.push_str("  Por tipo: ");
        out.push_str(
            &summary
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push_str("\n\nTop gaps por severity:\n");
        for g in self.gaps_detected.iter().take(max) {
            out.push_str(&format!(
                "  [{:?}] cid={} '{}' | severity={:.2} | {}\n",
                g.kind, g.concept_id, g.concept_label, g.severity, g.note,
            ));
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "Autonomy | introspections={} | goals_generated={} | last_introspect_tick={} | gaps_detected={}",
            self.n_introspections, self.n_goals_generated,
            self.last_introspection_tick, self.gaps_detected.len(),
        )
    }
}
