//! Task definition and execution
//!
//! This module provides the core `Task` struct and related types for defining and executing
//! individual units of work within a pipeline. Tasks are the fundamental building blocks
//! of Gaia pipelines and can represent any operation from simple data transformations to
//! complex distributed computations.
//!
//! # Task Structure
//!
//! Each task has:
//! - A unique identifier
//! - A human-readable name
//! - Optional description
//! - Dependencies on other tasks
//! - Execution logic
//! - Timeout and retry configurations
//!
//! # Examples
//!
//! ```
//! use gaia_core::{Task, Result};
//! use std::time::Duration;
//!
//! // Create a simple task
//! let task = Task::new("process-data", "Process Data")
//!     .with_description("Processes raw data into a structured format")
//!     .add_dependency("fetch-data")
//!     .with_timeout(Duration::from_secs(60))
//!     .with_retry_count(3);
//!
//! // Create a task with custom execution logic
//! let mut task_with_fn = Task::new("analyze-data", "Analyze Data")
//! .with_execution_fn(|_ctx| {
//!     Box::pin(async {
//!         // Perform analysis...
//!         Ok("Analysis complete")
//!     })
//! });
//! ```
//!
//! Tasks are typically added to a `Pipeline` and executed by an `Executor`.

use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use std::{collections::HashSet, pin::Pin};

use crate::Result;
use crate::executor::ExecutorContext;
use serde::{Deserialize, Serialize};

/// Trait for task execution results
///
/// This trait is implemented by all types that can be returned as the result of a task.
/// It provides a common interface for checking if a result is empty, which is useful
/// for conditional execution based on task results.
///
/// # Examples
///
/// ```
/// use gaia_core::task::TaskResult;
///
/// // String implements TaskResult
/// let result: Box<dyn TaskResult> = Box::new(String::from("Task output"));
/// assert!(!result.is_empty());
///
/// // Empty string is considered empty
/// let empty_result: Box<dyn TaskResult> = Box::new(String::new());
/// assert!(empty_result.is_empty());
/// ```
pub trait TaskResult: Debug + Any + Send + 'static {
    fn is_empty(&self) -> bool {
        false
    }
}

impl PartialEq for dyn TaskResult {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl TaskResult for () {
    fn is_empty(&self) -> bool {
        true
    }
}

impl TaskResult for &'static str {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TaskResult for String {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
impl TaskResult for Vec<u8> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl TaskResult for bool {
    fn is_empty(&self) -> bool {
        false
    }
}

impl TaskResult for i32 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for i64 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for i128 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for u32 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for u64 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for u128 {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for f32 {
    fn is_empty(&self) -> bool {
        self == &0.0
    }
}

impl TaskResult for f64 {
    fn is_empty(&self) -> bool {
        self == &0.0
    }
}

impl TaskResult for isize {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

impl TaskResult for usize {
    fn is_empty(&self) -> bool {
        self == &0
    }
}

/// Trait for casting task results to concrete types
///
/// This trait provides methods for safely casting a `TaskResult` to a concrete type.
/// It's useful when you need to extract specific data from a task result.
///
/// # Examples
///
/// ```
/// use gaia_core::task::{TaskResult, CastTask};
///
/// let result: Box<dyn TaskResult> = Box::new(42i32);
///
/// // Cast to the correct type
/// if let Some(number) = result.cast::<i32>() {
///     assert_eq!(*number, 42);
/// }
///
/// // Casting to the wrong type returns None
/// assert!(result.cast::<String>().is_none());
/// ```
pub trait CastTask {
    fn as_any(&self) -> &dyn Any;
    fn cast<T: 'static>(&self) -> Option<&T>;
}

impl CastTask for dyn TaskResult {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn cast<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

/// Status of a task execution
///
/// This enum represents the possible states of a task during its lifecycle.
/// Tasks start in the `Pending` state and transition through various states
/// as they are executed.
///
/// # States
///
/// - `Pending`: Task is waiting to be executed
/// - `Running`: Task is currently being executed
/// - `Completed`: Task has completed successfully with a result
/// - `TimedOut`: Task execution exceeded its timeout duration
/// - `Failed`: Task execution failed with an error
/// - `Cancelled`: Task was cancelled before completion
/// - `Skipped`: Task was skipped (e.g., due to conditional execution)
///
/// The `Completed` variant contains the result of the task execution.
#[derive(Debug, Default, PartialEq)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    #[default]
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed(Box<dyn TaskResult>),
    /// Task Timeout
    TimedOut,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task was skipped
    Skipped,
}

impl Clone for TaskStatus {
    fn clone(&self) -> Self {
        match self {
            TaskStatus::Pending => TaskStatus::Pending,
            TaskStatus::Running => TaskStatus::Running,
            TaskStatus::TimedOut => TaskStatus::TimedOut,
            TaskStatus::Failed => TaskStatus::Failed,
            TaskStatus::Cancelled => TaskStatus::Cancelled,
            TaskStatus::Skipped => TaskStatus::Skipped,
            TaskStatus::Completed(result) => TaskStatus::Completed({
                let result_ref: &dyn TaskResult = result.as_ref();
                let result_ptr: *const dyn TaskResult = result_ref as *const dyn TaskResult;

                // SAFETY: state_ptr is never null, it is safe to convert it to a NonNull pointer, this way we can safely convert it back to a Box
                // If it is ever found as null, this is a bug. It probably means the memory has been poisoned
                let nn_ptr = std::ptr::NonNull::new(result_ptr as *mut dyn TaskResult)
                .expect("TaskResult has been dropped, but this should never happen, ensure it is being cloned correctly."); // This should never happen, if it does, it's a bug
                let raw_ptr = nn_ptr.as_ptr();
                unsafe { Box::from_raw(raw_ptr) }
            }),
        }
    }
}

/// Type alias for a task execution function
///
/// This represents the signature of a function that can be used to execute a task.
/// The function takes an `ExecutorContext` and returns a `Future` that resolves to
/// a `Result` containing a boxed `TaskResult`.
///
/// # Examples
///
/// ```
/// use gaia_core::task::{TaskExecutionFn, TaskResult};
/// use gaia_core::executor::ExecutorContext;
/// use gaia_core::Result;
///
/// let execution_fn: TaskExecutionFn = Box::new(|_ctx| {
///     Box::pin(async {
///         // Perform task logic...
///         Ok(Box::new("Task completed") as Box<dyn TaskResult>)
///     })
/// });
/// ```
pub type TaskExecutionFn = Box<
    dyn FnMut(ExecutorContext) -> Pin<Box<dyn Future<Output = Result<Box<dyn TaskResult>>> + Send>>
        + Send
        + 'static,
>;

/// Represents a single task in a pipeline
///
/// A task is the fundamental unit of work in Gaia. Each task has a unique identifier,
/// a human-readable name, optional description, dependencies on other tasks, and
/// execution logic.
///
/// Tasks can be configured with timeouts and retry counts to handle failures gracefully.
/// They can also be serialized and deserialized for persistence and distribution.
///
/// # Examples
///
/// ```
/// use gaia_core::Task;
/// use std::time::Duration;
///
/// // Create a basic task
/// let task = Task::new("process-data", "Process Data")
///     .with_description("Processes raw data into a structured format")
///     .add_dependency("fetch-data")
///     .with_timeout(Duration::from_secs(60))
///     .with_retry_count(3);
/// ```
#[derive(Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,

