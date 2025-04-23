//! Macros for ergonomic pipeline and task definitions

#[macro_export]
macro_rules! define_pipeline {
    (
        $pipeline_id:ident$(:$parent_pipeline:expr)?, $pipeline_name:expr, {
            $( $task_id:ident: {
                name: $task_name:expr,
                $( description: $task_desc:expr, )?
                $( dependencies: [ $( $dep:ident ),* $(,)? ], )?
                $( timeout: $timeout:expr, )?
                $( retry_count: $retry:expr, )?
                handler: $exec_fn:expr $(,)?
            } ),* $(,)?
        }
    ) => {{
        let mut $pipeline_id = $crate::Pipeline::new(stringify!($pipeline_id), $pipeline_name);
        $(let _ = $pipeline_id.extend($parent_pipeline);)?
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
        $pipeline_id
    }};
}

#[cfg(test)]
mod tests {
    use crate::task::TaskStatus;

    #[test]
    fn test_define_pipeline_macro_basic() {
        let pipeline = define_pipeline!(
            test_pipeline, "Test Pipeline", {
                task1: {
                    name: "Task 1",
                    handler:  async || {
                        Ok(())
                    },
                },
                task2: {
                    name: "Task 2",
                    dependencies: [task1],
                    handler: async || {
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
}
