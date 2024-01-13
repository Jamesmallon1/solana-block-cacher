use crate::model::solana_block::SerializedSolanaBlock;
use log::{debug, error};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use std::thread;
use std::time::Duration;

/// A factory for creating instances of `BlockFetcher`.
///
/// This factory can create either a mock or a real Solana block fetcher based on the
/// provided configuration. It abstracts the creation process of block fetchers,
/// allowing for easy switching between mock and actual implementations.
///
/// # Fields
/// - `use_mock`: A boolean flag indicating whether to use a mock implementation.
/// - `rpc_url`: The URL of the Solana RPC client. Used when `use_mock` is `false`.
pub struct BlockFetcherFactory {
    use_mock: bool,
    rpc_url: String,
}

impl BlockFetcherFactory {
    /// Constructs a new `BlockFetcherFactory`.
    ///
    /// This method initializes a new factory with the specified configuration.
    ///
    /// # Parameters
    /// - `use_mock`: A boolean flag indicating whether to use a mock implementation.
    /// - `rpc_url`: A string slice representing the RPC URL for the Solana client.
    ///
    /// # Returns
    /// Returns a new instance of `BlockFetcherFactory`.
    pub fn new(use_mock: bool, rpc_url: &str) -> Self {
        BlockFetcherFactory {
            use_mock,
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Creates a block fetcher based on the factory's configuration.
    ///
    /// This method generates either a mock or a real implementation of `BlockFetcher`
    /// depending on the `use_mock` flag. It allows for flexible switching between
    /// a testing (mock) environment and a production environment.
    ///
    /// # Returns
    /// Returns a boxed object implementing `BlockFetcher` and `Send`.
    /// This could either be a `MockSolanaClient` or a `SolanaClient` based on the configuration.
    pub fn create_block_fetcher(&self) -> Box<dyn BlockFetcher + Send> {
        if self.use_mock {
            Box::new(MockSolanaClient::new())
        } else {
            Box::new(SolanaClient::new(&self.rpc_url))
        }
    }
}

pub trait BlockFetcher {
    /// Fetches a block from the blockchain by its slot number.
    ///
    /// Implementations of this method should retrieve the block corresponding
    /// to the provided slot number and return it in a serialized form.
    /// The method varies in its actual behavior based on the implementing struct.
    ///
    /// # Parameters
    /// - `slot`: The slot number of the block to fetch.
    ///
    /// # Returns
    /// Returns a `Result` containing either a `SerializedSolanaBlock` on success
    /// or a `GetBlockError` on failure.
    fn get_block(&self, slot: u64) -> Result<SerializedSolanaBlock, GetBlockError>;
}

pub struct SolanaClient {
    rpc_client: RpcClient,
}

impl BlockFetcher for SolanaClient {
    fn get_block(&self, slot: u64) -> Result<SerializedSolanaBlock, GetBlockError> {
        let block = self
            .rpc_client
            .get_block_with_config(
                slot,
                RpcBlockConfig {
                    encoding: None,
                    transaction_details: None,
                    rewards: None,
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                },
            )
            .map_err(|err| {
                if err.to_string().contains("-32009")
                    // todo: this is a timeout retry necessary
                    || err.to_string().contains("-32004")
                    || err.to_string().contains("-32007")
                {
                    // set verbosity lower as slot has been skipped or is not available for a specific slot
                    debug!("Could not retrieve block {} due to error: {}", slot, err)
                } else {
                    error!("Could not retrieve block {} due to error: {}", slot, err);
                }
                GetBlockError::RpcError
            })?;

        let data = serde_json::to_string(&block).map_err(|err| {
            error!("An error occurred serializing a Solana block: {}, Error: {}", slot, err);
            GetBlockError::SerializationError
        })?;

        Ok(SerializedSolanaBlock {
            slot_number: block.parent_slot as u64,
            data,
        })
    }
}

impl SolanaClient {
    fn new(rpc_url: &str) -> Self {
        SolanaClient {
            rpc_client: RpcClient::new(rpc_url),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GetBlockError {
    RpcError,
    SerializationError,
}

// mocking
pub struct MockSolanaClient {
    error: Option<GetBlockError>,
    get_block_returns: SerializedSolanaBlock,
}

impl BlockFetcher for MockSolanaClient {
    fn get_block(&self, _: u64) -> Result<SerializedSolanaBlock, GetBlockError> {
        // simulate the latency for a network call
        thread::sleep(Duration::from_millis(250));
        if self.error.is_some() {
            let clone = self.error.clone().unwrap();
            Err(clone)
        } else {
            Ok(self.get_block_returns.clone())
        }
    }
}

impl MockSolanaClient {
    pub fn new() -> Self {
        MockSolanaClient {
            error: None,
            get_block_returns: SerializedSolanaBlock {
                slot_number: 123,
                data: "{ \"name\": \"hello_world\" }".to_string(),
            },
        }
    }
}
