//! # Task - Work unit in the distributed grid

#![allow(dead_code)]

use std::time::Duration;

/// Tipo de tarea
#[derive(Debug, Clone)]
pub enum TaskType {
    Compute,
    DataProcessing,
    Simulation,
    Training,
    Rendering,
    Custom(Vec<u8>),
}

impl TaskType {
    pub fn flops_estimate(&self) -> u64 {
        match self {
            TaskType::Compute => 1_000_000,        // 1M FLOPs
            TaskType::DataProcessing => 5_000_000, // 5M FLOPs
            TaskType::Simulation => 20_000_000,    // 20M FLOPs
            TaskType::Training => 100_000_000,     // 100M FLOPs
            TaskType::Rendering => 50_000_000,     // 50M FLOPs
            TaskType::Custom(data) => {
                // Estimate based on data size
                (data.len() as u64) * 1000
            }
        }
    }
}

/// Estado de una tarea
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Una unidad de trabajo
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub priority: u32,
    pub data: Vec<u8>,
    pub result_data: Option<Vec<u8>>,
    pub created_at: u64,
    pub assigned_to: Option<String>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub status: TaskStatus,
}

impl Task {
    pub fn new(id: String, task_type: TaskType, data: Vec<u8>) -> Self {
        Self {
            id,
            task_type,
            priority: 5, // Default priority
            data,
            result_data: None,
            created_at: current_time_ms(),
            assigned_to: None,
            started_at: None,
            completed_at: None,
            status: TaskStatus::Pending,
        }
    }

    /// Asigna la tarea a un nodo
    pub fn assign(&mut self, node_id: &str) {
        self.assigned_to = Some(node_id.to_string());
        self.started_at = Some(current_time_ms());
        self.status = TaskStatus::Running;
    }

    /// Completa la tarea con resultado
    pub fn complete(&mut self, result: Vec<u8>) {
        self.result_data = Some(result);
        self.completed_at = Some(current_time_ms());
        self.status = TaskStatus::Completed;
    }

    /// Falla la tarea
    pub fn fail(&mut self) {
        self.completed_at = Some(current_time_ms());
        self.status = TaskStatus::Failed;
    }

    /// Tiempo de ejecución
    pub fn execution_time(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(Duration::from_millis(end - start)),
            _ => None,
        }
    }

    /// FLOPs estimados para esta tarea
    pub fn flops_required(&self) -> u64 {
        self.task_type.flops_estimate()
    }
}

/// Resultado de una tarea
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub result: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub node_id: String,
}

impl TaskResult {
    pub fn success(task_id: String, result: Vec<u8>, node_id: String, time_ms: u64) -> Self {
        Self {
            task_id,
            success: true,
            result: Some(result),
            error_message: None,
            execution_time_ms: time_ms,
            node_id,
        }
    }

    pub fn failure(task_id: String, error: String, node_id: String) -> Self {
        Self {
            task_id,
            success: false,
            result: None,
            error_message: Some(error),
            execution_time_ms: 0,
            node_id,
        }
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
    fn test_task_creation() {
        let task = Task::new("task1".to_string(), TaskType::Compute, vec![1, 2, 3]);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_assignment() {
        let mut task = Task::new("task1".to_string(), TaskType::Compute, vec![]);
        task.assign("node1");
        assert_eq!(task.status, TaskStatus::Running);
        assert!(task.started_at.is_some());
    }
}
