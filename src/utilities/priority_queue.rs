use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};

/// A generic priority queue implemented using a binary heap.
///
/// The `PriorityQueue` struct can store any type `T` that implements `Ord` and `PartialOrd`,
/// which are used to maintain the elements in a sorted order. The default behavior is that
/// the element with the highest value according to the `Ord` trait will be considered the highest priority.
pub struct PriorityQueue<T: Ord + PartialOrd> {
    heap: BinaryHeap<T>,
}

impl<T: Ord + PartialOrd> PriorityQueue<T> {
    /// Creates a new, empty `PriorityQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// ```
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }

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
    pub fn push(&mut self, block: T) {
        self.heap.push(block);
    }

    /// Adds multiple elements to the priority queue.
    ///
    /// This method takes a vector of elements and adds each of them to the priority queue.
    /// The position of each new element in the queue is determined by its priority relative to existing elements.
    /// The elements are added one by one, maintaining the heap property of the priority queue after each insertion.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of elements to be added to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::PriorityQueue;
    /// let mut pq = PriorityQueue::new();
    /// pq.push_many(vec![5, 3, 8]);
    /// assert_eq!(pq.pop(), Some(8)); // 8 has the highest priority
    /// ```
    pub fn push_many(&mut self, items: Vec<T>) {
        for item in items {
            self.heap.push(item);
        }
    }

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
    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

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
    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_queue_is_empty() {
        let pq: PriorityQueue<i32> = PriorityQueue::new();
        assert!(pq.peek().is_none());
    }

    #[test]
    fn test_push_and_peek() {
        let mut pq = PriorityQueue::new();
        pq.push(5);
        assert_eq!(pq.peek(), Some(&5));
        pq.push(10);
        assert_eq!(pq.peek(), Some(&10)); // 10 should be the new highest priority
    }

    #[test]
    fn test_push_many_and_pop_order() {
        let mut pq = PriorityQueue::new();
        pq.push_many(vec![3, 1, 4]);
        assert_eq!(pq.pop(), Some(4)); // 4 has the highest priority
        assert_eq!(pq.pop(), Some(3)); // followed by 3
        assert_eq!(pq.pop(), Some(1)); // and then 1
        assert_eq!(pq.pop(), None); // queue is empty now
    }

    /*#[test]
    fn test_wait_for_data() {
        let mut pq = Arc::new(PriorityQueue::new());
        let mut pq_clone = pq.clone();

        // Spawn a thread to push data after a delay
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            pq_clone.push(5);
        });

        // Main thread should wait and then proceed when data is pushed
        pq.wait_for_data();
        assert_eq!(pq.pop(), Some(5));
    }*/
}
