//! Rate Limiting types

use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self { max_requests, window_seconds }
    }
}

struct RateLimitEntry { count: u32, window_start: Instant }

impl RateLimitEntry { fn new() -> Self { Self { count: 1, window_start: Instant::now() } } }

/// Rate limiter storage
pub struct RateLimiter {
    entries: RwLock<HashMap<String, RateLimitEntry>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self { entries: RwLock::new(HashMap::new()), config }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut entries = self.entries.write();
        let window = Duration::from_secs(self.config.window_seconds);
        let entry = entries.entry(key.to_string()).or_insert_with(RateLimitEntry::new);

        if entry.window_start.elapsed() > window { *entry = RateLimitEntry::new(); }
        if entry.count > self.config.max_requests { return false; }
        entry.count += 1;
        true
    }
}
