//! Example data processing pipeline using Gaia

use std::time::Duration;

use gaia_core::Result;
use gaia_core::error::GaiaError;
use gaia_core::executor::Executor;
use gaia_core::pipeline;
use gaia_core::task::TaskStatus;

use rand::Rng;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Gaia Data Processing Pipeline Example ===");
    println!("This example demonstrates a data pipeline with multiple dependent tasks");

    let pipeline = pipeline! {
        data_processing, "Data Processing Pipeline" => {
            extract: {
                name: "Extract Data",
                // description: "Extract data from source",
                timeout: Duration::from_secs(10),
                retry_count: 3,
                handler: async |_| {
                    println!("🔍 Extracting data from source...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("✅ Data extraction complete");
                    Ok(())
                },
            },
            transform: {
                name: "Transform Data",
                description: "Transform extracted data",
                dependencies: [extract],
                timeout: Duration::from_secs(15),
                handler: async move |_| {
                    println!("🔄 Transforming data...");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.2) {
                        println!("❌ Data transformation failed, will retry...");
                        return Err(GaiaError::TaskExecutionFailed(
                            "Random transformation failure".to_string(),
                        ));
                    }
                    println!("✅ Data transformation complete");
                    Ok(())
                },
            },
            load: {
                name: "Load Data",
                description: "Load transformed data to destination",
                dependencies: [transform],
                timeout: Duration::from_secs(10),
                handler: async |context| {
                    match context.task_status("transform") {
                        Some(TaskStatus::Completed(_)) => {
                            println!("📥 Loading data to destination...");
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            println!("✅ Data loading complete");
                        }
                        _=>{}
                    }
                    Ok(())
                },
            },
            validate: {
                name: "Validate Data",
                description: "Validate loaded data",
                dependencies: [load],
                timeout: Duration::from_secs(5),
                handler: async |_| {
                    println!("✓ Validating loaded data...");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    println!("✅ Data validation complete");
                    Ok(())
                },
            },
        }
    };

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
    Ok(())
}
