mod utilities;

use clap::{arg, Parser};
use colored::*;
use fern::Dispatch;
use log::{info, LevelFilter};
use std::io;
use std::string::ToString;

const ERROR_LEVEL: &str = "ERROR";
const WARN_LEVEL: &str = "WARN";
const INFO_LEVEL: &str = "INFO";
const DEBUG_LEVEL: &str = "DEBUG";
const TRACE_LEVEL: &str = "TRACE";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Also displays debug logs
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// The path to the logging output file.
    ///
    /// Defaults to "output.log" in relative path to executable
    #[arg(short, long, default_value = "output.log")]
    log_output_file: String,

    /// The block number to start caching blocks from
    #[arg(short, long)]
    from_block_number: Option<u64>,

    /// The block number to cache blocks to
    #[arg(short, long)]
    to_block_number: Option<u64>,

    /// The RPC URL for connecting to the Solana MainNet.
    ///
    /// Defaults to Solana provided URL, more information here:
    /// https://docs.solana.com/cluster/rpc-endpoints
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    rpc_url: String,

    /// The output file to write the blocks collected to
    #[arg(short, long, default_value = "blocks.json")]
    blocks_output_file: String,

    /// The rate limit imposed on the cacher to prevent 429's on RPC.
    ///
    /// Defaults to 40 which is the correct value for the default
    /// rpc_url.
    #[arg(short, long, default_value = 40)]
    requests_per_second_rate_limit: u32
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    configure_logger(args.verbose, &args.log_output_file).expect("Failed to configure the applications logger.");
    info!("Initializing Solana Block Cacher..");

    // validate arguments
    if args.from_block_number.is_none() || args.to_block_number.is_none() {
        panic!("You must specify the --from-block-number and --to-block-number flags");
    }
}

/// Initializes the logging macros for the entire application. You can configure the logging level
/// directly here within the code.
///
/// # Arguments
///
/// * `verbose` - A boolean to determine the whether the logs are maximum verbosity
/// * `log_output_file` - The path to the logging file
fn configure_logger(verbose: bool, log_output_file: &String) -> Result<(), fern::InitError> {
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