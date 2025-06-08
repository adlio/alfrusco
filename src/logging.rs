use std::fs::File;

use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

use crate::config::ConfigProvider;
use crate::Error;

/// Initializes the default logger for alfrusco.
///
/// This sets up a logger that outputs to both stderr and a log file in the
/// workflow cache directory.
///
/// If a logger is already initialized, this function will return an error.
/// If the provider configuration fails, this function will also return an error.
///
/// # Errors
///
/// Returns an error if:
/// - A logger is already initialized
/// - The provider configuration cannot be retrieved
///
/// # Example
///
/// ```
/// use alfrusco::{execute, init_logging, Error};
/// use alfrusco::config::TestingProvider;
/// use tempfile::tempdir;
///
/// fn main() -> Result<(), Error> {
///     // For testing, use TestingProvider with a temporary directory
///     let temp_dir = tempdir().unwrap();
///     let provider = TestingProvider(temp_dir.path().to_path_buf());
///     
///     // Initialize logging before executing your workflow
///     init_logging(&provider)?;
///
///     // Then execute your workflow
///     // execute(&provider, ...);
///     Ok(())
/// }
/// ```
pub fn init_logging(provider: &dyn ConfigProvider) -> Result<(), Error> {
    // Get the workflow configuration - fail if this doesn't work
    let config = provider.config().map_err(|e| {
        Error::Config(format!(
            "Failed to get workflow configuration for logging: {e}"
        ))
    })?;

    // Set up log file path in the workflow cache directory
    let log_file_path = config.workflow_cache.join("workflow.log");

    // Create parent directories if they don't exist
    if let Some(parent) = log_file_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Configure colors for terminal output
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::White);

    // Create a file for logging
    let log_file = match File::create(&log_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Warning: Could not create log file: {e}");
            // Continue with stderr logging only
            return fern::Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "[{} {} {}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        colors.color(record.level()),
                        record.target(),
                        message
                    ))
                })
                .chain(std::io::stderr())
                .level(LevelFilter::Debug)
                .apply()
                .map_err(|e| {
                    Error::Logging(format!("Failed to initialize stderr-only logger: {e}"))
                });
        }
    };

    // Configure fern with both stderr and file outputs
    fern::Dispatch::new()
        // Format for all logs
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        // Output to stderr
        .chain(
            fern::Dispatch::new()
                .level(LevelFilter::Info)
                .chain(std::io::stderr()),
        )
        // Output to file
        .chain(
            fern::Dispatch::new()
                .level(LevelFilter::Debug)
                .chain(log_file),
        )
        // Set global log level
        .level(LevelFilter::Debug)
        // Apply configuration
        .apply()
        .map_err(|e| Error::Logging(format!("Failed to initialize logger: {e}")))
}
