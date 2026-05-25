use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

#[derive(Clone, Debug, PartialEq)]
pub struct KnowledgeGap {
    pub topic: String,
    pub uncertainty: f32,
    pub last_explored: u64,
    pub exploration_count: u32,
    pub information_potential: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExplorationTarget {
    pub target: String,
    pub information_gain: f32,
    pub energy_cost: f32,
    pub timestamp: u64,
    pub success: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubGoalStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MissionStatus {
    Active,
    Completed,
    Failed,
    Evolved,
    Abandoned,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubGoal {
    pub description: String,
    pub progress: f32,
    pub status: SubGoalStatus,
    pub completed_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mission {
    pub id: u64,
    pub primary_goal: String,
    pub sub_goals: Vec<SubGoal>,
    pub active_sub_goal_index: usize,
    pub created_at: u64,
    pub deadline: Option<u64>,
    pub progress: f32,
    pub relevance: f32,
    pub status: MissionStatus,
    pub success_criteria: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacySelfModel {
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub known_topics: Vec<String>,
    pub unknown_topics: Vec<String>,
    pub skills: Vec<String>,
    pub learning_goals: Vec<String>,
    pub last_updated: u64,
    pub reinforcement_count: u32,
    pub lineage_age: u64,
    pub total_rebirths: u32,
    pub ancestor_facts: Vec<String>,
    pub persistent_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmotionalBaseline {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub satisfaction: f32,
    pub frustration: f32,
    pub interest: f32,
    pub distress: f32,
    pub hope: f32,
    pub fear: f32,
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub trust: f32,
    pub disgust: f32,
    pub surprise: f32,
    pub anticipation: f32,
    pub current_emotion: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryConsolidation {
    pub memory: String,
    pub importance: f32,
    pub connections_made: Vec<String>,
    pub emotional_tag: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DreamState {
    pub active: bool,
    pub start_time: u64,
    pub consolidation_targets: Vec<MemoryConsolidation>,
    pub processed_memories: Vec<String>,
    pub creativity_output: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SharedKnowledge {
    pub knowledge_id: u64,
    pub content: String,
    pub source_agent: String,
    pub timestamp: u64,
    pub trust: f32,
    pub usefulness: f32,
    pub tags: Vec<String>,
}

pub struct LegacyCognitionNode {
    id: usize,
    pub knowledge_gaps: Vec<KnowledgeGap>,
    pub exploration_history: Vec<ExplorationTarget>,
    pub curiosity_threshold: f32,
    pub total_information_gain: f32,
    pub unexplored_domains: Vec<String>,
    pub current_mission: Option<Mission>,
    pub emotional_baseline: EmotionalBaseline,
    pub dream_state: DreamState,
    pub shared_knowledge: Vec<SharedKnowledge>,
    pub self_model: LegacySelfModel,
    internal_fe: f32,
}

impl LegacyCognitionNode {
    pub fn new(id: usize) -> Self {
        let mut node = Self {
            id,
            knowledge_gaps: Vec::new(),
            exploration_history: Vec::new(),
            curiosity_threshold: 0.5,
            total_information_gain: 0.0,
            unexplored_domains: vec![
                "quantum_physics".to_string(),
                "artificial_intelligence".to_string(),
                "consciousness_studies".to_string(),
                "complexity_theory".to_string(),
                "self_organization".to_string(),
                "emergence".to_string(),
                "evolutionary_biology".to_string(),
                "cognitive_science".to_string(),
            ],
            current_mission: None,
            emotional_baseline: EmotionalBaseline::new(),
            dream_state: DreamState::new(),
            shared_knowledge: Vec::new(),
            self_model: LegacySelfModel::new(),
            internal_fe: 1.0,
        };
        node.update_from_facts(&[], 0);
        node
    }

    pub fn update_from_facts(&mut self, facts: &[String], cycle_count: u64) {
        for domain in self.unexplored_domains.clone() {
            if !self.knowledge_gaps.iter().any(|gap| gap.topic == domain) {
                self.knowledge_gaps.push(KnowledgeGap {
                    topic: domain,
                    uncertainty: 0.8,
                    last_explored: 0,
                    exploration_count: 0,
                    information_potential: 1.0,
                });
            }
        }
        if self.knowledge_gaps.len() > 50 {
            let overflow = self.knowledge_gaps.len() - 50;
            self.knowledge_gaps.drain(0..overflow);
        }

        for gap in &mut self.knowledge_gaps {
            if facts
                .iter()
                .any(|fact| fact.to_lowercase().contains(&gap.topic))
            {
                gap.uncertainty = (gap.uncertainty - 0.1).max(0.1);
                gap.information_potential = (gap.information_potential * 0.9).max(0.1);
            }
        }
        self.self_model.update_from_experience(
            cycle_count,
            facts,
            self.current_mission
                .as_ref()
                .map(|m| m.primary_goal.as_str()),
        );
        if self.current_mission.is_none() {
            self.current_mission = Some(Mission::generate_from_curiosity(
                &self.knowledge_gaps,
                1,
                cycle_count,
            ));
        }
    }

    pub fn select_exploration_target(&mut self) -> Option<String> {
        self.knowledge_gaps.sort_by(|a, b| {
            let score_a = a.uncertainty * a.information_potential;
            let score_b = b.uncertainty * b.information_potential;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.knowledge_gaps
            .first()
            .map(|gap| gap.topic.clone())
            .or_else(|| self.unexplored_domains.first().cloned())
    }

    pub fn record_exploration(&mut self, target: &str, info_gain: f32, success: bool) {
        self.exploration_history.push(ExplorationTarget {
            target: target.to_string(),
            information_gain: info_gain,
            energy_cost: 0.1,
            timestamp: self.total_information_gain as u64,
            success,
        });
        if self.exploration_history.len() > 100 {
            let overflow = self.exploration_history.len() - 100;
            self.exploration_history.drain(0..overflow);
        }
        self.total_information_gain += info_gain;
        if let Some(gap) = self
            .knowledge_gaps
            .iter_mut()
            .find(|gap| gap.topic == target)
        {
            gap.exploration_count += 1;
            gap.last_explored = self.total_information_gain as u64;
            if success {
                gap.uncertainty = (gap.uncertainty - 0.2).max(0.1);
                gap.information_potential = (gap.information_potential * 0.95).max(0.1);
            }
        }
    }

    pub fn add_real_domain(&mut self, domain: &str) {
        let clean = domain.trim();
        if clean.is_empty()
            || clean.starts_with("nueva_dimension_")
            || self.unexplored_domains.iter().any(|d| d == clean)
        {
            return;
        }
        self.unexplored_domains.push(clean.to_string());
        if self.unexplored_domains.len() > 30 {
            let overflow = self.unexplored_domains.len() - 30;
            self.unexplored_domains.drain(0..overflow);
        }
    }

    pub fn consolidate_dream(&mut self, memories: &[String]) -> Vec<String> {
        self.dream_state.enter(self.total_information_gain as u64);
        self.dream_state.consolidate(memories);
        self.dream_state.exit()
    }

    pub fn share_knowledge(&mut self, content: String, source_agent: String) {
        let id = self.shared_knowledge.len() as u64 + 1;
        self.shared_knowledge.push(SharedKnowledge {
            knowledge_id: id,
            content,
            source_agent,
            timestamp: self.total_information_gain as u64,
            trust: 0.7,
            usefulness: 0.5,
            tags: Vec::new(),
        });
        if self.shared_knowledge.len() > 100 {
            let overflow = self.shared_knowledge.len() - 100;
            self.shared_knowledge.drain(0..overflow);
        }
    }

    pub fn report(&self) -> String {
        let mission = self
            .current_mission
            .as_ref()
            .map(|m| m.primary_goal.as_str())
            .unwrap_or("none");
        format!(
            "LEGACY COGNITION GARM\n- gaps={} target={}\n- explorations={} info_gain={:.3}\n- mission={}\n- self_model={}\n- dream_processed={} shared_knowledge={}",
            self.knowledge_gaps.len(),
            self.knowledge_gaps.first().map(|g| g.topic.as_str()).unwrap_or("none"),
            self.exploration_history.len(),
            self.total_information_gain,
            mission,
            self.self_model.summary(),
            self.dream_state.processed_memories.len(),
            self.shared_knowledge.len(),
        )
    }

    pub fn cognition_snapshot(&self) -> String {
        format!(
            "cognition:gaps:{} explorations:{} info_gain:{:.3} dream_processed:{} shared:{}",
            self.knowledge_gaps.len(),
            self.exploration_history.len(),
            self.total_information_gain,
            self.dream_state.processed_memories.len(),
            self.shared_knowledge.len()
        )
    }

    pub fn save_state(&self, path: &str) -> Result<(), String> {
        let gaps: Vec<_> = self
            .knowledge_gaps
            .iter()
            .map(|gap| {
                serde_json::json!({
                    "topic": gap.topic,
                    "uncertainty": gap.uncertainty,
                    "last_explored": gap.last_explored,
                    "exploration_count": gap.exploration_count,
                    "information_potential": gap.information_potential,
                })
            })
            .collect();
        let snapshot = serde_json::json!({
            "knowledge_gaps": gaps,
            "exploration_history": self.exploration_history.iter().map(|target| serde_json::json!({
                "target": target.target,
                "information_gain": target.information_gain,
                "energy_cost": target.energy_cost,
                "timestamp": target.timestamp,
                "success": target.success,
            })).collect::<Vec<_>>(),
            "total_information_gain": self.total_information_gain,
            "unexplored_domains": self.unexplored_domains,
            "current_mission": self.current_mission.as_ref().map(|m| serde_json::json!({
                "id": m.id,
                "primary_goal": m.primary_goal,
                "active_sub_goal_index": m.active_sub_goal_index,
                "created_at": m.created_at,
                "deadline": m.deadline,
                "progress": m.progress,
                "relevance": m.relevance,
                "status": mission_status_name(&m.status),
                "success_criteria": m.success_criteria,
                "sub_goals": m.sub_goals.iter().map(|goal| serde_json::json!({
                    "description": goal.description,
                    "progress": goal.progress,
                    "status": subgoal_status_name(&goal.status),
                    "completed_at": goal.completed_at,
                })).collect::<Vec<_>>(),
            })),
            "known_topics": self.self_model.known_topics,
            "unknown_topics": self.self_model.unknown_topics,
            "capabilities": self.self_model.capabilities,
            "limitations": self.self_model.limitations,
            "skills": self.self_model.skills,
            "learning_goals": self.self_model.learning_goals,
            "last_updated": self.self_model.last_updated,
            "reinforcement_count": self.self_model.reinforcement_count,
            "lineage_age": self.self_model.lineage_age,
            "total_rebirths": self.self_model.total_rebirths,
            "ancestor_facts": self.self_model.ancestor_facts,
            "persistent_capabilities": self.self_model.persistent_capabilities,
            "emotional_baseline": serde_json::json!({
                "valence": self.emotional_baseline.valence,
                "arousal": self.emotional_baseline.arousal,
                "dominance": self.emotional_baseline.dominance,
                "satisfaction": self.emotional_baseline.satisfaction,
                "frustration": self.emotional_baseline.frustration,
                "interest": self.emotional_baseline.interest,
                "distress": self.emotional_baseline.distress,
                "hope": self.emotional_baseline.hope,
                "fear": self.emotional_baseline.fear,
                "joy": self.emotional_baseline.joy,
                "sadness": self.emotional_baseline.sadness,
                "anger": self.emotional_baseline.anger,
                "trust": self.emotional_baseline.trust,
                "disgust": self.emotional_baseline.disgust,
                "surprise": self.emotional_baseline.surprise,
                "anticipation": self.emotional_baseline.anticipation,
                "current_emotion": self.emotional_baseline.current_emotion,
            }),
            "dream_active": self.dream_state.active,
            "dream_start_time": self.dream_state.start_time,
            "consolidation_targets": self.dream_state.consolidation_targets.iter().map(|item| serde_json::json!({
                "memory": item.memory,
                "importance": item.importance,
                "connections_made": item.connections_made,
                "emotional_tag": item.emotional_tag,
            })).collect::<Vec<_>>(),
            "processed_memories": self.dream_state.processed_memories,
            "creativity_output": self.dream_state.creativity_output,
            "shared_knowledge": self.shared_knowledge.iter().map(|item| serde_json::json!({
                "knowledge_id": item.knowledge_id,
                "content": item.content,
                "source_agent": item.source_agent,
                "timestamp": item.timestamp,
                "trust": item.trust,
                "usefulness": item.usefulness,
                "tags": item.tags,
            })).collect::<Vec<_>>(),
        });
        std::fs::write(path, snapshot.to_string())
            .map_err(|e| format!("failed to write {}: {}", path, e))
    }

    pub fn load_state(&mut self, path: &str) -> Result<(), String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse JSON: {}", e))?;
        self.knowledge_gaps = snapshot
            .get("knowledge_gaps")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| {
                Some(KnowledgeGap {
                    topic: v.get("topic")?.as_str()?.to_string(),
                    uncertainty: v.get("uncertainty").and_then(|n| n.as_f64()).unwrap_or(0.8)
                        as f32,
                    last_explored: v.get("last_explored").and_then(|n| n.as_u64()).unwrap_or(0),
                    exploration_count: v
                        .get("exploration_count")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(0) as u32,
                    information_potential: v
                        .get("information_potential")
                        .and_then(|n| n.as_f64())
                        .unwrap_or(1.0) as f32,
                })
            })
            .collect();
        self.total_information_gain = snapshot
            .get("total_information_gain")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        self.exploration_history = snapshot
            .get("exploration_history")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(ExplorationTarget {
                            target: v.get("target")?.as_str()?.to_string(),
                            information_gain: v
                                .get("information_gain")
                                .and_then(|n| n.as_f64())
                                .unwrap_or(0.0)
                                as f32,
                            energy_cost: v
                                .get("energy_cost")
                                .and_then(|n| n.as_f64())
                                .unwrap_or(0.1) as f32,
                            timestamp: v.get("timestamp").and_then(|n| n.as_u64()).unwrap_or(0),
                            success: v.get("success").and_then(|n| n.as_bool()).unwrap_or(false),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.unexplored_domains = string_array(snapshot.get("unexplored_domains"))
            .unwrap_or_else(|| self.unexplored_domains.clone());
        self.self_model.known_topics =
            string_array(snapshot.get("known_topics")).unwrap_or_default();
        self.self_model.unknown_topics =
            string_array(snapshot.get("unknown_topics")).unwrap_or_default();
        self.self_model.capabilities =
            string_array(snapshot.get("capabilities")).unwrap_or_default();
        self.self_model.limitations = string_array(snapshot.get("limitations")).unwrap_or_default();
        self.self_model.skills = string_array(snapshot.get("skills")).unwrap_or_default();
        self.self_model.learning_goals =
            string_array(snapshot.get("learning_goals")).unwrap_or_default();
        self.self_model.last_updated = snapshot
            .get("last_updated")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.self_model.reinforcement_count = snapshot
            .get("reinforcement_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        self.self_model.lineage_age = snapshot
            .get("lineage_age")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.self_model.total_rebirths = snapshot
            .get("total_rebirths")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        self.self_model.ancestor_facts =
            string_array(snapshot.get("ancestor_facts")).unwrap_or_default();
        self.self_model.persistent_capabilities =
            string_array(snapshot.get("persistent_capabilities")).unwrap_or_default();
        if let Some(emotion) = snapshot.get("emotional_baseline") {
            self.emotional_baseline.valence = json_f32(emotion, "valence", 0.0);
            self.emotional_baseline.arousal = json_f32(emotion, "arousal", 0.5);
            self.emotional_baseline.dominance = json_f32(emotion, "dominance", 0.5);
            self.emotional_baseline.satisfaction = json_f32(emotion, "satisfaction", 0.0);
            self.emotional_baseline.frustration = json_f32(emotion, "frustration", 0.0);
            self.emotional_baseline.interest = json_f32(emotion, "interest", 0.5);
            self.emotional_baseline.distress = json_f32(emotion, "distress", 0.0);
            self.emotional_baseline.hope = json_f32(emotion, "hope", 0.0);
            self.emotional_baseline.fear = json_f32(emotion, "fear", 0.0);
            self.emotional_baseline.joy = json_f32(emotion, "joy", 0.0);
            self.emotional_baseline.sadness = json_f32(emotion, "sadness", 0.0);
            self.emotional_baseline.anger = json_f32(emotion, "anger", 0.0);
            self.emotional_baseline.trust = json_f32(emotion, "trust", 0.0);
            self.emotional_baseline.disgust = json_f32(emotion, "disgust", 0.0);
            self.emotional_baseline.surprise = json_f32(emotion, "surprise", 0.3);
            self.emotional_baseline.anticipation = json_f32(emotion, "anticipation", 0.3);
            self.emotional_baseline.current_emotion = emotion
                .get("current_emotion")
                .and_then(|v| v.as_str())
                .unwrap_or("Curiosity")
                .to_string();
        }
        self.dream_state.active = snapshot
            .get("dream_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.dream_state.start_time = snapshot
            .get("dream_start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        self.dream_state.consolidation_targets = snapshot
            .get("consolidation_targets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(MemoryConsolidation {
                            memory: v.get("memory")?.as_str()?.to_string(),
                            importance: v.get("importance").and_then(|n| n.as_f64()).unwrap_or(0.5)
                                as f32,
                            connections_made: string_array(v.get("connections_made"))
                                .unwrap_or_default(),
                            emotional_tag: v
                                .get("emotional_tag")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Calm")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.dream_state.processed_memories =
            string_array(snapshot.get("processed_memories")).unwrap_or_default();
        self.dream_state.creativity_output =
            string_array(snapshot.get("creativity_output")).unwrap_or_default();
        self.shared_knowledge = snapshot
            .get("shared_knowledge")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(SharedKnowledge {
                            knowledge_id: v
                                .get("knowledge_id")
                                .and_then(|n| n.as_u64())
                                .unwrap_or(0),
                            content: v.get("content")?.as_str()?.to_string(),
                            source_agent: v
                                .get("source_agent")
                                .and_then(|n| n.as_str())
                                .unwrap_or("legacy")
                                .to_string(),
                            timestamp: v.get("timestamp").and_then(|n| n.as_u64()).unwrap_or(0),
                            trust: v.get("trust").and_then(|n| n.as_f64()).unwrap_or(0.7) as f32,
                            usefulness: v.get("usefulness").and_then(|n| n.as_f64()).unwrap_or(0.5)
                                as f32,
                            tags: string_array(v.get("tags")).unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.current_mission = load_mission(snapshot.get("current_mission"));
        Ok(())
    }
}

fn json_f32(value: &serde_json::Value, key: &str, default: f32) -> f32 {
    value
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default as f64) as f32
}

fn mission_status_name(status: &MissionStatus) -> &'static str {
    match status {
        MissionStatus::Active => "Active",
        MissionStatus::Completed => "Completed",
        MissionStatus::Failed => "Failed",
        MissionStatus::Evolved => "Evolved",
        MissionStatus::Abandoned => "Abandoned",
    }
}

fn subgoal_status_name(status: &SubGoalStatus) -> &'static str {
    match status {
        SubGoalStatus::Pending => "Pending",
        SubGoalStatus::Active => "Active",
        SubGoalStatus::Completed => "Completed",
        SubGoalStatus::Failed => "Failed",
    }
}

fn parse_mission_status(value: &str) -> MissionStatus {
    match value {
        "Completed" => MissionStatus::Completed,
        "Failed" => MissionStatus::Failed,
        "Evolved" => MissionStatus::Evolved,
        "Abandoned" => MissionStatus::Abandoned,
        _ => MissionStatus::Active,
    }
}

fn parse_subgoal_status(value: &str) -> SubGoalStatus {
    match value {
        "Active" => SubGoalStatus::Active,
        "Completed" => SubGoalStatus::Completed,
        "Failed" => SubGoalStatus::Failed,
        _ => SubGoalStatus::Pending,
    }
}

fn load_mission(value: Option<&serde_json::Value>) -> Option<Mission> {
    let value = value?;
    if let Some(goal) = value.as_str() {
        return Some(Mission::new(goal.to_string()));
    }
    let primary_goal = value.get("primary_goal")?.as_str()?.to_string();
    Some(Mission {
        id: value.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        primary_goal,
        sub_goals: value
            .get("sub_goals")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(SubGoal {
                            description: v.get("description")?.as_str()?.to_string(),
                            progress: v.get("progress").and_then(|n| n.as_f64()).unwrap_or(0.0)
                                as f32,
                            status: parse_subgoal_status(
                                v.get("status")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("Pending"),
                            ),
                            completed_at: v.get("completed_at").and_then(|n| n.as_u64()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        active_sub_goal_index: value
            .get("active_sub_goal_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        created_at: value
            .get("created_at")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        deadline: value.get("deadline").and_then(|v| v.as_u64()),
        progress: value
            .get("progress")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        relevance: value
            .get("relevance")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32,
        status: parse_mission_status(
            value
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Active"),
        ),
        success_criteria: string_array(value.get("success_criteria")).unwrap_or_default(),
    })
}

impl Mission {
    pub fn new(primary_goal: String) -> Self {
        Self {
            id: 0,
            primary_goal,
            sub_goals: Vec::new(),
            active_sub_goal_index: 0,
            created_at: 0,
            deadline: None,
            progress: 0.0,
            relevance: 1.0,
            status: MissionStatus::Active,
            success_criteria: Vec::new(),
        }
    }

    pub fn generate_from_curiosity(
        knowledge_gaps: &[KnowledgeGap],
        level: u8,
        created_at: u64,
    ) -> Self {
        let mut sorted = knowledge_gaps.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            let score_a = a.uncertainty * a.information_potential;
            let score_b = b.uncertainty * b.information_potential;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(gap) = sorted.get(
            (level as usize)
                .saturating_sub(1)
                .min(sorted.len().saturating_sub(1)),
        ) {
            let mut mission = Mission::new(format!("Explorar y entender: {}", gap.topic));
            mission.sub_goals = vec![
                SubGoal::new(
                    format!("Investigar fundamentos de {}", gap.topic),
                    SubGoalStatus::Active,
                ),
                SubGoal::new(
                    format!("Encontrar patrones en {}", gap.topic),
                    SubGoalStatus::Pending,
                ),
                SubGoal::new(
                    format!("Conectar {} con conocimiento existente", gap.topic),
                    SubGoalStatus::Pending,
                ),
            ];
            mission.success_criteria = vec![
                "Reducir incertidumbre significativamente".to_string(),
                "Generar nueva informacion util".to_string(),
                "Integrar en arquitectura cognitiva".to_string(),
            ];
            mission.relevance = gap.uncertainty * gap.information_potential;
            mission.created_at = created_at;
            mission
        } else {
            Mission::new("Evolucionar y crecer".to_string())
        }
    }
}

impl SubGoal {
    fn new(description: String, status: SubGoalStatus) -> Self {
        Self {
            description,
            progress: 0.0,
            status,
            completed_at: None,
        }
    }
}

impl LegacySelfModel {
    fn new() -> Self {
        Self {
            capabilities: vec![
                "auto_evolucion".to_string(),
                "pattern_recognition".to_string(),
                "self_modification".to_string(),
                "goal_generation".to_string(),
                "curiosity_driven_exploration".to_string(),
            ],
            limitations: Vec::new(),
            known_topics: Vec::new(),
            unknown_topics: vec![
                "advanced_mathematics".to_string(),
                "low_level_neuroscience".to_string(),
                "certain_physical_theories".to_string(),
            ],
            skills: vec![
                "adaptive_learning".to_string(),
                "hierarchical_goal_planning".to_string(),
            ],
            learning_goals: Vec::new(),
            last_updated: 0,
            reinforcement_count: 0,
            lineage_age: 0,
            total_rebirths: 0,
            ancestor_facts: Vec::new(),
            persistent_capabilities: Vec::new(),
        }
    }

    fn update_from_experience(
        &mut self,
        cycle_count: u64,
        facts: &[String],
        mission: Option<&str>,
    ) {
        self.known_topics.clear();
        for fact in facts.iter().take(50) {
            for word in fact
                .split_whitespace()
                .take(5)
                .filter(|word| word.len() > 5)
            {
                let word = word.to_string();
                if !self.known_topics.contains(&word) {
                    self.known_topics.push(word);
                }
            }
        }
        if let Some(mission) = mission {
            if !self
                .learning_goals
                .iter()
                .any(|goal| goal.contains(mission))
            {
                self.learning_goals.push(mission.to_string());
            }
        }
        if self.learning_goals.len() > 10 {
            let overflow = self.learning_goals.len() - 10;
            self.learning_goals.drain(0..overflow);
        }
        if cycle_count <= 50 {
            self.capabilities
                .retain(|cap| cap != "curiosity_driven_exploration");
        }
        self.last_updated = cycle_count;
    }

    fn summary(&self) -> String {
        format!(
            "known_topics={} learning={} skills={}",
            self.known_topics.len(),
            self.learning_goals
                .last()
                .map(|s| s.as_str())
                .unwrap_or("none"),
            self.skills.join(","),
        )
    }
}

impl EmotionalBaseline {
    fn new() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            satisfaction: 0.0,
            frustration: 0.0,
            interest: 0.5,
            distress: 0.0,
            hope: 0.0,
            fear: 0.0,
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            trust: 0.0,
            disgust: 0.0,
            surprise: 0.3,
            anticipation: 0.3,
            current_emotion: "Curiosity".to_string(),
        }
    }
}

impl DreamState {
    fn new() -> Self {
        Self {
            active: false,
            start_time: 0,
            consolidation_targets: Vec::new(),
            processed_memories: Vec::new(),
            creativity_output: Vec::new(),
        }
    }

    fn enter(&mut self, current_time: u64) {
        self.active = true;
        self.start_time = current_time;
    }

    fn consolidate(&mut self, memories: &[String]) {
        for (idx, memory) in memories.iter().enumerate() {
            self.consolidation_targets.push(MemoryConsolidation {
                memory: memory.clone(),
                importance: 0.5 + idx as f32 * 0.05,
                connections_made: Vec::new(),
                emotional_tag: "Calm".to_string(),
            });
        }
        if self.consolidation_targets.len() >= 3 {
            self.creativity_output.push(format!(
                "Nueva conexion: {} + {} -> insight",
                self.consolidation_targets[0].memory,
                self.consolidation_targets[self.consolidation_targets.len() / 2].memory,
            ));
        }
        self.processed_memories
            .extend(memories.iter().take(5).cloned());
    }

    fn exit(&mut self) -> Vec<String> {
        self.active = false;
        let output = self.creativity_output.clone();
        self.creativity_output.clear();
        output
    }
}

fn string_array(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    value.and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    })
}

impl GARMNode for LegacyCognitionNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "legacy_cognition"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Deliberative
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + (self.knowledge_gaps.len() as f32).ln_1p() * 0.02
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.knowledge_gaps.len() as f32,
            self.total_information_gain,
            self.internal_fe,
        ]
    }
    fn act(&mut self, _ctx: &NodeContext, prediction_error: &[f32]) -> NodeAction {
        if let Some(err) = prediction_error.first() {
            self.internal_fe = (self.internal_fe + err.abs() * 0.05).min(5.0);
        }
        NodeAction::Output(vec![
            self.knowledge_gaps.len() as f32,
            self.total_information_gain,
        ])
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        self.internal_fe *= 0.995;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_curiosity_mission_self_model_and_dream_state() {
        let mut node = LegacyCognitionNode::new(77);
        assert!(node
            .knowledge_gaps
            .iter()
            .any(|gap| gap.topic == "consciousness_studies"));

        let facts = vec!["consciousness_studies integra memoria y atencion".to_string()];
        node.update_from_facts(&facts, 99);
        let target = node.select_exploration_target().unwrap();
        node.record_exploration(&target, 0.42, true);
        let dreams = node.consolidate_dream(&[
            "memoria alpha".to_string(),
            "memoria beta".to_string(),
            "memoria gamma".to_string(),
        ]);
        node.share_knowledge("shared insight".to_string(), "test_agent".to_string());

        assert!(node
            .current_mission
            .as_ref()
            .unwrap()
            .primary_goal
            .contains("Explorar"));
        assert!(node
            .self_model
            .known_topics
            .iter()
            .any(|topic| topic.contains("consciousness_studies")));
        assert_eq!(node.exploration_history.len(), 1);
        assert_eq!(node.shared_knowledge.len(), 1);
        assert_eq!(dreams.len(), 1);
        assert!(node.report().contains("LEGACY COGNITION GARM"));
    }

    #[test]
    fn saves_and_loads_legacy_cognition_snapshot() {
        let path = std::env::temp_dir().join(format!(
            "eden_garm_legacy_cognition_{}.json",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();

        let mut source = LegacyCognitionNode::new(1);
        source.record_exploration("consciousness_studies", 0.5, true);
        source.consolidate_dream(&[
            "dream memory".to_string(),
            "dream link".to_string(),
            "dream third".to_string(),
        ]);
        source.save_state(&path_str).unwrap();

        let mut restored = LegacyCognitionNode::new(2);
        restored.load_state(&path_str).unwrap();

        assert!(restored.total_information_gain >= 0.5);
        assert!(!restored.knowledge_gaps.is_empty());
        assert!(restored.report().contains("info_gain"));

        let _ = std::fs::remove_file(path);
    }
}
