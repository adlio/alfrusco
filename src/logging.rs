use std::fs::File;

use fern::colors::{Color, ColoredLevelConfig};
use log::{LevelFilter, SetLoggerError};

use crate::config::ConfigProvider;

/// Initializes the default logger for alfrusco.
///
/// This sets up a logger that outputs to both stderr and a log file in the
/// workflow cache directory.
///
/// If a logger is already initialized, this function will return an error.
///
/// # Example
///
/// ```
/// use alfrusco::{execute, init_logging};
/// use alfrusco::config::AlfredEnvProvider;
///
/// // Initialize logging before executing your workflow
/// let _ = init_logging(&AlfredEnvProvider);
///
/// // Then execute your workflow
/// // execute(&AlfredEnvProvider, ...);
/// ```
pub fn init_logging(provider: &dyn ConfigProvider) -> Result<(), SetLoggerError> {
    // Get the workflow configuration
    let config = match provider.config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Could not get workflow configuration: {e}");
            // We can't create a SetLoggerError directly, so we'll try to set a dummy logger
            // which will fail if a logger is already set, giving us a SetLoggerError
            struct DummyLogger;
            impl log::Log for DummyLogger {
                fn enabled(&self, _: &log::Metadata) -> bool {
                    false
                }
                fn log(&self, _: &log::Record) {}
                fn flush(&self) {}
            }
            return log::set_logger(&DummyLogger).map(|_| ());
        }
    };

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
                .apply();
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
}
