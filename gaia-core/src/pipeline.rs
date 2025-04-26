//! Pipeline definition and management
//!
//! This module provides the core `Pipeline` struct which represents a collection of tasks
//! with dependencies between them. Pipelines are the main building blocks of Gaia and
//! allow defining complex workflows with proper dependency management.
//!
//! # Examples
//!
//! ```
//! use gaia_core::{pipeline, Pipeline, Result};
//!
//! fn create_data_processing_pipeline() -> Result<Pipeline> {
//!     let pipeline = pipeline!(
//!         data_pipeline, "Data Processing Pipeline", schedule: "0 0 * * *" => {
//!             extract: {
//!                 name: "Extract Data",
//!                 description: "Extract data from source",
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
//!     // Validate the pipeline to ensure there are no circular dependencies
//!     pipeline.validate()?;
//!
//!     Ok(pipeline)
//! }
//! ```
//!
//! Pipelines can be executed using the `Executor` from the executor module:
//!
//! ```
//! use gaia_core::{Executor, Pipeline};
//!
//! async fn run_pipeline(pipeline: Pipeline) {
//!     let executor = Executor::new();
//!     let result = executor.execute_pipeline(pipeline).await;
//!     match result {
//!         Ok(_) => println!("Pipeline executed successfully"),
//!         Err(e) => println!("Pipeline execution failed: {}", e),
//!     }
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::state::PipelineState;
use crate::task::Task;
use crate::{GaiaError, Result};

/// Represents a pipeline of tasks with dependencies
///
/// A pipeline is a collection of tasks with dependencies between them. The pipeline
/// ensures that tasks are executed in the correct order based on their dependencies.
///
/// Pipelines can be scheduled to run at specific times using cron expressions and
/// can be executed using the `Executor` from the executor module.
///
/// # Fields
///
/// * `id` - Unique identifier for the pipeline
/// * `name` - Human-readable name of the pipeline
/// * `description` - Optional description of what the pipeline does
/// * `schedule` - Optional cron schedule expression for the pipeline
/// * `tasks` - Map of task IDs to tasks that make up the pipeline
/// * `state` - Current state of the pipeline during execution
#[derive(Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique identifier for the pipeline
    pub id: String,

    /// Human-readable name of the pipeline
    pub name: String,

    /// Description of what the pipeline does
    pub description: Option<String>,

    /// Cron schedule expression for the pipeline
    pub schedule: Option<String>,

    /// Tasks that make up the pipeline
    pub tasks: HashMap<String, Task>,

    /// Current state of the pipeline
    #[serde(skip)]
    pub state: Arc<Mutex<PipelineState>>,
}

