//! Pipeline graph visualization component

use dioxus::prelude::*;
use gaia_core::task::Task;
use gaia_core::{pipeline::Pipeline, task::TaskStatus};
use std::collections::{HashMap, HashSet};

/// Component to visualize pipeline tasks and their dependencies as a graph
#[component]
pub fn PipelineGraph(pipeline: Pipeline) -> Element {
    // Calculate positions for each task node based on dependencies
    let task_positions = calculate_task_positions(&pipeline);

    let width = 800;
    let height = 600;

    let paths = pipeline
        .tasks
        .iter()
        .flat_map(|(task_id, task)| {
            let deps = task
                .dependencies
                .iter()
                .filter_map(|dep_id| {
                    if let (Some(from_pos), Some(to_pos)) =
                        (task_positions.get(dep_id), task_positions.get(task_id))
                    {
                        let (from_x, from_y) = (from_pos.0 + 75, from_pos.1 + 30); // Center of the from node
                        let (to_x, to_y) = (to_pos.0, to_pos.1 + 30); // Left edge of the to node
                        Some((task_id, dep_id, from_x, from_y, to_x, to_y))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            deps
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "pipeline-graph-container relative", style: "width: {width}px; height: {height}px;",
            // Draw edges (connections) between tasks first so they appear behind nodes
            svg {
                width: "{width}",
                height: "{height}",
                class: "absolute top-0 left-0 w-full h-full",
                // Draw the edges between tasks
                for (task_id, dep_id, from_x, from_y, to_x, to_y) in paths {
                    path {
                        key: "{dep_id}-{task_id}",
                        d: "M{from_x},{from_y} C{(from_x + to_x) / 2},{from_y} {(from_x + to_x) / 2},{to_y} {to_x},{to_y}",
                        stroke: "#94a3b8", // slate-400
                        stroke_width: "2",
                        fill: "none",
                        marker_end: "url(#arrowhead)"
                    }
                },
                // Define the arrowhead marker
                defs {
                    marker {
                        id: "arrowhead",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "0",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon { points: "0 0, 10 3.5, 0 7", fill: "#94a3b8" }
                    }
                }
            }

            // Draw the task nodes
            for (task_id, task) in pipeline.tasks.iter() {
                if let Some((x, y)) = task_positions.get(task_id) {
                    {
                        let status_class = match task.status {
                            TaskStatus::Pending => "bg-gray-100 border-gray-300",
                            TaskStatus::Running => "bg-blue-100 border-blue-300",
                            TaskStatus::Completed => "bg-green-100 border-green-300",
                            TaskStatus::Failed => "bg-red-100 border-red-300",
                            TaskStatus::Cancelled => "bg-yellow-100 border-yellow-300",
                            TaskStatus::TimedOut => "bg-orange-100 border-orange-300",
                            TaskStatus::Skipped => "bg-purple-100 border-purple-300",
                        };

                        rsx! {
                            div {
                                key: "{task_id}",
                                class: "absolute p-3 rounded-lg shadow-md border-2 {status_class} w-40",
                                style: "left: {x}px; top: {y}px;",
                                div { class: "font-bold text-sm truncate", "{task.name}" }
                                div { class: "text-xs text-gray-600 truncate", "{task_id}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Calculate positions for each task in the pipeline based on dependencies
fn calculate_task_positions(pipeline: &Pipeline) -> HashMap<String, (i32, i32)> {
    let mut positions = HashMap::new();
    let mut levels = HashMap::new();

    // Determine the level of each task based on its dependencies
    for (task_id, _task) in &pipeline.tasks {
        calculate_task_level(task_id, &pipeline.tasks, &mut levels, &mut HashSet::new());
    }

    // Group tasks by level
    let mut tasks_by_level: HashMap<i32, Vec<&String>> = HashMap::new();
    for (task_id, level) in &levels {
        tasks_by_level.entry(*level).or_default().push(task_id);
    }

    // Calculate positions based on levels
    let level_spacing = 200; // Horizontal spacing between levels
    let task_spacing = 120; // Vertical spacing between tasks at the same level

    let max_level = levels.values().copied().max().unwrap_or(0);

    for level in 0..=max_level {
        if let Some(tasks) = tasks_by_level.get(&level) {
            let level_x = level * level_spacing;
            let total_height = tasks.len() as i32 * task_spacing;
            let start_y = (600 - total_height) / 2; // Center vertically in the 600px height

            for (i, task_id) in tasks.iter().enumerate() {
                let y = start_y + (i as i32 * task_spacing);
                positions.insert((*task_id).clone(), (level_x, y));
            }
        }
    }

    positions
}

/// Recursively calculate the level of a task based on its dependencies
fn calculate_task_level(
    task_id: &str,
    tasks: &HashMap<String, Task>,
    levels: &mut HashMap<String, i32>,
    visited: &mut HashSet<String>,
) -> i32 {
    // If we've already calculated this task's level, return it
    if let Some(level) = levels.get(task_id) {
        return *level;
    }

    // Mark as visited to detect cycles
    visited.insert(task_id.to_string());

    // Get the task
    let task = match tasks.get(task_id) {
        Some(t) => t,
        None => return 0, // Task not found
    };

    // If no dependencies, level is 0
    if task.dependencies.is_empty() {
        levels.insert(task_id.to_string(), 0);
        return 0;
    }

    // Calculate the maximum level of dependencies
    let mut max_dep_level = -1;
    for dep_id in &task.dependencies {
        // Avoid cycles
        if visited.contains(dep_id) {
            continue;
        }

        let dep_level = calculate_task_level(dep_id, tasks, levels, visited);
        max_dep_level = max_dep_level.max(dep_level);
    }

    // This task's level is one more than the maximum level of its dependencies
    let level = max_dep_level + 1;
    levels.insert(task_id.to_string(), level);

    level
}
