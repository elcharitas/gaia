//! Example web crawler pipeline using Gaia

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gaia_core::Result;
use gaia_core::error::GaiaError;
use gaia_core::executor::{Executor, ExecutorConfig};
use gaia_core::monitoring::Monitor;
use gaia_core::pipeline::Pipeline;
use gaia_core::task::Task;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("=== Gaia Web Crawler Pipeline Example ===");
    println!("This example demonstrates a web crawler pipeline with multiple dependent tasks");

    // Create a new pipeline
    let mut pipeline = Pipeline::new("web-crawler", "Web Crawler Pipeline")
        .with_description("A pipeline that crawls websites, extracts data, and generates reports");

    // Create discover URLs task
    let discover_task = Task::new("discover", "Discover URLs")
        .with_description("Discover URLs to crawl")
        .with_timeout(Duration::from_secs(10))
        .with_execution_fn(|| async {
            println!("🔍 Discovering URLs to crawl...");
            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("✅ Discovered 5 URLs to crawl");
            Ok(())
        });

    // Add discover task to pipeline
    pipeline.add_task(discover_task)?;

    // Create fetch task with dependency on discover
    let mut fetch_task = Task::new("fetch", "Fetch Content")
        .with_description("Fetch content from discovered URLs")
        .with_timeout(Duration::from_secs(20))
        .with_retry_count(3)
        .with_execution_fn(|| async {
            println!("📥 Fetching content from URLs...");
            // Simulate work with potential network issues
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Simulate occasional network failure
            if rand::random::<f32>() < 0.3 {
                // 30% chance of failure
                println!("⚠️ Network error while fetching, will retry...");
                return Err(GaiaError::TaskExecutionFailed(
                    "Network connection error".to_string(),
                ));
            }

            println!("✅ Successfully fetched content from all URLs");
            Ok(())
        });
    // Add dependency on discover task
    fetch_task.add_dependency("discover");
    // Add fetch task to pipeline
    pipeline.add_task(fetch_task)?;

    // Create parse task with dependency on fetch
    let mut parse_task = Task::new("parse", "Parse Content")
        .with_description("Parse fetched content")
        .with_timeout(Duration::from_secs(15))
        .with_execution_fn(|| async {
            println!("🔄 Parsing content...");
            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("✅ Content parsed successfully");
            Ok(())
        });
    // Add dependency on fetch task
    parse_task.add_dependency("fetch");

    // Add parse task to pipeline
    pipeline.add_task(parse_task)?;

    // Create extract data task with dependency on parse
    let mut extract_task = Task::new("extract", "Extract Data")
        .with_description("Extract structured data from parsed content")
        .with_timeout(Duration::from_secs(10))
        .with_execution_fn(|| async {
            println!("📊 Extracting structured data...");
            // Simulate work
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("✅ Data extracted successfully");
            Ok(())
        });

    // Add dependency on parse task
    extract_task.add_dependency("parse");
    // Add extract task to pipeline
    pipeline.add_task(extract_task)?;

    // Create report task with dependency on extract
    let mut report_task = Task::new("report", "Generate Report")
        .with_description("Generate report from extracted data")
        .with_timeout(Duration::from_secs(5))
        .with_execution_fn(|| async {
            println!("📝 Generating report...");
            // Simulate work
            tokio::time::sleep(Duration::from_millis(800)).await;
            println!("✅ Report generated successfully");
            Ok(())
        });
    // Add dependency on extract task
    report_task.add_dependency("extract");

    // Add report task to pipeline
    pipeline.add_task(report_task)?;

    // Validate the pipeline
    pipeline.validate()?;

    // Create executor with custom config
    let executor_config = ExecutorConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_tasks: 2,
        continue_on_failure: true, // Continue even if some tasks fail
    };
    let executor = Executor::with_config(executor_config);

    // Execute the pipeline
    println!("\n🚀 Executing pipeline: {}", pipeline.name);
    let pipeline_arc = Arc::new(Mutex::new(pipeline));
    executor.execute_pipeline(pipeline_arc.clone()).await?;

    // Create a monitor and collect metrics
    let mut monitor = Monitor::new();
    monitor.collect_metrics(&pipeline_arc.lock().unwrap())?;

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
