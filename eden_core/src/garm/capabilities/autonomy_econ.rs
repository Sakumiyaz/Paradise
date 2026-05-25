// EDEN GARM AutonomyEcon — Autonomia economica y gestion de recursos.
// 100% Rust puro, 0 LLM, 0 red.
//
// Recursos: tiempo de CPU, memoria RAM, energia (simulada), conocimiento
// Planning jerarquico: metas a largo plazo -> submetas -> acciones concretas
// Auto-evaluacion: comparar progreso esperado vs real, ajustar planes

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Resource {
    pub name: String,
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32, // per tick
    pub consumption_rate: f32,
}

impl Resource {
    pub fn new(name: &str, max: f32, regen: f32, consume: f32) -> Self {
        Resource {
            name: name.to_string(),
            current: max,
            max,
            regeneration_rate: regen,
            consumption_rate: consume,
        }
    }
    pub fn tick(&mut self) {
        self.current =
            (self.current + self.regeneration_rate - self.consumption_rate).clamp(0.0, self.max);
    }
    pub fn fraction(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug)]
pub struct HierarchicalGoal {
    pub id: u64,
    pub label: String,
    pub parent_id: Option<u64>,
    pub sub_goals: Vec<u64>,
    pub progress: f32, // 0..1
    pub priority: f32,
    pub deadline_tick: u64,
    pub completed: bool,
    pub expected_outcome: String,
    pub actual_outcome: String,
}

#[derive(Clone, Debug)]
pub struct AutonomyEcon {
    pub resources: HashMap<String, Resource>,
    pub goals: HashMap<u64, HierarchicalGoal>,
    pub goal_stack: Vec<u64>,
    pub next_goal_id: u64,
    pub n_evaluations: u64,
    pub n_plan_adjustments: u64,
    pub total_ticks: u64,
}

impl AutonomyEcon {
    pub fn new() -> Self {
        let mut resources = HashMap::new();
        resources.insert(
            "cpu_time".to_string(),
            Resource::new("CPU time", 100.0, 10.0, 5.0),
        );
        resources.insert(
            "memory".to_string(),
            Resource::new("Memory", 1000.0, 50.0, 30.0),
        );
        resources.insert(
            "energy".to_string(),
            Resource::new("Energy", 100.0, 2.0, 3.0),
        );
        resources.insert(
            "knowledge".to_string(),
            Resource::new("Knowledge", 10000.0, 5.0, 0.0),
        );
        AutonomyEcon {
            resources,
            goals: HashMap::new(),
            goal_stack: Vec::new(),
            next_goal_id: 1,
            n_evaluations: 0,
            n_plan_adjustments: 0,
            total_ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        self.total_ticks += 1;
        for res in self.resources.values_mut() {
            res.tick();
        }
        // Auto-generate goal if resources are low
        let low_resources: Vec<String> = self
            .resources
            .iter()
            .filter(|(_, res)| res.fraction() < 0.2)
            .map(|(name, _)| name.clone())
            .collect();
        for name in low_resources {
            let label = format!("acquire more {}", name);
            let id = self.push_goal(&label, 0.9, self.total_ticks + 100);
            if let Some(g) = self.goals.get_mut(&id) {
                g.expected_outcome = format!("{} restored to 50%", name);
            }
        }
    }

    pub fn push_goal(&mut self, label: &str, priority: f32, deadline: u64) -> u64 {
        let id = self.next_goal_id;
        self.next_goal_id += 1;
        let goal = HierarchicalGoal {
            id,
            label: label.to_string(),
            parent_id: None,
            sub_goals: Vec::new(),
            progress: 0.0,
            priority,
            deadline_tick: deadline,
            completed: false,
            expected_outcome: String::new(),
            actual_outcome: String::new(),
        };
        self.goals.insert(id, goal);
        self.goal_stack.push(id);
        id
    }

    pub fn decompose_goal(&mut self, parent_id: u64, sub_labels: &[String]) {
        let mut sub_ids = Vec::new();
        for lbl in sub_labels {
            let id = self.next_goal_id;
            self.next_goal_id += 1;
            sub_ids.push(id);
            self.goals.insert(
                id,
                HierarchicalGoal {
                    id,
                    label: lbl.clone(),
                    parent_id: Some(parent_id),
                    sub_goals: Vec::new(),
                    progress: 0.0,
                    priority: 0.5,
                    deadline_tick: self.total_ticks + 50,
                    completed: false,
                    expected_outcome: String::new(),
                    actual_outcome: String::new(),
                },
            );
        }
        if let Some(g) = self.goals.get_mut(&parent_id) {
            g.sub_goals.extend(&sub_ids);
        }
    }

    pub fn evaluate_progress(&mut self, goal_id: u64, actual: &str) {
        self.n_evaluations += 1;
        if let Some(g) = self.goals.get_mut(&goal_id) {
            g.actual_outcome = actual.to_string();
            // Simple evaluation: if actual contains expected keywords, progress up
            if !g.expected_outcome.is_empty()
                && actual
                    .to_lowercase()
                    .contains(&g.expected_outcome.to_lowercase())
            {
                g.progress = (g.progress + 0.3).min(1.0);
            } else {
                g.progress = (g.progress + 0.1).min(1.0);
            }
            if g.progress >= 1.0 {
                g.completed = true;
            }
            // If actual diverges from expected, adjust
            if !actual
                .to_lowercase()
                .contains(&g.expected_outcome.to_lowercase())
                && !g.expected_outcome.is_empty()
            {
                self.n_plan_adjustments += 1;
            }
        }
    }

    pub fn active_goals(&self) -> Vec<&HierarchicalGoal> {
        self.goals.values().filter(|g| !g.completed).collect()
    }

    pub fn report_resources(&self) -> String {
        let mut out = "Recursos:\n".to_string();
        for (_, r) in self.resources.iter() {
            out.push_str(&format!(
                "  {} | {:.1}/{:.1} ({:.1}%)\n",
                r.name,
                r.current,
                r.max,
                r.fraction() * 100.0
            ));
        }
        out
    }

    pub fn report_goals(&self) -> String {
        let mut out = "Metas jerarquicas:\n".to_string();
        let active: Vec<&HierarchicalGoal> = self.active_goals();
        for g in active.iter().take(10) {
            let indent = if g.parent_id.is_some() { "  " } else { "" };
            out.push_str(&format!(
                "{}[{}] '{}' | p={:.2} | prog={:.1}% | deadline=t{}\n",
                indent,
                g.id,
                g.label,
                g.priority,
                g.progress * 100.0,
                g.deadline_tick
            ));
        }
        out
    }

    pub fn status(&self) -> String {
        let completed = self.goals.values().filter(|g| g.completed).count();
        format!("AutonomyEcon | resources={} | goals={} | completed={} | active={} | evals={} | adjustments={}",
            self.resources.len(), self.goals.len(), completed,
            self.goals.len() - completed, self.n_evaluations, self.n_plan_adjustments,
        )
    }
}
