//! Pipeline definition and management

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::state::PipelineState;
use crate::task::Task;
use crate::{GaiaError, Result};

/// Represents a pipeline of tasks with dependencies
#[derive(Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique identifier for the pipeline
    pub id: String,

    /// Human-readable name of the pipeline
    pub name: String,

    /// Description of what the pipeline does
    pub description: Option<String>,

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
    /// Create a new pipeline with the given ID and name
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tasks: HashMap::new(),
            state: Arc::new(Mutex::new(PipelineState::new())),
        }
    }

    pub fn extend(&mut self, other: Self) -> Result<()> {
        for (_, task) in other.tasks {
            self.add_task(task.clone())?;
        }
        Ok(())
    }

    /// Add a task to the pipeline
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Set the description for this pipeline
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Validate the pipeline for circular dependencies
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

    /// Check if the pipeline has a cycle
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
    use crate::task::TaskStatus;
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
        let task = Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };

        let result = pipeline.add_task(task);
        assert!(result.is_ok());
        assert_eq!(pipeline.tasks.len(), 1);
        assert!(pipeline.tasks.contains_key("task-1"));
    }

    #[test]
    fn test_get_task() {
        let mut pipeline = Pipeline::new("pipeline-1", "Test Pipeline");
        let task = Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };

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
            let task = Task {
                id: format!("task-{}", i),
                name: format!("Test Task {}", i),
                description: Some(format!("A test task {}", i)),
                dependencies: HashSet::new(),
                timeout: None,
                retry_count: 0,
                status: TaskStatus::Pending,
                execution_fn: None,
            };
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
        let task1 = Task {
            id: "task-1".to_string(),
            name: "Test Task 1".to_string(),
            description: Some("A test task 1".to_string()),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };
        pipeline.add_task(task1).unwrap();

        // Add second task with dependency on first
        let mut deps = HashSet::new();
        deps.insert("task-1".to_string());
        let task2 = Task {
            id: "task-2".to_string(),
            name: "Test Task 2".to_string(),
            description: Some("A test task 2".to_string()),
            dependencies: deps,
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };
        pipeline.add_task(task2).unwrap();

        // Verify dependency relationship
        let task2 = pipeline.get_task("task-2").unwrap();
        assert_eq!(task2.dependencies.len(), 1);
        assert!(task2.dependencies.contains("task-1"));
    }
}
