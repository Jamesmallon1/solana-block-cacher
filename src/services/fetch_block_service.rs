use std::sync::Mutex;
use crate::utilities::rate_limiter::RateLimiter;
use crate::utilities::threading::ThreadPool;

pub struct FetchBlockService {
    rate_limiter: Mutex<RateLimiter>,
    thread_pool: ThreadPool
}

impl FetchBlockService {
    pub fn new(rate_limiter: Mutex<RateLimiter>, thread_pool: ThreadPool) -> Self {
        FetchBlockService {
            rate_limiter,
            thread_pool
        }
    }
}