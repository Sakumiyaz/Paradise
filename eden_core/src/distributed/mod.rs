//! # DISTRIBUTED - Supercomputing Grid
//!
//! Grid computacional P2P para task distribution, load balancing,
//! computing power agregado que rivaliza con JARVIS.
//! Sin dependencias externas - 100% Rust.

#![allow(dead_code)]

mod compute;
mod node;
mod scheduler;
mod task;

pub use compute::ComputeEngine;
pub use node::{ComputeNode, NodeCapabilities, NodeInfo, NodeStatus};
pub use scheduler::{LoadBalancer, Scheduler};
pub use task::{Task, TaskResult, TaskStatus, TaskType};

/// Nodo computacional en el grid
pub struct ComputeGrid {
    nodes: std::collections::HashMap<String, ComputeNode>,
    scheduler: Scheduler,
    tasks: std::collections::HashMap<String, Task>,
    results: std::collections::HashMap<String, TaskResult>,
}

impl ComputeGrid {
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
            scheduler: Scheduler::new(),
            tasks: std::collections::HashMap::new(),
            results: std::collections::HashMap::new(),
        }
    }

    /// Registra un nodo en el grid
    pub fn register_node(&mut self, node: ComputeNode) {
        let node_info = node.info.clone();
        self.nodes.insert(node_info.id.clone(), node);
        self.scheduler.add_node(&node_info);
    }

    /// Desregistra un nodo
    pub fn unregister_node(&mut self, node_id: &str) -> Option<ComputeNode> {
        self.scheduler.remove_node(node_id);
        self.nodes.remove(node_id)
    }

    /// Submit una tarea al grid
    pub fn submit_task(&mut self, task: Task) -> String {
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        task_id
    }

    /// Obtiene resultado de una tarea
    pub fn get_result(&self, task_id: &str) -> Option<&TaskResult> {
        self.results.get(task_id)
    }

    /// Obtiene estado del grid
    pub fn stats(&self) -> GridStats {
        GridStats {
            active_nodes: self
                .nodes
                .values()
                .filter(|n| n.status == NodeStatus::Active)
                .count(),
            total_nodes: self.nodes.len(),
            pending_tasks: self
                .tasks
                .values()
                .filter(|t| t.status == TaskStatus::Pending)
                .count(),
            completed_tasks: self.results.len(),
            total_compute_power: self
                .nodes
                .values()
                .map(|n| n.capabilities.compute_power)
                .sum(),
        }
    }

    /// Balancea carga entre nodos
    pub fn rebalance(&mut self) {
        self.scheduler.balance_load(&self.nodes);
    }
}

impl Default for ComputeGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del grid
#[derive(Debug, Clone)]
pub struct GridStats {
    pub active_nodes: usize,
    pub total_nodes: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
    pub total_compute_power: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = ComputeGrid::new();
        assert_eq!(grid.stats().total_nodes, 0);
    }
}

// ============================================================================
// ADVANCED DISTRIBUTED COMPUTING - Enhanced Grid Computing
// ============================================================================

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Genetic algorithm-based load balancer
pub struct GeneticLoadBalancer {
    population_size: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    generations: usize,
}

#[derive(Debug, Clone)]
pub struct Chromosome {
    pub task_assignments: Vec<(String, String)>,
    pub fitness: f64,
}

impl GeneticLoadBalancer {
    pub fn new() -> Self {
        GeneticLoadBalancer {
            population_size: 50,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            generations: 100,
        }
    }

    /// Optimizes task distribution using genetic algorithms
    pub fn optimize(&self, tasks: &[Task], nodes: &[ComputeNode]) -> HashMap<String, String> {
        let mut population = self.initialize_population(tasks, nodes);
        let mut best_solution: Option<Chromosome> = None;

        for _ in 0..self.generations {
            for chromosome in &mut population {
                chromosome.fitness = self.evaluate_fitness(chromosome, nodes);
            }

            population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

            if let Some(best) = population.first() {
                if best_solution.is_none() || best.fitness > best_solution.as_ref().unwrap().fitness
                {
                    best_solution = Some(best.clone());
                }
            }

            population = self.evolve_population(population, nodes);
        }

        let mut assignments = HashMap::new();
        if let Some(best) = best_solution {
            for (task_id, node_id) in best.task_assignments {
                assignments.insert(task_id, node_id);
            }
        }
        assignments
    }

