use std::io;
use colored::Colorize;
use fern::Dispatch;
use log::LevelFilter;
use crate::{DEBUG_LEVEL, ERROR_LEVEL, INFO_LEVEL, TRACE_LEVEL, WARN_LEVEL};

/// Initializes the logging macros for the entire application. You can configure the logging level
/// directly here within the code.
///
/// # Arguments
///
/// * `verbose` - A boolean to determine the whether the logs are maximum verbosity
/// * `log_output_file` - The path to the logging file
pub fn configure_logger(verbose: bool, log_output_file: &String) -> Result<(), fern::InitError> {
    let mut verbosity = LevelFilter::Info;
    if verbose {
        verbosity = LevelFilter::Debug;
    }

    // configure a logger for the console to include the ANSI color codes
    let console_dispatch = Dispatch::new()
        // format: specify log line format
        .format(|out, message, record| {
            let level_string = match record.level() {
                log::Level::Error => ERROR_LEVEL.red(),
                log::Level::Warn => WARN_LEVEL.yellow(),
                log::Level::Info => INFO_LEVEL.blue(),
                log::Level::Debug => DEBUG_LEVEL.into(),
                log::Level::Trace => TRACE_LEVEL.cyan(),
            };
            out.finish(format_args!(
                "{} [{}] [{}] - {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string().blue(),
                record.target().to_uppercase().green(),
                level_string,
                message
            ))
        })
        // output: specify log output
        .chain(io::stdout()) // log to stdout
        .level(verbosity);

    // configure a logger for the file to exclude ANSI color codes
    let file_dispatch = Dispatch::new()
        // format: specify log line format
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] - {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string(),
                record.target().to_uppercase(),
                record.level(),
                message
            ))
        })
        // output: specify log output
        .chain(fern::log_file(log_output_file)?) // log to a file
        .level(verbosity);

    // implement both loggers on the base dispatch logger
    Dispatch::new().chain(file_dispatch).chain(console_dispatch).apply()?;

    Ok(())
}