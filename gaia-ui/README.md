# Gaia UI

A Dioxus frontend for Gaia pipeline management and monitoring.

## Overview

Gaia UI provides a visual interface for tracking and managing Gaia pipelines. It allows you to:

- View all pipelines and their current status
- Track individual task progress within pipelines
- Visualize task dependencies as a graph
- Monitor execution metrics and performance

## Features

- **Dashboard**: Overview of active, completed, and failed pipelines
- **Pipeline Visualization**: Interactive graph showing task dependencies
- **Task Tracking**: Real-time status updates for all tasks
- **Metrics**: Charts and graphs showing performance metrics

## Getting Started

### Prerequisites

- Rust and Cargo installed
- Gaia Core library

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/elcharitas/gaia.git
   cd gaia
   ```

2. Build the UI:
   ```bash
   cd gaia-ui
   cargo build
   ```

### Running the Application

To run the desktop application:

```bash
cargo run --bin gaia-ui
```

To build for web:

```bash
cargo run --bin gaia-ui --features web --no-default-features
```

## Project Structure

- `src/main.rs` - Main application entry point and routing
- `src/components/` - Reusable UI components
  - `pipeline_graph.rs` - Pipeline visualization component
  - `metrics_visualization.rs` - Charts and metrics display

## Integration with Gaia Core

Gaia UI integrates directly with the Gaia Core library to display and interact with pipelines. It uses the following core components:

- `Pipeline` - For pipeline structure and task relationships
- `Task` - For individual task information and status
- `Monitor` - For collecting and displaying metrics
- `Executor` - For controlling pipeline execution

## License

MIT
