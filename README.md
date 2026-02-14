# alfrusco

[![Crates.io](https://img.shields.io/crates/v/alfrusco.svg)](https://crates.io/crates/alfrusco)
[![Documentation](https://docs.rs/alfrusco/badge.svg)](https://docs.rs/alfrusco)
[![CI](https://github.com/adlio/alfrusco/actions/workflows/ci.yml/badge.svg)](https://github.com/adlio/alfrusco/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/adlio/alfrusco/graph/badge.svg)](https://codecov.io/gh/adlio/alfrusco)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A Rust library for building [Alfred](https://www.alfredapp.com/) workflows. It handles Alfred's JSON protocol, provides builder patterns for creating items, and includes support for background jobs, clipboard operations, and logging.

## Features

- Builder patterns for creating Alfred items
- Async/await support
- Background jobs that don't block Alfred's UI
- Rich text and Markdown clipboard operations
- Fuzzy search and sorting
- Access to workflow directories and configuration
- Structured logging
- URL items with clipboard modifiers
- Testing utilities

## Installation

Add alfrusco to your `Cargo.toml`:

```toml
[dependencies]
alfrusco = "0.3"

# For async workflows
tokio = { version = "1", features = ["full"] }

# For command-line argument parsing (recommended)
clap = { version = "4", features = ["derive", "env"] }
```

## Quick Start

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
    let _ = alfrusco::init_logging(&AlfredEnvProvider);
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

## Core Concepts

### Items

Items represent choices in the Alfred selection UI:

```rust
use alfrusco::Item;

let item = Item::new("My Title")
    .subtitle("Additional information")
    .arg("argument-passed-to-action")
    .uid("unique-identifier")
    .valid(true)
    .icon_from_image("/path/to/icon.png")
    .copy_text("Text copied with Cmd+C")
    .large_type_text("Text shown in large type with Cmd+L")
    .quicklook_url("https://example.com")
    .var("CUSTOM_VAR", "value")
    .autocomplete("text for tab completion");
```

### Workflow Configuration

Alfrusco handles Alfred's environment variables through configuration providers:

```rust
use alfrusco::config::{AlfredEnvProvider, TestingProvider};

// For production (reads from Alfred environment variables)
let provider = AlfredEnvProvider;

// For testing (uses temporary directories)
let temp_dir = tempfile::tempdir().unwrap();
let provider = TestingProvider(temp_dir.path().to_path_buf());
```

### Error Handling

Custom error types integrate with Alfred:

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

// Errors become Alfred items automatically
impl Runnable for MyWorkflow {
    type Error = MyWorkflowError;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        Err(MyWorkflowError::InvalidInput("Missing required field".to_string()))
    }
}
```

## Advanced Features

### Background Jobs

Run tasks without blocking Alfred's UI:

```rust
use std::process::Command;
use std::time::Duration;

impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let cache_file = workflow.cache_dir().join("releases.json");

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(format!(
                "curl -s https://api.github.com/repos/rust-lang/rust/releases/latest > {}",
                cache_file.display()
            ));

        // Run in background, refresh every 30 seconds
        workflow.run_in_background(
            "github-releases",
            Duration::from_secs(30),
            cmd
        );

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

        Ok(())
    }
}
```

Background jobs track their status and show messages like "Last succeeded 2 minutes ago, running for 3s". Failed jobs are retried automatically.

### URL Items

URL items include modifiers for copying links in different formats:

```rust
use alfrusco::URLItem;

let url_item = URLItem::new("Rust Documentation", "https://doc.rust-lang.org/")
    .subtitle("The Rust Programming Language Documentation")
    .short_title("Rust Docs")
    .long_title("The Rust Programming Language Official Documentation")
    .icon_for_filetype("public.html")
    .copy_text("doc.rust-lang.org");

let item: Item = url_item.into();
```

Modifier keys:
- Cmd: Copy as Markdown link
- Alt: Copy as rich text link
- Cmd+Shift: Copy as Markdown with short title
- Alt+Shift: Copy as rich text with short title
- Cmd+Ctrl: Copy as Markdown with long title
- Alt+Ctrl: Copy as rich text with long title

### Filtering and Sorting

Enable fuzzy search:

```rust
impl Runnable for SearchWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let query = self.query.join(" ");
        workflow.set_filter_keyword(query);

        workflow.append_items(vec![
            Item::new("Apple").subtitle("Fruit"),
            Item::new("Banana").subtitle("Yellow fruit"),
            Item::new("Carrot").subtitle("Orange vegetable"),
        ]);

        Ok(())
    }
}
```

#### Boosting Item Priority

Use boost to influence ranking:

```rust
use alfrusco::{Item, BOOST_HIGH, BOOST_MODERATE};

workflow.append_items(vec![
    Item::new("Preferred Result")
        .subtitle("This ranks higher")
        .boost(BOOST_HIGH),
    Item::new("Normal Result")
        .subtitle("Standard ranking"),
    Item::new("Slightly Preferred")
        .subtitle("Moderate boost")
        .boost(BOOST_MODERATE),
]);
```

Boost constants:
- `BOOST_SLIGHT` (25)
- `BOOST_LOW` (50)
- `BOOST_MODERATE` (75)
- `BOOST_HIGH` (100)
- `BOOST_HIGHER` (150)
- `BOOST_HIGHEST` (200)

Boost only affects non-sticky items. Use `.sticky(true)` for items that should always appear first.

### Workflow Directories

```rust
impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        let data_dir = workflow.data_dir();
        let config_file = data_dir.join("config.json");

        let cache_dir = workflow.cache_dir();
        let temp_file = cache_dir.join("temp_data.json");

        std::fs::write(config_file, "{\"setting\": \"value\"}")?;

        Ok(())
    }
}
```

### Response Caching and Rerun

```rust
use std::time::Duration;

