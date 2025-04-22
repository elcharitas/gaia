//! Gaia UI - Dioxus frontend for Gaia pipeline management and monitoring

use dioxus::prelude::*;
// use dioxus_free_icons::icons::bs_icons::{BsArrowRepeat, BsPauseFill, BsPlayFill, BsStopFill};
use dioxus_router::prelude::*;
use gaia_core::task::TaskStatus;
use gaia_core::{Pipeline, Task, monitoring::Monitor};

// Import custom components
mod components;
use components::{MetricsTable, MetricsVisualization, Navigation, PipelineGraph};

// Define the main routes for the application
#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/pipelines")]
    Pipelines {},
    #[route("/pipelines/:id")]
    PipelineDetails { id: String },
    #[route("/tasks")]
    Tasks {},
    #[route("/tasks/:id")]
    TaskDetails { id: String },
    #[route("/metrics")]
    Metrics {},
}

// Main application component
fn app() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50",

            // Main content area
            main {
                class: "py-4",
                Router::<Route> {}
            }
        }
    }
}

// Home page component
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Gaia Pipeline Manager" }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                DashboardCard {
                    title: "Active Pipelines",
                    value: "3",
                    icon: rsx! { },
                    color: "bg-green-100 text-green-800"
                }
                DashboardCard {
                    title: "Completed Pipelines",
                    value: "12",
                    icon: rsx! { },
                    color: "bg-blue-100 text-blue-800"
                }
                DashboardCard {
                    title: "Failed Pipelines",
                    value: "2",
                    icon: rsx! { },
                    color: "bg-red-100 text-red-800"
                }
            }
            div { class: "mt-8",
                h2 { class: "text-2xl font-bold mb-4", "Recent Pipelines" }
                PipelineList {}
            }
        }
    }
}

// Dashboard card component for metrics
#[component]
fn DashboardCard(
    title: &'static str,
    value: &'static str,
    icon: Element,
    color: &'static str,
) -> Element {
    rsx! {
        div { class: "p-6 rounded-lg shadow-md {color}",
            div { class: "flex justify-between items-center",
                div {
                    h3 { class: "text-lg font-semibold", "{title}" }
                    p { class: "text-3xl font-bold mt-2", "{value}" }
                }
                div { class: "text-2xl" }
            }
        }
    }
}

