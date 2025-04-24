//! Integration tests for gaia-core

use gaia_core::executor::Executor;
use gaia_core::pipeline::Pipeline;
use gaia_core::task::Task;

#[cfg(feature = "tokio")]
use tokio;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_pipeline_execution_with_dependencies() {
    use gaia_core::task::{CastTask, TaskStatus};

    let executor = Executor::new();
    let mut pipeline = Pipeline::new("pipeline-deps", "Pipeline with Dependencies");

    let task1 = Task::new("task-1", "First Task").with_execution_fn(async |_| Ok(2));
    pipeline.add_task(task1).unwrap();

    let task2 = Task::new("task-2", "Second Task")
        .add_dependency("task-1")
        .with_execution_fn(async |ctx| {
            match ctx.task_status("task-1") {
                Some(result) => match result {
                    TaskStatus::Completed(value) => {
                        let value = value.cast::<i32>();
                        assert_eq!(value, Some(&2));
                    }
                    _ => panic!("Dependency task result not as expected"),
                },
                _ => panic!("Dependency task result not as expected"),
            }

            Ok(3)
        });
    pipeline.add_task(task2).unwrap();

    let result = executor.execute_pipeline(pipeline).await;

    assert!(result.is_ok());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_executor_configuration_affects_execution() {
    let executor = Executor::new();
    let mut pipeline = Pipeline::new("pipeline-config", "Pipeline for Config Testing");

    for i in 1..=4 {
        let task = Task::new(format!("task-{}", i), "Test Task");
        pipeline.add_task(task).unwrap();
    }

    let result = executor.execute_pipeline(pipeline).await;

    assert!(result.is_ok());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_pipeline_with_error_handling() {
    let executor = Executor::new();
    let mut pipeline = Pipeline::new("pipeline-errors", "Pipeline with Error Handling");

    for i in 1..=4 {
        let task = Task::new(format!("task-{}", i), "Test Task");
        pipeline.add_task(task).unwrap();
    }

    let result = executor.execute_pipeline(pipeline).await;

    assert!(result.is_ok());
}
