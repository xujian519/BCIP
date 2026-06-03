//! 混合检索引擎 — 向量 + 法律库 + 图谱融合检索。

use serde::Deserialize;
use serde::Serialize;

/// 检索片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedSnippet {
    pub source: String,
    pub title: String,
    pub text: String,
    pub score: f64,
}

/// 混合检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub snippets: Vec<RetrievedSnippet>,
    pub graph_expanded: bool,
    pub total_sources: usize,
}

/// 混合检索配置
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub vector_weight: f64,
    pub graph_weight: f64,
    pub legal_weight: f64,
    pub top_k: usize,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.4,
            graph_weight: 0.3,
            legal_weight: 0.3,
            top_k: 10,
        }
    }
}

/// 混合检索器
pub struct HybridSearcher {
    config: HybridSearchConfig,
}

impl HybridSearcher {
    pub fn new(config: HybridSearchConfig) -> Self {
        Self { config }
    }

    /// 执行混合检索
    pub fn search(
        &self,
        _query: &str,
        vector_results: Vec<RetrievedSnippet>,
        graph_results: Vec<RetrievedSnippet>,
        legal_results: Vec<RetrievedSnippet>,
    ) -> HybridSearchResult {
        let graph_expanded = !graph_results.is_empty();
        let mut all_snippets = Vec::new();

        // 加权合并
        for mut s in vector_results {
            s.score *= self.config.vector_weight;
            all_snippets.push(s);
        }
        for mut s in graph_results {
            s.score *= self.config.graph_weight;
            all_snippets.push(s);
        }
        for mut s in legal_results {
            s.score *= self.config.legal_weight;
            all_snippets.push(s);
        }

        // 按分数排序
        all_snippets.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 去重(基于标题)
        let mut seen = std::collections::HashSet::new();
        let mut unique = Vec::new();
        for s in all_snippets {
            if seen.insert(s.title.clone()) {
                unique.push(s);
            }
        }

        // 限制数量
        let snippets: Vec<_> = unique.into_iter().take(self.config.top_k).collect();

        HybridSearchResult {
            total_sources: snippets.len(),
            graph_expanded,
            snippets,
        }
    }
}

/// 引用扩展 — 在原始引用列表基础上添加关联引用。
///
/// 对以 "D" 开头的引用，自动追加 `{id}-related` 形式的关联项。
/// 用于模拟用户点击后的引用链条展开。
pub fn expand_citations(citations: &[String]) -> Vec<String> {
    let mut expanded = Vec::new();
    for citation in citations {
        expanded.push(citation.clone());
        // 模拟二跳扩展:添加相关引用
        if citation.starts_with("D") {
            expanded.push(format!("{citation}-related"));
        }
    }
    expanded
}

/// 根据引用次数对现有技术结果进行加权提升。
///
/// 引用越多，加权因子越大（上限 1.5×）。仅作用于来源为 `patent_db` 的结果。
pub fn boost_prior_art(results: &mut [RetrievedSnippet], citation_count: u32) {
    let boost = 1.0 + (citation_count as f64 * 0.05).min(0.5);
    for result in results.iter_mut() {
        if result.source == "patent_db" {
            result.score *= boost;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_search() {
        let searcher = HybridSearcher::new(HybridSearchConfig::default());

        let vector = vec![RetrievedSnippet {
            source: "vector".into(),
            title: "向量结果1".into(),
            text: "内容1".into(),
            score: 0.9,
        }];
        let graph = vec![RetrievedSnippet {
            source: "graph".into(),
            title: "图谱结果1".into(),
            text: "内容2".into(),
            score: 0.8,
        }];
        let legal = vec![];

        let result = searcher.search("测试", vector, graph, legal);
        assert!(!result.snippets.is_empty());
    }

    #[test]
    fn test_citation_expand() {
        let citations = vec!["D1".into(), "D2".into()];
        let expanded = expand_citations(&citations);
        assert!(expanded.len() >= 2);
    }

    #[test]
    fn test_prior_art_boost() {
        let mut results = vec![RetrievedSnippet {
            source: "patent_db".into(),
            title: "专利1".into(),
            text: "内容".into(),
            score: 0.8,
        }];
        boost_prior_art(&mut results, 10);
        assert!(results[0].score > 0.8);
    }
}
