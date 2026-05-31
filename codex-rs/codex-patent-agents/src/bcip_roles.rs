//! BCIP 内置 Agent 角色注册
//!
//! 提供专利领域 9 个专业角色的 BCIP 格式配置。
//!
//! ## 使用方式
//!
//! 这些角色可以通过 BCIP 的 Agent 系统调用，或通过 @角色名 方式触发。
//!
//! ## 角色列表
//!
//! - `retriever`: 检索专家 — 多源专利检索、检索式构建和对比文件筛选
//! - `analyzer`: 分析专家 — 权利要求解析、技术特征提取、四层对比和本质识别
//! - `writer`: 撰写专家 — 专利文件撰写（说明书/权利要求/摘要）
//! - `novelty_checker`: 新颖性评估专家 — 三步法新颖性判断、逐特征对比
//! - `creativity_checker`: 创造性评估专家 — 问题-解决方案法创造力分析
//! - `infringement_checker`: 侵权分析专家 — 全面覆盖+等同原则侵权分析
//! - `invalidity_checker`: 无效分析专家 — 无效理由和证据分析
//! - `reviewer`: 文件审查专家 — 格式规范和内容质量审查
//! - `quality_checker`: 质量评估专家 — 多维度专利质量评估
//!
//! ## 配置文件路径
//!
//! 这些角色的配置文件位于 `codex-rs/codex-patent-agents/assets/bcip/` 目录下，
//! 采用 BCIP Agent Role 配置格式（包含 `developer_instructions` 字段）。

use std::collections::BTreeMap;
use std::path::Path;

/// 返回所有专利 Agent 角色的配置映射
///
/// 键是角色名称，值是配置文件的相对路径。
pub fn patent_agent_role_configs() -> BTreeMap<String, &'static Path> {
    let mut roles = BTreeMap::new();

    // 注册所有专利角色，配置文件位于 codex-patent-agents crate 内部
    // 这些路径会在运行时由 codex-patent-agents crate 通过 include_str! 加载
    roles.insert("retriever".to_string(), Path::new("patent/retriever.toml"));
    roles.insert("analyzer".to_string(), Path::new("patent/analyzer.toml"));
    roles.insert("writer".to_string(), Path::new("patent/writer.toml"));
    roles.insert(
        "novelty_checker".to_string(),
        Path::new("patent/novelty_checker.toml"),
    );
    roles.insert(
        "creativity_checker".to_string(),
        Path::new("patent/creativity_checker.toml"),
    );
    roles.insert(
        "infringement_checker".to_string(),
        Path::new("patent/infringement_checker.toml"),
    );
    roles.insert(
        "invalidity_checker".to_string(),
        Path::new("patent/invalidity_checker.toml"),
    );
    roles.insert("reviewer".to_string(), Path::new("patent/reviewer.toml"));
    roles.insert(
        "quality_checker".to_string(),
        Path::new("patent/quality_checker.toml"),
    );

    roles
}

/// 获取角色的配置文件内容
///
/// 提供编译时嵌入的配置文件内容，避免运行时文件读取。
/// 注意：此函数需要配合 include_str! 宏在调用处使用。
///
/// ## 使用示例
///
/// ```rust,ignore
/// let retriever_config = include_str!("../assets/bcip/retriever.toml");
/// let analyzer_config = include_str!("../assets/bcip/analyzer.toml");
/// // ... 其他角色配置
/// ```
pub fn config_file_contents(path: &Path) -> Option<&'static str> {
    // 这些内容将在调用处通过 include_str! 嵌入
    // 这里只提供路径映射逻辑
    const CONFIGS: &[(&str, &str)] = &[
        (
            "patent/retriever.toml",
            include_str!("../assets/bcip/retriever.toml"),
        ),
        (
            "patent/analyzer.toml",
            include_str!("../assets/bcip/analyzer.toml"),
        ),
        (
            "patent/writer.toml",
            include_str!("../assets/bcip/writer.toml"),
        ),
        (
            "patent/novelty_checker.toml",
            include_str!("../assets/bcip/novelty_checker.toml"),
        ),
        (
            "patent/creativity_checker.toml",
            include_str!("../assets/bcip/creativity_checker.toml"),
        ),
        (
            "patent/infringement_checker.toml",
            include_str!("../assets/bcip/infringement_checker.toml"),
        ),
        (
            "patent/invalidity_checker.toml",
            include_str!("../assets/bcip/invalidity_checker.toml"),
        ),
        (
            "patent/reviewer.toml",
            include_str!("../assets/bcip/reviewer.toml"),
        ),
        (
            "patent/quality_checker.toml",
            include_str!("../assets/bcip/quality_checker.toml"),
        ),
    ];

    CONFIGS
        .iter()
        .find(|(p, _)| *p == path.to_str().unwrap_or(""))
        .map(|(_, content)| *content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patent_agent_role_configs() {
        let configs = patent_agent_role_configs();
        assert_eq!(configs.len(), 9);
        assert!(configs.contains_key("retriever"));
        assert!(configs.contains_key("analyzer"));
        assert!(configs.contains_key("writer"));
        assert!(configs.contains_key("novelty_checker"));
        assert!(configs.contains_key("creativity_checker"));
        assert!(configs.contains_key("infringement_checker"));
        assert!(configs.contains_key("invalidity_checker"));
        assert!(configs.contains_key("reviewer"));
        assert!(configs.contains_key("quality_checker"));
    }

    #[test]
    fn test_config_file_contents() {
        use std::path::Path;
        let path = Path::new("patent/retriever.toml");
        let contents = config_file_contents(path);
        assert!(contents.is_some());
        let contents = contents.unwrap();
        assert!(contents.contains("专利检索专家"));
        assert!(contents.contains("developer_instructions"));
    }
}
