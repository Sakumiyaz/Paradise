//! Eden Async Executor - Native Async/Await Runtime
//! 
//! Implements an async runtime using only std library features.
//! No external crates required - pure Rust async primitives.
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod runtime;
pub mod task;
pub mod async_io;
pub mod timer;

pub use runtime::EdenRuntime;
pub use task::{EdenTask, TaskId, TaskPriority};
