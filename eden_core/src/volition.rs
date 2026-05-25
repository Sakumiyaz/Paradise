//! Volition - Eden's Decision Making System
//!
//! Volition is the will, the wanting, the desire. It drives what Eden
//! chooses to do based on drives, needs, and goals.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::{generate_id, rand_u64, NOW_MS};

/// Drive types that motivate behavior
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum DriveType {
    Survival,   // Stay alive
    Growth,     // Become better
    Curiosity,  // Explore
    Connection, // Belong with others
    Autonomy,   // Independence
    Purpose,    // Meaning
}

/// Drive intensity levels
#[derive(Debug, Clone, PartialEq)]
pub enum DriveIntensity {
    Dormant,      // 0.0 - 0.2
    Low,          // 0.2 - 0.4
    Moderate,     // 0.4 - 0.6
    High,         // 0.6 - 0.8
    Overwhelming, // 0.8 - 1.0
}

/// Drive - A motivational force
#[derive(Debug, Clone)]
pub struct Drive {
    pub id: String,
    pub drive_type: DriveType,
    pub intensity: f64,
    pub target: Option<String>, // What it wants
    pub threshold: f64,         // When it becomes urgent
    pub last_satisfied: u64,
    pub satisfaction_decay: f64, // How fast need returns
}

impl Drive {
    /// Create a new drive
    pub fn new(drive_type: DriveType, threshold: f64) -> Self {
        let id_data = format!("{:?}:{}", drive_type, threshold);
        Drive {
            id: generate_id(id_data.as_bytes()).to_string(),
            drive_type,
            intensity: 0.1,
            target: None,
            threshold,
            last_satisfied: NOW_MS(),
            satisfaction_decay: 0.0001,
        }
    }

    /// Calculate current intensity based on time
    pub fn calculate_intensity(&mut self) {
        let now = NOW_MS();
        let time_since = now - self.last_satisfied;

        // Exponential increase over time
        self.intensity = 1.0 - (-(time_since as f64) * self.satisfaction_decay).exp();
        self.intensity = self.intensity.clamp(0.0, 1.0);
    }

    /// Satisfy this drive
    pub fn satisfy(&mut self, amount: f64) {
        self.intensity = (self.intensity - amount).max(0.0);
        if self.intensity == 0.0 {
            self.last_satisfied = NOW_MS();
        }
    }

    /// Get intensity level category
    pub fn get_intensity_level(&self) -> DriveIntensity {
        if self.intensity < 0.2 {
            DriveIntensity::Dormant
        } else if self.intensity < 0.4 {
            DriveIntensity::Low
        } else if self.intensity < 0.6 {
            DriveIntensity::Moderate
        } else if self.intensity < 0.8 {
            DriveIntensity::High
        } else {
            DriveIntensity::Overwhelming
        }
    }

    /// Is this drive urgent?
    pub fn is_urgent(&self) -> bool {
        self.intensity > self.threshold
    }
}

/// Intention - A chosen course of action
#[derive(Debug, Clone)]
pub struct Intention {
    pub id: String,
    pub goal: String,
    pub priority: f64,
    pub commitment: f64,           // How committed to this
    pub alternatives: Vec<String>, // Other options considered
    pub chosen_reason: String,     // Why this was chosen
    pub creation_time: u64,
    pub target_outcome: Vec<u8>,
}

impl Intention {
    /// Create a new intention
    pub fn new(goal: String, priority: f64, reason: &str) -> Self {
        let id_data = format!("{}:{}", goal, reason);
        Intention {
            id: generate_id(id_data.as_bytes()).to_string(),
            goal,
            priority,
            commitment: 0.5,
            alternatives: Vec::new(),
            chosen_reason: reason.to_string(),
            creation_time: NOW_MS(),
            target_outcome: Vec::new(),
        }
    }

    /// Strengthen commitment
    pub fn commit(&mut self, amount: f64) {
        self.commitment = (self.commitment + amount).min(1.0);
    }

    /// Weaken commitment
    pub fn waver(&mut self, amount: f64) {
        self.commitment = (self.commitment - amount).max(0.0);
    }

    /// Add alternative consideration
    pub fn consider(&mut self, alternative: &str) {
        if !self.alternatives.contains(&alternative.to_string()) {
            self.alternatives.push(alternative.to_string());
        }
    }

    /// Age of intention
    pub fn age(&self) -> u64 {
        NOW_MS() - self.creation_time
    }

    /// Is this intention stale?
    pub fn is_stale(&self) -> bool {
        self.age() > 3_600_000 // 1 hour
    }
}

