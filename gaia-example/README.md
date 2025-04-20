# Gaia Example Crate

This example crate demonstrates the power and flexibility of Gaia, a Rust-based pipeline management system. It showcases how to define, configure, and execute complex pipelines with dependent tasks, error handling, and monitoring capabilities.

## Examples

This crate includes two example applications that demonstrate different use cases for Gaia:

### 1. Data Processing Pipeline

A data processing pipeline that demonstrates the Extract-Transform-Load (ETL) pattern with validation:

- **Extract**: Simulates extracting data from a source
- **Transform**: Processes the extracted data with retry capabilities
- **Load**: Loads the transformed data to a destination
- **Validate**: Validates the loaded data

Run with:

```bash
cargo run --bin data-processing
```

### 2. Web Crawler Pipeline

A web crawler pipeline that demonstrates handling network operations with dependencies:

- **Discover**: Finds URLs to crawl
- **Fetch**: Retrieves content from discovered URLs with retry logic
- **Parse**: Parses the fetched content
- **Extract**: Extracts structured data from parsed content
- **Report**: Generates a report from the extracted data

Run with:

```bash
cargo run --bin web-crawler
```

## Key Features Demonstrated

- **Task Dependencies**: Define complex task relationships and execution order
- **Error Handling**: Implement retry logic and failure recovery
- **Timeout Management**: Set execution time limits for tasks
- **Monitoring**: Collect and display metrics about pipeline execution
- **Concurrency Control**: Configure maximum concurrent task execution
- **Pipeline Validation**: Validate pipeline configuration before execution

## Implementation Details

Both examples showcase:

- Creating pipelines with multiple dependent tasks
- Defining custom execution functions for tasks
- Configuring task properties (timeouts, retry counts)
- Handling task failures and retries
- Collecting and displaying execution metrics

These examples demonstrate how Gaia can be used to manage complex workflows with clear dependency management, robust error handling, and comprehensive monitoring.
