//! Error types for Gaia
//!
//! This module defines the error types that can occur during Gaia operations.
//! The `GaiaError` enum provides specific error variants for different failure scenarios
//! that can occur during pipeline definition, validation, and execution.
//!
//! # Examples
//!
//! ```
//! use gaia_core::{pipeline, Result};
//!
//! fn create_invalid_pipeline() -> Result<()> {
//!     // Sample pipeline with circular dependencies
//!     let pipeline = pipeline!(
//!         pipeline_1, "Invalid Pipeline" => {
//!             task_1: {
//!                 name: "Task 1",
//!                 dependencies: [task_2],
//!                 handler: async |_| {
//!                     Ok(())
//!                 },
//!             },
//!             task_2: {
//!                 name: "Task 2",
//!                 dependencies: [task_1],
//!                 handler: async |_| {
//!                     Ok(())
//!                 },
//!             },
//!         }
//!     );
//!     
//!     // This will return Err(GaiaError::CircularDependency)
//!     pipeline.validate()
//! }
//!
//! fn handle_errors() {
//!     match create_invalid_pipeline() {
//!         Ok(_) => println!("Pipeline is valid"),
//!         Err(e) => println!("Pipeline validation failed: {}", e),
//!     }
//! }
//! ```

use std::io;
use thiserror::Error;

/// Errors that can occur in Gaia operations
///
/// This enum represents all possible errors that can occur during Gaia operations.
/// Each variant provides specific information about what went wrong, making it
/// easier to diagnose and handle errors appropriately.
///
/// Most error variants include additional context information, such as the ID of
/// the task or pipeline that caused the error, or a description of what went wrong.
#[derive(Error, Debug)]
pub enum GaiaError {
    /// Error when a task is not found in a pipeline
    ///
    /// This error occurs when trying to access or execute a task that doesn't exist
    /// in the pipeline, typically when referencing a task by ID that hasn't been added.
    ///
    /// The string parameter contains the ID of the task that wasn't found.
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Error when a pipeline is not found in the system
    ///
    /// This error occurs when trying to access or execute a pipeline that doesn't exist,
    /// typically when referencing a pipeline by ID that hasn't been registered.
    ///
    /// The string parameter contains the ID of the pipeline that wasn't found.
    #[error("Pipeline not found: {0}")]
    PipelineNotFound(String),

    /// Error when a task configuration is not valid
    ///
    /// This error occurs when a task has invalid configuration, such as
    /// invalid parameters, missing required fields, or other validation issues.
    ///
    /// The string parameter contains a description of why the task is invalid.
    #[error("Task is not valid: {0}")]
    TaskNotValid(String),

    /// Error when a task fails during execution
    ///
    /// This error occurs when a task's execution function returns an error or
    /// throws an exception. This typically indicates a runtime failure rather
    /// than a configuration issue.
    ///
    /// The string parameter contains details about why the execution failed.
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// Error when a task execution exceeds its timeout duration
    ///
    /// This error occurs when a task takes longer to execute than its configured
    /// timeout duration. This is often used to prevent tasks from running indefinitely.
    #[error("Task timed out")]
    TaskTimeout,

    /// Error when a pipeline contains circular dependencies
    ///
    /// This error occurs during pipeline validation when a circular dependency is detected
    /// between tasks (e.g., task A depends on task B, which depends on task A).
    /// Circular dependencies make it impossible to determine a valid execution order.
    #[error("Circular dependency detected in pipeline")]
    CircularDependency,

    /// Error when a task dependency is not satisfied
    ///
    /// This error occurs when a task is scheduled to run, but one of its dependencies
    /// has not completed successfully. This can happen if a dependency task failed,
    /// timed out, or was cancelled.
    ///
    /// The string parameter contains the ID of the dependency that wasn't satisfied.
    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    /// Error from underlying IO operations
    ///
    /// This error occurs when an IO operation fails, such as reading from or writing
    /// to a file, network connection, or other IO resource.
    ///
    /// This variant wraps a standard Rust `io::Error` to provide more details.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error during serialization or deserialization
    ///
    /// This error occurs when serializing a pipeline or task to JSON or deserializing
    /// from JSON fails. This typically happens when the JSON structure doesn't match
    /// the expected format.
    ///
    /// This variant wraps a `serde_json::Error` to provide more details.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error for cases not covered by other variants
    ///
    /// This is a catch-all error variant for cases that don't fit into the more
    /// specific categories. The string parameter contains a description of the error.
    ///
    /// When possible, it's better to use or create a more specific error variant
    /// rather than using this generic one.
    #[error("{0}")]
    Other(String),
}
