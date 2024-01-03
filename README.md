# Solana Block Cacher
![build and testing](https://github.com/Jamesmallon1/rust-simple-event-driven-microservices/actions/workflows/main-ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/Jamesmallon1/solana-block-cacher/graph/badge.svg?token=8pB0srxCoq)](https://codecov.io/gh/Jamesmallon1/solana-block-cacher)

## Overview
Solana Block Cacher is a high-performance, command-line interface (CLI) tool built in Rust. It is designed to efficiently pull blocks from the Solana blockchain, respecting the specified rate limits. This tool intelligently measures the user's connection speed and the rate limit to calculate the optimum number of threads for fetching blocks. The blocks are processed in batches of 50 and written to a JSON file in an efficient manner.

## Features
- **Fast Block Fetching:** Leveraging the speed of Rust, it pulls blocks as fast as the rate limit allows.
- **Connection Speed Optimization:** Automatically adjusts the number of threads based on the user's connection speed and rate limits.
- **Customizable Block Range:** Users can specify the starting and ending Solana slot numbers to cache.
- **Efficient Batch Processing:** Handles blocks in batches of 50 for efficient processing.
- **Verbose Logging:** Option to enable detailed debug logs.

## Installation
Before installing Solana Block Cacher, ensure you have Cargo installed on your system. If not, you can install it following the instructions on the [official Cargo website](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To install Solana Block Cacher, use the following command:
```bash
cargo install solana-block-cacher
```

## Usage
To use Solana Block Cacher, run the command with the desired arguments. Below are the available options:
```bash
solana-block-cacher [OPTIONS]
```

### Options
- `-v, --verbose`: Enables verbose logging (default: false).
- `-l, --log_output_file <LOG_OUTPUT_FILE>`: Path to the logging output file (default: "output.log").
- `-f, --from_slot <FROM_SLOT>`: The Solana slot number to start caching blocks from.
- `-t, --to_slot <TO_SLOT>`: The Solana slot number to cache blocks until.
- `-r, --rpc_url <RPC_URL>`: The HTTP RPC URL for connecting to Solana MainNet.
- `-o, --output_file <OUTPUT_FILE>`: The file to write the blocks to (default: "blocks.json").
- `--rate_limit <RATE_LIMIT>`: The rate limit for the cacher (default: 50).
- `-w, --window <WINDOW>`: The time window for the rate limit in seconds (default: 1).

### Example
```bash
solana-block-cacher --from_slot 12345 --to_slot 54321 --rpc_url https://example-rpc-url.com --output_file my_blocks.json
```

## Optimum Number of Threads Calculation
Given the following inputs:
- $R_l$ (Rate Limit): The maximum number of requests allowed per time window.
- $W$ (Window of Time): The duration of the time window in milliseconds.
- $T$ (Average Time per Request): The average time it takes to pull blocks from the network in milliseconds.

The Optimum Number of Threads ($O_n$) can be calculated using the formula:

$$
O_n = \frac{R_l}{\min\left(\frac{W}{T}, R_l\right)}
$$

Where:
- $\frac{R_l}{\frac{W}{T}}$ calculates the effective number of requests that a single thread can handle within the time window $W$.
- $\min\left(\frac{W}{T}, R_l\right)$ ensures that the effective number of requests per thread does not exceed the rate limit.
- $O_n$ is rounded up to the nearest whole number as the number of threads must be an integer.

## Contributing
Contributions are welcome! Please feel free to submit pull requests or open issues for improvements and suggestions.

Prior to contributing please ensure you read [CONTRIBUTING.md](CONTRIBUTING.md).

## License
This project is open source and available under MIT.

#### Happy Coding! 🚀🦀

