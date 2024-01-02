use std::cmp::Ordering;

// the constant size of batches within the system
pub const BATCH_SIZE: u64 = 50;

/// A batch of serialized Solana blocks.
///
/// `BlockBatch` represents a collection of Solana blocks that have been serialized and are ready for processing or storage.
/// Each `BlockBatch` is associated with a sequence number indicating its order in the overall sequence of batches.
///
/// # Fields
///
/// - `sequence_number`: A unique identifier representing the order of this batch in the sequence of all batches.
/// - `batch`: A vector of `SerializedSolanaBlock`, representing the individual blocks contained in this batch.
///
/// # Examples
///
/// ```
/// use your_crate::{BlockBatch, SerializedSolanaBlock};
///
/// let block_batch = BlockBatch {
///     sequence_number: 1,
///     batch: vec![SerializedSolanaBlock {
///         slot_number: 123,
///         data: "block_data".to_string(),
///     }],
/// };
/// ```
#[derive(Debug)]
pub struct BlockBatch {
    pub(crate) sequence_number: u64,
    pub(crate) batch: Vec<SerializedSolanaBlock>,
}

impl BlockBatch {
    /// Creates a new `BlockBatch` with a calculated sequence number.
    ///
    /// The sequence number is determined based on the starting slot of the entire operation (`global_start_slot`)
    /// and the starting slot of this specific batch (`batch_start_slot`), divided by the batch size.
    ///
    /// # Parameters
    ///
    /// - `global_start_slot`: The starting slot number for the entire batch processing operation.
    /// - `batch_start_slot`: The starting slot number for this specific batch.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `BlockBatch` with an empty batch and a calculated sequence number.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::{BlockBatch, SerializedSolanaBlock};
    ///
    /// let global_start_slot = 100.0;
    /// let batch_start_slot = 150.0;
    /// let block_batch = BlockBatch::new(global_start_slot, batch_start_slot);
    ///
    /// assert_eq!(block_batch.sequence_number, 2);
    /// ```
    pub fn new(global_start_slot: f64, batch_start_slot: f64) -> Self {
        let sequence_number = (((batch_start_slot - global_start_slot) / BATCH_SIZE as f64).floor() + 1.0) as u64;
        let batch: Vec<SerializedSolanaBlock> = vec![];

        BlockBatch { sequence_number, batch }
    }

    /// Adds a `SerializedSolanaBlock` to the batch.
    ///
    /// This method appends a given `SerializedSolanaBlock` to the end of the `batch` vector in the `BlockBatch`.
    ///
    /// # Parameters
    ///
    /// - `block`: The `SerializedSolanaBlock` to be added to the batch.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::{BlockBatch, SerializedSolanaBlock};
    ///
    /// let mut block_batch = BlockBatch::new(100.0, 150.0);
    /// let solana_block = SerializedSolanaBlock {
    ///     slot_number: 123,
    ///     data: "block_data".to_string(),
    /// };
    ///
    /// block_batch.push(solana_block);
    /// assert_eq!(block_batch.batch.len(), 1);
    /// ```
    pub fn push(&mut self, block: SerializedSolanaBlock) {
        self.batch.push(block);
    }
}

/// A serialized representation of a Solana block.
///
/// `SerializedSolanaBlock` contains the data for a Solana block, along with its slot number.
/// The `data` field is a string representation, potentially in a serialized format like JSON, that contains
/// the full information of the block.
///
/// # Fields
///
/// - `slot_number`: The slot number of the Solana block. This is a unique identifier for the block in the Solana blockchain.
/// - `data`: The serialized data of the block, stored as a `String`.
///
/// # Examples
///
/// ```
/// use your_crate::SerializedSolanaBlock;
///
/// let serialized_block = SerializedSolanaBlock {
///     slot_number: 123,
///     data: "{\"transactions\": [], \"rewards\": []}".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct SerializedSolanaBlock {
    pub(crate) slot_number: u64,
    pub(crate) data: String,
}

// the below are the implementations of traits required by the priority queue for ordering
impl Eq for BlockBatch {}

impl PartialEq<Self> for BlockBatch {
    fn eq(&self, other: &Self) -> bool {
        self.sequence_number == other.sequence_number
    }
}

impl PartialOrd<Self> for BlockBatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sequence_number.partial_cmp(&other.sequence_number)
    }
}

impl Ord for BlockBatch {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.sequence_number, other.sequence_number) {
            (a, b) => a.cmp(&b),
        }
    }
}

/// A wrapper struct that reverses the order of comparisons for the contained value.
///
/// This struct is useful when you want to change the order of sorting or any other operation
/// that relies on comparison operators. It simply wraps a value of any type that implements
/// the necessary comparison traits (`PartialEq`, `Eq`, `PartialOrd`, `Ord`).
///
/// # Examples
///
/// ```
/// let mut vec = vec![Reverse(3), Reverse(1), Reverse(2)];
/// vec.sort(); // Will sort in reverse order
/// assert_eq!(vec, vec![Reverse(3), Reverse(2), Reverse(1)]);
/// ```
#[derive(Debug)]
pub struct Reverse<T>(pub T);

impl<T: PartialEq> PartialEq for Reverse<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Reverse<T> {}

impl<T: PartialOrd> PartialOrd for Reverse<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_batch_new() {
        let global_start_slot = 100.0;
        let batch_start_slot = 150.0;
        let block_batch = BlockBatch::new(global_start_slot, batch_start_slot);

        assert_eq!(block_batch.sequence_number, 2);
        assert_eq!(block_batch.batch.len(), 0);
    }

    #[test]
    fn test_block_batch_push() {
        let mut block_batch = BlockBatch::new(100.0, 150.0);
        let solana_block = SerializedSolanaBlock {
            slot_number: 123,
            data: "block_data".to_string(),
        };

        block_batch.push(solana_block);
        assert_eq!(block_batch.batch.len(), 1);
        assert_eq!(block_batch.batch[0].slot_number, 123);
        assert_eq!(block_batch.batch[0].data, "block_data");
    }

    #[test]
    fn test_block_batch_ordering() {
        let batch1 = BlockBatch::new(100.0, 150.0);
        let batch2 = BlockBatch::new(100.0, 200.0);

        assert!(batch1 < batch2);
        assert!(batch2 > batch1);
        assert_ne!(batch1, batch2);
    }

    #[test]
    fn test_reverse_ordering() {
        let mut vec = vec![Reverse(3), Reverse(1), Reverse(2)];
        vec.sort();
        assert_eq!(vec, vec![Reverse(3), Reverse(2), Reverse(1)]);
    }
}
