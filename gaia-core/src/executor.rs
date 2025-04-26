//! Task and pipeline execution logic
//!
//! This module provides the execution engine for Gaia pipelines. It handles the scheduling
//! and execution of tasks based on their dependencies, monitors their progress, and collects
//! performance metrics.
//!
//! The executor is responsible for:
//! - Validating pipelines before execution
//! - Determining the execution order of tasks based on dependencies
//! - Running tasks concurrently when possible
//! - Handling task timeouts and retries
//! - Collecting execution metrics
//!
//! # Examples
//!
//! ```
//! use gaia_core::{pipeline, Executor, executor::ExecutorConfig, Result};
//! use std::time::Duration;
//!
//! async fn run_example_pipeline() -> Result<()> {
//!     let pipeline = pipeline!(
//!         data_pipeline, "Data Processing" => {
//!             extract: {
//!                 name: "Extract Data",
//!                 handler: async |_| {
//!                     // Extract data implementation
//!                     Ok(())
//!                 },
//!             },
//!             transform: {
//!                 name: "Transform Data",
//!                 dependencies: [extract],
//!                 handler: async |_| {
//!                     // Transform data implementation
//!                     Ok(())
//!                 },
//!             },
//!         }
//!     );
//!     
//!     // Create an executor with custom configuration
//!     let executor = Executor::with_config(
//!         ExecutorConfig {
//!             default_timeout: Duration::from_secs(300), // 5 minutes
//!             max_concurrent_tasks: 8,
//!             continue_on_failure: false,
//!         }
//!     );
//!     
//!     // Execute the pipeline
//!     let monitor = executor.execute_pipeline(pipeline).await?;
//!     
//!     // Access execution metrics
//!     for metric in monitor.get_metrics() {
//!         println!("Metric: {} = {}", metric.name, metric.value);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::GaiaError;
use crate::monitoring::Monitor;
use crate::pipeline::Pipeline;
use crate::task::{Task, TaskResult, TaskStatus};

/// Handles the execution of pipelines and their tasks
///
/// The `Executor` is responsible for running pipelines and their tasks in the correct order,
/// respecting dependencies between tasks, and handling failures appropriately.
///
/// It provides methods for executing entire pipelines or individual tasks, and can be
/// configured with custom timeout, concurrency, and failure handling settings.
#[derive(Debug)]
pub struct Executor {
    /// Configuration for the executor
    config: ExecutorConfig,
}

/// Context provided to task execution functions
///
/// This struct provides access to the current state of the pipeline execution,
/// allowing tasks to check the status of other tasks and make decisions based
/// on that information.
///
/// It's passed to task execution functions when they're invoked by the executor.
pub struct ExecutorContext {
    task_statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

impl ExecutorContext {
    /// Creates a new executor context
    ///
    /// This is used internally by the executor to create a context for task execution.
    ///
    /// # Arguments
    ///
    /// * `task_statuses` - Shared map of task IDs to their current status
    /// Gets the current status of a task
    ///
    /// This method allows a task to check the status of another task in the pipeline.
    /// This is useful for conditional execution based on the results of other tasks.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to check
    ///
    /// # Returns
    ///
    /// * `Some(TaskStatus)` - The current status of the task if it exists
    /// * `None` - If the task doesn't exist in the pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::task::{TaskStatus, TaskResult};
    /// use gaia_core::executor::ExecutorContext;
    ///
    /// fn check_dependency(ctx: &ExecutorContext) -> bool {
    ///     match ctx.task_status("dependency-task") {
    ///         Some(TaskStatus::Completed(_)) => true,
    ///         _ => false,
    ///     }
    /// }
    /// ```
    pub fn task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.task_statuses.lock().unwrap().get(task_id).cloned()
    }
}

