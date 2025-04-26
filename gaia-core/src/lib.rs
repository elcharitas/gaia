//! Core library for Gaia pipeline management and execution
//!
//! Gaia is a flexible pipeline management system designed for defining, executing, and monitoring
//! complex workflows. It provides a robust framework for creating task-based pipelines with
//! dependency management, concurrent execution, error handling, and performance monitoring.
//!
//! # Key Features
//!
//! - Define complex pipelines with dependencies between tasks
//! - Execute tasks concurrently when possible
//! - Handle failures with configurable retry and fallback strategies
//! - Monitor execution status and performance metrics
//! - Visualize pipeline execution through the companion UI crate
//!
//! # Architecture
//!
//! Gaia is built around several core concepts:
//!
//! - **Tasks**: Individual units of work with inputs, outputs, and dependencies
//! - **Pipelines**: Collections of tasks with defined execution order
//! - **Executor**: Engine that runs pipelines and manages task execution
//! - **Monitor**: System for collecting and analyzing performance metrics
//!
//! # Example
//!
//! ```
//! use gaia_core::{pipeline, Executor, Result};
//! use std::time::Duration;
//!
//! async fn example() -> Result<()> {
//!     let pipeline = pipeline!(
//!         data_pipeline, "Data Processing", schedule: "0 0 * * *" => {
//!             extract: {
//!                 name: "Extract Data",
//!                 description: "Extract data from source",
//!                 timeout: 60,
//!                 handler: async |_| {
//!                     // Extract data implementation
//!                     Ok(())
//!                 },
//!             },
//!             transform: {
//!                 name: "Transform Data",
//!                 description: "Transform extracted data",
//!                 dependencies: [extract],
//!                 handler: async |_| {
//!                     // Transform data implementation
//!                     Ok(())
//!                 },
//!             },
//!             load: {
//!                 name: "Load Data",
//!                 description: "Load transformed data",
//!                 dependencies: [transform],
//!                 handler: async |_| {
//!                     // Load data implementation
//!                     Ok(())
//!                 },
//!             },
//!         }
//!     );
//!
//!     // Execute the pipeline
//!     let executor = Executor::new();
//!     let monitor = executor.execute_pipeline(pipeline).await?;
//!
//!     // Analyze metrics
//!     for metric in monitor.get_metrics() {
//!         println!("Metric: {} = {}", metric.name, metric.value);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`pipeline`]: Pipeline definition and management
//! - [`task`]: Task definition and execution
//! - [`executor`]: Task and pipeline execution logic
//! - [`state`]: Pipeline and task state management
//! - [`monitoring`]: Pipeline and task monitoring functionality
//! - [`error`]: Error types for Gaia
//! - [`macros`]: Utility macros for Gaia

pub mod error;
pub mod executor;
pub mod macros;
pub mod monitoring;
pub mod pipeline;
pub mod state;
pub mod task;

/// Re-export commonly used types
pub use error::GaiaError;
pub use executor::Executor;
pub use pipeline::Pipeline;
pub use state::PipelineState;
pub use task::Task;

/// Result type for Gaia operations
pub type Result<T> = std::result::Result<T, GaiaError>;

/// Gaia version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
