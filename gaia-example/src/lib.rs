//! Shared utilities for Gaia examples

use std::time::Duration;
use std::sync::{Arc, Mutex};

use gaia_core::monitoring::Monitor;
use gaia_core::pipeline::Pipeline;
use gaia_core::Result;

/// Prints a formatted header for an example
pub fn print_example_header(title: &str, description: &str) {
    println!("=== Gaia {} Example ===", title);
    println!("{}", description);
}

/// Prints metrics collected from a pipeline execution
pub fn print_metrics(pipeline: &Arc<Mutex<Pipeline>>) -> Result<()> {
    // Create a monitor and collect metrics
    let mut monitor = Monitor::new();
    monitor.collect_metrics(&pipeline.lock().unwrap())?;
    
    // Display metrics
    println!("\n📊 Pipeline Metrics:");
    for metric in monitor.get_metrics() {
        let labels = metric.labels.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {} = {} ({})", metric.name, metric.value, labels);
    }
    
    println!("\n✨ Pipeline execution completed successfully!");
    Ok(())
}

/// Simulates work with a random delay between min and max duration
pub async fn simulate_work(task_name: &str, min_ms: u64, max_ms: u64) {
    let duration = Duration::from_millis(
        min_ms + rand::random::<u64>() % (max_ms - min_ms)
    );
    println!("⏳ {} working for {} ms...", task_name, duration.as_millis());
    tokio::time::sleep(duration).await;
}