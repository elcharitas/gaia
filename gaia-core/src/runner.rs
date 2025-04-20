//! Task runner definition and implementation

use std::future::Future;
use std::pin::Pin;

use crate::Result;
use crate::error::GaiaError;
use crate::task::{Task, TaskStatus};

/// Trait for objects that can be executed as part of a pipeline
pub trait Runnable {
    /// Execute the task and return a result
    fn run(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Update the status of the task
    fn set_status(&mut self, status: TaskStatus);

    /// Get the ID of the task
    fn id(&self) -> &str;

    /// Get the name of the task
    fn name(&self) -> &str;

    /// Set a custom execution function for this task

    fn with_execution_fn<F, Fut>(&mut self, execution_fn: F) -> &mut Self
    where
        F: FnMut() -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;
}

/// Implementation of Runnable for the Task struct
impl Runnable for Task {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            // Update status to Running
            self.status = TaskStatus::Running;

            let result = if let Some(execution_fn) = &mut self.execution_fn {
                // Execute the custom execution function if available
                match execution_fn().await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        self.status = TaskStatus::Failed;
                        Err(GaiaError::TaskExecutionFailed(format!(
                            "Task {} failed: {}",
                            self.id, e
                        )))
                    }
                }
            } else {
                // Default implementation for backward compatibility
                // Check if we should simulate a failure (for testing)
                if self.id.contains("fail") {
                    self.status = TaskStatus::Failed;
                    Err(GaiaError::TaskExecutionFailed(format!(
                        "Task {} failed",
                        self.id
                    )))
                } else {
                    Ok(())
                }
            };

            // Update status to Completed if successful
            if result.is_ok() {
                self.status = TaskStatus::Completed;
            }

            result
        })
    }

    fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn with_execution_fn<F, Fut>(&mut self, execution_fn: F) -> &mut Self
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

    #[tokio::test]
    async fn test_task_run_success() {
        let mut task = Task::new("task-1", "Test Task");
        let result = task.run().await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_task_run_failure() {
        let mut task = Task::new("task-fail", "Failing Task");
        let result = task.run().await;
        assert!(result.is_err());
        assert_eq!(task.status, TaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_task_status_methods() {
        let mut task = Task::new("task-status", "Status Test Task");
        assert_eq!(task.status, TaskStatus::Pending);

        task.set_status(TaskStatus::Running);
        assert_eq!(task.status, TaskStatus::Running);

        task.set_status(TaskStatus::Completed);
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_task_execution_fn() {
        let mut task = Task::new("task-execution", "Execution Test Task");
        task.with_execution_fn(async move || Ok(()));
        let result = task.run().await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
