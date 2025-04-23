//! Task runner definition and implementation

use std::future::Future;
use std::pin::Pin;

use crate::error::GaiaError;
use crate::task::{Task, TaskStatus};

/// Trait for objects that can be executed as part of a pipeline
pub(super) trait Runnable {
    /// Execute the task and return a result
    fn run(&mut self) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>>;
}

/// Implementation of Runnable for the Task struct
impl Runnable for Task {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + '_>> {
        Box::pin(async move {
            // Update status to Running
            self.status = TaskStatus::Running;

            let result = if let Some(execution_fn) = &mut self.execution_fn {
                let result: Result<crate::Result<()>, _> = if let Some(timeout) = self.timeout {
                    #[cfg(feature = "tokio")]
                    let exec = tokio::time::timeout(timeout, execution_fn()).await;
                    #[cfg(not(feature = "tokio"))]
                    let exec = Ok::<crate::Result<()>, ()>(execution_fn().await);
                    exec
                } else {
                    Ok(execution_fn().await)
                };
                match result {
                    Ok(res) => match res {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            self.status = TaskStatus::Failed;
                            Err(GaiaError::TaskExecutionFailed(format!(
                                "Task {} failed: {}",
                                self.id, e
                            )))
                        }
                    },
                    Err(_) => {
                        self.status = TaskStatus::TimedOut;
                        Err(GaiaError::TaskTimeout)
                    }
                }
            } else {
                Ok(())
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
    #[cfg(feature = "tokio")]
    use tokio;

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_task_run_success() {
        let mut task = Task::new("task-1", "Test Task");
        let result = task.run().await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_task_run_failure() {
        let mut task = Task::new("task-fail", "Failing Task").with_execution_fn(async move || {
            Err(GaiaError::TaskExecutionFailed(
                "Execution failed".to_string(),
            ))
        });
        let result = task.run().await;
        assert!(result.is_err());
        assert_eq!(task.status, TaskStatus::Failed);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_task_execution_fn() {
        let mut task = Task::new("task-execution", "Execution Test Task")
            .with_execution_fn(async move || Ok(()));
        let result = task.run().await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
