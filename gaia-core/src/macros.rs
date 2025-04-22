//! Macros for ergonomic pipeline and task definitions

#[macro_export]
macro_rules! define_pipeline {
    (
        $pipeline_id:ident, $pipeline_name:expr, {
            $( $task_id:ident: {
                name: $task_name:expr,
                $( description: $task_desc:expr, )?
                $( dependencies: [ $( $dep:ident ),* $(,)? ], )?
                $( timeout: $timeout:expr, )?
                $( retry_count: $retry:expr, )?
                exec: $exec_fn:expr $(,)?
            } ),* $(,)?
        }
    ) => {{
        let mut pipeline = $crate::Pipeline::new(stringify!($pipeline_id), $pipeline_name);
        $(
            let mut deps = std::collections::HashSet::new();
            $( $( deps.insert(stringify!($dep).to_string()); )* )?
            let mut task = $crate::Task::new(stringify!($task_id).to_string(), $task_name.to_string())
                $(.with_description($task_desc.to_string()))?
                $(.with_timeout($timeout))?
                $(.with_retry_count($retry))?;
                (task as $crate::runner::Runnable).with_execution_fn($exec_fn);
            pipeline.add_task(task).unwrap();
        )*
        pipeline
    }};
}

#[cfg(test)]
mod tests {
    use crate::task::TaskStatus;

    fn dummy_exec() -> fn() -> () {
        || ()
    }

    #[test]
    fn test_define_pipeline_macro_basic() {
        let pipeline = define_pipeline!(
            test_pipeline, "Test Pipeline", {
                task1: {
                    name: "Task 1",
                    exec: dummy_exec(),
                },
                task2: {
                    name: "Task 2",
                    dependencies: [task1],
                    exec: dummy_exec(),
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
