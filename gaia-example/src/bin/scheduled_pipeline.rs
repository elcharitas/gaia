use std::time::Duration;

use gaia_core::{Executor, Result, pipeline};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Gaia Scheduled Pipeline Example ===");
    println!("This example demonstrates a pipeline with a cron schedule");

    let pipeline = pipeline!(
        scheduled_pipeline, "Scheduled Pipeline", schedule: "0 0 * * *" => {
            daily_task: {
                name: "Daily Task",
                description: "A task that runs daily at midnight",
                timeout: Duration::from_secs(10),
                handler: async |_| {
                    println!("🕛 Running daily task at midnight...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("✅ Daily task completed");
                    Ok(())
                },
            },
            report: {
                name: "Generate Report",
                description: "Generate a report after the daily task",
                dependencies: [daily_task],
                timeout: Duration::from_secs(5),
                handler: async |_| {
                    println!("📊 Generating daily report...");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    println!("✅ Report generated");
                    Ok(())
                },
            },
        }
    );

    println!(
        "\n📅 Pipeline Schedule: {}",
        pipeline.schedule.as_ref().unwrap()
    );
    println!("\n🚀 Executing pipeline: {}", pipeline.name);

    let executor = Executor::new();
    let monitor = executor.execute_pipeline(pipeline).await?;

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
    println!("\n📝 Note: In a real application, you would use the cron schedule");
    println!("   to determine when to execute the pipeline instead of running it immediately.");
    Ok(())
}