    fn initialize_population(&self, tasks: &[Task], nodes: &[ComputeNode]) -> Vec<Chromosome> {
        let mut population = Vec::new();
        let mut rng = RandSimple::new();

        for _ in 0..self.population_size {
            let mut assignments = Vec::new();
            for task in tasks {
                if let Some(node) = nodes.iter().nth(rng.next_index(nodes.len())) {
                    assignments.push((task.id.clone(), node.info.id.clone()));
                }
            }
            population.push(Chromosome {
                task_assignments: assignments,
                fitness: 0.0,
            });
        }

        population
    }

    fn evaluate_fitness(&self, chromosome: &Chromosome, _nodes: &[ComputeNode]) -> f64 {
        let mut node_loads: HashMap<String, u64> = HashMap::new();

        for (_, node_id) in &chromosome.task_assignments {
            *node_loads.entry(node_id.clone()).or_insert(0) += 1;
        }

        if node_loads.is_empty() {
            return 0.0;
        }

        let loads: Vec<u64> = node_loads.values().cloned().collect();
        let avg_load: f64 = loads.iter().sum::<u64>() as f64 / loads.len() as f64;

        let variance: f64 = loads
            .iter()
            .map(|l| {
                let diff = *l as f64 - avg_load;
                diff * diff
            })
            .sum::<f64>()
            / loads.len() as f64;

        1.0 / (1.0 + variance)
    }

    fn evolve_population(
        &self,
        population: Vec<Chromosome>,
        nodes: &[ComputeNode],
    ) -> Vec<Chromosome> {
        let mut new_population = Vec::new();
        let elite_count = (self.population_size / 10).max(1);
        new_population.extend(population.iter().take(elite_count).cloned());

        let mut rng = RandSimple::new();

        while new_population.len() < self.population_size {
            let parent1 = self.tournament_select(&population, &mut rng);
            let parent2 = self.tournament_select(&population, &mut rng);

            let mut child = if rng.next_f64() < self.crossover_rate {
                self.crossover(&parent1, &parent2)
            } else {
                parent1.clone()
            };

            if rng.next_f64() < self.mutation_rate {
                self.mutate(&mut child, nodes, &mut rng);
            }

            new_population.push(child);
        }

        new_population
    }

    fn tournament_select(&self, population: &[Chromosome], rng: &mut RandSimple) -> Chromosome {
        let tournament_size = 5;
        let mut best: Option<Chromosome> = None;

        for _ in 0..tournament_size {
            let idx = rng.next_index(population.len());
            if best.is_none() || population[idx].fitness > best.as_ref().unwrap().fitness {
                best = Some(population[idx].clone());
            }
        }

        best.unwrap_or_else(|| population[0].clone())
    }

    fn crossover(&self, parent1: &Chromosome, parent2: &Chromosome) -> Chromosome {
        let mut child_assignments = Vec::new();
        let min_len = parent1
            .task_assignments
            .len()
            .min(parent2.task_assignments.len());

        for i in 0..min_len {
            if i % 2 == 0 {
                child_assignments.push(parent1.task_assignments[i].clone());
            } else {
                child_assignments.push(parent2.task_assignments[i].clone());
            }
        }

        Chromosome {
            task_assignments: child_assignments,
            fitness: 0.0,
        }
    }

    fn mutate(&self, chromosome: &mut Chromosome, nodes: &[ComputeNode], rng: &mut RandSimple) {
        if !chromosome.task_assignments.is_empty() && !nodes.is_empty() {
            let idx = rng.next_index(chromosome.task_assignments.len());
            if let Some(node) = nodes.iter().nth(rng.next_index(nodes.len())) {
                chromosome.task_assignments[idx].1 = node.info.id.clone();
            }
        }
    }
}

impl Default for GeneticLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple random generator
struct RandSimple(u64);

impl RandSimple {
    fn new() -> Self {
        RandSimple(timestamp_quantum())
    }

    fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 33) as f64) / (u32::MAX as f64)
    }

    fn next_index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        (self.0 % len as u64) as usize
    }
}

/// Fault tolerance manager
pub struct FaultToleranceManager {
    node_health: HashMap<String, NodeHealth>,
    checkpoint_interval: u64,
}

