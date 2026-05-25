// EDEN GARM Intention Hierarchy — Hierarchical goal stack with temporal credit assignment
// Goals decompose into sub-goals. Success/failure propagates credit up the hierarchy (TD-like).

#[derive(Clone, Debug)]
pub struct Goal {
    pub id: u64,
    pub label: String,
    pub priority: f32, // 0..1
    pub parent_id: Option<u64>,
    pub sub_goals: Vec<u64>,
    pub deadline_tick: Option<u64>,
    pub progress: f32, // 0..1
    pub completed: bool,
    pub failed: bool,
}

pub struct GoalStack {
    pub goals: std::collections::HashMap<u64, Goal>,
    pub stack: Vec<u64>, // top of stack = current active goal
    pub next_id: u64,
    pub gamma: f32, // TD discount factor
}

impl GoalStack {
    pub fn new() -> Self {
        GoalStack {
            goals: std::collections::HashMap::new(),
            stack: Vec::new(),
            next_id: 1,
            gamma: 0.9,
        }
    }

    pub fn push(
        &mut self,
        label: &str,
        priority: f32,
        deadline_tick: Option<u64>,
        parent_id: Option<u64>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let goal = Goal {
            id,
            label: label.to_string(),
            priority,
            parent_id,
            sub_goals: Vec::new(),
            deadline_tick,
            progress: 0.0,
            completed: false,
            failed: false,
        };
        self.goals.insert(id, goal);
        if let Some(pid) = parent_id {
            if let Some(parent) = self.goals.get_mut(&pid) {
                parent.sub_goals.push(id);
            }
        }
        self.stack.push(id);
        id
    }

    pub fn pop(&mut self) -> Option<u64> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<&Goal> {
        self.stack.last().and_then(|id| self.goals.get(id))
    }

    pub fn complete_goal(&mut self, id: u64, reward: f32) {
        if let Some(goal) = self.goals.get_mut(&id) {
            goal.completed = true;
            goal.progress = 1.0;
        }
        // TD credit assignment up the hierarchy
        let mut current_id = Some(id);
        let mut discounted = reward;
        while let Some(gid) = current_id {
            if let Some(goal) = self.goals.get_mut(&gid) {
                goal.progress = (goal.progress + discounted).min(1.0);
                discounted *= self.gamma;
                current_id = goal.parent_id;
            } else {
                break;
            }
        }
    }

    pub fn fail_goal(&mut self, id: u64, penalty: f32) {
        if let Some(goal) = self.goals.get_mut(&id) {
            goal.failed = true;
            goal.progress = (goal.progress - penalty).max(0.0);
        }
        // Propagate penalty up
        let mut current_id = self.goals.get(&id).and_then(|g| g.parent_id);
        let mut discounted = penalty * self.gamma;
        while let Some(gid) = current_id {
            if let Some(goal) = self.goals.get_mut(&gid) {
                goal.progress = (goal.progress - discounted).max(0.0);
                discounted *= self.gamma;
                current_id = goal.parent_id;
            } else {
                break;
            }
        }
    }

    /// Push a high-level goal derived from a drive. It auto-decomposes when active.
    pub fn push_drive_goal(&mut self, drive_label: &str, discomfort: f32, tick: u64) -> u64 {
        let priority = discomfort.clamp(0.0, 1.0);
        let root = self.push(
            &format!("drive:{}", drive_label),
            priority,
            Some(tick + 50),
            None,
        );
        // Auto-decompose based on drive
        match drive_label {
            "curiosity" => {
                let _ = self.push(
                    "explore_unknown",
                    priority * 0.8,
                    Some(tick + 20),
                    Some(root),
                );
            }
            "efficiency" => {
                let _ = self.push("reduce_load", priority * 0.8, Some(tick + 20), Some(root));
            }
            "stability" => {
                let _ = self.push(
                    "predict_better",
                    priority * 0.8,
                    Some(tick + 20),
                    Some(root),
                );
            }
            _ => {}
        }
        root
    }

    /// Peek the top goal id without removing it.
    pub fn pop_top(&self) -> Option<u64> {
        self.stack.last().copied()
    }

    /// TD update for a goal and its ancestors given a reward.
    pub fn td_update(&mut self, id: u64, reward: f32, _tick: u64) {
        self.complete_goal(id, reward);
    }

    pub fn status(&self) -> String {
        let top_label = self
            .top()
            .map(|g| g.label.clone())
            .unwrap_or_else(|| "none".to_string());
        format!(
            "Goals | stack: {} | top: {} | total: {}",
            self.stack.len(),
            top_label,
            self.goals.len()
        )
    }
}