    /// Human-readable name of the task
    pub name: String,

    /// Description of what the task does
    pub description: Option<String>,

    /// IDs of tasks that must complete before this task can start
    pub dependencies: HashSet<String>,

    /// Maximum time the task is allowed to run
    pub timeout: Option<Duration>,

    /// Number of retry attempts if the task fails
    pub retry_count: u32,

    /// Current status of the task
    #[serde(skip)]
    pub status: TaskStatus,

    /// Custom execution function (not serialized)
    #[serde(skip)]
    pub execution_fn: Option<TaskExecutionFn>,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            dependencies: self.dependencies.clone(),
            timeout: self.timeout,
            retry_count: self.retry_count,
            status: TaskStatus::Pending,
            execution_fn: None,
        }
    }
}

impl Task {
    /// Create a new task with the given ID and name
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        }
    }

    /// Add a dependency to this task
    pub fn add_dependency(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.insert(dependency_id.into());
        self
    }

    pub fn with_dependencies(mut self, dependencies: HashSet<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Set the description for this task
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the timeout for this task
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the retry count for this task
    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn with_execution_fn<F, Fut, R: TaskResult>(mut self, execution_fn: F) -> Self
    where
        F: FnMut(ExecutorContext) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<R>> + Send + 'static,
    {
        self.execution_fn = Some(Box::new(move |e| {
            let mut execution_fn = execution_fn.clone();
            Box::pin(async move {
                let result = execution_fn(e).await;
                match result {
                    Ok(r) => Ok(Box::new(r) as Box<dyn TaskResult>),
                    Err(e) => Err(e),
                }
            })
        }));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let task = Task::new("task-1", "Test Task");
        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, None);
        assert!(task.dependencies.is_empty());
        assert_eq!(task.timeout, None);
        assert_eq!(task.retry_count, 0);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_with_description() {
        let task = Task::new("task-1", "Test Task").with_description("A test task");
        assert_eq!(task.description, Some("A test task".to_string()));
    }

    #[test]
    fn test_task_with_timeout() {
        let timeout = Duration::from_secs(60);
        let task = Task::new("task-1", "Test Task").with_timeout(timeout);
        assert_eq!(task.timeout, Some(timeout));
    }

    #[test]
    fn test_task_with_retry_count() {
        let task = Task::new("task-1", "Test Task").with_retry_count(3);
        assert_eq!(task.retry_count, 3);
    }

    #[test]
    fn test_task_add_dependency() {
        let task = Task::new("task-1", "Test Task");
        let task = task.add_dependency("dependency-1");
        assert!(task.dependencies.contains("dependency-1"));
        assert_eq!(task.dependencies.len(), 1);

        // Add another dependency
        let task = task.add_dependency("dependency-2");
        assert!(task.dependencies.contains("dependency-2"));
        assert_eq!(task.dependencies.len(), 2);

        // Add duplicate dependency
        let task = task.add_dependency("dependency-1");
        assert_eq!(task.dependencies.len(), 2); // Should still be 2
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new("task-1", "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);

        // Transition to Running
        task.status = TaskStatus::Running;
        assert_eq!(task.status, TaskStatus::Running);

        // Transition to Completed
        task.status = TaskStatus::Completed(Box::new(()));
        assert_eq!(matches!(task.status, TaskStatus::Completed(_)), true);

        // Transition to Failed
        task.status = TaskStatus::Failed;
        assert_eq!(task.status, TaskStatus::Failed);

        // Transition to Cancelled
        task.status = TaskStatus::Cancelled;
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_task_builder_pattern() {
        let task = Task::new("task-1", "Test Task")
            .with_description("A test task")
            .with_timeout(Duration::from_secs(60))
            .with_retry_count(3);

        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, Some("A test task".to_string()));
        assert_eq!(task.timeout, Some(Duration::from_secs(60)));
        assert_eq!(task.retry_count, 3);
        assert_eq!(task.status, TaskStatus::Pending);
    }
}