#[derive(Debug, Clone)]
pub struct NodeHealth {
    pub node_id: String,
    pub last_heartbeat: u64,
    pub failure_count: u32,
    pub reputation: f32,
    pub is_healthy: bool,
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Restart,
    MigrateTasks,
    PartialReboot,
    FullReboot,
}

impl FaultToleranceManager {
    pub fn new() -> Self {
        FaultToleranceManager {
            node_health: HashMap::new(),
            checkpoint_interval: 300,
        }
    }

    pub fn update_health(&mut self, node_id: &str, is_alive: bool) {
        let now = timestamp_quantum();

        if is_alive {
            if let Some(health) = self.node_health.get_mut(node_id) {
                health.last_heartbeat = now;
                health.is_healthy = true;
            } else {
                self.node_health.insert(
                    node_id.to_string(),
                    NodeHealth {
                        node_id: node_id.to_string(),
                        last_heartbeat: now,
                        failure_count: 0,
                        reputation: 0.5,
                        is_healthy: true,
                    },
                );
            }
        } else if let Some(health) = self.node_health.get_mut(node_id) {
            health.failure_count += 1;
            health.reputation *= 0.9;
            health.is_healthy = false;
        }
    }

    pub fn detect_failures(&self, threshold_secs: u64) -> Vec<String> {
        let now = timestamp_quantum();
        let mut failed_nodes = Vec::new();

        for (node_id, health) in &self.node_health {
            if now - health.last_heartbeat > threshold_secs {
                failed_nodes.push(node_id.clone());
            }
        }

        failed_nodes
    }

    pub fn get_recovery_strategy(&self, node_id: &str) -> RecoveryStrategy {
        if let Some(health) = self.node_health.get(node_id) {
            if health.failure_count == 0 {
                RecoveryStrategy::Restart
            } else if health.failure_count < 3 {
                RecoveryStrategy::MigrateTasks
            } else if health.failure_count < 5 {
                RecoveryStrategy::PartialReboot
            } else {
                RecoveryStrategy::FullReboot
            }
        } else {
            RecoveryStrategy::Restart
        }
    }

    pub fn get_node_reputation(&self, node_id: &str) -> f32 {
        self.node_health
            .get(node_id)
            .map(|h| h.reputation)
            .unwrap_or(0.5)
    }
}

impl Default for FaultToleranceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Self-healing compute grid
pub struct SelfHealingGrid {
    fault_manager: FaultToleranceManager,
    backup_nodes: HashMap<String, ComputeNode>,
    migration_buffer: Vec<Task>,
}

impl SelfHealingGrid {
    pub fn new() -> Self {
        SelfHealingGrid {
            fault_manager: FaultToleranceManager::new(),
            backup_nodes: HashMap::new(),
            migration_buffer: Vec::new(),
        }
    }

    pub fn heal_node(&mut self, node_id: &str) -> bool {
        let strategy = self.fault_manager.get_recovery_strategy(node_id);

        match strategy {
            RecoveryStrategy::Restart => true,
            RecoveryStrategy::MigrateTasks => !self.migration_buffer.is_empty(),
            RecoveryStrategy::PartialReboot => true,
            RecoveryStrategy::FullReboot => self.backup_nodes.contains_key(node_id),
        }
    }

    pub fn checkpoint_state(&self, _node_id: &str) -> Option<Vec<u8>> {
        None
    }
}

impl Default for SelfHealingGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Grid-wide consensus mechanism
pub struct GridConsensus {
    quorum_size: usize,
    active_proposals: HashMap<String, ConsensusProposal>,
    decided_values: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct ConsensusProposal {
    pub proposal_id: String,
    pub value: String,
    pub proposer: String,
    pub votes_for: HashSet<String>,
    pub votes_against: HashSet<String>,
    pub created_at: u64,
}

impl GridConsensus {
    pub fn new() -> Self {
        GridConsensus {
            quorum_size: 3,
            active_proposals: HashMap::new(),
            decided_values: HashMap::new(),
        }
    }

    pub fn propose(&mut self, proposal_id: &str, value: &str, proposer: &str) -> bool {
        if self.decided_values.contains_key(proposal_id) {
            return false;
        }

        self.active_proposals.insert(
            proposal_id.to_string(),
            ConsensusProposal {
                proposal_id: proposal_id.to_string(),
                value: value.to_string(),
                proposer: proposer.to_string(),
                votes_for: HashSet::new(),
                votes_against: HashSet::new(),
                created_at: timestamp_quantum(),
            },
        );

        true
    }

