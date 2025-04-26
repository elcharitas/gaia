//! Pipeline and task state management
//!
//! This module provides structures for tracking the execution state of pipelines and tasks.
//! It maintains information about start and end times, task status, retry counts, and errors.
//!
//! The state management system is used internally by the executor to track progress
//! and is also used by the monitoring system to collect performance metrics.
//!
//! # Examples
//!
//! ```ignore
//! use gaia_core::{pipeline, state::PipelineState, task::TaskStatus};
//!
//! let pipeline = pipeline!(
//!     data_pipeline, "Data Processing" => {
//!         task_1: {
//!             name: "Task 1",
//!             handler: async |_| {
//!                 Ok(())
//!             },
//!         },
//!     }
//! );
//!
//! // Access the pipeline state
//! let mut state = pipeline.state.lock().unwrap();
//!
//! // Mark the pipeline as started
//! state.start();
//!
//! // Update a task state
//! state.update_task("task_1", TaskStatus::Running);
//!
//! // Later, mark the task as completed
//! state.update_task("task_1", TaskStatus::Completed(()));
//!
//! // Mark the pipeline as complete
//! state.complete(true); // true indicates success
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::task::TaskStatus;

/// Represents the current state of a pipeline execution
///
/// This struct tracks the overall state of a pipeline, including start and end times,
/// the state of each task, and whether the pipeline has completed successfully.
///
/// The pipeline state is updated by the executor during execution and can be used
/// to monitor progress, collect metrics, and determine the final outcome.
#[derive(Default, Debug)]
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
///
/// This struct tracks the state of an individual task within a pipeline, including
/// its current status, execution times, retry count, and any error information.
///
/// Task states are managed by the `PipelineState` and updated during pipeline execution.
/// They provide detailed information about each task's progress and outcome.
#[derive(Debug)]
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
    /// Creates a new, empty pipeline state
    ///
    /// Initializes a pipeline state with no start or end time, no task states,
    /// and marked as not complete.
    pub(super) fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            task_states: HashMap::new(),
            is_complete: false,
            is_successful: false,
        }
    }

    /// Marks the pipeline as started
    ///
    /// Sets the start time to the current time. This method is called by the executor
    /// when pipeline execution begins.
    pub(super) fn start(&mut self) {
        self.start_time = Some(Instant::now().elapsed().as_millis() as u64);
    }

    /// Marks the pipeline as complete
    ///
    /// Sets the end time to the current time and updates the completion status.
    /// This method is called by the executor when pipeline execution finishes.
    ///
    /// # Arguments
    ///
    /// * `success` - Whether the pipeline completed successfully
    pub(super) fn complete(&mut self, success: bool) {
        self.end_time = Some(Instant::now().elapsed().as_millis() as u64);
        self.is_complete = true;
        self.is_successful = success;
    }

    /// Gets the duration of the pipeline execution
    ///
    /// Calculates the duration between the start and end times of the pipeline.
    /// If the pipeline is still running, calculates the duration from start until now.
    ///
    /// # Returns
    ///
    /// * `Some(Duration)` - The duration if the pipeline has started
    /// * `None` - If the pipeline hasn't started yet
    pub(super) fn duration(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(Duration::from_millis(end - start)),
            (Some(start), None) => Some(Duration::from_millis(
                Instant::now().elapsed().as_millis() as u64 - start,
            )),
            _ => None,
        }
    }

    /// Updates the state of a task
    ///
    /// Creates or updates a task state with the given status. This method is called
    /// by the executor when a task's status changes.
    ///
    /// The method automatically updates start and end times based on the status:
    /// - Sets start time when a task transitions to Running
    /// - Sets end time when a task reaches a terminal state (Completed, Failed, Cancelled)
    ///
    /// # Arguments
    ///
    /// * `task_id` - The unique identifier of the task
    /// * `status` - The new status of the task
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
                if res.is_empty() {
                    task_state.status = TaskStatus::Completed(res);
                }
            }
            _ => {
                task_state.status = status;
            }
        }
    }
}
