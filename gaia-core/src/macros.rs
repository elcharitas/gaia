//! Macros for ergonomic pipeline and task definitions
//!
//! This module provides macros that simplify the creation of pipelines and tasks
//! with a more declarative syntax. The macros reduce boilerplate code and make
//! pipeline definitions more readable and maintainable.
//!
//! # Examples
//!
//! ```
//! use gaia_core::pipeline;
//! use std::time::Duration;
//!
//! // Create a data processing pipeline with three tasks
//! let pipeline = pipeline!(
//!     data_pipeline, "Data Processing Pipeline", schedule: "0 0 * * *" => {
//!         // Extract task with timeout
//!         extract: {
//!             name: "Extract Data",
//!             description: "Extract data from source system",
//!             timeout: 60, // 60 seconds
//!             handler: |_ctx| Box::pin(async {
//!                 // Extraction logic would go here
//!                 Ok("Data extracted")
//!             }),
//!         },
//!         
//!         // Transform task with dependency on extract
//!         transform: {
//!             name: "Transform Data",
//!             description: "Transform extracted data",
//!             dependencies: [extract],
//!             retry_count: 3,
//!             handler: |ctx| Box::pin(async move {
//!                 // Check if extract task completed successfully
//!                 if let Some(status) = ctx.task_status("extract") {
//!                     // Transformation logic would go here
//!                     Ok("Data transformed")
//!                 } else {
//!                     Err(gaia_core::GaiaError::DependencyNotSatisfied("extract".to_string()))
//!                 }
//!             }),
//!         },
//!         
//!         // Load task with dependency on transform
//!         load: {
//!             name: "Load Data",
//!             description: "Load transformed data",
//!             dependencies: [transform],
//!             handler: |_ctx| Box::pin(async {
//!                 // Loading logic would go here
//!                 Ok("Data loaded")
//!             }),
//!         },
//!     }
//! );
//!
//! // The pipeline can now be validated and executed
//! ```
//!
//! The macro syntax supports:
//! - Pipeline ID, name, and schedule
//! - Task definitions with names, descriptions, dependencies
//! - Timeout and retry configurations
//! - Execution handlers as async closures

