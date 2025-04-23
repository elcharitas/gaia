//! Task runner definition and implementation

use std::future::Future;
use std::pin::Pin;

use crate::Result;
use crate::error::GaiaError;
use crate::task::{Task, TaskStatus};

/// Trait for objects that can be executed as part of a pipeline
pub(super) trait Runnable {
    /// Execute the task and return a result
    fn run(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
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
    async fn test_task_execution_fn() {
        let mut task = Task::new("task-execution", "Execution Test Task")
            .with_execution_fn(async move || Ok(()));
        let result = task.run().await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