// Pipeline list component
#[component]
fn PipelineList() -> Element {
    // In a real app, this would fetch data from the backend
    let pipelines = vec![
        (
            "data-processing",
            "Data Processing Pipeline",
            "Running",
            "90%",
        ),
        ("web-crawler", "Web Crawler Pipeline", "Completed", "100%"),
        ("etl-pipeline", "ETL Pipeline", "Failed", "65%"),
    ];

    rsx! {
        div { class: "overflow-x-auto",
            table { class: "min-w-full bg-white rounded-lg overflow-hidden",
                thead { class: "bg-gray-100",
                    tr {
                        th { class: "py-3 px-4 text-left", "ID" }
                        th { class: "py-3 px-4 text-left", "Name" }
                        th { class: "py-3 px-4 text-left", "Status" }
                        th { class: "py-3 px-4 text-left", "Progress" }
                        th { class: "py-3 px-4 text-left", "Actions" }
                    }
                }
                tbody {
                    for (id, name, status, progress) in pipelines.iter() {
                        tr { key: "{id}", class: "border-b border-gray-200 hover:bg-gray-50",
                            td { class: "py-3 px-4", "{id}" }
                            td { class: "py-3 px-4", "{name}" }
                            td { class: "py-3 px-4",
                                span { class: "px-2 py-1 rounded-full text-xs", "{status}" }
                            }
                            td { class: "py-3 px-4",
                                div { class: "w-full bg-gray-200 rounded-full h-2.5",
                                    div { class: "bg-blue-600 h-2.5 rounded-full", style: "width: {progress}" }
                                }
                            }
                            td { class: "py-3 px-4",
                                Link { to: Route::PipelineDetails { id: id.to_string() }, class: "text-blue-600 hover:underline", "View Details" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Pipeline details component
#[component]
fn PipelineDetails(id: String) -> Element {
    // In a real app, this would fetch the specific pipeline data
    let pipeline_name = format!("Pipeline {}", id);
    let tasks = vec![
        ("extract", "Extract Data", TaskStatus::Completed),
        ("transform", "Transform Data", TaskStatus::Running),
        ("load", "Load Data", TaskStatus::Pending),
        ("validate", "Validate Data", TaskStatus::Pending),
    ];

    rsx! {
        div { class: "container mx-auto p-4",
            div { class: "flex items-center mb-6",
                Link { to: Route::Pipelines {}, class: "text-blue-600 hover:underline mr-4", "← Back to Pipelines" }
                h1 { class: "text-3xl font-bold", "{pipeline_name}" }
            }

            div { class: "bg-white rounded-lg shadow-md p-6 mb-6",
                h2 { class: "text-xl font-bold mb-4", "Pipeline Information" }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        p { class: "text-gray-600", "ID:" }
                        p { class: "font-semibold", "{id}" }
                    }
                    div {
                        p { class: "text-gray-600", "Status:" }
                        span { class: "px-2 py-1 rounded-full text-xs bg-green-100 text-green-800", "Running" }
                    }
                    div {
                        p { class: "text-gray-600", "Started:" }
                        p { class: "font-semibold", "2023-06-15 14:30:25" }
                    }
                    div {
                        p { class: "text-gray-600", "Duration:" }
                        p { class: "font-semibold", "00:15:32" }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow-md p-6 mb-6",
                h2 { class: "text-xl font-bold mb-4", "Tasks" }
                div { class: "overflow-x-auto",
                    table { class: "min-w-full",
                        thead { class: "bg-gray-100",
                            tr {
                                th { class: "py-3 px-4 text-left", "ID" }
                                th { class: "py-3 px-4 text-left", "Name" }
                                th { class: "py-3 px-4 text-left", "Status" }
                                th { class: "py-3 px-4 text-left", "Dependencies" }
                                th { class: "py-3 px-4 text-left", "Duration" }
                            }
                        }
                        tbody {
                            for (id, name, status) in tasks.iter() {
                                tr { key: "{id}", class: "border-b border-gray-200",
                                    td { class: "py-3 px-4", "{id}" }
                                    td { class: "py-3 px-4", "{name}" }
                                    td { class: "py-3 px-4",
                                        {
                                            let (status_text, status_class) = match status {
                                                TaskStatus::Pending => ("Pending", "bg-gray-100 text-gray-800"),
                                                TaskStatus::Running => ("Running", "bg-green-100 text-green-800"),
                                                TaskStatus::Completed => ("Completed", "bg-blue-100 text-blue-800"),
                                                TaskStatus::Failed => ("Failed", "bg-red-100 text-red-800"),
                                                TaskStatus::Cancelled => ("Cancelled", "bg-yellow-100 text-yellow-800"),
                                            };
                                            rsx! {
                                                span { class: "px-2 py-1 rounded-full text-xs {status_class}", "{status_text}" }
                                            }
                                        }
                                    }
                                    td { class: "py-3 px-4", "-" }
                                    td { class: "py-3 px-4", "00:05:12" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow-md p-6",
                h2 { class: "text-xl font-bold mb-4", "Pipeline Visualization" }
                div { class: "border border-gray-200 rounded-lg p-4 overflow-auto",
                    // Create a sample pipeline for visualization
                    // In a real app, this would use the actual pipeline data
                    {
                        let mut sample_pipeline = Pipeline::new(id.clone(), pipeline_name.clone());

                        // Add the same tasks we're displaying in the table
                        let extract_task = Task::new("extract", "Extract Data");
                        sample_pipeline.add_task(extract_task).unwrap_or_default();

                        let mut transform_task = Task::new("transform", "Transform Data");
                        transform_task.add_dependency("extract");
                        sample_pipeline.add_task(transform_task).unwrap_or_default();

                        let mut load_task = Task::new("load", "Load Data");
                        load_task.add_dependency("transform");
                        sample_pipeline.add_task(load_task).unwrap_or_default();

                        let mut validate_task = Task::new("validate", "Validate Data");
                        validate_task.add_dependency("load");
                        sample_pipeline.add_task(validate_task).unwrap_or_default();

                        // Update task statuses to match our display
                        if let Some(task) = sample_pipeline.tasks.get_mut("extract") {
                            task.status = TaskStatus::Completed;
                        }
                        if let Some(task) = sample_pipeline.tasks.get_mut("transform") {
                            task.status = TaskStatus::Running;
                        }

                        rsx! {
                            PipelineGraph { pipeline: sample_pipeline }
                        }
                    }
                }
            }
        }
    }
}

// Pipelines list page component
#[component]
fn Pipelines() -> Element {
    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Pipelines" }
            PipelineList {}
        }
    }
}

// Tasks list page component
#[component]
fn Tasks() -> Element {
    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Tasks" }
            // Task list would go here
        }
    }
}

// Task details component
#[component]
fn TaskDetails(id: String) -> Element {
    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Task Details: {id}" }
            // Task details would go here
        }
    }
}

// Metrics page component
#[component]
fn Metrics() -> Element {
    // In a real app, this would fetch metrics from actual pipelines
    // For now, we'll create a sample monitor with metrics
    let monitor = create_sample_monitor();

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Pipeline Metrics" }

            // Display metrics visualizations
            MetricsVisualization { monitor: monitor.clone() }

            // Display metrics table
            div { class: "mt-8 bg-white rounded-lg shadow-md p-6",
                h2 { class: "text-xl font-bold mb-4", "Metrics Details" }
                MetricsTable { monitor: monitor }
            }
        }
    }
}

// Create a sample monitor with metrics for demonstration
fn create_sample_monitor() -> Monitor {
    let mut monitor = Monitor::new();

    // Add some sample metrics
    // In a real app, these would come from actual pipeline execution

    // Pipeline duration metrics
    monitor.add_metric(
        "pipeline.duration",
        120.5, // 2 minutes
        vec![
            ("pipeline_id".to_string(), "data-processing".to_string()),
            (
                "pipeline_name".to_string(),
                "Data Processing Pipeline".to_string(),
            ),
        ],
    );

    monitor.add_metric(
        "pipeline.duration",
        45.2, // 45 seconds
        vec![
            ("pipeline_id".to_string(), "web-crawler".to_string()),
            (
                "pipeline_name".to_string(),
                "Web Crawler Pipeline".to_string(),
            ),
        ],
    );

    // Task duration metrics
    for (task_id, task_name, duration) in [
        ("extract", "Extract Data", 15.3),
        ("transform", "Transform Data", 42.1),
        ("load", "Load Data", 8.7),
        ("validate", "Validate Data", 5.2),
    ] {
        monitor.add_metric(
            "task.duration",
            duration,
            vec![
                ("task_id".to_string(), task_id.to_string()),
                ("task_name".to_string(), task_name.to_string()),
                ("pipeline_id".to_string(), "data-processing".to_string()),
            ],
        );
    }

    // Task status metrics
    for (task_id, task_name, status) in [
        ("extract", "Extract Data", 2.0),     // Completed
        ("transform", "Transform Data", 1.0), // Running
        ("load", "Load Data", 0.0),           // Pending
        ("validate", "Validate Data", 0.0),   // Pending
    ] {
        monitor.add_metric(
            "task.status",
            status,
            vec![
                ("task_id".to_string(), task_id.to_string()),
                ("task_name".to_string(), task_name.to_string()),
                ("pipeline_id".to_string(), "data-processing".to_string()),
            ],
        );
    }

    monitor
}

fn main() {
    // Initialize logger
    env_logger::init();
    dioxus::launch(app);
}