impl PartialEq for Pipeline {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Pipeline {
    /// Creates a new pipeline with the given ID and name
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the pipeline
    /// * `name` - Human-readable name of the pipeline
    ///
    /// # Returns
    ///
    /// A new pipeline with no tasks, no description, and no schedule
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::Pipeline;
    ///
    /// let pipeline = Pipeline::new("pipeline-1", "My Pipeline");
    /// assert_eq!(pipeline.id, "pipeline-1");
    /// assert_eq!(pipeline.name, "My Pipeline");
    /// ```
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            schedule: None,
            tasks: HashMap::new(),
            state: Arc::new(Mutex::new(PipelineState::new())),
        }
    }

    /// Extends this pipeline with tasks from another pipeline
    ///
    /// This method adds all tasks from the `other` pipeline to this pipeline.
    /// It's useful for combining multiple smaller pipelines into a larger one.
    ///
    /// # Arguments
    ///
    /// * `other` - The pipeline whose tasks will be added to this pipeline
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all tasks were added successfully
    /// * `Err` if there was an error adding any task
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Pipeline, Task};
    ///
    /// let mut main_pipeline = Pipeline::new("main", "Main Pipeline");
    /// let mut sub_pipeline = Pipeline::new("sub", "Sub Pipeline");
    ///
    /// sub_pipeline.add_task(Task::new("task-1", "Task 1")).unwrap();
    /// main_pipeline.extend(sub_pipeline).unwrap();
    ///
    /// assert!(main_pipeline.get_task("task-1").is_some());
    /// ```
    pub fn extend(&mut self, other: Self) -> Result<()> {
        for (_, task) in other.tasks {
            self.add_task(task.clone())?;
        }
        Ok(())
    }

    /// Adds a task to the pipeline
    ///
    /// # Arguments
    ///
    /// * `task` - The task to add to the pipeline
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the task was added successfully
    /// * `Err` if there was an error adding the task
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Pipeline, Task};
    ///
    /// let mut pipeline = Pipeline::new("pipeline-1", "My Pipeline");
    /// let task = Task::new("task-1", "My Task");
    ///
    /// pipeline.add_task(task).unwrap();
    /// assert!(pipeline.get_task("task-1").is_some());
    /// ```
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Gets a task by ID
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to get
    ///
    /// # Returns
    ///
    /// * `Some(&Task)` if the task exists in the pipeline
    /// * `None` if the task does not exist in the pipeline
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Pipeline, Task};
    ///
    /// let mut pipeline = Pipeline::new("pipeline-1", "My Pipeline");
    /// pipeline.add_task(Task::new("task-1", "My Task")).unwrap();
    ///
    /// let task = pipeline.get_task("task-1");
    /// assert!(task.is_some());
    /// assert_eq!(task.unwrap().name, "My Task");
    ///
    /// let non_existent = pipeline.get_task("non-existent");
    /// assert!(non_existent.is_none());
    /// ```
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Sets the description for this pipeline
    ///
    /// # Arguments
    ///
    /// * `description` - A description of what the pipeline does
    ///
    /// # Returns
    ///
    /// The pipeline with the description set
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::Pipeline;
    ///
    /// let pipeline = Pipeline::new("pipeline-1", "My Pipeline")
    ///     .with_description("This pipeline does something important");
    ///
    /// assert_eq!(pipeline.description, Some("This pipeline does something important".to_string()));
    /// ```
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the cron schedule for this pipeline
    ///
    /// # Arguments
    ///
    /// * `schedule` - A cron expression defining when the pipeline should run
    ///
    /// # Returns
    ///
    /// The pipeline with the schedule set
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::Pipeline;
    ///
    /// let pipeline = Pipeline::new("pipeline-1", "My Pipeline")
    ///     .with_schedule("0 0 * * *"); // Run daily at midnight
    ///
    /// assert_eq!(pipeline.schedule, Some("0 0 * * *".to_string()));
    /// ```
    pub fn with_schedule(mut self, schedule: impl Into<String>) -> Self {
        self.schedule = Some(schedule.into());
        self
    }

    /// Validates the pipeline for circular dependencies
    ///
    /// This method checks that there are no circular dependencies between tasks
    /// in the pipeline. It also verifies that all task dependencies exist in the pipeline.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pipeline is valid
    /// * `Err(GaiaError::CircularDependency)` if there is a circular dependency
    /// * `Err(GaiaError::TaskNotFound)` if a task depends on a non-existent task
    ///
    /// # Examples
    ///
    /// ```
    /// use gaia_core::{Pipeline, Task, Result};
    ///
    /// fn create_valid_pipeline() -> Result<()> {
    ///     let mut pipeline = Pipeline::new("pipeline-1", "Valid Pipeline");
    ///
    ///     pipeline.add_task(Task::new("task-1", "Task 1"))?;
    ///     pipeline.add_task(
    ///         Task::new("task-2", "Task 2")
    ///             .add_dependency("task-1")
    ///     )?;
    ///
    ///     // This should succeed
    ///     pipeline.validate()
    /// }
    ///
    /// fn create_invalid_pipeline() -> Result<()> {
    ///     let mut pipeline = Pipeline::new("pipeline-2", "Invalid Pipeline");
    ///
    ///     pipeline.add_task(
    ///         Task::new("task-1", "Task 1")
    ///             .add_dependency("task-2")
    ///     )?;
    ///     pipeline.add_task(
    ///         Task::new("task-2", "Task 2")
    ///             .add_dependency("task-1")
    ///     )?;
    ///
    ///     // This should fail with CircularDependency
    ///     pipeline.validate()
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        for (_, task) in &self.tasks {
            if !visited.contains(&task.id) {
                if self.has_cycle(task, &mut visited, &mut stack)? {
                    return Err(GaiaError::CircularDependency);
                }
            }
        }
        Ok(())
    }

    /// Checks if the pipeline has a cycle starting from the given task
    ///
    /// This is a helper method used by `validate()` to detect circular dependencies
    /// using a depth-first search algorithm.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to start the cycle check from
    /// * `visited` - Set of task IDs that have been visited in any path
    /// * `stack` - Set of task IDs in the current path
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if a cycle was detected
    /// * `Ok(false)` if no cycle was detected
    /// * `Err` if a task dependency doesn't exist
    fn has_cycle(
        &self,
        task: &Task,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<bool> {
        if stack.contains(&task.id) {
            return Ok(true);
        }
        if visited.contains(&task.id) {
            return Ok(false);
        }
        visited.insert(task.id.clone());
        stack.insert(task.id.clone());
        for dep in &task.dependencies {
            if let Some(dep_task) = self.get_task(dep) {
                if self.has_cycle(dep_task, visited, stack)? {
                    return Ok(true);
                }
            } else {
                return Err(GaiaError::TaskNotFound(dep.to_string()));
            }
        }
        stack.remove(&task.id);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_pipeline_new() {
        let pipeline = Pipeline::new("pipeline-1", "Test Pipeline");
        assert_eq!(pipeline.id, "pipeline-1");
        assert_eq!(pipeline.name, "Test Pipeline");
        assert_eq!(pipeline.description, None);
        assert!(pipeline.tasks.is_empty());
    }

    #[test]
    fn test_pipeline_with_description() {
        let pipeline =
            Pipeline::new("pipeline-1", "Test Pipeline").with_description("A test pipeline");
        assert_eq!(pipeline.description, Some("A test pipeline".to_string()));
    }

    #[test]
    fn test_add_task() {
        let mut pipeline = Pipeline::new("pipeline-1", "Test Pipeline");
        let task = Task::new("task-1", "Test Task").with_description("A test task");

        let result = pipeline.add_task(task);
        assert!(result.is_ok());
        assert_eq!(pipeline.tasks.len(), 1);
        assert!(pipeline.tasks.contains_key("task-1"));
    }

    #[test]
    fn test_get_task() {
        let mut pipeline = Pipeline::new("pipeline-1", "Test Pipeline");
        let task = Task::new("task-1", "Test Task").with_description("A test task");

        pipeline.add_task(task).unwrap();

        let retrieved_task = pipeline.get_task("task-1");
        assert!(retrieved_task.is_some());
        let retrieved_task = retrieved_task.unwrap();
        assert_eq!(retrieved_task.id, "task-1");
        assert_eq!(retrieved_task.name, "Test Task");

        // Test getting a non-existent task
        let non_existent = pipeline.get_task("non-existent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_multiple_tasks() {
        let mut pipeline = Pipeline::new("pipeline-1", "Test Pipeline");

        // Add multiple tasks
        for i in 1..=3 {
            let task = Task::new(format!("task-{}", i), format!("Test Task {}", i))
                .with_description(format!("A test task {}", i));
            pipeline.add_task(task).unwrap();
        }

        assert_eq!(pipeline.tasks.len(), 3);
        assert!(pipeline.tasks.contains_key("task-1"));
        assert!(pipeline.tasks.contains_key("task-2"));
        assert!(pipeline.tasks.contains_key("task-3"));
    }

    #[test]
    fn test_task_with_dependencies() {
        let mut pipeline = Pipeline::new("pipeline-1", "Test Pipeline");

        // Add first task
        let task1 = Task::new("task-1", "Test Task 1").with_description("A test task 1");
        pipeline.add_task(task1).unwrap();

        // Add second task with dependency on first
        let mut deps = HashSet::new();
        deps.insert("task-1".to_string());
        let task2 = Task::new("task-2", "Test Task 2")
            .with_description("A test task 2")
            .with_dependencies(deps);
        pipeline.add_task(task2).unwrap();

        // Verify dependency relationship
        let task2 = pipeline.get_task("task-2").unwrap();
        assert_eq!(task2.dependencies.len(), 1);
        assert!(task2.dependencies.contains("task-1"));
    }
}
