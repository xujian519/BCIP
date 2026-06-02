use std::num::NonZeroUsize;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use codex_utils_cache::BlockingLruCache;
use codex_utils_cache::sha1_digest;

/// 缓存的工具结果。
#[derive(Clone)]
struct CachedToolResult {
    output: String,
    inserted_at: Instant,
}

/// 按工具名配置的缓存策略。
struct CachePolicy {
    ttl: Duration,
    capacity: NonZeroUsize,
}

/// 工具结果缓存，按工具名分桶。
struct ToolResultCache {
    buckets: Vec<(String, BlockingLruCache<[u8; 20], CachedToolResult>)>,
    policies: Vec<(String, CachePolicy)>,
}

impl ToolResultCache {
    fn new(policies: Vec<(String, CachePolicy)>) -> Self {
        let buckets = policies
            .iter()
            .map(|(name, policy)| (name.clone(), BlockingLruCache::new(policy.capacity)))
            .collect();
        Self { buckets, policies }
    }

    fn get(&self, tool_name: &str, arguments: &str) -> Option<String> {
        let (bucket, policy) = self.find_bucket(tool_name)?;
        let key = cache_key(tool_name, arguments);
        let cached = bucket.get(&key)?;
        if cached.inserted_at.elapsed() > policy.ttl {
            return None;
        }
        Some(cached.output.clone())
    }

    fn insert(&self, tool_name: &str, arguments: &str, output: String) {
        let Some((bucket, _policy)) = self.find_bucket(tool_name) else {
            return;
        };
        let key = cache_key(tool_name, arguments);
        bucket.insert(
            key,
            CachedToolResult {
                output,
                inserted_at: Instant::now(),
            },
        );
    }

    fn find_bucket(
        &self,
        tool_name: &str,
    ) -> Option<(&BlockingLruCache<[u8; 20], CachedToolResult>, &CachePolicy)> {
        self.buckets
            .iter()
            .zip(self.policies.iter())
            .find(|((name, _), _)| name == tool_name)
            .map(|((_, cache), (_, policy))| (cache, policy))
    }
}

fn cache_key(tool_name: &str, arguments: &str) -> [u8; 20] {
    let mut input = String::with_capacity(tool_name.len() + 1 + arguments.len());
    input.push_str(tool_name);
    input.push('\0');
    input.push_str(arguments);
    sha1_digest(input.as_bytes())
}

// ── 静态配置 ──

static TOOL_RESULT_CACHE: LazyLock<ToolResultCache> = LazyLock::new(|| {
    ToolResultCache::new(vec![
        (
            "legal_qa".into(),
            CachePolicy {
                ttl: Duration::from_secs(3600),
                capacity: NonZeroUsize::new(200).unwrap(),
            },
        ),
        (
            "legal_basis_refs".into(),
            CachePolicy {
                ttl: Duration::from_secs(3600),
                capacity: NonZeroUsize::new(200).unwrap(),
            },
        ),
        (
            "patent_search".into(),
            CachePolicy {
                ttl: Duration::from_secs(300),
                capacity: NonZeroUsize::new(100).unwrap(),
            },
        ),
        (
            "format_rules".into(),
            CachePolicy {
                ttl: Duration::from_secs(7200),
                capacity: NonZeroUsize::new(200).unwrap(),
            },
        ),
        (
            "ipc_search".into(),
            CachePolicy {
                ttl: Duration::from_secs(7200),
                capacity: NonZeroUsize::new(200).unwrap(),
            },
        ),
    ])
});

/// 查询缓存，命中返回 Some(output)。
pub fn get_cached_result(tool_name: &str, arguments: &str) -> Option<String> {
    TOOL_RESULT_CACHE.get(tool_name, arguments)
}

/// 缓存工具结果。
pub fn cache_tool_result(tool_name: &str, arguments: &str, output: String) {
    TOOL_RESULT_CACHE.insert(tool_name, arguments, output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_deterministic() {
        let k1 = cache_key("test", r#"{"q":"hello"}"#);
        let k2 = cache_key("test", r#"{"q":"hello"}"#);
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_differs_for_different_args() {
        let k1 = cache_key("test", r#"{"q":"hello"}"#);
        let k2 = cache_key("test", r#"{"q":"world"}"#);
        assert_ne!(k1, k2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cache_hit_and_miss() {
        let cache = ToolResultCache::new(vec![(
            "test_tool".into(),
            CachePolicy {
                ttl: Duration::from_secs(60),
                capacity: NonZeroUsize::new(10).unwrap(),
            },
        )]);

        assert!(cache.get("test_tool", "args").is_none());
        cache.insert("test_tool", "args", "result".into());
        assert_eq!(cache.get("test_tool", "args"), Some("result".into()));
        assert!(cache.get("test_tool", "other_args").is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cache_miss_for_uncached_tool() {
        let cache = ToolResultCache::new(vec![(
            "cached_tool".into(),
            CachePolicy {
                ttl: Duration::from_secs(60),
                capacity: NonZeroUsize::new(10).unwrap(),
            },
        )]);
        cache.insert("uncached_tool", "args", "result".into());
        assert!(cache.get("uncached_tool", "args").is_none());
    }
}
