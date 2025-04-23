//! Task definition and execution

use std::future::Future;
use std::time::Duration;
use std::{collections::HashSet, pin::Pin};

use crate::Result;
use serde::{Deserialize, Serialize};

/// Status of a task execution
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    #[default]
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task Timeout
    TimedOut,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task was skipped
    Skipped,
}

/// Type alias for a task execution function
pub type TaskExecutionFn =
    Box<dyn FnMut() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + 'static>;

/// Represents a single task in a pipeline
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

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            dependencies: self.dependencies.clone(),
            timeout: self.timeout,
            retry_count: self.retry_count,
            status: self.status.clone(),
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

    pub fn with_execution_fn<F, Fut>(mut self, execution_fn: F) -> Self
    where
        F: FnMut() -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.execution_fn = Some(Box::new(move || {
            let mut execution_fn = execution_fn.clone();
            Box::pin(async move { execution_fn().await })
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
        task.status = TaskStatus::Completed;
        assert_eq!(task.status, TaskStatus::Completed);

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
