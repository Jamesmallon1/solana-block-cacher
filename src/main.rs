mod services;
mod utilities;

use crate::services::fetch_block_service::FetchBlockService;
use crate::utilities::priority_queue::PriorityQueue;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;
use crate::utilities::{logging, threading};
use clap::{arg, Parser};
use log::info;
use std::sync::{Arc, Mutex};
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
    output_file: String,

    /// The rate limit imposed on the cacher to prevent 429's on RPC.
    ///
    /// Defaults to 40 which is the correct value for the default
    /// rpc_url.
    #[arg(long, default_value = "1")]
    rate_limit: u32,

    /// The amount of seconds that the rate limit can occur in.
    ///
    /// For example the default solana rpc allows for 40 requests every 10 seconds
    #[arg(short = 'w', long, default_value = "2")]
    window: u32,
}

fn main() {
    let timer = Instant::now();
    let args = Args::parse();
    logging::configure_logger(args.verbose, &args.log_output_file)
        .expect("Failed to configure the applications logger.");
    info!("Initializing Solana Block Cacher..");

    // validate arguments
    if args.from_block_number.is_none() || args.to_block_number.is_none() {
        panic!("You must specify the --from-block-number and --to-block-number flags");
    }

    info!(
        "Initializing block rate limiter to {} requests every {} seconds",
        args.rate_limit, args.window
    );

    let rl = Arc::new(Mutex::new(RateLimiter::new(
        args.rate_limit as usize,
        Duration::from_secs(args.window as u64),
    )));

    let number_of_worker_threads =
        threading::get_optimum_number_of_threads(&args.rpc_url, args.rate_limit, args.window);
    let tp = ThreadPool::new(number_of_worker_threads);
    //let pq = Arc::new(Mutex::new(PriorityQueue:::new()));
    let fbs = FetchBlockService::new(rl, tp);
    fbs.fetch_blocks(
        args.from_block_number.unwrap(),
        args.to_block_number.unwrap(),
        &args.rpc_url,
    );
    info!("Program completed in {:?}", timer.elapsed())
}
