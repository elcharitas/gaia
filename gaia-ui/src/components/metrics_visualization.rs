//! Metrics visualization components for pipeline monitoring

use dioxus::prelude::*;
// use dioxus_charts::{BarChart, LineChart, PieChart};
use gaia_core::monitoring::{Metric, Monitor};
use std::collections::HashMap;

#[component]
pub fn MetricsVisualization(monitor: Monitor) -> Element {
    let metrics = monitor.get_metrics();

    // Group metrics by type
    let mut task_duration_metrics = Vec::new();
    let mut task_status_metrics = Vec::new();
    let mut pipeline_metrics = Vec::new();

    for metric in metrics {
        match metric.name.as_str() {
            "task.duration" => task_duration_metrics.push(metric.clone()),
            "task.status" => task_status_metrics.push(metric.clone()),
            _ if metric.name.starts_with("pipeline.") => pipeline_metrics.push(metric.clone()),
            _ => {} // Ignore other metrics for now
        }
    }

    rsx! {
        div { class: "metrics-container space-y-8",
            // Pipeline Duration Chart
            if !pipeline_metrics.is_empty() {
                div { class: "bg-white rounded-lg shadow-md p-6",
                    h3 { class: "text-xl font-bold mb-4", "Pipeline Metrics" }
                    PipelineMetricsChart { metrics: pipeline_metrics }
                }
            }

            // Task Duration Chart
            if !task_duration_metrics.is_empty() {
                div { class: "bg-white rounded-lg shadow-md p-6",
                    h3 { class: "text-xl font-bold mb-4", "Task Duration" }
                    TaskDurationChart { metrics: task_duration_metrics }
                }
            }

            // Task Status Chart
            if !task_status_metrics.is_empty() {
                div { class: "bg-white rounded-lg shadow-md p-6",
                    h3 { class: "text-xl font-bold mb-4", "Task Status Distribution" }
                    TaskStatusChart { metrics: task_status_metrics }
                }
            }
        }
    }
}

/// Component to visualize pipeline-level metrics
#[component]
fn PipelineMetricsChart(metrics: Vec<Metric>) -> Element {
    // Extract data for the chart
    let labels = metrics
        .iter()
        .map(|m| {
            // Get pipeline name from labels
            m.labels
                .iter()
                .find(|(k, _)| k == "pipeline_name")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        })
        .collect::<Vec<_>>();

    let values = metrics.iter().map(|m| m.value).collect::<Vec<_>>();

    rsx! {
        div { class: "h-64",
            // BarChart {
            //     labels: labels,
            //     series: values,
            //     dataset_labels: vec!["Duration (seconds)".to_string()],
            //     colors: vec!["#3b82f6".to_string()], // blue-500
            //     x_axis_title: "Pipeline".to_string(),
            //     y_axis_title: "Duration (s)".to_string(),
            // }
        }
    }
}

/// Component to visualize task duration metrics
#[component]
fn TaskDurationChart(metrics: Vec<Metric>) -> Element {
    // Extract data for the chart
    let mut task_names = Vec::new();
    let mut durations = Vec::new();

    for metric in &metrics {
        let task_name = metric
            .labels
            .iter()
            .find(|(k, _)| k == "task_name")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        task_names.push(task_name);
        durations.push(metric.value);
    }

    rsx! {
        div { class: "h-64",
            // BarChart {
            //     labels: task_names,
            //     data: durations,
            //     dataset_labels: vec!["Duration (seconds)".to_string()],
            //     colors: vec!["#10b981".to_string()], // emerald-500
            //     x_axis_title: "Task".to_string(),
            //     y_axis_title: "Duration (s)".to_string(),
            // }
        }
    }
}

/// Component to visualize task status distribution
#[component]
fn TaskStatusChart(metrics: Vec<Metric>) -> Element {
    // Count tasks by status
    let mut status_counts = HashMap::new();

    for metric in &metrics {
        let status_value = metric.value as i32;
        let status_name = match status_value {
            0 => "Pending",
            1 => "Running",
            2 => "Completed",
            3 => "Failed",
            4 => "Cancelled",
            _ => "Unknown",
        };

        *status_counts.entry(status_name).or_insert(0) += 1;
    }

    let labels = status_counts
        .keys()
        .cloned()
        .map(String::from)
        .collect::<Vec<_>>();
    let values = status_counts
        .values()
        .cloned()
        .map(|v| v as f64)
        .collect::<Vec<_>>();
    let colors = vec![
        "#94a3b8".to_string(), // slate-400 (Pending)
        "#3b82f6".to_string(), // blue-500 (Running)
        "#10b981".to_string(), // emerald-500 (Completed)
        "#ef4444".to_string(), // red-500 (Failed)
        "#f59e0b".to_string(), // amber-500 (Cancelled)
    ];

    rsx! {
        div { class: "h-64 flex justify-center",
            // PieChart {
            //     labels: labels,
            //     data: values,
            //     colors: colors,
            // }
        }
    }
}

/// Component to display a table of all metrics
#[component]
pub fn MetricsTable(monitor: Monitor) -> Element {
    let metrics = monitor.get_metrics();

    rsx! {
        div { class: "overflow-x-auto",
            table { class: "min-w-full bg-white rounded-lg overflow-hidden",
                thead { class: "bg-gray-100",
                    tr {
                        th { class: "py-3 px-4 text-left", "Metric" }
                        th { class: "py-3 px-4 text-left", "Value" }
                        th { class: "py-3 px-4 text-left", "Labels" }
                        th { class: "py-3 px-4 text-left", "Timestamp" }
                    }
                }
                tbody {
                    for metric in metrics.iter() {
                        {
                            let labels_str = metric.labels.iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ");

                            rsx! {
                                tr { key: "{metric.name}-{labels_str}", class: "border-b border-gray-200 hover:bg-gray-50",
                                    td { class: "py-3 px-4 font-medium", "{metric.name}" }
                                    td { class: "py-3 px-4", "{metric.value}" }
                                    td { class: "py-3 px-4 text-sm", "{labels_str}" }
                                    td { class: "py-3 px-4 text-sm text-gray-500", "Just now" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
