//! Task module for Eden's async executor
//! 
//! Provides task management with priorities and scheduling.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};

/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self(0)
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Waiting,
    Completed,
    Failed,
}

/// Internal task representation
struct TaskInner {
    id: TaskId,
    future: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    state: TaskState,
    priority: TaskPriority,
    spawned_at: Instant,
    last_wake: Instant,
    result: Option<()>,
}

impl TaskInner {
    fn new(id: TaskId, future: Pin<Box<dyn Future<Output = ()> + Send>>, priority: TaskPriority) -> Self {
        let now = Instant::now();
        Self {
            id,
            future: Some(future),
            state: TaskState::Ready,
            priority,
            spawned_at: now,
            last_wake: now,
            result: None,
        }
    }
}

/// Shared task handle for external access
pub struct EdenTask {
    inner: Arc<Mutex<TaskInner>>,
    notifier: Arc<Condvar>,
}

impl EdenTask {
    fn new(id: TaskId, future: Pin<Box<dyn Future<Output = ()> + Send>>, priority: TaskPriority) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TaskInner::new(id, future, priority))),
            notifier: Arc::new(Condvar::new()),
        }
    }
    
    pub fn id(&self) -> TaskId {
        self.inner.lock().unwrap().id
    }
    
    pub fn state(&self) -> TaskState {
        self.inner.lock().unwrap().state
    }
    
    pub fn priority(&self) -> TaskPriority {
        self.inner.lock().unwrap().priority
    }
    
    /// Poll the task's future
    fn poll(&self, cx: &mut Context<'_>) -> Poll<()> {
        let mut inner = self.inner.lock().unwrap();
        
        let future = match inner.future.as_mut() {
            Some(f) => f,
            None => return Poll::Ready(inner.result.take().unwrap_or(())),
        };
        
        // Create a waker that notifies this task
        let waker = self.make_waker();
        let mut cx = Context::from_waker(&waker);
        
        inner.state = TaskState::Running;
        let result = future.poll(&mut cx);
        
        match &result {
            Poll::Ready(_) => {
                inner.state = TaskState::Completed;
                inner.result = Some(());
                self.notifier.notify_all();
                Poll::Ready(())
            }
            Poll::Pending => {
                inner.state = TaskState::Waiting;
                Poll::Pending
            }
        }
    }
    
    fn make_waker(&self) -> Waker {
        let data = Arc::into_raw(self.inner.clone()) as *const ();
        
        let vtable = Box::new(RawWakerVTable::new(
            |ptr| {
                // Clone: increase reference count
                unsafe { Arc::from_raw(ptr as *const Mutex<TaskInner>) };
                RawWaker::new(ptr, RawWakerVTable::new(clone, clone, wake, wake_by_ref, drop, drop))
            },
            |ptr| {
                // Wake: notify the condvar
                unsafe {
                    let inner = Arc::from_raw(ptr as *const Mutex<TaskInner>);
                    let _guard = inner.lock().unwrap();
                    // We can't easily notify without the condvar, so this is a simplification
                    drop(inner);
                }
            },
            |ptr| {
                // Wake by ref
                unsafe {
                    let inner = Arc::from_raw(ptr as *const Mutex<TaskInner>);
                    drop(inner);
                }
            },
            |ptr| {
                // Drop
                unsafe { Arc::from_raw(ptr as *const Mutex<TaskInner>) };
            },
        ));
        
        // Simplified waker - use default
        Waker::noop()
    }
}

impl std::fmt::Debug for EdenTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        f.debug_struct("EdenTask")
            .field("id", &inner.id)
            .field("state", &inner.state)
            .field("priority", &inner.priority)
            .finish()
    }
}

/// Task scheduler using priority queue
pub struct TaskScheduler {
    tasks: BinaryHeap<ScheduledTask>,
    next_id: u64,
}

struct ScheduledTask {
    id: TaskId,
    priority: TaskPriority,
    task: Arc<Mutex<TaskInner>>,
    notifier: Arc<Condvar>,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first (lower enum value)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => self.id.as_u64().cmp(&other.id.as_u64()),
            ord => ord.reverse(),
        }
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: BinaryHeap::with_capacity(1024),
            next_id: 1,
        }
    }
    
    /// Spawn a new task
    pub fn spawn<F>(&mut self, future: F, priority: TaskPriority) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = TaskId::new(self.next_id);
        self.next_id += 1;
        
        let inner = Arc::new(Mutex::new(TaskInner::new(
            id,
            Box::pin(future),
            priority,
        )));
        let notifier = Arc::new(Condvar::new());
        
        self.tasks.push(ScheduledTask {
            id,
            priority,
            task: inner,
            notifier,
        });
        
        id
    }
    
    /// Get next ready task
    pub fn pop_ready(&mut self) -> Option<(TaskId, Arc<Mutex<TaskInner>>, Arc<Condvar>)> {
        self.tasks.pop().map(|st| {
            (st.id, st.task, st.notifier)
        })
    }
    
    /// Check if there are pending tasks
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    
    /// Number of pending tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_task_id() {
        let id = TaskId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_priority_ordering() {
        let priorities = vec![
            TaskPriority::Idle,
            TaskPriority::Low,
            TaskPriority::Normal,
            TaskPriority::High,
            TaskPriority::Critical,
        ];
        
        // Critical should be first in max-heap
        let mut sorted = priorities.clone();
        sorted.sort();
        sorted.reverse();
        
        assert_eq!(sorted[0], TaskPriority::Critical);
    }

    #[test]
    fn test_scheduler_spawn() {
        let mut scheduler = TaskScheduler::new();
        
        let id = scheduler.spawn(async { 42 }, TaskPriority::Normal);
        assert_eq!(id.as_u64(), 1);
        
        let id2 = scheduler.spawn(async { 43 }, TaskPriority::High);
        assert_eq!(id2.as_u64(), 2);
        
        assert_eq!(scheduler.len(), 2);
    }

    #[tokio::test] // Note: this test uses tokio for convenience, but the core doesn't require it
    async fn test_task_priority_ordering() {
        let mut scheduler = TaskScheduler::new();
        
        // Spawn in reverse priority order
        scheduler.spawn(async {}, TaskPriority::Low);
        scheduler.spawn(async {}, TaskPriority::Critical);
        scheduler.spawn(async {}, TaskPriority::Normal);
        scheduler.spawn(async {}, TaskPriority::High);
        
        // Critical should be first
        let (id, _, _) = scheduler.pop_ready().unwrap();
        assert_eq!(id.as_u64(), 2); // Second spawned (Critical)
    }
}
