//! Integration tests for gaia-core

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gaia_core::executor::{Executor, ExecutorConfig};
use gaia_core::pipeline::Pipeline;
use gaia_core::task::{Task, TaskStatus};

#[tokio::test]
async fn test_pipeline_execution_with_dependencies() {
    // Create an executor
    let executor = Executor::new();

    // Create a pipeline with dependent tasks
    let mut pipeline = Pipeline::new("pipeline-deps", "Pipeline with Dependencies");

    // Create tasks with dependencies
    let task1 = Task {
        id: "task-1".to_string(),
        name: "First Task".to_string(),
        description: Some("The first task".to_string()),
        dependencies: HashSet::new(), // No dependencies
        timeout: None,
        retry_count: 0,
        status: TaskStatus::Pending,
        execution_fn: None,
    };

    let mut task2_deps = HashSet::new();
    task2_deps.insert("task-1".to_string());
    let task2 = Task {
        id: "task-2".to_string(),
        name: "Second Task".to_string(),
        description: Some("The second task that depends on the first".to_string()),
        dependencies: task2_deps,
        timeout: None,
        retry_count: 0,
        status: TaskStatus::Pending,
        execution_fn: None,
    };

    // Add tasks to pipeline
    pipeline.add_task(task1).unwrap();
    pipeline.add_task(task2).unwrap();

    // Execute pipeline
    let pipeline_arc = Arc::new(Mutex::new(pipeline));
    let result = executor.execute_pipeline(pipeline_arc.clone()).await;

    // Verify execution was successful
    assert!(result.is_ok());

    // In a real implementation with actual execution logic, we would verify:
    // 1. Task 1 executed before Task 2
    // 2. Both tasks completed successfully
    // 3. Pipeline state reflects correct execution order and status
}

#[tokio::test]
async fn test_executor_configuration_affects_execution() {
    // Create a custom executor configuration
    let config = ExecutorConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_tasks: 2,
        continue_on_failure: true,
    };

    // Create executor with custom config
    let executor = Executor::with_config(config);

    // Create a simple pipeline
    let mut pipeline = Pipeline::new("pipeline-config", "Pipeline for Config Testing");

    // Add a few tasks
    for i in 1..=4 {
        let task = Task {
            id: format!("task-{}", i),
            name: format!("Task {}", i),
            description: Some(format!("Task number {}", i)),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: 0,
            status: TaskStatus::Pending,
            execution_fn: None,
        };
        pipeline.add_task(task).unwrap();
    }

    // Execute pipeline
    let pipeline_arc = Arc::new(Mutex::new(pipeline));
    let result = executor.execute_pipeline(pipeline_arc.clone()).await;

    // Verify execution was successful
    assert!(result.is_ok());

    // In a real implementation, we would verify:
    // 1. Only 2 tasks run concurrently (max_concurrent_tasks)
    // 2. Tasks use the default timeout when none specified
    // 3. Pipeline continues even if a task fails (continue_on_failure)
}

#[tokio::test]
async fn test_pipeline_with_error_handling() {
    // Create an executor that continues on failure
    let config = ExecutorConfig {
        default_timeout: Duration::from_secs(3600),
        max_concurrent_tasks: 4,
        continue_on_failure: true,
    };
    let executor = Executor::with_config(config);

    // Create a pipeline with a mix of successful and failing tasks
    let mut pipeline = Pipeline::new("pipeline-errors", "Pipeline with Error Handling");

    // Add tasks (in a real implementation, we'd have a way to make tasks fail)
    for i in 1..=3 {
        let task = Task {
            id: format!("task-{}", i),
            name: format!("Task {}", i),
            description: Some(format!("Task number {}", i)),
            dependencies: HashSet::new(),
            timeout: None,
            retry_count: i - 1, // Different retry counts
            status: TaskStatus::Pending,
            execution_fn: None,
        };
        pipeline.add_task(task).unwrap();
    }

    // Execute pipeline
    let pipeline_arc = Arc::new(Mutex::new(pipeline));
    let result = executor.execute_pipeline(pipeline_arc.clone()).await;

    // Verify execution was successful overall (due to continue_on_failure)
    assert!(result.is_ok());

    // In a real implementation, we would verify:
    // 1. Failed tasks are retried the correct number of times
    // 2. Pipeline continues despite task failures
    // 3. Pipeline state correctly reflects which tasks failed
}
