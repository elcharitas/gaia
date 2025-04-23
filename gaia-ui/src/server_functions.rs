//! Server functions for Gaia UI (Dioxus Fullstack)

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineDto {
    pub id: String,
    pub name: String,
    pub status: String,
    pub progress: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskDto {
    pub id: String,
    pub name: String,
    pub status: String,
    pub dependencies: Vec<String>,
}

#[server]
pub async fn list_pipelines() -> Result<Vec<PipelineDto>, ServerFnError> {
    // TODO: Replace with real gaia_core logic
    Ok(vec![
        PipelineDto {
            id: "data-processing".into(),
            name: "Data Processing Pipeline".into(),
            status: "Running".into(),
            progress: "90%".into(),
        },
        PipelineDto {
            id: "web-crawler".into(),
            name: "Web Crawler Pipeline".into(),
            status: "Completed".into(),
            progress: "100%".into(),
        },
        PipelineDto {
            id: "etl-pipeline".into(),
            name: "ETL Pipeline".into(),
            status: "Failed".into(),
            progress: "65%".into(),
        },
    ])
}

#[server]
pub async fn get_pipeline_details(
    id: String,
) -> Result<(PipelineDto, Vec<TaskDto>), ServerFnError> {
    // TODO: Replace with real gaia_core logic
    let pipeline = PipelineDto {
        id: id.clone(),
        name: format!("Pipeline {}", id),
        status: "Running".into(),
        progress: "90%".into(),
    };
    let tasks = vec![
        TaskDto {
            id: "extract".into(),
            name: "Extract Data".into(),
            status: "Completed".into(),
            dependencies: vec![],
        },
        TaskDto {
            id: "transform".into(),
            name: "Transform Data".into(),
            status: "Running".into(),
            dependencies: vec!["extract".into()],
        },
        TaskDto {
            id: "load".into(),
            name: "Load Data".into(),
            status: "Pending".into(),
            dependencies: vec!["transform".into()],
        },
        TaskDto {
            id: "validate".into(),
            name: "Validate Data".into(),
            status: "Pending".into(),
            dependencies: vec!["load".into()],
        },
    ];
    Ok((pipeline, tasks))
}

#[server]
pub async fn get_metrics() -> Result<String, ServerFnError> {
    // TODO: Replace with real gaia_core logic
    // let mut monitor = Monitor::new();
    // ... add sample metrics as in create_sample_monitor ...
    Ok(String::new())
}