#[macro_export]
macro_rules! pipeline {
    (
        $(#[$pipeline_meta:meta])* // Allow doc comments before pipeline definition
        $pipeline_id:ident$(:$parent_pipeline:ident)? $(, $pipeline_name:literal)? $(, schedule: $schedule:expr)? => {
            $(#[$task_group_meta:meta])* // Allow doc comments before task group
            $( $(#[$task_meta:meta])* $task_id:ident: {
                $(#[$task_prop_meta:meta])* // Allow doc comments before task properties
                name: $task_name:expr,
                $( $(#[$desc_meta:meta])* description: $task_desc:expr, )?
                $( $(#[$deps_meta:meta])* dependencies: [ $( $dep:ident ),* $(,)? ], )?
                $( $(#[$timeout_meta:meta])* timeout: $timeout:expr, )?
                $( $(#[$retry_meta:meta])* retry_count: $retry:expr, )?
                $(#[$handler_meta:meta])* handler: $exec_fn:expr $(,)?
            } ),* $(,)?
        }
    ) => {{
        let mut $pipeline_id = $crate::Pipeline::new(stringify!($pipeline_id), {
            #[allow(unused_mut, unused_assignments)]
            let mut name = stringify!($pipeline_id);
            $(name = $pipeline_name;)?
            name
        });
        {
            $(let _ = $pipeline_id.extend($parent_pipeline);)?
            $($pipeline_id = $pipeline_id.with_schedule($schedule);)?
            $(
                #[allow(unused_mut)]
                let $task_id = $crate::Task::new(stringify!($task_id).to_string(), $task_name)
                    $(.with_description($task_desc.to_string()))?
                    $($(.add_dependency(stringify!($dep).to_string()))*)?
                    $(.with_timeout({
                        macro_rules! process_timeout {
                            ($t:literal) => { std::time::Duration::from_secs($t) };
                            ($t:expr) => { $t };
                        }
                        process_timeout!($timeout)
                    }))?
                    $(.with_retry_count($retry))?
                    .with_execution_fn($exec_fn);
                $pipeline_id.add_task($task_id).unwrap();
            )*
        }
        $pipeline_id
    }};
}

#[cfg(test)]
mod tests {
    use crate::task::TaskStatus;

    #[test]
    fn test_define_pipeline_macro_basic() {
        let pipeline = pipeline!(
            test_pipeline, "Test Pipeline" => {
                task1: {
                    name: "Task 1",
                    handler:  async |_| {
                        Ok(())
                    },
                },
                task2: {
                    name: "Task 2",
                    dependencies: [task1],
                    handler: async |_| {
                        Ok(())
                    },
                },
            }
        );
        assert_eq!(pipeline.id, "test_pipeline");
        assert_eq!(pipeline.name, "Test Pipeline");
        assert_eq!(pipeline.tasks.len(), 2);
        assert!(pipeline.tasks.contains_key("task1"));
        assert!(pipeline.tasks.contains_key("task2"));
        let task1 = pipeline.tasks.get("task1").unwrap();
        let task2 = pipeline.tasks.get("task2").unwrap();
        assert_eq!(task1.name, "Task 1");
        assert_eq!(task2.name, "Task 2");
        assert!(task2.dependencies.contains("task1"));
        assert_eq!(task1.status, TaskStatus::Pending);
    }

    #[test]
    fn test_define_pipeline_macro_without_name() {
        let pipeline = pipeline!(
            test_pipeline => {
                task1: {
                    name: "Task 1",
                    handler:  async |_| {
                        Ok(())
                    },
                },
            }
        );
        assert_eq!(pipeline.id, "test_pipeline");
        assert_eq!(pipeline.name, "test_pipeline");
        assert_eq!(pipeline.tasks.len(), 1);
        assert!(pipeline.tasks.contains_key("task1"));
        let task1 = pipeline.tasks.get("task1").unwrap();
        assert_eq!(task1.name, "Task 1");
        assert_eq!(task1.status, TaskStatus::Pending);
    }

    #[test]
    fn test_define_pipeline_macro_with_schedule() {
        let pipeline = pipeline!(
            test_pipeline, "Test Pipeline", schedule: "0 0 * * *" => {
                task1: {
                    name: "Task 1",
                    handler:  async |_| {
                        Ok(())
                    },
                },
            }
        );
        assert_eq!(pipeline.id, "test_pipeline");
        assert_eq!(pipeline.name, "Test Pipeline");
        assert_eq!(pipeline.schedule, Some("0 0 * * *".to_string()));
        assert_eq!(pipeline.tasks.len(), 1);
        assert!(pipeline.tasks.contains_key("task1"));
    }

    #[test]
    fn test_define_pipeline_macro_with_schedule_without_name() {
        let pipeline = pipeline!(
            test_pipeline, schedule: "0 0 * * *" => {
                task1: {
                    name: "Task 1",
                    handler:  async |_| {
                        Ok(())
                    },
                },
            }
        );
        assert_eq!(pipeline.id, "test_pipeline");
        assert_eq!(pipeline.name, "test_pipeline");
        assert_eq!(pipeline.schedule, Some("0 0 * * *".to_string()));
        assert_eq!(pipeline.tasks.len(), 1);
        assert!(pipeline.tasks.contains_key("task1"));
    }

    #[test]
    fn test_define_pipeline_macro_with_timeout_options() {
        use std::time::Duration;

        // Test with integer literal timeout
        let pipeline_int = pipeline!(
            test_pipeline_int => {
                task1: {
                    name: "Task with Integer Timeout",
                    timeout: 15,  // 15 seconds as integer literal
                    handler: async |_| {
                        Ok(())
                    },
                },
            }
        );

        // Test with Duration expression timeout
        let pipeline_duration = pipeline!(
            test_pipeline_duration => {
                task1: {
                    name: "Task with Duration Timeout",
                    timeout: Duration::from_secs(30),  // Duration expression
                    handler: async |_| {
                        Ok(())
                    },
                },
            }
        );

        // Verify integer literal was converted to Duration
        let task_int = pipeline_int.tasks.get("task1").unwrap();
        assert_eq!(task_int.timeout, Some(Duration::from_secs(15)));

        // Verify Duration expression was used directly
        let task_duration = pipeline_duration.tasks.get("task1").unwrap();
        assert_eq!(task_duration.timeout, Some(Duration::from_secs(30)));
    }
}
