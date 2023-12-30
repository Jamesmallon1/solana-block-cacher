use std::collections::BinaryHeap;

/// A generic priority queue implemented using a binary heap.
///
/// The `PriorityQueue` struct can store any type `T` that implements `Ord` and `PartialOrd`,
/// which are used to maintain the elements in a sorted order. The default behavior is that
/// the element with the highest value according to the `Ord` trait will be considered the highest priority.
pub struct PriorityQueue<T: Ord + PartialOrd> {
    heap: BinaryHeap<T>,
}

pub trait Queue<'a, T>
where
    T: 'a + Ord + PartialOrd,
{
    /// Creates a new, empty `PriorityQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// ```
    fn new() -> Self;

    /// Adds an element to the priority queue.
    ///
    /// The position of the new element in the queue is determined by its priority relative to existing elements.
    ///
    /// # Arguments
    ///
    /// * `block` - The element to be added to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// ```
    fn push(&mut self, block: T);

    /// Removes and returns the highest priority element from the queue, if it is not empty.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// assert_eq!(pq.pop(), Some(5));
    /// assert_eq!(pq.pop(), None);
    /// ```
    fn pop(&mut self) -> Option<T>;

    /// Returns a reference to the highest priority element in the queue without removing it.
    ///
    /// This method returns `None` if the queue is empty. Otherwise, it returns `Some` with a reference to
    /// the element with the highest priority. The returned value is wrapped in an `Option` for safe handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push(5);
    /// pq.push(3);
    /// assert_eq!(pq.peek(), Some(&5));
    /// ```
    fn peek(&'a self) -> Option<&'a T>;
}

impl<'a, T: 'a + Ord + PartialOrd> Queue<'a, T> for PriorityQueue<T> {
    fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }

    fn push(&mut self, block: T) {
        self.heap.push(block);
    }

    fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    fn peek(&'a self) -> Option<&'a T> {
        self.heap.peek()
    }
}

// mock implementation for testing
pub struct MockQueue<T> {
    to_pop: Option<T>,
    to_peak: Option<T>,
}

impl<'a, T: 'a + Ord + PartialOrd> Queue<'a, T> for MockQueue<T> {
    fn new() -> Self {
        MockQueue {
            to_pop: None,
            to_peak: None,
        }
    }

    fn push(&mut self, _: T) {}

    fn pop(&mut self) -> Option<T> {
        self.to_pop.take()
    }

    fn peek(&'a self) -> Option<&'a T> {
        self.to_peak.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_queue_new() {
        let pq: PriorityQueue<i32> = PriorityQueue::new();
        assert!(pq.heap.is_empty());
    }

    #[test]
    fn test_priority_queue_push_and_peek() {
        let mut pq = PriorityQueue::new();
        pq.push(3);
        pq.push(5);
        pq.push(1);

        assert_eq!(pq.peek(), Some(&5));
    }

    #[test]
    fn test_priority_queue_pop() {
        let mut pq = PriorityQueue::new();
        pq.push(3);
        pq.push(5);
        pq.push(1);

        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(3));
        assert_eq!(pq.pop(), Some(1));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_priority_queue_peek_empty() {
        let pq: PriorityQueue<i32> = PriorityQueue::new();
        assert_eq!(pq.peek(), None);
    }
}
