use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct MessageDedup {
    seen: HashSet<String>,
    max_size: usize,
    last_sweep: Instant,
}

impl MessageDedup {
    pub fn new(max_size: usize) -> Self {
        Self {
            seen: HashSet::new(),
            max_size,
            last_sweep: Instant::now(),
        }
    }

    pub fn is_duplicate(&mut self, key: &str) -> bool {
        self.maybe_sweep();
        if self.seen.contains(key) {
            true
        } else {
            self.seen.insert(key.to_string());
            false
        }
    }

    fn maybe_sweep(&mut self) {
        if self.seen.len() > self.max_size || self.last_sweep.elapsed() > Duration::from_secs(300) {
            self.seen.clear();
            self.last_sweep = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let mut dedup = MessageDedup::new(1000);
        assert!(!dedup.is_duplicate("msg-1"));
        assert!(dedup.is_duplicate("msg-1"));
        assert!(!dedup.is_duplicate("msg-2"));
    }
}