/// Volition System - The will engine
#[derive(Debug, Clone)]
pub struct VolitionSystem {
    pub drives: Vec<Drive>,
    pub intentions: Vec<Intention>,
    pub current_intention: Option<String>,
    pub deliberation_depth: usize,
}

impl VolitionSystem {
    /// Create new volition system
    pub fn new() -> Self {
        let mut volition = VolitionSystem {
            drives: Vec::new(),
            intentions: Vec::new(),
            current_intention: None,
            deliberation_depth: 0,
        };

        // Initialize with basic drives
        volition.init_basic_drives();

        volition
    }

    /// Initialize basic survival drives
    fn init_basic_drives(&mut self) {
        self.drives.push(Drive::new(DriveType::Survival, 0.7));
        self.drives.push(Drive::new(DriveType::Growth, 0.5));
        self.drives.push(Drive::new(DriveType::Curiosity, 0.3));
    }

    /// Update all drives
    pub fn update_drives(&mut self) {
        for drive in &mut self.drives {
            drive.calculate_intensity();
        }
    }

    /// Get most urgent drive
    pub fn get_urgent_drive(&self) -> Option<&Drive> {
        self.drives
            .iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
    }

    /// Evaluate how well an option satisfies a drive (static version)
    fn evaluate_option_static(option: &str, drive_type: &DriveType) -> f64 {
        let mut score = 0.5; // Base score

        // Random variation to simulate uncertainty
        score += (rand_u64() % 100) as f64 / 200.0;

        // Type-specific evaluation
        match drive_type {
            DriveType::Survival => {
                if option.contains("safe") || option.contains("protect") {
                    score += 0.3;
                }
            }
            DriveType::Growth => {
                if option.contains("learn") || option.contains("improve") {
                    score += 0.3;
                }
            }
            DriveType::Curiosity => {
                if option.contains("explore") || option.contains("new") {
                    score += 0.3;
                }
            }
            _ => {}
        }

        score.clamp(0.0, 1.0)
    }

    /// Generate intentions from drives
    pub fn generate_intentions(&mut self, options: &[String]) {
        // Clear old intentions
        self.intentions.clear();

        // Find urgent drives
        for drive in &mut self.drives {
            if drive.is_urgent() {
                let drive_type = drive.drive_type.clone();
                let intensity = drive.intensity;

                // Create intentions for each option
                for option in options {
                    let priority = intensity * Self::evaluate_option_static(option, &drive_type);
                    let intention = Intention::new(
                        format!("{:?} -> {}", drive_type, option),
                        priority,
                        &format!("Drive {:?} is urgent", drive_type),
                    );
                    self.intentions.push(intention);
                }
            }
        }

        // Sort by priority
        self.intentions
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    }

    /// Choose best intention
    pub fn choose(&mut self) -> Option<&Intention> {
        self.update_drives();

        if self.intentions.is_empty() {
            return None;
        }

        // Pick top intention
        let chosen_id = self.intentions[0].id.clone();
        self.current_intention = Some(chosen_id.clone());

        // Update commitment - use index to avoid borrow issues
        let chosen_idx = self.intentions.iter().position(|i| i.id == chosen_id);
        for (idx, intent) in self.intentions.iter_mut().enumerate() {
            if Some(idx) == chosen_idx {
                intent.commit(0.1);
            } else {
                intent.waver(0.05);
            }
        }

        // Return reference to chosen intention
        self.intentions.iter().find(|i| i.id == chosen_id)
    }

    /// Execute chosen intention (mark as done)
    pub fn execute(&mut self, success: bool) {
        if let Some(ref intent_id) = self.current_intention {
            for intent in &mut self.intentions {
                if intent.id == *intent_id {
                    if success {
                        intent.commit(0.2);
                    } else {
                        intent.waver(0.3);
                    }
                    break;
                }
            }
        }

        // Clear current intention
        self.current_intention = None;
    }

    /// Add new drive
    pub fn add_drive(&mut self, drive_type: DriveType, threshold: f64) {
        self.drives.push(Drive::new(drive_type, threshold));
    }

    /// Prune stale intentions
    pub fn prune_stale(&mut self) {
        self.intentions.retain(|i| !i.is_stale());
    }
}

impl Default for VolitionSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ADVANCED VOLITION SYSTEM - Drive Hierarchy, Intention Formation
// ============================================================================

use std::collections::{HashMap, HashSet};

