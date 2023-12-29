use crate::{DEBUG_LEVEL, ERROR_LEVEL, INFO_LEVEL, TRACE_LEVEL, WARN_LEVEL};
use colored::Colorize;
use fern::Dispatch;
use log::LevelFilter;
use std::io;

/// Initializes the logging macros for the entire application. You can configure the logging level
/// directly here within the code.
///
/// # Arguments
///
/// * `verbose` - A boolean to determine the whether the logs are maximum verbosity
/// * `log_output_file` - The path to the logging file
pub fn configure_logger(verbose: bool, log_output_file: &str) -> Result<(), fern::InitError> {
    let verbosity = determine_verbosity(verbose);
    let console_dispatch = configure_console_logger(verbosity);
    let file_dispatch = configure_file_logger(verbosity, log_output_file);

    combine_loggers(console_dispatch, file_dispatch)
}

fn determine_verbosity(verbose: bool) -> LevelFilter {
    if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

fn configure_console_logger(verbosity: LevelFilter) -> Dispatch {
    // configure a logger for the console to include the ANSI color codes
    Dispatch::new()
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
        .level(verbosity)
        .into()
}

fn configure_file_logger(verbosity: LevelFilter, log_output_file: &str) -> Dispatch {
    Dispatch::new()
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
        .chain(fern::log_file(log_output_file).unwrap()) // log to a file
        .level(verbosity)
}

fn combine_loggers(console_dispatch: Dispatch, file_dispatch: Dispatch) -> Result<(), fern::InitError> {
    Dispatch::new().chain(file_dispatch).chain(console_dispatch).apply()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{self, info};
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn test_determine_verbosity() {
        assert_eq!(determine_verbosity(true), LevelFilter::Debug);
        assert_eq!(determine_verbosity(false), LevelFilter::Info);
    }

    #[test]
    fn test_configure_file_logger() {
        let log_file = "test_log_1.log";
        let file_logger = configure_file_logger(LevelFilter::Debug, log_file);
        assert!(Path::new(log_file).exists());

        // Cleanup
        fs::remove_file(log_file).unwrap();
    }

    #[test]
    fn test_configure_logger() {
        let log_file = "test_log_2.log";
        assert!(configure_logger(true, log_file).is_ok());
        info!("testing, testing, 123");
        assert!(Path::new(log_file).exists());

        let mut file = File::open(log_file).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(!contents.is_empty());

        fs::remove_file(log_file).unwrap();
    }
}
