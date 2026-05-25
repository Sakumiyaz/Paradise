//! # Scheduler - Task scheduling and load balancing

#![allow(dead_code)]

use super::{NodeCapabilities, NodeInfo};
use std::collections::HashMap;
/// Wrapper for f32 that implements Ord for use in collections
#[derive(Debug, Clone)]
struct OrdScore(f32);

impl PartialEq for OrdScore {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for OrdScore {}

impl PartialOrd for OrdScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrdScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse the order so higher scores come first
        other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Información de scheduling para un nodo
#[derive(Debug, Clone)]
pub struct NodeScheduleInfo {
    node_id: String,
    available_power: u64, // compute power adjusted by load
    score: f32,           // scheduling score
}

/// Scheduler de tareas
pub struct Scheduler {
    nodes: HashMap<String, NodeScheduleInfo>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Añade un nodo al scheduler
    pub fn add_node(&mut self, info: &NodeInfo) {
        self.nodes.insert(
            info.id.clone(),
            NodeScheduleInfo {
                node_id: info.id.clone(),
                available_power: 1000, // Default
                score: 1.0,
            },
        );
    }

    /// Remueve un nodo
    pub fn remove_node(&mut self, node_id: &str) -> Option<NodeScheduleInfo> {
        self.nodes.remove(node_id)
    }

    /// Selecciona mejor nodo para una tarea
    pub fn select_node(&self, required_power: u64) -> Option<String> {
        self.nodes
            .iter()
            .filter(|(_, info)| info.available_power >= required_power)
            .max_by(|(_, a), (_, b)| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| id.clone())
    }

    /// Actualiza información de un nodo
    pub fn update_node(&mut self, node_id: &str, load: f32, capabilities: &NodeCapabilities) {
        if let Some(info) = self.nodes.get_mut(node_id) {
            info.available_power = (capabilities.compute_power as f32 * (1.0 - load)) as u64;
            info.score = info.available_power as f32 / (load + 0.1);
        }
    }

    /// Balancea carga entre nodos
    pub fn balance_load(&mut self, nodes: &HashMap<String, super::ComputeNode>) {
        let total_load: f32 = nodes.values().map(|n| n.current_load).sum();

        let avg_load = if !nodes.is_empty() {
            total_load / nodes.len() as f32
        } else {
            0.0
        };

        // Redistribute based on capacity
        for (node_id, node) in nodes {
            if let Some(info) = self.nodes.get_mut(node_id) {
                let capacity_factor = node.capabilities.cores as f32 / 32.0;
                let target_load = avg_load * capacity_factor;
                let load_delta = node.current_load - target_load;

                // Update available power based on load delta
                let power_adjustment = (load_delta * node.capabilities.compute_power as f32) as i64;
                info.available_power =
                    (info.available_power as i64 + power_adjustment).max(0) as u64;
            }
        }
    }

    /// Obtiene estadísticas del scheduler
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            node_count: self.nodes.len(),
            avg_available_power: if !self.nodes.is_empty() {
                self.nodes.values().map(|n| n.available_power).sum::<u64>()
                    / self.nodes.len() as u64
            } else {
                0
            },
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub node_count: usize,
    pub avg_available_power: u64,
}

/// Load Balancer implementa estrategias de balanceo
pub struct LoadBalancer {
    strategy: BalanceStrategy,
}

#[derive(Debug, Clone)]
pub enum BalanceStrategy {
    RoundRobin,
    LeastLoaded,
    PowerBased,
    Random,
}

impl LoadBalancer {
    pub fn new(strategy: BalanceStrategy) -> Self {
        Self { strategy }
    }

    /// Selecciona nodo según estrategia
    pub fn select<'a, 'b: 'a>(
        &self,
        node_ids: &[&'a str],
        nodes: &'b HashMap<String, super::ComputeNode>,
    ) -> Option<&'a str> {
        if node_ids.is_empty() {
            return None;
        }

        match self.strategy {
            BalanceStrategy::RoundRobin => {
                // Simple: return first available
                node_ids.first().copied()
            }
            BalanceStrategy::LeastLoaded => node_ids
                .iter()
                .filter_map(|id| nodes.get(*id))
                .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
                .map(|n| n.info.id.as_str()),
            BalanceStrategy::PowerBased => node_ids
                .iter()
                .filter_map(|id| nodes.get(*id))
                .max_by(|a, b| {
                    let power_a = a.capabilities.compute_power as f32 * (1.0 - a.current_load);
                    let power_b = b.capabilities.compute_power as f32 * (1.0 - b.current_load);
                    power_a.partial_cmp(&power_b).unwrap()
                })
                .map(|n| n.info.id.as_str()),
            BalanceStrategy::Random => {
                use std::time::SystemTime;
                let seed = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as usize;
                Some(node_ids[seed % node_ids.len()])
            }
        }
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(BalanceStrategy::LeastLoaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler() {
        let mut scheduler = Scheduler::new();
        scheduler.add_node(&NodeInfo::new(
            "node1".to_string(),
            "Test".to_string(),
            "localhost".to_string(),
            8080,
        ));

        let selected = scheduler.select_node(500);
        assert_eq!(selected, Some("node1".to_string()));
    }
}
