//! Pipeline and task monitoring functionality

use std::time::Instant;

use crate::state::TaskState;
use crate::task::TaskStatus;

/// Represents a metric collected during pipeline execution
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
#[derive(Clone, PartialEq)]
pub struct Monitor {
    /// Metrics collected during pipeline execution
    metrics: Vec<Metric>,
}

impl Monitor {
    /// Create a new monitor
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Collect metrics for a specific task
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

    /// Add a metric to the collection
    pub fn add_metric(&mut self, name: &str, value: f64, labels: Vec<(String, String)>) {
        self.metrics.push(Metric {
            name: name.to_string(),
            value,
            timestamp: Instant::now(),
            labels,
        });
    }

    /// Get all collected metrics
    pub fn get_metrics(&self) -> &[Metric] {
        &self.metrics
    }
}