impl Runnable for MyWorkflow {
    type Error = alfrusco::Error;

    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error> {
        workflow.cache(Duration::from_secs(300), true);
        workflow.rerun(Duration::from_secs(30));
        workflow.skip_knowledge(true);

        workflow.append_item(Item::new("Cached result"));
        Ok(())
    }
}
```

## Testing

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

## Examples

The `examples/` directory contains runnable examples. They require Alfred environment variables:

```bash
# Using the run script
./run-example.sh static_output
./run-example.sh success --message "Custom message"
./run-example.sh random_user search_term
./run-example.sh url_items
./run-example.sh sleep --duration-in-seconds 10
./run-example.sh error --file-path nonexistent.txt

# Using Make
make examples-help
make example-static_output

# Manual setup
export alfred_workflow_bundleid="com.example.test"
export alfred_workflow_cache="/tmp/cache"
export alfred_workflow_data="/tmp/data"
export alfred_version="5.0"
export alfred_version_build="2058"
export alfred_workflow_name="Test Workflow"
cargo run --example static_output
```

## API Reference

### `Item`

- `new(title)` - Create item
- `subtitle(text)` - Set subtitle
- `arg(value)` / `args(values)` - Set arguments
- `valid(bool)` - Set actionable
- `uid(id)` - Set unique identifier
- `icon_from_image(path)` / `icon_for_filetype(type)` - Set icons
- `copy_text(text)` / `large_type_text(text)` - Set text operations
- `quicklook_url(url)` - Enable Quick Look
- `var(key, value)` - Set workflow variables
- `autocomplete(text)` - Set tab completion
- `modifier(modifier)` - Add modifier actions
- `sticky(bool)` - Pin to top
- `boost(value)` - Adjust ranking

### `URLItem`

- `new(title, url)` - Create URL item
- `subtitle(text)` - Override subtitle
- `short_title(text)` / `long_title(text)` - Alternative titles for modifiers
- `display_title(text)` - Override display title
- `copy_text(text)` - Set copy text
- `icon_from_image(path)` / `icon_for_filetype(type)` - Set icons

### `Workflow`

- `append_item(item)` / `append_items(items)` - Add items
- `prepend_item(item)` / `prepend_items(items)` - Add items to beginning
- `set_filter_keyword(query)` - Enable filtering
- `data_dir()` / `cache_dir()` - Get directories
- `run_in_background(name, max_age, command)` - Run background job

### `Response`

- `cache(duration, loose_reload)` - Set caching
- `rerun(interval)` - Set refresh interval
- `skip_knowledge(bool)` - Control knowledge integration

### Traits

```rust
trait Runnable {
    type Error: WorkflowError;
    fn run(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}

#[async_trait]
trait AsyncRunnable {
    type Error: WorkflowError;
    async fn run_async(self, workflow: &mut Workflow) -> Result<(), Self::Error>;
}

trait WorkflowError: std::error::Error {
    fn error_item(&self) -> Item { /* default implementation */ }
}
```

### Configuration

- `AlfredEnvProvider` - Reads from Alfred environment variables
- `TestingProvider` - Uses temporary directories

### Execution

- `execute(provider, runnable, writer)` - Run synchronous workflow
- `execute_async(provider, runnable, writer)` - Run async workflow
- `init_logging(provider)` - Initialize logging

## Development

```bash
git clone https://github.com/adlio/alfrusco.git
cd alfrusco
cargo build
cargo test
cargo nextest run  # recommended
cargo tarpaulin --out html  # coverage
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes and add tests
4. Run `cargo nextest run`, `cargo clippy`, `cargo fmt`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE).

## Support

- [Documentation](https://docs.rs/alfrusco)
- [Issues](https://github.com/adlio/alfrusco/issues)
- [Discussions](https://github.com/adlio/alfrusco/discussions)
