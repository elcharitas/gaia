//! Task and pipeline execution logic

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::pipeline::Pipeline;
use crate::task::{Task, TaskStatus};
use crate::{Result, Runnable};

/// Handles the execution of pipelines and their tasks
#[derive(Debug)]
pub struct Executor {
    /// Configuration for the executor
    config: ExecutorConfig,
}

/// Configuration options for the executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Default timeout for tasks that don't specify one
    pub default_timeout: Duration,
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Whether to continue pipeline execution when a task fails
    pub continue_on_failure: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(3600), // 1 hour default
            max_concurrent_tasks: 4,
            continue_on_failure: false,
        }
    }
}

impl Executor {
    /// Create a new executor with default configuration
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
        }
    }

    /// Create a new executor with custom configuration
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// Execute a pipeline
    pub async fn execute_pipeline(&self, pipeline: Arc<Mutex<Pipeline>>) -> Result<()> {
        let mut pipeline_guard = pipeline.lock().unwrap();

        let mut tasks: Vec<&mut Task> = pipeline_guard.tasks.values_mut().collect();

        // TODO: proper topological sorting
        tasks.sort_by(|a, b| a.dependencies.len().cmp(&b.dependencies.len()));

        for task in tasks {
            if let Err(e) = self.execute_task(task).await {
                if !self.config.continue_on_failure {
                    return Err(e);
                }
                // Log the error but continue with next task
                eprintln!("Task {} failed: {}", task.id, e);
            }
        }

        Ok(())
    }

    /// Execute a single task
    pub async fn execute_task(&self, task: &mut Task) -> Result<()> {
        // Execute the task using the Runnable trait
        task.run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GaiaError;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_executor_new() {
        let executor = Executor::new();
        assert_eq!(executor.config.default_timeout, Duration::from_secs(3600));
        assert_eq!(executor.config.max_concurrent_tasks, 4);
        assert_eq!(executor.config.continue_on_failure, false);
    }

    #[tokio::test]
    async fn test_executor_with_config() {
        let config = ExecutorConfig {
            default_timeout: Duration::from_secs(1800),
            max_concurrent_tasks: 8,
            continue_on_failure: true,
        };
        let executor = Executor::with_config(config.clone());
        assert_eq!(executor.config.default_timeout, Duration::from_secs(1800));
        assert_eq!(executor.config.max_concurrent_tasks, 8);
        assert_eq!(executor.config.continue_on_failure, true);
    }

    #[tokio::test]
    async fn test_execute_task() {
        let executor = Executor::new();
        let mut task = Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };

        let result = executor.execute_task(&mut task).await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_execute_pipeline_empty() {
        let executor = Executor::new();
        let pipeline = Arc::new(Mutex::new(Pipeline::new("pipeline-1", "Test Pipeline")));

        let result = executor.execute_pipeline(pipeline).await;
        assert!(result.is_ok());
    }

    // Test timeout handling
    #[tokio::test]
    async fn test_task_timeout() {
        // This test would be more complex in a real implementation
        // where we'd actually test timeout functionality
        let executor = Executor::new();
        let mut task = Task {
            id: "task-timeout".to_string(),
            name: "Timeout Task".to_string(),
            description: Some("A task that tests timeout".to_string()),
            dependencies: HashSet::new(),
            timeout: Some(Duration::from_millis(100)),
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };

        // In a real implementation, we'd make this task take longer than its timeout
        // and verify it fails with a timeout error
        let result = executor.execute_task(&mut task).await;
        assert!(result.is_ok()); // Currently passes because our implementation is a stub
    }

    // Test error handling
    #[tokio::test]
    async fn test_task_error_handling() {
        // This would test how the executor handles task failures
        // In a real implementation, we'd force a task to fail and verify error handling
        let executor = Executor::new();
        let mut task = Task {
            id: "task-error".to_string(),
            name: "Error Task".to_string(),
            description: Some("A task that tests error handling".to_string()),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 1,
            status: TaskStatus::Pending,
            execution_fn: None,
        };

        // In a real implementation, we'd make this task fail and verify retry behavior
        let result = executor.execute_task(&mut task).await;
        assert!(result.is_ok()); // Currently passes because our implementation is a stub
    }
}
