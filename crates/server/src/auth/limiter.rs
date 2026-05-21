use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(60);
const DEFAULT_LIMIT: usize = 5;
const MAX_TRACKED_BUCKETS: usize = 1024;

#[derive(Debug)]
pub struct RateLimiter {
    inner: Mutex<HashMap<IpAddr, Vec<Instant>>>,
    limit: usize,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(DEFAULT_LIMIT)
    }
}

impl RateLimiter {
    pub fn new(limit: usize) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            limit,
        }
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut guard = self.inner.lock().expect("rate limiter lock");
        if guard.len() > MAX_TRACKED_BUCKETS {
            guard.retain(|_, hits| hits.iter().any(|hit| now.duration_since(*hit) < WINDOW));
        }
        let hits = guard.entry(ip).or_default();
        hits.retain(|hit| now.duration_since(*hit) < WINDOW);
        if hits.len() >= self.limit {
            return false;
        }
        hits.push(now);
        true
    }
}
