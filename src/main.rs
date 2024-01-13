mod model;
mod networking;
mod services;
mod utilities;

use crate::model::solana_block::{BlockBatch, Reverse};
use crate::networking::BlockFetcherFactory;
use crate::services::fetch_block_service::FetchBlockService;
use crate::services::write_service::WriteService;
use crate::utilities::priority_queue::PriorityQueue;
use crate::utilities::priority_queue::Queue;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;
use crate::utilities::{logging, threading};
use clap::{arg, Parser};
use log::info;
use solana_client::client_error::reqwest::Url;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tokio::time::Instant;

const ERROR_LEVEL: &str = "ERROR";
const WARN_LEVEL: &str = "WARN";
const INFO_LEVEL: &str = "INFO";
const DEBUG_LEVEL: &str = "DEBUG";
const TRACE_LEVEL: &str = "TRACE";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Displays debug logs from the application and dependencies
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// The path to the logging output file.
    ///
    /// Defaults to "output.log" in relative path to executable
    #[arg(short, long, default_value = "output.log")]
    log_output_file: String,

    /// The Solana slot number to start caching blocks from
    #[arg(short, long)]
    from_slot: Option<u64>,

    /// The Solana slot number to cache blocks until
    #[arg(short, long)]
    to_slot: Option<u64>,

    /// The HTTP RPC URL for connecting to the Solana MainNet.
    /// this should be in this format: https://crimson-chaotic-bird.solana-mainnet.quiknode.pro/... don't use ws://..
    ///
    /// You can sign up for a QuickNode Solana RPC by visiting:
    /// https://www.quicknode.com?tap_a=67226-09396e&tap_s=4369813-07359f&utm_source=affiliate&utm_campaign=generic&utm_content=affiliate_landing_page&utm_medium=generic
    #[arg(short, long, default_value = "")]
    rpc_url: String,

    /// The output file to write the blocks collected to
    #[arg(short, long, default_value = "blocks.json")]
    output_file: String,

    /// The rate limit imposed on the cacher to prevent 429's on RPC.
    ///
    /// Defaults to 50 which is the default value for a $49/mo QuickNode subscription
    #[arg(long, default_value = "50")]
    rate_limit: u32,

    /// The amount of seconds that the rate limit can occur in.
    ///
    /// Defaults to 1 which is what the window is measured in at QuickNode
    #[arg(short, long, default_value = "1")]
    window: u32,
}

fn main() {
    let timer = Instant::now();
    let args = Args::parse();
    logging::configure_logger(args.verbose, &args.log_output_file)
        .expect("Failed to configure the applications logger.");
    info!("Initializing Solana Block Cacher..");

    validate_arguments(&args);
    info!(
        "Initializing block rate limiter to {} requests every {} seconds",
        args.rate_limit, args.window
    );

    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
        args.rate_limit as usize,
        Duration::from_secs(args.window as u64),
    )));
    let fetcher_factory = BlockFetcherFactory::new(false, &args.rpc_url);
    let number_of_worker_threads = threading::get_optimum_number_of_threads(
        fetcher_factory.create_block_fetcher(),
        args.rate_limit,
        args.window,
        args.from_slot.unwrap(),
        args.to_slot.unwrap(),
    );
    let thread_pool = ThreadPool::new(number_of_worker_threads);
    let priority_queue = Arc::new(Mutex::new(PriorityQueue::<Reverse<BlockBatch>>::new()));
    let condvar = Arc::new(Condvar::new());
    let mut write_service = WriteService::new(priority_queue.clone(), condvar.clone());
    write_service.initialize(&args.output_file);
    let mut fetch_block_service = FetchBlockService::new(
        priority_queue.clone(),
        rate_limiter,
        thread_pool,
        condvar.clone(),
        fetcher_factory,
    );
    fetch_block_service.fetch_blocks(args.from_slot.unwrap(), args.to_slot.unwrap());
    write_service.complete_json_file(&args.output_file);
    info!("All blocks cached in {:?}", timer.elapsed());
}

fn validate_arguments(args: &Args) {
    if args.from_slot.is_none() || args.to_slot.is_none() {
        panic!("You must specify the --from-slot and --to-slot flags");
    }

    if args.rpc_url.is_empty() || Url::parse(&args.rpc_url).is_err() {
        panic!("HTTP RPC URL is invalid");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "You must specify the --from-slot and --to-slot flags")]
    fn test_missing_from_and_to_slot() {
        let args = Args {
            verbose: false,
            log_output_file: "".to_string(),
            from_slot: None,
            to_slot: None,
            rpc_url: "".to_string(),
            output_file: "".to_string(),
            rate_limit: 0,
            window: 0,
        };
        validate_arguments(&args);
    }

    #[test]
    fn test_valid_arguments() {
        let args = Args {
            verbose: false,
            log_output_file: "".to_string(),
            from_slot: Some(0),
            to_slot: Some(10),
            rpc_url: "http://localhost:8545".to_string(),
            output_file: "".to_string(),
            rate_limit: 0,
            window: 0,
        };
        validate_arguments(&args); // Should not panic
    }

    #[test]
    #[should_panic(expected = "HTTP RPC URL is invalid")]
    fn test_invalid_rpc_url() {
        let args = Args {
            verbose: false,
            log_output_file: "".to_string(),
            from_slot: Some(0),
            to_slot: Some(10),
            rpc_url: "not_a_valid_url".to_string(),
            output_file: "".to_string(),
            rate_limit: 0,
            window: 0,
        };
        validate_arguments(&args);
    }
}