    pub fn vote(&mut self, proposal_id: &str, voter: &str, approve: bool) -> bool {
        if let Some(proposal) = self.active_proposals.get_mut(proposal_id) {
            if approve {
                proposal.votes_for.insert(voter.to_string());
            } else {
                proposal.votes_against.insert(voter.to_string());
            }

            let total_nodes = proposal.votes_for.len() + proposal.votes_against.len();
            if total_nodes >= self.quorum_size {
                let approved = proposal.votes_for.len() > proposal.votes_against.len();
                self.decided_values
                    .insert(proposal_id.to_string(), if approved { 1 } else { 0 });
                self.active_proposals.remove(proposal_id);
                return approved;
            }
        }

        false
    }

    pub fn is_decided(&self, proposal_id: &str) -> bool {
        self.decided_values.contains_key(proposal_id)
    }
}

impl Default for GridConsensus {
    fn default() -> Self {
        Self::new()
    }
}

/// Node reputation system
pub struct ReputationSystem {
    node_scores: HashMap<String, RepScore>,
}

#[derive(Debug, Clone)]
pub struct RepScore {
    pub node_id: String,
    pub reliability: f32,
    pub performance: f32,
    pub cooperation: f32,
    pub total_score: f32,
    pub history: Vec<RepEvent>,
}

#[derive(Debug, Clone)]
pub struct RepEvent {
    pub timestamp: u64,
    pub event_type: RepEventType,
    pub delta: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum RepEventType {
    TaskCompletion,
    TaskFailure,
    HeartbeatMiss,
    MaliciousBehavior,
    ResourceDonation,
}

impl ReputationSystem {
    pub fn new() -> Self {
        ReputationSystem {
            node_scores: HashMap::new(),
        }
    }

    pub fn record_event(&mut self, node_id: &str, event: RepEvent) {
        let score = self
            .node_scores
            .entry(node_id.to_string())
            .or_insert(RepScore {
                node_id: node_id.to_string(),
                reliability: 0.5,
                performance: 0.5,
                cooperation: 0.5,
                total_score: 0.5,
                history: Vec::new(),
            });

        score.history.push(event.clone());

        match event.event_type {
            RepEventType::TaskCompletion => {
                score.reliability = (score.reliability + 0.05).min(1.0);
                score.performance = (score.performance + 0.02).min(1.0);
            }
            RepEventType::TaskFailure => {
                score.reliability = (score.reliability - 0.1).max(0.0);
            }
            RepEventType::MaliciousBehavior => {
                score.reliability = (score.reliability - 0.2).max(0.0);
                score.cooperation = (score.cooperation - 0.2).max(0.0);
            }
            RepEventType::ResourceDonation => {
                score.cooperation = (score.cooperation + 0.05).min(1.0);
            }
            _ => {}
        }

        score.total_score = (score.reliability + score.performance + score.cooperation) / 3.0;
    }

    pub fn get_score(&self, node_id: &str) -> f32 {
        self.node_scores
            .get(node_id)
            .map(|s| s.total_score)
            .unwrap_or(0.5)
    }

    pub fn get_ranked_nodes(&self) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .node_scores
            .iter()
            .map(|(id, s)| (id.clone(), s.total_score))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores
    }
}

impl Default for ReputationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Work stealing algorithm
pub struct WorkStealer {
    idle_workers: Vec<String>,
    work_queue: Vec<Task>,
}

impl WorkStealer {
    pub fn new() -> Self {
        WorkStealer {
            idle_workers: Vec::new(),
            work_queue: Vec::new(),
        }
    }

    pub fn register_idle(&mut self, worker_id: &str) {
        self.idle_workers.push(worker_id.to_string());
    }

    pub fn unregister_idle(&mut self, worker_id: &str) {
        self.idle_workers.retain(|w| w != worker_id);
    }

    pub fn steal_work(&mut self) -> Option<Task> {
        self.work_queue.pop()
    }

    pub fn offer_work(&mut self, task: Task) {
        self.work_queue.push(task);
    }

    pub fn balance(&mut self) {
        if self.idle_workers.len() > 1 && !self.work_queue.is_empty() {
            self.idle_workers.remove(0);
        }
    }
}

impl Default for WorkStealer {
    fn default() -> Self {
        Self::new()
    }
}

fn timestamp_quantum() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
