//! # Node - Compute node representation

#![allow(dead_code)]

use std::time::Duration;

/// Información de un nodo computacional
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub registered_at: u64,
}

impl NodeInfo {
    pub fn new(id: String, name: String, host: String, port: u16) -> Self {
        Self {
            id,
            name,
            host,
            port,
            registered_at: current_time_ms(),
        }
    }
}

/// Estado de un nodo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeStatus {
    Active,
    Idle,
    Busy,
    Offline,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Offline
    }
}

/// Capacidades de un nodo
#[derive(Debug, Clone)]
pub struct NodeCapabilities {
    pub compute_power: u64,    // FLOPs estimate
    pub memory_bytes: u64,     // RAM total
    pub available_memory: u64, // RAM disponible
    pub cores: u32,            // CPU cores
    pub has_gpu: bool,
    pub gpu_power: Option<u64>, // GPU FLOPs si tiene
}

impl NodeCapabilities {
    pub fn new() -> Self {
        Self {
            compute_power: 1000,           // Default 1K FLOPs (simulated)
            memory_bytes: 1_073_741_824,   // 1GB default
            available_memory: 536_870_912, // 512MB disponible
            cores: 4,
            has_gpu: false,
            gpu_power: None,
        }
    }

    pub fn desktop() -> Self {
        Self {
            compute_power: 10_000,
            memory_bytes: 8_589_934_592,
            available_memory: 4_294_967_296,
            cores: 8,
            has_gpu: false,
            gpu_power: None,
        }
    }

    pub fn server() -> Self {
        Self {
            compute_power: 100_000,
            memory_bytes: 68_719_476_736,
            available_memory: 34_359_738_368,
            cores: 32,
            has_gpu: true,
            gpu_power: Some(1_000_000),
        }
    }
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

/// Nodo computacional completo
#[derive(Debug, Clone)]
pub struct ComputeNode {
    pub info: NodeInfo,
    pub status: NodeStatus,
    pub capabilities: NodeCapabilities,
    pub current_load: f32, // 0.0 - 1.0
    pub active_tasks: Vec<String>,
    pub last_heartbeat: u64,
}

impl ComputeNode {
    pub fn new(info: NodeInfo, capabilities: NodeCapabilities) -> Self {
        Self {
            info,
            status: NodeStatus::Idle,
            capabilities,
            current_load: 0.0,
            active_tasks: Vec::new(),
            last_heartbeat: current_time_ms(),
        }
    }

    /// Verifica si el nodo está vivo
    pub fn is_alive(&self) -> bool {
        let elapsed = current_time_ms() - self.last_heartbeat;
        elapsed < 30_000 // 30 seconds timeout
    }

    /// Actualiza heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = current_time_ms();
    }

    /// Añade tarea activa
    pub fn assign_task(&mut self, task_id: &str) {
        self.active_tasks.push(task_id.to_string());
        self.status = NodeStatus::Busy;
        self.update_load();
    }

    /// Completa tarea
    pub fn complete_task(&mut self, task_id: &str) {
        self.active_tasks.retain(|t| t != task_id);
        self.update_load();

        if self.active_tasks.is_empty() {
            self.status = NodeStatus::Idle;
        }
    }

    fn update_load(&mut self) {
        let max_tasks = self.capabilities.cores as f32;
        self.current_load = (self.active_tasks.len() as f32 / max_tasks).min(1.0);
    }

    /// Estima tiempo de ejecución de una tarea
    pub fn estimate_time(&self, flops_required: u64) -> Duration {
        let available_power = self.capabilities.compute_power as f32 * (1.0 - self.current_load);
        let seconds = flops_required as f32 / available_power;
        Duration::from_secs_f32(seconds)
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let info = NodeInfo::new(
            "node1".to_string(),
            "Test Node".to_string(),
            "localhost".to_string(),
            8080,
        );
        let node = ComputeNode::new(info, NodeCapabilities::new());
        assert_eq!(node.status, NodeStatus::Idle);
    }
}
