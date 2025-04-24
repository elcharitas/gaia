//! Pipeline and task state management

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::task::TaskStatus;

/// Represents the current state of a pipeline execution
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PipelineState {
    /// When the pipeline started executing
    pub start_time: Option<u64>,

    /// When the pipeline finished executing
    pub end_time: Option<u64>,

    /// Current status of each task in the pipeline
    pub task_states: HashMap<String, TaskState>,

    /// Whether the pipeline execution is complete
    pub is_complete: bool,

    /// Whether the pipeline execution was successful
    pub is_successful: bool,
}

/// Represents the current state of a task execution
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskState {
    /// Current status of the task
    pub status: TaskStatus,

    /// When the task started executing
    pub start_time: Option<u64>,

    /// When the task finished executing
    pub end_time: Option<u64>,

    /// How many times the task has been retried
    pub retry_count: u32,

    /// Error message if the task failed
    pub error: Option<String>,
}

impl PipelineState {
    /// Create a new pipeline state
    pub(super) fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            task_states: HashMap::new(),
            is_complete: false,
            is_successful: false,
        }
    }

    /// Mark the pipeline as started
    pub(super) fn start(&mut self) {
        self.start_time = Some(Instant::now().elapsed().as_millis() as u64);
    }

    /// Mark the pipeline as complete
    pub(super) fn complete(&mut self, success: bool) {
        self.end_time = Some(Instant::now().elapsed().as_millis() as u64);
        self.is_complete = true;
        self.is_successful = success;
    }

    /// Get the duration of the pipeline execution
    pub(super) fn duration(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(Duration::from_millis(end - start)),
            (Some(start), None) => Some(Duration::from_millis(
                Instant::now().elapsed().as_millis() as u64 - start,
            )),
            _ => None,
        }
    }

    /// Update the state of a task
    pub(super) fn update_task(&mut self, task_id: impl Into<String>, status: TaskStatus) {
        let task_state = self
            .task_states
            .entry(task_id.into())
            .or_insert_with(|| TaskState {
                status: TaskStatus::Pending,
                start_time: None,
                end_time: None,
                retry_count: 0,
                error: None,
            });

        match status {
            TaskStatus::Running if task_state.start_time.is_none() => {
                task_state.start_time = Some(Instant::now().elapsed().as_millis() as u64);
            }
            TaskStatus::Completed(_) | TaskStatus::Failed | TaskStatus::Cancelled => {
                if task_state.end_time.is_none() {
                    task_state.end_time = Some(Instant::now().elapsed().as_millis() as u64);
                }
            }
            _ => {}
        }
        match status {
            TaskStatus::Completed(res) => {
                if res != () {
                    task_state.status = status;
                }
            }
            _ => {
                task_state.status = status;
            }
        }
    }
}