/// Configuration options for the executor
///
/// This struct allows customizing the behavior of the executor, including
/// timeout durations, concurrency limits, and failure handling.
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
    /// Creates a new executor with default configuration
    ///
    /// The default configuration includes:
    /// - 1 hour default timeout for tasks
    /// - 4 concurrent tasks maximum
    /// - Continue pipeline execution when a task fails
    /// Creates a new executor with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::Executor;
    ///
    /// let executor = Executor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
        }
    }

    /// Creates a new executor with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration for the executor
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Executor, executor::ExecutorConfig};
    /// use std::time::Duration;
    ///
    /// let config = ExecutorConfig {
    ///     default_timeout: Duration::from_secs(300), // 5 minutes
    ///     max_concurrent_tasks: 8,
    ///     continue_on_failure: false,
    /// };
    ///
    /// let executor = Executor::with_config(config);
    /// ```
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// Executes a pipeline and returns execution metrics
    ///
    /// This method executes all tasks in the pipeline in the correct order based on
    /// their dependencies. It handles concurrent execution of independent tasks and
    /// collects metrics about the execution.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline to execute
    ///
    /// # Returns
    ///
    /// * `Ok(Monitor)` - Execution completed successfully, with collected metrics
    /// * `Err(GaiaError)` - Execution failed
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Executor, Pipeline, Task};
    ///
    /// async fn run_pipeline() {
    ///     let mut pipeline = Pipeline::new("pipeline-1", "Example Pipeline");
    ///     pipeline.add_task(Task::new("task-1", "Example Task")).unwrap();
    ///
    ///     let executor = Executor::new();
    ///     match executor.execute_pipeline(pipeline).await {
    ///         Ok(monitor) => {
    ///             println!("Pipeline executed successfully");
    ///             for metric in monitor.get_metrics() {
    ///                 println!("Metric: {} = {}", metric.name, metric.value);
    ///             }
    ///         },
    ///         Err(e) => println!("Pipeline execution failed: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn execute_pipeline(&self, mut pipeline: Pipeline) -> crate::Result<Monitor> {
        let mut monitor = Monitor::new();

        pipeline.validate()?; // Validate the pipeline before execution

        let mut tasks: Vec<&mut Task> = pipeline.tasks.values_mut().collect();
        let mut pipeline_state = pipeline.state.lock().unwrap();

        pipeline_state.start();

        if !tasks.is_empty() {
            // TODO: proper topological sorting
            tasks.sort_by(|a, b| a.dependencies.len().cmp(&b.dependencies.len()));

            let max_concurrent = self.config.max_concurrent_tasks;
            let task_map: HashMap<String, usize> = tasks
                .iter()
                .enumerate()
                .map(|(i, t)| (t.id.clone(), i))
                .collect();

            let task_statuses = Arc::new(Mutex::new(HashMap::<String, TaskStatus>::new()));
            let mut pending: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
            let mut running = FuturesUnordered::new();
            let tasks_arc = Arc::new(Mutex::new(tasks));

            while !pending.is_empty() || running.len() > 0 {
                let mut ready = vec![];
                {
                    let statuses = task_statuses.lock().unwrap();
                    for id in pending.iter() {
                        let idx = *task_map.get(id).unwrap();
                        let task = &tasks_arc.lock().unwrap()[idx];
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
                        (id_clone, result)
                    };

                    if let Some(task_state) = pipeline_state.task_states.get(&id) {
                        monitor.collect_task_metrics(&id, &task_name, &task_state);
                    }

                    running.push(exec);
                    pending.remove(&id);
                }

                if let Some((task_id, Ok(result))) = running.next().await {
                    pipeline_state.update_task(&task_id, TaskStatus::Completed(result));
                    task_statuses
                        .lock()
                        .unwrap()
                        .insert(task_id, TaskStatus::Completed(Box::new(())));
                };
            }
        }

        pipeline_state.complete(true);

        // Collect pipeline-level metrics
        if let Some(duration) = pipeline_state.duration() {
            monitor.add_metric(
                "pipeline.duration",
                duration.as_secs_f64(),
                vec![
                    ("pipeline_id".to_string(), pipeline.id.clone()),
                    ("pipeline_name".to_string(), pipeline.name.clone()),
                ],
            );
        }

        // Collect task-level metrics
        for (task_id, task_state) in &pipeline_state.task_states {
            if let Some(task) = pipeline.tasks.get(task_id) {
                monitor.collect_task_metrics(task_id, task.name.as_str(), task_state);
            }
        }

        Ok(monitor)
    }

    /// Executes a single task
    ///
    /// This method executes a single task and returns its result. It handles timeouts
    /// and execution errors, and updates the task's status accordingly.
    ///
    /// This is primarily used internally by `execute_pipeline`, but can also be used
    /// to execute individual tasks outside of a pipeline context.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to execute
    /// * `task_statuses` - Shared map of task IDs to their current status
    ///
    /// # Returns
    ///
    /// * `Ok(Box<dyn TaskResult>)` - Task executed successfully with a result
    /// * `Err(GaiaError)` - Task execution failed
    pub async fn execute_task(
        &self,
        task: &mut Task,
        task_statuses: Arc<Mutex<HashMap<String, TaskStatus>>>,
    ) -> crate::Result<Box<dyn TaskResult>> {
        let result = if let Some(execution_fn) = &mut task.execution_fn {
            let result: Result<crate::Result<Box<dyn TaskResult>>, _> = if let Some(timeout) =
                task.timeout
            {
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
                    Ok(res) => Ok(res),
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
            Ok(Box::new(()) as Box<dyn TaskResult>)
        };

        if result.is_ok() {
            task.status = TaskStatus::Completed(Box::new(()));
            Ok(Box::new(()))
        } else {
            result
        }
    }
}

// Helper to check if all dependencies of a task are completed
fn is_ready(task: &Task, statuses: &HashMap<String, TaskStatus>) -> bool {
    task.dependencies
        .iter()
        .all(|dep| matches!(statuses.get(dep), Some(TaskStatus::Completed(_))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskStatus;

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
        let mut task = Task::new("task-1", "Test Task").with_execution_fn(async |_| {
            println!("Executing task...");
            Ok(())
        });

        let result = executor.execute_task(&mut task, Arc::default()).await;
        assert!(result.is_ok());
        assert_eq!(matches!(task.status, TaskStatus::Completed(_)), true);
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
            Err::<(), GaiaError>(crate::error::GaiaError::TaskExecutionFailed(
                "Test error".to_string(),
            ))
        });
        let result = executor.execute_task(&mut task, Arc::default()).await;
        assert!(result.is_err());
    }
}
