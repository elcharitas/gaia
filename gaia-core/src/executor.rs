//! Task and pipeline execution logic

use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::GaiaError;
use crate::monitoring::Monitor;
use crate::pipeline::Pipeline;
use crate::task::{Task, TaskStatus};

/// Handles the execution of pipelines and their tasks
#[derive(Debug)]
pub struct Executor {
    /// Configuration for the executor
    config: ExecutorConfig,
}

pub struct ExecutorContext {
    task_statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

impl ExecutorContext {
    pub fn task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.task_statuses.lock().unwrap().get(task_id).cloned()
    }
}

/// Configuration options for the executor
#[derive(Debug)]
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
            continue_on_failure: true,
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
    pub async fn execute_pipeline(&self, mut pipeline: Pipeline) -> crate::Result<Monitor> {
        let mut monitor = Monitor::new();

        pipeline.validate()?; // Validate the pipeline before execution

        pipeline.state.lock().unwrap().start();

        let mut tasks: Vec<&mut Task> = pipeline.tasks.values_mut().collect();

        let task_states = &pipeline.state.lock().unwrap().task_states;

        // TODO: proper topological sorting
        tasks.sort_by(|a, b| a.dependencies.len().cmp(&b.dependencies.len()));

        let max_concurrent = self.config.max_concurrent_tasks;
        let continue_on_failure = self.config.continue_on_failure;
        let task_map: HashMap<String, usize> = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        let task_statuses = Arc::new(Mutex::new(HashMap::<String, TaskStatus>::new()));
        let mut pending: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let mut running = FuturesUnordered::new();
        let tasks_arc = Arc::new(Mutex::new(tasks));

        // Helper to check if all dependencies of a task are completed
        let is_ready = |task: &Task, statuses: &HashMap<String, TaskStatus>| {
            task.dependencies
                .iter()
                .all(|dep| matches!(statuses.get(dep), Some(TaskStatus::Completed)))
        };

        while !pending.is_empty() || running.len() > 0 {
            // Find ready tasks
            let mut ready = vec![];
            {
                let statuses = task_statuses.lock().unwrap();
                for id in pending.iter() {
                    let idx = *task_map.get(id).unwrap();
                    let task = &tasks_arc.lock().unwrap()[idx];
                    println!("Task {} is ready: {}", task.id, is_ready(task, &statuses));
                    if is_ready(task, &statuses) {
                        ready.push(id.clone());
                    }
                }
            }

            // Spawn up to max_concurrent tasks
            for id in ready.into_iter().take(max_concurrent - running.len()) {
                let idx = *task_map.get(&id).unwrap();
                let mut task = tasks_arc.lock().unwrap()[idx].clone();
                let statuses = Arc::clone(&task_statuses);
                let id_clone = id.clone();
                let task_name = task.name.clone();
                task.status = TaskStatus::Running;
                let exec = async move {
                    let result = self.execute_task(&mut task, statuses.clone()).await;
                    let mut statuses = statuses.lock().unwrap();
                    statuses.insert(
                        id_clone.clone(),
                        if result.is_ok() {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed
                        },
                    );
                    (id_clone, result)
                };
                if let Some(task_state) = task_states.get(&id) {
                    monitor.collect_task_metrics(&id, &task_name, &task_state);
                }
                running.push(exec);
                pending.remove(&id);
            }
            if let Some((task_id, result)) = running.next().await {
                if let Err(e) = result {
                    if !continue_on_failure {
                        pipeline.state.lock().unwrap().complete(false);
                        pipeline
                            .state
                            .lock()
                            .unwrap()
                            .update_task(task_id.clone(), TaskStatus::Failed);
                        monitor.collect_metrics(&pipeline)?;
                        return Err(e);
                    }
                    eprintln!("Task {} failed: {}", task_id, e);
                } else {
                    pipeline
                        .state
                        .lock()
                        .unwrap()
                        .update_task(task_id.clone(), TaskStatus::Completed);
                }
            }
        }

        pipeline.state.lock().unwrap().complete(true);

        monitor.collect_metrics(&pipeline)?;

        Ok(monitor)
    }

    /// Execute a single task
    pub async fn execute_task(
        &self,
        task: &mut Task,
        task_statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
    ) -> crate::Result<()> {
        let result = if let Some(execution_fn) = &mut task.execution_fn {
            let result: Result<crate::Result<()>, _> = if let Some(timeout) = task.timeout {
                #[cfg(feature = "tokio")]
                let exec =
                    tokio::time::timeout(timeout, execution_fn(ExecutorContext { task_statuses }))
                        .await;
                #[cfg(not(feature = "tokio"))]
                let exec = Ok::<crate::Result<()>, ()>(
                    execution_fn(ExecutorContext { task_statuses }).await,
                );
                exec
            } else {
                Ok(execution_fn(ExecutorContext { task_statuses }).await)
            };
            match result {
                Ok(res) => match res {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        task.status = TaskStatus::Failed;
                        Err(GaiaError::TaskExecutionFailed(format!(
                            "Task {} failed: {}",
                            task.id, e
                        )))
                    }
                },
                Err(_) => {
                    task.status = TaskStatus::TimedOut;
                    Err(GaiaError::TaskTimeout)
                }
            }
        } else {
            Ok(())
        };

        // Update status to Completed if successful
        if result.is_ok() {
            task.status = TaskStatus::Completed;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskStatus;
    use std::collections::HashSet;

    #[cfg(feature = "tokio")]
    use tokio;

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_executor_new() {
        let executor = Executor::new();
        assert_eq!(executor.config.default_timeout, Duration::from_secs(3600));
        assert_eq!(executor.config.max_concurrent_tasks, 4);
        assert_eq!(executor.config.continue_on_failure, true);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_executor_with_config() {
        let config = ExecutorConfig {
            default_timeout: Duration::from_secs(1800),
            max_concurrent_tasks: 8,
            continue_on_failure: true,
        };
        let executor = Executor::with_config(config);
        assert_eq!(executor.config.default_timeout, Duration::from_secs(1800));
        assert_eq!(executor.config.max_concurrent_tasks, 8);
        assert_eq!(executor.config.continue_on_failure, true);
    }

    #[cfg(feature = "tokio")]
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

        let result = executor.execute_task(&mut task, Arc::default()).await;
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_execute_pipeline_empty() {
        let executor = Executor::new();
        let pipeline = Pipeline::new("pipeline-1", "Test Pipeline");

        let result = executor.execute_pipeline(pipeline).await;
        assert!(result.is_ok());
    }

    // Test timeout handling
    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_task_timeout() {
        let executor = Executor::new();
        let mut task = Task::new("task-timeout", "Timeout Task")
            .with_timeout(Duration::from_millis(100))
            .with_execution_fn(async |_| {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            });

        let result = executor.execute_task(&mut task, Arc::default()).await;
        assert!(result.is_err());
    }

    // Test error handling
    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_task_error_handling() {
        // This would test how the executor handles task failures
        let executor = Executor::new();
        let mut task = Task::new("task-error", "Error Task").with_execution_fn(async |_| {
            Err(crate::error::GaiaError::TaskExecutionFailed(
                "Test error".to_string(),
            ))
        });
        let result = executor.execute_task(&mut task, Arc::default()).await;
        assert!(result.is_err());
    }
}
