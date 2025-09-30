# alfrusco

[![Crates.io](https://img.shields.io/crates/v/alfrusco.svg)](https://crates.io/crates/alfrusco)
[![Documentation](https://docs.rs/alfrusco/badge.svg)](https://docs.rs/alfrusco)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful, ergonomic Rust library for building [Alfred](https://www.alfredapp.com/) workflows with ease. Alfrusco
handles the complexity of Alfred's JSON protocol, provides rich item building capabilities, and includes advanced
features like background jobs, clipboard operations, and comprehensive logging.

## Features

- **Simple & Ergonomic API** - Intuitive builder patterns for creating Alfred items
- **Async Support** - Full async/await support for modern Rust applications
- **Background Jobs** - Run long-running tasks without blocking Alfred's UI
- **Clipboard Integration** - Built-in support for rich text and Markdown clipboard operations
- **Smart Filtering** - Automatic fuzzy search and sorting of results
- **Workflow Management** - Easy access to workflow directories and configuration
- **Comprehensive Logging** - Structured logging with file and console output
- **URL Items** - Specialized support for URL-based workflow items
- **Environment Handling** - Robust configuration management for Alfred environments
- **Testing Support** - Built-in testing utilities and mocking capabilities

## 📦 Installation

Add alfrusco to your `Cargo.toml`:

```toml
[dependencies]
alfrusco = "0.2"

# For async workflows
tokio = { version = "1", features = ["full"] }

# For command-line argument parsing (recommended)
clap = { version = "4", features = ["derive", "env"] }
```

## 🚀 Quick Start

### Basic Synchronous Workflow

```rust
use alfrusco::{execute, Item, Runnable, Workflow};
use alfrusco::config::AlfredEnvProvider;
use clap::Parser;

#[derive(Parser)]
struct MyWorkflow {
    query: Vec<String>,
}

impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let query = self.query.join(" ");

        workflow.append_item(
            Item::new(format!("Hello, {}!", query))
                .subtitle("This is a basic Alfred workflow")
                .arg(&query)
                .valid(true)
        );

        Ok(())
    }
}

fn main() {
    // Initialize logging (optional but recommended)
    let _ = alfrusco::init_logging(&AlfredEnvProvider);

    // Parse command line arguments and execute workflow
    let command = MyWorkflow::parse();
    execute(&AlfredEnvProvider, command, &mut std::io::stdout());
}
```

### Async Workflow with HTTP Requests

```rust
use alfrusco::{execute_async, AsyncRunnable, Item, Workflow, WorkflowError};
use alfrusco::config::AlfredEnvProvider;
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
struct ApiWorkflow {
    query: Vec<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    results: Vec<ApiResult>,
}

#[derive(Deserialize)]
struct ApiResult {
    title: String,
    description: String,
    url: String,
}

#[async_trait::async_trait]
impl AsyncRunnable for ApiWorkflow {
    type Error = Box<dyn WorkflowError>;

    async fn run_async(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let query = self.query.join(" ");
        workflow.set_filter_keyword(query.clone());

        let url = format!("https://api.example.com/search?q={}", query);
        let response: ApiResponse = reqwest::get(&url)
            .await?
            .json()
            .await?;

        let items: Vec<Item> = response.results
            .into_iter()
            .map(|result| {
                Item::new(&result.title)
                    .subtitle(&result.description)
                    .arg(&result.url)
                    .quicklook_url(&result.url)
                    .valid(true)
            })
            .collect();

        workflow.append_items(items);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let _ = alfrusco::init_logging(&AlfredEnvProvider);
    let command = ApiWorkflow::parse();
    execute_async(&AlfredEnvProvider, command, &mut std::io::stdout()).await;
}
```

## 🏗️ Core Concepts

### Items

Items are the building blocks of Alfred workflows. Each item represents a choice in the Alfred selection UI:

```rust
use alfrusco::Item;

let item = Item::new("My Title")
.subtitle("Additional information")
.arg("argument-passed-to-action")
.uid("unique-identifier")
.valid(true)
.icon_from_image("/path/to/icon.png")
.copy_text("Text copied with ⌘C")
.large_type_text("Text shown in large type with ⌘L")
.quicklook_url("https://example.com")
.var("CUSTOM_VAR", "value")
.autocomplete("text for tab completion");
```

### Workflow Configuration

Alfrusco automatically handles Alfred's environment variables through configuration providers:

```rust
use alfrusco::config::{AlfredEnvProvider, TestingProvider};

// For production (reads from Alfred environment variables)
let provider = AlfredEnvProvider;

// For testing (uses temporary directories)
let temp_dir = tempfile::tempdir().unwrap();
let provider = TestingProvider(temp_dir.path().to_path_buf());
```

### Error Handling

Implement custom error types that work seamlessly with Alfred:

```rust
use alfrusco::{WorkflowError, Item};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyWorkflowError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl WorkflowError for MyWorkflowError {}

// Errors automatically become Alfred items
impl Runnable for MyWorkflow {
    type Error = MyWorkflowError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        // If this returns an error, Alfred will show it as an item
        Err(MyWorkflowError::InvalidInput("Missing required field".to_string()))
    }
}
```

## 🔧 Advanced Features

### Background Jobs

Run long-running tasks without blocking Alfred's UI. This example fetches GitHub release data in the background and
caches it to disk, showing cached results immediately while refreshing stale data:

```rust
use std::process::Command;
use std::time::Duration;

impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let cache_file = workflow.cache_dir().join("releases.json");

        // Set up a command to fetch data and save to cache
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(format!(
                "curl -s https://api.github.com/repos/rust-lang/rust/releases/latest > {}",
                cache_file.display()
            ));

        // Run the command in the background, refresh every 30 seconds
        workflow.run_in_background(
            "github-releases",
            Duration::from_secs(30),
            cmd
        );

        // Check if we have cached data to display
        if cache_file.exists() {
            if let Ok(data) = std::fs::read_to_string(&cache_file) {
                if let Ok(release) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(tag) = release["tag_name"].as_str() {
                        workflow.append_item(
                            Item::new(format!("Latest Rust: {}", tag))
                                .subtitle("Click to view release notes")
                                .arg(release["html_url"].as_str().unwrap_or(""))
                                .valid(true)
                        );
                    }
                }
            }
        }

        // run_in_background automatically shows a status item when the job is stale

        Ok(())
    }
}
```

**Enhanced Background Job Features:**

- **Smart Status Tracking**: Jobs show detailed status messages like "Last succeeded 2 minutes ago (14:32:15), running
  for 3s" or "Last failed 5 minutes ago (14:29:42), running for 1s"
- **Automatic Retry Logic**: Failed jobs are automatically retried even if they ran recently, ensuring eventual success
- **Context-Aware Icons**: Visual indicators show job status at a glance:
    - ✅ Success jobs show completion icon
    - ❌ Failed jobs show error icon
    - 🔄 Retry attempts show sync icon
    - 🕐 First-time runs show clock icon
- **Secure Shell Escaping**: Arguments with spaces and special characters are properly escaped for security
- **Robust Last-Run Tracking**: All job executions are tracked regardless of success/failure for accurate status
  reporting

**Background Job Status Messages:**

When a background job is running, Alfred will display informative status items:

```
Background Job 'github-releases'
Last succeeded 2 minutes ago (14:32:15), running for 3s
```

This gives users clear visibility into:

- When the job last ran successfully or failed
- The exact time of the last execution
- How long the current execution has been running
- Visual context through appropriate icons

### URL Items with Rich Clipboard Support

Create URL items with automatic clipboard integration:

```rust
use alfrusco::URLItem;

let url_item = URLItem::new("Rust Documentation", "https://doc.rust-lang.org/")
    .subtitle("The Rust Programming Language Documentation")
    .short_title("Rust Docs")  // Used in Cmd+Shift modifier
    .long_title("The Rust Programming Language Official Documentation")  // Used in Cmd+Ctrl modifier
    .icon_for_filetype("public.html")
    .copy_text("doc.rust-lang.org");

// Convert to regular Item (happens automatically when added to workflow)
let item: Item = url_item.into();
```

URL items automatically include modifiers for copying links:

- **⌘ (Cmd)**: Copy as Markdown link `[title](url)`
- **⌥ (Alt)**: Copy as rich text link (HTML)
- **⌘⇧ (Cmd+Shift)**: Copy as Markdown with short title
- **⌥⇧ (Alt+Shift)**: Copy as rich text with short title
- **⌘⌃ (Cmd+Ctrl)**: Copy as Markdown with long title
- **⌥⌃ (Alt+Ctrl)**: Copy as rich text with long title

### Smart Filtering and Sorting

Enable automatic fuzzy search and sorting of results:

```rust
impl Runnable for SearchWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let query = self.query.join(" ");

        // Enable filtering - results will be automatically filtered and sorted
        workflow.set_filter_keyword(query);

        // Add items - they'll be filtered based on the query
        workflow.append_items(vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Yellow fruit"),
            Item::new("Carrot").subtitle("Orange vegetable"),
        ]);

        Ok(())
    }
}
```

### Workflow Directories

Access workflow-specific data and cache directories:

```rust
impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        // Access workflow data directory (persistent storage)
        let data_dir = workflow.data_dir();
        let config_file = data_dir.join("config.json");

        // Access workflow cache directory (temporary storage)
        let cache_dir = workflow.cache_dir();
        let temp_file = cache_dir.join("temp_data.json");

        // Use directories for file operations
        std::fs::write(config_file, "{\"setting\": \"value\"}")?;

        Ok(())
    }
}
```

### Response Caching and Rerun

Control Alfred's caching behavior and automatic refresh:

```rust
use std::time::Duration;

impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        // Cache results for 5 minutes, allow loose reload
        workflow.cache(Duration::from_secs(300), true);

        // Automatically rerun every 30 seconds
        workflow.rerun(Duration::from_secs(30));

        // Skip Alfred's knowledge base integration
        workflow.skip_knowledge(true);

        workflow.append_item(Item::new("Cached result"));
        Ok(())
    }
}
```

## 🧪 Testing

Alfrusco provides comprehensive testing support with shared utilities and organized test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use alfrusco::config::TestingProvider;
    use tempfile::tempdir;

    #[test]
    fn test_my_workflow() {
        let workflow = MyWorkflow {
            query: vec!["test".to_string()],
        };

        // Use TestingProvider for isolated testing
        let temp_dir = tempdir().unwrap();
        let provider = TestingProvider(temp_dir.path().to_path_buf());

        let mut buffer = Vec::new();
        alfrusco::execute(&provider, workflow, &mut buffer);

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Hello, test!"));
    }

    #[tokio::test]
    async fn test_async_workflow() {
        let workflow = AsyncWorkflow {
            query: vec!["async".to_string()],
        };

        let temp_dir = tempdir().unwrap();
        let provider = TestingProvider(temp_dir.path().to_path_buf());

        let mut buffer = Vec::new();
        alfrusco::execute_async(&provider, workflow, &mut buffer).await;

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("async"));
    }
}
```

### ### Test Organization

Alfrusco maintains a comprehensive test suite with **112 tests** across organized test files:

- **`background_job_integration_tests.rs`** - Complete background job lifecycle testing (6 tests)
- **`clipboard_tests.rs`** - Clipboard functionality testing (4 tests)
- **`config_tests.rs`** - Configuration and environment testing (8 tests)
- **`error_injection_tests.rs`** - Error handling and edge cases (2 tests)
- **`error_tests.rs`** - Error type behavior (7 tests)
- **`logging_tests.rs`** - Logging functionality (1 test)
- **`runnable_tests.rs`** - Trait implementation testing (4 tests)
- **`tests/common/mod.rs`** - Shared test utilities and helpers

### Shared Test Utilities

The `tests/common/mod.rs` module provides reusable testing utilities that eliminate code duplication and ensure
consistent test setup across the entire test suite. This includes helper functions for creating test workflows, managing
temporary directories, and common test operations.

```

## 📚 Examples

The `examples/` directory contains complete, runnable examples. Since these examples use `AlfredEnvProvider`, they require Alfred environment variables to be set. We provide a convenient script to run them with mock environment variables:

### Running Examples

**Option 1: Using the run script (recommended)**
```bash
# Basic static output
./run-example.sh static_output

# Success workflow with custom message  
./run-example.sh success --message "Custom message"

# Async API example
./run-example.sh random_user search_term

# URL items demonstration
./run-example.sh url_items

# Background job example
./run-example.sh sleep --duration-in-seconds 10

# Error handling example
./run-example.sh error --file-path nonexistent.txt
```

**Option 2: Using Make targets**

```bash
# List all available examples
make examples-help

# Run specific examples
make example-static_output
make example-success
make example-url_items
```

**Option 3: Manual environment setup**

```bash
# Set required Alfred environment variables
export alfred_workflow_bundleid="com.example.test"
export alfred_workflow_cache="/tmp/cache"
export alfred_workflow_data="/tmp/data"
export alfred_version="5.0"
export alfred_version_build="2058"
export alfred_workflow_name="Test Workflow"

# Then run normally
cargo run --example static_output
```

### Example Descriptions

- **static_output** - Basic workflow that returns static items without user input
- **success** - Simple workflow demonstrating command-line argument parsing
- **random_user** - Async workflow that fetches data from an external API with fuzzy filtering
- **url_items** - Demonstrates URL items with automatic clipboard integration and modifiers
- **sleep** - Shows background job execution with status monitoring
- **error** - Demonstrates custom error types and error item generation
- **async_success** - Basic async workflow example
- **async_error** - Async workflow with error handling examples

## 📖 API Reference

### Core Types

#### `Item`

The primary building block for Alfred workflow results.

**Key Methods:**

- `new(title)` - Create a new item with a title
- `subtitle(text)` - Set subtitle text
- `arg(value)` / `args(values)` - Set arguments passed to actions
- `valid(bool)` - Set whether the item is actionable
- `uid(id)` - Set unique identifier for Alfred's learning
- `icon_from_image(path)` / `icon_for_filetype(type)` - Set item icons
- `copy_text(text)` / `large_type_text(text)` - Set text operations
- `quicklook_url(url)` - Enable Quick Look preview
- `var(key, value)` - Set workflow variables
- `autocomplete(text)` - Set tab completion text
- `modifier(modifier)` - Add keyboard modifier actions

#### `URLItem`

Specialized item type for URLs with automatic clipboard integration.

**Key Methods:**

- `new(title, url)` - Create a URL item
- `subtitle(text)` - Override default URL subtitle
- `short_title(text)` / `long_title(text)` - Set alternative titles for modifiers
- `display_title(text)` - Override displayed title while preserving link title
- `copy_text(text)` - Set custom copy text
- `icon_from_image(path)` / `icon_for_filetype(type)` - Set icons

#### `Workflow`

Main workflow execution context.

**Key Methods:**

- `append_item(item)` / `append_items(items)` - Add items to results
- `prepend_item(item)` / `prepend_items(items)` - Add items to beginning
- `set_filter_keyword(query)` - Enable fuzzy filtering
- `data_dir()` / `cache_dir()` - Access workflow directories
- `run_in_background(name, max_age, command)` - Execute background jobs

#### `Response`

Controls Alfred's response behavior.

**Key Methods:**

- `cache(duration, loose_reload)` - Set caching behavior
- `rerun(interval)` - Set automatic refresh interval
- `skip_knowledge(bool)` - Control Alfred's knowledge integration

### Traits

#### `Runnable`

For synchronous workflows.

```rust
trait Runnable {
    type Error: WorkflowError;
    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}
```

#### `AsyncRunnable`

For asynchronous workflows.

```rust
#[async_trait]
trait AsyncRunnable {
    type Error: WorkflowError;
    async fn run_async(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}
```

#### `WorkflowError`

For custom error types that integrate with Alfred.

```rust
trait WorkflowError: std::error::Error {
    fn error_item(&self) -> Item { /* default implementation */ }
}
```

### Configuration

#### `AlfredEnvProvider`

Production configuration provider that reads from Alfred environment variables.

#### `TestingProvider`

Testing configuration provider that uses temporary directories.

### Execution Functions

- `execute(provider, runnable, writer)` - Execute synchronous workflow
- `execute_async(provider, runnable, writer)` - Execute asynchronous workflow
- `init_logging(provider)` - Initialize structured logging## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/adlio/alfrusco.git
cd alfrusco
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with nextest (recommended)
cargo nextest run

# Run tests serially (for debugging flaky tests)
make test-serial

# Run with coverage
cargo tarpaulin --out html
```

### Running Examples

Examples require Alfred environment variables. Use the provided script:

```bash
# Basic static output
./run-example.sh static_output

# Success workflow with custom message
./run-example.sh success --message "Hello World"

# Async API example
./run-example.sh random_user john

# URL items demonstration
./run-example.sh url_items

# Background job example
./run-example.sh sleep --duration-in-seconds 5

# Error handling example
./run-example.sh error --file-path /nonexistent/file.txt

# Or use Make targets
make example-static_output
make examples-help  # See all available examples
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to
discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for your changes
5. Ensure all tests pass (`cargo nextest run`)
6. Run clippy (`cargo clippy`)
7. Format your code (`cargo fmt`)
8. Commit your changes (`git commit -m 'Add some amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Ensure clippy passes without warnings (`cargo clippy`)
- Add documentation for public APIs
- Include tests for new functionality
- Update examples if adding new features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- 📖 [Documentation](https://docs.rs/alfrusco)
- 🐛 [Issue Tracker](https://github.com/adlio/alfrusco/issues)
- 💬 [Discussions](https://github.com/adlio/alfrusco/discussions)

---

Made with ❤️ for the Alfred and Rust communities.