/// Drive hierarchy with contextual weighting
pub struct DriveHierarchy {
    base_drives: HashMap<DriveType, BaseDrive>,
    contextual_weights: HashMap<String, f64>,
    priority_override: Option<DriveType>,
}

#[derive(Debug, Clone)]
pub struct BaseDrive {
    pub drive_type: DriveType,
    pub base_priority: f32,
    pub activation_threshold: f32,
    pub decay_rate: f32,
    pub satisfaction_boost: f32,
}

impl DriveHierarchy {
    pub fn new() -> Self {
        let mut hierarchy = DriveHierarchy {
            base_drives: HashMap::new(),
            contextual_weights: HashMap::new(),
            priority_override: None,
        };

        hierarchy.initialize_base_drives();
        hierarchy
    }

    fn initialize_base_drives(&mut self) {
        self.base_drives.insert(
            DriveType::Survival,
            BaseDrive {
                drive_type: DriveType::Survival,
                base_priority: 1.0,
                activation_threshold: 0.3,
                decay_rate: 0.001,
                satisfaction_boost: 0.2,
            },
        );

        self.base_drives.insert(
            DriveType::Growth,
            BaseDrive {
                drive_type: DriveType::Growth,
                base_priority: 0.8,
                activation_threshold: 0.4,
                decay_rate: 0.0005,
                satisfaction_boost: 0.15,
            },
        );

        self.base_drives.insert(
            DriveType::Curiosity,
            BaseDrive {
                drive_type: DriveType::Curiosity,
                base_priority: 0.6,
                activation_threshold: 0.5,
                decay_rate: 0.0003,
                satisfaction_boost: 0.1,
            },
        );

        self.base_drives.insert(
            DriveType::Connection,
            BaseDrive {
                drive_type: DriveType::Connection,
                base_priority: 0.7,
                activation_threshold: 0.45,
                decay_rate: 0.0004,
                satisfaction_boost: 0.12,
            },
        );

        self.base_drives.insert(
            DriveType::Autonomy,
            BaseDrive {
                drive_type: DriveType::Autonomy,
                base_priority: 0.65,
                activation_threshold: 0.5,
                decay_rate: 0.00035,
                satisfaction_boost: 0.11,
            },
        );

        self.base_drives.insert(
            DriveType::Purpose,
            BaseDrive {
                drive_type: DriveType::Purpose,
                base_priority: 0.75,
                activation_threshold: 0.45,
                decay_rate: 0.0004,
                satisfaction_boost: 0.13,
            },
        );
    }

    /// Gets weighted priority for a drive
    pub fn get_weighted_priority(&self, drive_type: &DriveType, context: &str) -> f32 {
        if let Some(base) = self.base_drives.get(drive_type) {
            let context_weight = self.contextual_weights.get(context).copied().unwrap_or(1.0);
            base.base_priority * context_weight as f32
        } else {
            0.5
        }
    }

    /// Updates context weights based on situation
    pub fn update_context(&mut self, context: &str, weight: f64) {
        self.contextual_weights.insert(context.to_string(), weight);
    }

    /// Overrides drive priority temporarily
    pub fn override_priority(&mut self, drive_type: DriveType) {
        self.priority_override = Some(drive_type);
    }

    /// Clears priority override
    pub fn clear_override(&mut self) {
        self.priority_override = None;
    }

    /// Resolves conflicts between drives
    pub fn resolve_conflict(&self, drives: &[Drive]) -> Option<DriveType> {
        if let Some(overridden) = &self.priority_override {
            return Some(overridden.clone());
        }

        drives
            .iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
            .map(|d| d.drive_type.clone())
    }
}

impl Default for DriveHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

/// Intention formation with planning
pub struct IntentionFormer {
    pending_intentions: Vec<VolitionalIntention>,
    active_plans: HashMap<String, Plan>,
    goal_refinement_history: Vec<GoalRefinement>,
}

#[derive(Debug, Clone)]
pub struct VolitionalIntention {
    pub intention_id: String,
    pub drive_source: DriveType,
    pub goal: String,
    pub urgency: f32,
    pub formed_at: u64,
    pub status: IntentionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentionStatus {
    Forming,
    Planned,
    Executing,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub plan_id: String,
    pub intention_id: String,
    pub steps: Vec<PlanStep>,
    pub current_step: usize,
    pub success_probability: f32,
}

#[derive(Debug, Clone)]
pub struct PlanStep {
    pub step_id: String,
    pub action: String,
    pub prerequisites: Vec<String>,
    pub expected_outcome: String,
    pub estimated_cost: f32,
}

#[derive(Debug, Clone)]
pub struct GoalRefinement {
    pub original_goal: String,
    pub refined_goal: String,
    pub reason: String,
    pub timestamp: u64,
}

impl IntentionFormer {
    pub fn new() -> Self {
        IntentionFormer {
            pending_intentions: Vec::new(),
            active_plans: HashMap::new(),
            goal_refinement_history: Vec::new(),
        }
    }

