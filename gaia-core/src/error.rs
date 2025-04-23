//! Error types for Gaia

use std::io;
use thiserror::Error;
use tokio::time::error::Elapsed;

/// Errors that can occur in Gaia operations
#[derive(Error, Debug)]
pub enum GaiaError {
    /// Error when a task is not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Error when a pipeline is not found
    #[error("Pipeline not found: {0}")]
    PipelineNotFound(String),

    /// Error when a task is not valid
    #[error("Task is not valid: {0}")]
    TaskNotValid(String),

    /// Error when a task fails to execute
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// Error when a task fails due to timeout
    #[error("Task timed out: {0}")]
    TaskTimeout(Elapsed),

    /// Error when a pipeline has a circular dependency
    #[error("Circular dependency detected in pipeline")]
    CircularDependency,

    /// Error when a task dependency is not satisfied
    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
