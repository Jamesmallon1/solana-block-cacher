use std::cmp::Ordering;

// the constant size of batches within the system
pub const BATCH_SIZE: u64 = 100;

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
pub struct BlockBatch {
    pub(crate) sequence_number: u64,
    pub(crate) batch: Vec<SerializedSolanaBlock>,
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