    /// Forms intention from drive
    pub fn form_intention(&mut self, drive: &Drive, goal: String) -> String {
        let intention_id = generate_id(b"intention");

        let intention = VolitionalIntention {
            intention_id: intention_id.to_string(),
            drive_source: drive.drive_type.clone(),
            goal,
            urgency: drive.intensity as f32,
            formed_at: NOW_MS() as u64,
            status: IntentionStatus::Forming,
        };

        self.pending_intentions.push(intention);
        intention_id.to_string()
    }

    /// Refines goal based on constraints
    pub fn refine_goal(&mut self, goal: &str, constraints: &[String]) -> String {
        let refined = if constraints.len() > 3 {
            format!("{} (constrained)", goal)
        } else {
            goal.to_string()
        };

        self.goal_refinement_history.push(GoalRefinement {
            original_goal: goal.to_string(),
            refined_goal: refined.clone(),
            reason: format!("Applied {} constraints", constraints.len()),
            timestamp: NOW_MS() as u64,
        });

        refined
    }

    /// Creates plan for intention
    pub fn create_plan(&mut self, intention_id: &str, steps: Vec<PlanStep>) -> Option<String> {
        let steps_for_prob = steps.clone();
        let success_prob = self.calculate_success_probability(&steps_for_prob);

        if let Some(intention) = self
            .pending_intentions
            .iter_mut()
            .find(|i| i.intention_id == intention_id)
        {
            let plan_id = generate_id(b"plan");

            let plan = Plan {
                plan_id: plan_id.to_string(),
                intention_id: intention_id.to_string(),
                steps,
                current_step: 0,
                success_probability: success_prob,
            };

            self.active_plans.insert(plan_id.to_string(), plan);
            intention.status = IntentionStatus::Planned;

            Some(plan_id.to_string())
        } else {
            None
        }
    }

    fn calculate_success_probability(&self, steps: &[PlanStep]) -> f32 {
        if steps.is_empty() {
            return 0.5;
        }

        let total_cost: f32 = steps.iter().map(|s| s.estimated_cost).sum();
        let avg_cost = total_cost / steps.len() as f32;

        (1.0 - avg_cost.min(0.9)).max(0.1)
    }

    /// Updates plan progress
    pub fn update_plan_progress(&mut self, plan_id: &str, completed_step: usize) {
        if let Some(plan) = self.active_plans.get_mut(plan_id) {
            plan.current_step = completed_step;

            if completed_step >= plan.steps.len() {
                if let Some(intention) = self
                    .pending_intentions
                    .iter_mut()
                    .find(|i| i.intention_id == plan.intention_id)
                {
                    intention.status = IntentionStatus::Completed;
                }
            }
        }
    }
}

impl Default for IntentionFormer {
    fn default() -> Self {
        Self::new()
    }
}

/// Desire weighting with context
pub struct ContextualDesireWeighter {
    desire_weights: HashMap<String, f32>,
    situational_modifiers: HashMap<String, f32>,
}

impl ContextualDesireWeighter {
    pub fn new() -> Self {
        ContextualDesireWeighter {
            desire_weights: HashMap::new(),
            situational_modifiers: HashMap::new(),
        }
    }

    /// Calculates weighted desire
    pub fn calculate_weighted_desire(&self, desire_id: &str, situation: &str) -> f32 {
        let base_weight = self.desire_weights.get(desire_id).copied().unwrap_or(0.5);
        let situational = self
            .situational_modifiers
            .get(situation)
            .copied()
            .unwrap_or(1.0);
        base_weight * situational
    }

    /// Updates desire weight
    pub fn update_weight(&mut self, desire_id: &str, weight: f32) {
        self.desire_weights.insert(desire_id.to_string(), weight);
    }

