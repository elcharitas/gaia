//! Gaia UI - Dioxus frontend for Gaia pipeline management and monitoring

use dioxus::prelude::*;
// use dioxus_free_icons::icons::bs_icons::{BsArrowRepeat, BsPauseFill, BsPlayFill, BsStopFill};
use dioxus_router::prelude::*;
mod server_functions;
use server_functions::{get_metrics, get_pipeline_details, list_pipelines};
mod components;

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
    let pipelines = use_resource(|| async move { list_pipelines().await.unwrap_or_default() });
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
                    for pipeline in pipelines.read().clone().unwrap().iter() {
                        tr { key: "{pipeline.id}", class: "border-b border-gray-200 hover:bg-gray-50",
                            td { class: "py-3 px-4", "{pipeline.id}" }
                            td { class: "py-3 px-4", "{pipeline.name}" }
                            td { class: "py-3 px-4",
                                span { class: "px-2 py-1 rounded-full text-xs", "{pipeline.status}" }
                            }
                            td { class: "py-3 px-4",
                                div { class: "w-full bg-gray-200 rounded-full h-2.5",
                                    div { class: "bg-blue-600 h-2.5 rounded-full", style: "width: {pipeline.progress}" }
                                }
                            }
                            td { class: "py-3 px-4",
                                Link { to: Route::PipelineDetails { id: pipeline.id.clone() }, class: "text-blue-600 hover:underline", "View Details" }
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
    let details = use_resource(move || {
        let value = id.clone();
        async move { get_pipeline_details(value.clone()).await.ok() }
    });
    rsx! {
        div { class: "container mx-auto p-4",
            Link { to: Route::Pipelines {}, class: "text-blue-600 hover:underline mr-4", "← Back to Pipelines" }
            if let Some((pipeline, tasks)) = details.read().as_ref().and_then(|d| d.as_ref()) {
                h1 { class: "text-3xl font-bold", "{pipeline.name}" }
                div { class: "bg-white rounded-lg shadow-md p-6 mb-6",
                    h2 { class: "text-xl font-bold mb-4", "Pipeline Information" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { p { class: "text-gray-600", "ID:" } p { class: "font-semibold", "{pipeline.id}" } }
                        div { p { class: "text-gray-600", "Status:" } span { class: "px-2 py-1 rounded-full text-xs bg-green-100 text-green-800", "{pipeline.status}" } }
                        div { p { class: "text-gray-600", "Progress:" } p { class: "font-semibold", "{pipeline.progress}" } }
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
                                }
                            }
                            tbody {
                                for task in tasks.iter() {
                                    tr { key: "{task.id}", class: "border-b border-gray-200 hover:bg-gray-50",
                                        td { class: "py-3 px-4", "{task.id}" }
                                        td { class: "py-3 px-4", "{task.name}" }
                                        td { class: "py-3 px-4", "{task.status}" }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                h1 { class: "text-3xl font-bold", "Loading..." }
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
    let monitor = use_resource(|| async move { get_metrics().await.unwrap_or_default() });
    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-6", "Pipeline Metrics" }
            // MetricsVisualization { monitor: monitor.read().clone().unwrap() }
            // MetricsTable { monitor: monitor.read().clone().unwrap() }
        }
    }
}

fn main() {
    // Initialize logger
    env_logger::init();
    dioxus::launch(app);
}
