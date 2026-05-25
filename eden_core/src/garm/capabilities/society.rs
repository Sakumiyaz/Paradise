// EDEN GARM Society — Fase 6: Global Workspace as Attention Bottleneck
// Winner-take-all with refractory periods, global ignition threshold,
// and repetition suppression. Serial conscious processing.

use std::collections::HashMap;

pub trait AsAnyMut {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[derive(Clone, Debug)]
pub struct Broadcast {
    pub content: String,
    pub confidence: f32,
    pub agent_name: String,
    pub tick: u64,
}

pub trait CognitiveAgent: AsAnyMut + Send {
    fn name(&self) -> &'static str;
    fn propose(&mut self, tick: u64) -> Option<Broadcast>;
    fn receive_broadcast(&mut self, broadcast: &Broadcast);
}

pub struct PerceptionAgent {
    pub last_input_len: usize,
    pub novelty: f32, // 0..1, higher = more novel input
}

impl AsAnyMut for PerceptionAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for PerceptionAgent {
    fn name(&self) -> &'static str {
        "perception"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        if self.last_input_len == 0 && self.novelty < 0.1 {
            return None;
        }
        let conf = ((self.last_input_len as f32 / 50.0) + self.novelty).clamp(0.0, 1.0);
        Some(Broadcast {
            content: format!(
                "input_len={} | novelty={:.2}",
                self.last_input_len, self.novelty
            ),
            confidence: conf,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct GoalAgent {
    pub goals_pending: usize,
    pub discomfort: f32,
}

impl AsAnyMut for GoalAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for GoalAgent {
    fn name(&self) -> &'static str {
        "goal"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        if self.goals_pending == 0 && self.discomfort < 0.5 {
            return None;
        }
        let conf = (self.discomfort * 0.7 + (self.goals_pending as f32 / 10.0).clamp(0.0, 0.3))
            .clamp(0.0, 1.0);
        Some(Broadcast {
            content: format!(
                "goals={} | discomfort={:.2}",
                self.goals_pending, self.discomfort
            ),
            confidence: conf,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct MemoryAgent {
    pub retrieval_count: u32,
    pub concept_load: usize,
}

impl AsAnyMut for MemoryAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for MemoryAgent {
    fn name(&self) -> &'static str {
        "memory"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        let load =
            (self.retrieval_count as f32 / 100.0 + self.concept_load as f32 / 50.0).clamp(0.0, 1.0);
        if load < 0.2 {
            return None;
        }
        Some(Broadcast {
            content: format!("load={:.2}", load),
            confidence: load,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct MetaAgent {
    pub error_recent: f32,
    pub curiosity: f32,
}

impl AsAnyMut for MetaAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for MetaAgent {
    fn name(&self) -> &'static str {
        "metacognition"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        let urgency = (self.error_recent / 3.0 + self.curiosity * 0.3).clamp(0.0, 1.0);
        if urgency < 0.2 {
            return None;
        }
        Some(Broadcast {
            content: format!(
                "error={:.3} | curiosity={:.2}",
                self.error_recent, self.curiosity
            ),
            confidence: urgency,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct ExplorationAgent {
    pub curiosity: f32,
    pub n_concepts: usize,
}

impl AsAnyMut for ExplorationAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for ExplorationAgent {
    fn name(&self) -> &'static str {
        "exploration"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        if self.curiosity < 0.5 {
            return None;
        }
        let conf = (self.curiosity * 0.6 + (self.n_concepts as f32 / 100.0).clamp(0.0, 0.4))
            .clamp(0.0, 1.0);
        Some(Broadcast {
            content: format!(
                "curiosity={:.2} | concepts={}",
                self.curiosity, self.n_concepts
            ),
            confidence: conf,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct SocialAgent {
    pub n_peers: usize,
    pub last_message_age: u64,
}

impl AsAnyMut for SocialAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for SocialAgent {
    fn name(&self) -> &'static str {
        "social"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        if self.n_peers == 0 {
            return None;
        }
        let urgency = (self.n_peers as f32 / 10.0
            + (1.0 / (1.0 + self.last_message_age as f32)).clamp(0.0, 0.5))
        .clamp(0.0, 1.0);
        Some(Broadcast {
            content: format!(
                "peers={} | last_msg_age={}",
                self.n_peers, self.last_message_age
            ),
            confidence: urgency,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct CreativityAgent {
    pub novel_combinations: usize,
    pub inferred_relations: usize,
}

impl AsAnyMut for CreativityAgent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CognitiveAgent for CreativityAgent {
    fn name(&self) -> &'static str {
        "creativity"
    }
    fn propose(&mut self, tick: u64) -> Option<Broadcast> {
        if self.novel_combinations == 0 && self.inferred_relations == 0 {
            return None;
        }
        let conf = ((self.novel_combinations as f32 / 5.0).clamp(0.0, 0.5)
            + (self.inferred_relations as f32 / 10.0).clamp(0.0, 0.5))
        .clamp(0.0, 1.0);
        Some(Broadcast {
            content: format!(
                "novel_combos={} | inferred={}",
                self.novel_combinations, self.inferred_relations
            ),
            confidence: conf,
            agent_name: self.name().to_string(),
            tick,
        })
    }
    fn receive_broadcast(&mut self, _broadcast: &Broadcast) {}
}

pub struct GlobalWorkspace {
    pub agents: Vec<Box<dyn CognitiveAgent>>,
    pub last_broadcast: Option<Broadcast>,
    pub history: Vec<Broadcast>,
    pub max_history: usize,
    pub refractory: HashMap<String, u64>, // agent_name -> remaining ticks
    pub global_threshold: f32,            // minimum confidence for broadcast
    pub default_refractory: u64,          // ticks winner is silenced
    pub last_winner: Option<String>,      // for repetition suppression
    pub suppress_repeat_penalty: f32,     // confidence penalty for repeat
    pub n_broadcasts: u64,
    pub n_skipped: u64, // ticks below threshold
}

impl GlobalWorkspace {
    pub fn new() -> Self {
        let mut agents: Vec<Box<dyn CognitiveAgent>> = Vec::new();
        agents.push(Box::new(PerceptionAgent {
            last_input_len: 0,
            novelty: 0.0,
        }));
        agents.push(Box::new(GoalAgent {
            goals_pending: 0,
            discomfort: 0.0,
        }));
        agents.push(Box::new(MemoryAgent {
            retrieval_count: 0,
            concept_load: 0,
        }));
        agents.push(Box::new(MetaAgent {
            error_recent: 0.0,
            curiosity: 0.0,
        }));
        agents.push(Box::new(ExplorationAgent {
            curiosity: 0.0,
            n_concepts: 0,
        }));
        agents.push(Box::new(SocialAgent {
            n_peers: 0,
            last_message_age: 999,
        }));
        agents.push(Box::new(CreativityAgent {
            novel_combinations: 0,
            inferred_relations: 0,
        }));
        GlobalWorkspace {
            agents,
            last_broadcast: None,
            history: Vec::new(),
            max_history: 100,
            refractory: HashMap::new(),
            global_threshold: 0.35,
            default_refractory: 2,
            last_winner: None,
            suppress_repeat_penalty: 0.15,
            n_broadcasts: 0,
            n_skipped: 0,
        }
    }

    /// One step of the workspace: agents compete, winner broadcasted, others inhibited.
    pub fn tick(&mut self, tick: u64) -> Option<Broadcast> {
        // 1. Decrement refractory periods
        let mut to_remove = Vec::new();
        for (name, rem) in &mut self.refractory {
            if *rem > 0 {
                *rem -= 1;
            }
            if *rem == 0 {
                to_remove.push(name.clone());
            }
        }
        for name in to_remove {
            self.refractory.remove(&name);
        }

        // 2. Collect proposals, respecting refractory and threshold
        let mut proposals: Vec<Broadcast> = Vec::new();
        for agent in &mut self.agents {
            if let Some(mut p) = agent.propose(tick) {
                // Skip if in refractory
                if self.refractory.contains_key(&p.agent_name) {
                    continue;
                }
                // Suppress repeat winner
                if self.last_winner.as_ref() == Some(&p.agent_name) {
                    p.confidence = (p.confidence - self.suppress_repeat_penalty).max(0.0);
                }
                // Global threshold (ignition)
                if p.confidence >= self.global_threshold {
                    proposals.push(p);
                }
            }
        }

        // 3. Winner-take-all
        let winner = proposals.into_iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(ref broadcast) = winner {
            // Set refractory for winner (prevents monopolization)
            self.refractory
                .insert(broadcast.agent_name.clone(), self.default_refractory);
            self.last_winner = Some(broadcast.agent_name.clone());
            self.n_broadcasts += 1;
            // Broadcast to all agents (they can update internal state)
            for agent in &mut self.agents {
                agent.receive_broadcast(broadcast);
            }
            self.history.push(broadcast.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        } else {
            self.n_skipped += 1;
            self.last_winner = None;
        }

        self.last_broadcast = winner.clone();
        winner
    }

    pub fn set_perception_input(&mut self, input_len: usize) {
        for agent in &mut self.agents {
            if let Some(p) = agent.as_any_mut().downcast_mut::<PerceptionAgent>() {
                p.last_input_len = input_len;
            }
        }
    }

    pub fn set_goal_state(&mut self, count: usize, discomfort: f32) {
        for agent in &mut self.agents {
            if let Some(g) = agent.as_any_mut().downcast_mut::<GoalAgent>() {
                g.goals_pending = count;
                g.discomfort = discomfort;
            }
        }
    }

    pub fn set_memory_load(&mut self, load: usize) {
        for agent in &mut self.agents {
            if let Some(m) = agent.as_any_mut().downcast_mut::<MemoryAgent>() {
                m.concept_load = load;
                m.retrieval_count = load as u32;
            }
        }
    }

    pub fn set_meta_state(&mut self, error: f32, curiosity: f32) {
        for agent in &mut self.agents {
            if let Some(m) = agent.as_any_mut().downcast_mut::<MetaAgent>() {
                m.error_recent = error;
                m.curiosity = curiosity;
            }
        }
    }

    pub fn set_novelty(&mut self, novelty: f32) {
        for agent in &mut self.agents {
            if let Some(p) = agent.as_any_mut().downcast_mut::<PerceptionAgent>() {
                p.novelty = novelty;
            }
        }
    }

    pub fn set_exploration_state(&mut self, curiosity: f32, n_concepts: usize) {
        for agent in &mut self.agents {
            if let Some(e) = agent.as_any_mut().downcast_mut::<ExplorationAgent>() {
                e.curiosity = curiosity;
                e.n_concepts = n_concepts;
            }
        }
    }

    pub fn set_social_state(&mut self, n_peers: usize, last_message_age: u64) {
        for agent in &mut self.agents {
            if let Some(s) = agent.as_any_mut().downcast_mut::<SocialAgent>() {
                s.n_peers = n_peers;
                s.last_message_age = last_message_age;
            }
        }
    }

    pub fn set_creativity_state(&mut self, novel_combinations: usize, inferred_relations: usize) {
        for agent in &mut self.agents {
            if let Some(c) = agent.as_any_mut().downcast_mut::<CreativityAgent>() {
                c.novel_combinations = novel_combinations;
                c.inferred_relations = inferred_relations;
            }
        }
    }

    pub fn status(&self) -> String {
        let winner = self
            .last_broadcast
            .as_ref()
            .map(|b| b.agent_name.clone())
            .unwrap_or_else(|| "none".to_string());
        let active: Vec<String> = self
            .refractory
            .iter()
            .map(|(n, r)| format!("{}:{}", n, r))
            .collect();
        format!(
            "Society | agents={} | winner={} | broadcasts={} | skipped={} | refractory=[{}]",
            self.agents.len(),
            winner,
            self.n_broadcasts,
            self.n_skipped,
            active.join(",")
        )
    }
}