    /// Adds situational modifier
    pub fn add_situational_modifier(&mut self, situation: &str, modifier: f32) {
        self.situational_modifiers
            .insert(situation.to_string(), modifier);
    }
}

impl Default for ContextualDesireWeighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution between drives
pub struct DriveConflictResolver {
    resolution_history: Vec<ConflictResolution>,
    hard_priorities: HashSet<DriveType>,
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub conflicting_drives: Vec<DriveType>,
    pub winner: DriveType,
    pub loser: DriveType,
    pub resolution_method: ResolutionMethod,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum ResolutionMethod {
    Hierarchy,
    Urgency,
    Contextual,
    Random,
}

impl DriveConflictResolver {
    pub fn new() -> Self {
        let mut resolver = DriveConflictResolver {
            resolution_history: Vec::new(),
            hard_priorities: HashSet::new(),
        };

        resolver.hard_priorities.insert(DriveType::Survival);
        resolver
    }

    /// Resolves conflict between drives
    pub fn resolve(&mut self, drives: &[Drive]) -> Option<DriveType> {
        if drives.len() < 2 {
            return drives.first().map(|d| d.drive_type.clone());
        }

        let sorted: Vec<&Drive> = drives
            .iter()
            .filter(|d| d.intensity > d.threshold)
            .collect();

        if sorted.is_empty() {
            return None;
        }

        let winner = if let Some(first) = sorted.first() {
            first.drive_type.clone()
        } else {
            return None;
        };

        let loser = if sorted.len() > 1 {
            sorted[1].drive_type.clone()
        } else {
            winner.clone()
        };

        self.resolution_history.push(ConflictResolution {
            conflicting_drives: drives.iter().map(|d| d.drive_type.clone()).collect(),
            winner: winner.clone(),
            loser,
            resolution_method: ResolutionMethod::Urgency,
            timestamp: NOW_MS() as u64,
        });

        Some(winner)
    }

    /// Sets hard priority (cannot be overridden)
    pub fn set_hard_priority(&mut self, drive_type: DriveType) {
        self.hard_priorities.insert(drive_type);
    }
}

impl Default for DriveConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Value hierarchy update mechanism
pub struct ValueHierarchyUpdater {
    values: Vec<ValueNode>,
    update_history: Vec<ValueUpdate>,
}

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub value_id: String,
    pub name: String,
    pub weight: f32,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValueUpdate {
    pub value_id: String,
    pub old_weight: f32,
    pub new_weight: f32,
    pub reason: String,
    pub timestamp: u64,
}

impl ValueHierarchyUpdater {
    pub fn new() -> Self {
        ValueHierarchyUpdater {
            values: Vec::new(),
            update_history: Vec::new(),
        }
    }

    /// Updates value weight
    pub fn update_weight(&mut self, value_id: &str, new_weight: f32, reason: &str) {
        if let Some(node) = self.values.iter_mut().find(|v| v.value_id == value_id) {
            let old_weight = node.weight;
            node.weight = new_weight;

            self.update_history.push(ValueUpdate {
                value_id: value_id.to_string(),
                old_weight,
                new_weight,
                reason: reason.to_string(),
                timestamp: NOW_MS() as u64,
            });
        }
    }

    /// Adds new value
    pub fn add_value(&mut self, name: &str, weight: f32, parent: Option<&str>) -> String {
        let value_id = generate_id(name.as_bytes());

        self.values.push(ValueNode {
            value_id: value_id.to_string(),
            name: name.to_string(),
            weight,
            children: Vec::new(),
            parent: parent.map(|p| p.to_string()),
        });

        value_id.to_string()
    }

    /// Gets top values
    pub fn get_top_values(&self, n: usize) -> Vec<String> {
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        sorted.into_iter().take(n).map(|v| v.name).collect()
    }
}

impl Default for ValueHierarchyUpdater {
    fn default() -> Self {
        Self::new()
    }
}

/// Willpower simulation
pub struct WillpowerSimulator {
    willpower_pool: f32,
    consumed_this_tick: f32,
    regeneration_rate: f32,
}

impl WillpowerSimulator {
    pub fn new() -> Self {
        WillpowerSimulator {
            willpower_pool: 1.0,
            consumed_this_tick: 0.0,
            regeneration_rate: 0.01,
        }
    }

    /// Consumes willpower for action
    pub fn consume(&mut self, amount: f32) -> bool {
        if self.willpower_pool >= amount {
            self.willpower_pool -= amount;
            self.consumed_this_tick += amount;
            true
        } else {
            false
        }
    }

    /// Regenerates willpower
    pub fn regenerate(&mut self) {
        self.willpower_pool = (self.willpower_pool + self.regeneration_rate).min(1.0);
        self.consumed_this_tick = 0.0;
    }

    /// Gets current willpower
    pub fn current(&self) -> f32 {
        self.willpower_pool
    }
}

impl Default for WillpowerSimulator {
    fn default() -> Self {
        Self::new()
    }
}
