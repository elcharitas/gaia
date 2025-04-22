//! Example data processing pipeline using Gaia

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gaia_core::Result;
use gaia_core::error::GaiaError;
use gaia_core::executor::{Executor, ExecutorConfig};
use gaia_core::pipeline::Pipeline;
use gaia_core::task::Task;

use rand::Rng;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("=== Gaia Data Processing Pipeline Example ===");
    println!("This example demonstrates a data pipeline with multiple dependent tasks");

    // Create a new pipeline
    let mut pipeline = Pipeline::new("data-processing", "Data Processing Pipeline")
        .with_description(
            "A pipeline that demonstrates data extraction, transformation, and loading",
        );

    // Create tasks with dependencies
    let extract_task = Task::new("extract", "Extract Data")
        .with_description("Extract data from source")
        .with_timeout(Duration::from_secs(10))
        .with_retry_count(3)
        .with_execution_fn(async || {
            println!("🔍 Extracting data from source...");
            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("✅ Data extraction complete");
            Ok(())
        });

    // Add extract task to pipeline
    pipeline.add_task(extract_task)?;

    // Create transform task with dependency on extract
    let mut transform_task = Task::new("transform", "Transform Data")
        .with_description("Transform extracted data")
        .with_timeout(Duration::from_secs(15))
        .with_execution_fn(async || {
            println!("🔄 Transforming data...");
            // Simulate work with random chance of failure
            tokio::time::sleep(Duration::from_secs(2)).await;

            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.2) {
                // 20% chance of failure
                println!("❌ Data transformation failed, will retry...");
                return Err(GaiaError::TaskExecutionFailed(
                    "Random transformation failure".to_string(),
                ));
            }

            println!("✅ Data transformation complete");
            Ok(())
        });

    // Add dependency on extract task
    transform_task.add_dependency("extract");

    // Add transform task to pipeline
    pipeline.add_task(transform_task)?;

    // Create load task with dependency on transform
    let mut load_task = Task::new("load", "Load Data")
        .with_description("Load transformed data to destination")
        .with_timeout(Duration::from_secs(10))
        .with_execution_fn(async || {
            println!("📥 Loading data to destination...");
            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("✅ Data loading complete");
            Ok(())
        });

    // Add dependency on transform task
    load_task.add_dependency("transform");

    // Add load task to pipeline
    pipeline.add_task(load_task)?;

    // Create validate task with dependency on load
    let mut validate_task = Task::new("validate", "Validate Data")
        .with_description("Validate loaded data")
        .with_timeout(Duration::from_secs(5))
        .with_execution_fn(async || {
            println!("✓ Validating loaded data...");
            // Simulate work
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("✅ Data validation complete");
            Ok(())
        });
    // Add dependency on load task
    validate_task.add_dependency("load");

    // Add validate task to pipeline
    pipeline.add_task(validate_task)?;

    // Validate the pipeline
    pipeline.validate()?;

    // Create executor with custom config
    let executor_config = ExecutorConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_tasks: 2,
        continue_on_failure: true,
    };
    let executor = Executor::with_config(executor_config);

    // Execute the pipeline
    println!("\n🚀 Executing pipeline: {}", pipeline.name);
    let pipeline_arc = Arc::new(Mutex::new(pipeline));
    let monitor = executor.execute_pipeline(pipeline_arc).await?;

    // Display metrics
    println!("\n📊 Pipeline Metrics:");
    for metric in monitor.get_metrics() {
        let labels = metric
            .labels
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {} = {} ({})", metric.name, metric.value, labels);
    }

    println!("\n✨ Pipeline execution completed successfully!");
    Ok(())
}
