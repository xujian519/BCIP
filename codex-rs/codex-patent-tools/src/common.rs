//! 通用工具函数。
//!
//! 提供各模块共享的默认值和辅助函数。

/// 默认搜索限制数量。
pub fn default_limit() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_limit_returns_10() {
        assert_eq!(default_limit(), 10);
    }
}
