//! Pipeline and task monitoring functionality
//!
//! This module provides tools for monitoring pipeline execution and collecting performance metrics.
//! It allows tracking task status, duration, and retry counts, which can be used for performance analysis,
//! debugging, and visualization.
//!
//! # Examples
//!
//! ```
//! use gaia_core::monitoring::Monitor;
//!
//! // Create a new monitor
//! let mut monitor = Monitor::new();
//!
//! // Add custom metrics
//! monitor.add_metric(
//!     "pipeline.memory_usage",
//!     128.5, // MB
//!     vec![("pipeline_id".to_string(), "pipeline-1".to_string())]
//! );
//!
//! // Retrieve collected metrics
//! let metrics = monitor.get_metrics();
//! ```
//!
//! The metrics collected by this module can be visualized using components from the `gaia-ui` crate
//! or exported to external monitoring systems.

use std::time::Instant;

use crate::state::TaskState;
use crate::task::TaskStatus;

/// Represents a metric collected during pipeline execution
///
/// Metrics are the core data structure for monitoring in Gaia. Each metric has a name,
/// a numeric value, a timestamp, and a set of labels that provide context about what
/// the metric represents.
///
/// # Metric Types
///
/// The monitoring system collects several types of metrics by default:
///
/// * `task.status` - Numeric representation of task status (0=Pending, 1=Running, 2=Completed, etc.)
/// * `task.duration` - Time taken to execute a task in milliseconds
/// * `task.retries` - Number of times a task was retried
///
/// Custom metrics can be added using the `Monitor::add_metric` method.
#[derive(Clone, PartialEq)]
pub struct Metric {
    /// Name of the metric
    pub name: String,

    /// Value of the metric
    pub value: f64,

    /// When the metric was collected
    pub timestamp: Instant,

    /// Labels/tags associated with the metric
    pub labels: Vec<(String, String)>,
}

/// Monitors pipeline execution and collects metrics
///
/// The `Monitor` struct is responsible for collecting and storing metrics during pipeline
/// execution. It provides methods for adding custom metrics and retrieving all collected metrics.
///
/// Metrics are automatically collected for tasks during pipeline execution, including:
/// - Task status changes
/// - Task execution duration
/// - Retry attempts
///
/// The collected metrics can be used for performance analysis, debugging, and visualization.
#[derive(Clone, PartialEq)]
pub struct Monitor {
    /// Metrics collected during pipeline execution
    metrics: Vec<Metric>,
}

impl Monitor {
    /// Creates a new, empty monitor instance
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::monitoring::Monitor;
    ///
    /// let monitor = Monitor::new();
    /// assert!(monitor.get_metrics().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Collects metrics for a specific task based on its current state
    ///
    /// This method is called internally by the executor during pipeline execution.
    /// It collects metrics about task status, duration, and retry count.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The unique identifier of the task
    /// * `task_name` - The human-readable name of the task
    /// * `state` - The current state of the task
    pub(super) fn collect_task_metrics(
        &mut self,
        task_id: &str,
        task_name: &str,
        state: &TaskState,
    ) {
        // Add task status metric
        let status_value = match state.status {
            TaskStatus::Pending => 0.0,
            TaskStatus::Running => 1.0,
            TaskStatus::Completed(_) => 2.0,
            TaskStatus::Failed => 3.0,
            TaskStatus::TimedOut => 4.0,
            TaskStatus::Skipped => 5.0,
            TaskStatus::Cancelled => 6.0,
        };

        self.add_metric(
            "task.status",
            status_value,
            vec![
                ("task_id".to_string(), task_id.to_string()),
                ("task_name".to_string(), task_name.to_string()),
            ],
        );

        // Add task duration metric if available
        if let (Some(start), Some(end)) = (state.start_time, state.end_time) {
            let duration = (end - start) as f64;
            self.add_metric(
                "task.duration",
                duration,
                vec![
                    ("task_id".to_string(), task_id.to_string()),
                    ("task_name".to_string(), task_name.to_string()),
                ],
            );
        }

        // Add retry count metric
        if state.retry_count > 0 {
            self.add_metric(
                "task.retries",
                state.retry_count as f64,
                vec![
                    ("task_id".to_string(), task_id.to_string()),
                    ("task_name".to_string(), task_name.to_string()),
                ],
            );
        }
    }

    /// Adds a custom metric to the collection
    ///
    /// This method allows adding arbitrary metrics to the monitoring system.
    /// The timestamp is automatically set to the current time.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the metric (e.g., "pipeline.memory_usage")
    /// * `value` - The numeric value of the metric
    /// * `labels` - A vector of key-value pairs providing context for the metric
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::monitoring::Monitor;
    ///
    /// let mut monitor = Monitor::new();
    ///
    /// // Add a custom metric for pipeline memory usage
    /// monitor.add_metric(
    ///     "pipeline.memory_usage",
    ///     128.5, // MB
    ///     vec![("pipeline_id".to_string(), "pipeline-1".to_string())]
    /// );
    /// ```
    pub fn add_metric(&mut self, name: &str, value: f64, labels: Vec<(String, String)>) {
        self.metrics.push(Metric {
            name: name.to_string(),
            value,
            timestamp: Instant::now(),
            labels,
        });
    }

    /// Returns a slice containing all collected metrics
    ///
    /// This method provides read-only access to all metrics that have been collected
    /// by this monitor instance.
    ///
    /// # Returns
    ///
    /// A slice of `Metric` objects in the order they were added
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::monitoring::Monitor;
    ///
    /// let mut monitor = Monitor::new();
    /// monitor.add_metric("test.metric", 42.0, vec![]);
    ///
    /// let metrics = monitor.get_metrics();
    /// assert_eq!(metrics.len(), 1);
    /// assert_eq!(metrics[0].name, "test.metric");
    /// assert_eq!(metrics[0].value, 42.0);
    /// ```
    pub fn get_metrics(&self) -> &[Metric] {
        &self.metrics
    }
}
