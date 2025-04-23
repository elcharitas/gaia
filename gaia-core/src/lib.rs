//! Core library for Gaia pipeline management and execution

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
