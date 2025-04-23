//! Example web crawler pipeline using Gaia

use std::time::Duration;

use gaia_core::Result;
use gaia_core::define_pipeline;
use gaia_core::error::GaiaError;
use gaia_core::executor::Executor;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Gaia Web Crawler Pipeline Example ===");
    println!("This example demonstrates a web crawler pipeline with multiple dependent tasks");

    let base_pipeline = define_pipeline!(
        web_crawler, "Web Crawler Pipeline", {
            discover: {
                name: "Discover URLs",
                description: "Discover URLs to crawl",
                timeout: Duration::from_secs(10),
                handler: async |_| {
                    println!("🔍 Discovering URLs to crawl...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("✅ Discovered 5 URLs to crawl");
                    Ok(())
                },
            },
            fetch: {
                name: "Fetch Content",
                description: "Fetch content from discovered URLs",
                dependencies: [discover],
                timeout: Duration::from_secs(20),
                retry_count: 3,
                handler: async |_| {
                    println!("📥 Fetching content from URLs...");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.3) {
                        println!("⚠️ Network error while fetching, will retry...");
                        return Err(GaiaError::TaskExecutionFailed(
                            "Network connection error".to_string(),
                        ));
                    }
                    println!("✅ Successfully fetched content from all URLs");
                    Ok(())
                },
            },
            parse: {
                name: "Parse Content",
                description: "Parse fetched content",
                dependencies: [fetch],
                timeout: Duration::from_secs(15),
                handler: async |_| {
                    println!("🔄 Parsing content...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("✅ Content parsed successfully");
                    Ok(())
                },
            },
            extract: {
                name: "Extract Data",
                description: "Extract structured data from parsed content",
                dependencies: [parse],
                timeout: Duration::from_secs(10),
                handler: async |_| {
                    println!("📊 Extracting structured data...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("✅ Data extracted successfully");
                    Ok(())
                },
            },
            report: {
                name: "Generate Report",
                description: "Generate report from extracted data",
                dependencies: [extract],
                timeout: Duration::from_secs(5),
                handler: async |_| {
                    println!("📝 Generating report...");
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    println!("✅ Report generated successfully");
                    Ok(())
                },
            },
        }
    );

    // Inherit from base_pipeline and add a new task
    let pipeline = define_pipeline!(
        web_crawler_extended : base_pipeline, "Web Crawler Pipeline (Extended)", {
            summarize: {
                name: "Summarize Results",
                description: "Summarize the crawling results",
                dependencies: [report],
                timeout: Duration::from_secs(3),
                handler: async |_| {
                    println!("📈 Summarizing crawling results...");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    println!("✅ Summary complete");
                    Ok(())
                },
            },
        }
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
    Ok(())
}
