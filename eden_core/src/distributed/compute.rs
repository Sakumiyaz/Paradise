//! # Compute - Grid computation execution
#![allow(unused_imports)]
#![allow(dead_code)]
use std::time::UNIX_EPOCH;

use super::{
    ComputeGrid, ComputeNode, GridStats, NodeCapabilities, NodeInfo, Task, TaskResult, TaskType,
};
use std::collections::{HashMap, VecDeque};

/// mensaje del protocolo de comunicación entre nodos
#[derive(Debug, Clone)]
pub enum ComputeMessage {
    TaskSubmit { task: Task },
    TaskResult { result: TaskResult },
    Heartbeat { node_id: String, load: f32 },
    NodeRegister { node: ComputeNode },
    NodeUnregister { node_id: String },
}

/// Engine de ejecución distribuida
pub struct ComputeEngine {
    grid: ComputeGrid,
    pending_queue: VecDeque<Task>,
    result_cache: HashMap<String, TaskResult>,
}

impl ComputeEngine {
    pub fn new() -> Self {
        Self {
            grid: ComputeGrid::new(),
            pending_queue: VecDeque::new(),
            result_cache: HashMap::new(),
        }
    }

    /// Registra un nodo
    pub fn register_node(&mut self, id: String, name: String, host: String, port: u16) {
        let info = NodeInfo::new(id.clone(), name, host, port);
        let caps = NodeCapabilities::desktop();
        let node = ComputeNode::new(info, caps);
        self.grid.register_node(node);
    }

    /// Submite una tarea
    pub fn submit(&mut self, task_type: TaskType, data: Vec<u8>) -> String {
        let id = format!("task_{}", self.pending_queue.len());
        let task = Task::new(id.clone(), task_type, data);
        self.pending_queue.push_back(task.clone());
        self.grid.submit_task(task);
        id
    }

    /// Obtiene resultado
    pub fn get_result(&self, task_id: &str) -> Option<&TaskResult> {
        self.grid.get_result(task_id)
    }

    /// Procesa mensajes del protocolo
    pub fn process_message(&mut self, msg: ComputeMessage) -> Option<ComputeMessage> {
        match msg {
            ComputeMessage::TaskSubmit { task } => {
                // Add to pending
                self.pending_queue.push_back(task);
                None
            }
            ComputeMessage::TaskResult { result } => {
                // Cache result
                self.result_cache.insert(result.task_id.clone(), result);
                None
            }
            ComputeMessage::Heartbeat { .. } => {
                // Update node load in scheduler
                // Note: This requires &mut access to scheduler which we don't have direct access to here
                // In a real implementation, we'd need a different approach
                None
            }
            ComputeMessage::NodeRegister { node } => {
                self.grid.register_node(node);
                None
            }
            ComputeMessage::NodeUnregister { node_id } => {
                self.grid.unregister_node(&node_id);
                None
            }
        }
    }

    /// Ejecuta una tarea (simula ejecución)
    pub fn execute_task(&self, task: &mut Task) -> Vec<u8> {
        // Simulate computation based on task type
        match task.task_type {
            TaskType::Compute => {
                // Simple CPU computation simulation
                let result: u64 = (0..1000).map(|i| i * i).sum();
                result.to_le_bytes().to_vec()
            }
            TaskType::DataProcessing => {
                // Process data
                let mut result = task.data.clone();
                for byte in &mut result {
                    *byte = byte.rotate_right(2);
                }
                result
            }
            TaskType::Simulation => {
                // Return simulation result marker
                vec![b'S', b'I', b'M', b'U', b'L']
            }
            TaskType::Training => {
                // Neural network training simulation
                vec![b'T', b'R', b'A', b'I', b'N']
            }
            TaskType::Rendering => {
                vec![b'R', b'E', b'N', b'D']
            }
            TaskType::Custom(ref data) => data.clone(),
        }
    }

    /// Stats del engine
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            grid: self.grid.stats(),
            pending: self.pending_queue.len(),
            cached_results: self.result_cache.len(),
        }
    }
}

impl Default for ComputeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del engine
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub grid: GridStats,
    pub pending: usize,
    pub cached_results: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = ComputeEngine::new();
        assert_eq!(engine.stats().grid.total_nodes, 0);
    }

    #[test]
    fn test_task_submission() {
        let mut engine = ComputeEngine::new();
        let task_id = engine.submit(TaskType::Compute, vec![1, 2, 3]);
        assert!(!task_id.is_empty());
    }
}
