use codex_patent_knowledge::CitationTracker;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SearchMode;
use codex_patent_knowledge::UnifiedSearch;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoKnowledgeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub inject_kg: bool,
    #[serde(default = "default_true")]
    pub inject_law: bool,
    #[serde(default = "default_true")]
    pub inject_cards: bool,
    #[serde(default)]
    pub semantic: bool,
    #[serde(default = "default_max_context")]
    pub max_context_items: usize,
}

fn default_true() -> bool {
    true
}
fn default_max_context() -> usize {
    5
}

impl Default for AutoKnowledgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            inject_kg: true,
            inject_law: true,
            inject_cards: true,
            semantic: false,
            max_context_items: 5,
        }
    }
}

pub struct RoleKeywords;

impl RoleKeywords {
    pub fn for_role(role_id: &str) -> &'static str {
        match role_id {
            "novelty_checker" => "新颖性 单独对比 现有技术 抵触申请 实质审查 对比文件",
            "creativity_checker" => "创造性 三步法 技术启示 非显而易见 结合启示 最接近现有技术",
            "infringement_checker" => "全面覆盖 等同侵权 合法来源 间接侵权 保护范围 权利要求解释",
            "invalidity_checker" => "无效宣告 修改超范围 理由变更 证据认定 无效理由 请求人",
            "writer" => {
                "权利要求 说明书 充分公开 形式要求 撰写规范 必要技术特征 独立权利要求 从属权利要求 实施例 有益效果 技术领域 背景技术 具体实施方式 摘要 其特征在于"
            }
            "analyzer" => {
                "权利要求 技术特征 保护范围 功能性特征 最小技术单元 独立权利要求 从属权利要求"
            }
            "retriever" => "专利检索 检索式 对比文件 IPC分类 关键词 语义检索 布尔检索",
            "reviewer" => {
                "形式审查 格式规范 CNIPA 权利要求格式 说明书完整性 发明名称 附图说明 附图标记 引用检查 统一性 具体实施方式 摘要 独立权利要求 从属权利要求 商业宣传 术语一致性"
            }
            "quality_checker" => {
                "质量评估 清楚性 支持性 保护范围 可实施性 权利要求层次 模糊用语 商业用语 实施例 充分公开 形式审查 法言法语 单一性 技术领域 背景技术 发明内容 具体实施方式 禁止用语 术语一致性"
            }
            _ => "专利法 新颖性 创造性 实用性",
        }
    }

    pub fn for_role_and_task(role_id: &str, task: &str) -> String {
        let base = Self::for_role(role_id);
        if task.is_empty() {
            base.to_string()
        } else {
            format!("{base} {task}")
        }
    }
}

pub struct KnowledgeContext {
    search: &'static UnifiedSearch,
    config: AutoKnowledgeConfig,
}

impl KnowledgeContext {
    pub fn new(
        _kg_path: &str,
        _law_db_path: &str,
        _card_index_path: &str,
        _semantic_index_path: Option<&str>,
        config: AutoKnowledgeConfig,
    ) -> Self {
        Self {
            search: UnifiedSearch::global(),
            config,
        }
    }

    /// 为指定角色和任务生成知识上下文（注入到 Agent prompt 中）
    pub fn resolve(&self, role_id: &str, task_description: &str) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let combined_query = RoleKeywords::for_role_and_task(role_id, task_description);
        let mode = if self.config.semantic {
            SearchMode::Hybrid
        } else {
            SearchMode::KeywordEnhanced
        };
        let config = SearchConfig {
            query: combined_query,
            limit: self.config.max_context_items,
            search_kg: self.config.inject_kg,
            search_law: self.config.inject_law,
            search_cards: self.config.inject_cards,
            mode,
            ..Default::default()
        };
        let results = self.search.search(&config);
        CitationTracker::citation_prefix(&results)
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roles::PatentAgentRole;

    #[test]
    fn test_role_keywords_all_9_roles() {
        for role in PatentAgentRole::all() {
            let kw = RoleKeywords::for_role(role.role_id());
            assert!(
                !kw.is_empty(),
                "missing keywords for role: {}",
                role.role_id()
            );
        }
    }

    #[test]
    fn test_disabled_context_returns_empty() {
        let ctx = KnowledgeContext::new(
            "../codex-patent-assets/patent_kg.db",
            "../codex-patent-assets/laws.db",
            "../codex-patent-assets/card-index.json",
            None,
            AutoKnowledgeConfig {
                enabled: false,
                ..Default::default()
            },
        );
        let result = ctx.resolve("novelty_checker", "判断新颖性");
        assert!(result.is_empty());
    }

    #[test]
    #[ignore = "requires local asset files"]
    fn test_enabled_context_returns_non_empty() {
        let ctx = KnowledgeContext::new(
            "../codex-patent-assets/patent_kg.db",
            "../codex-patent-assets/laws.db",
            "../codex-patent-assets/card-index.json",
            None,
            AutoKnowledgeConfig {
                enabled: true,
                max_context_items: 3,
                ..Default::default()
            },
        );
        let result = ctx.resolve("novelty_checker", "新颖性判断");
        assert!(!result.is_empty());
        assert!(result.contains("基于以下知识来源"));
    }

    #[test]
    fn test_role_keywords_for_role_and_task() {
        let q = RoleKeywords::for_role_and_task("creativity_checker", "判断某个发明的创造性");
        assert!(q.contains("三步法"));
        assert!(q.contains("判断某个发明的创造性"));
    }
}
