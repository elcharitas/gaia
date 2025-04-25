//! Macros for ergonomic pipeline and task definitions

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
                    $(.with_timeout($timeout))?
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
}
