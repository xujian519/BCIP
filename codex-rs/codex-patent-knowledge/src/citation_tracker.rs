use codex_patent_core::SearchResult;

#[derive(Debug, Clone)]
pub struct CitationMeta {
    pub source_title: String,
    pub source_path: String,
    pub source_db: String,
    pub item_type: String,
    pub score: f64,
}

pub struct CitationTracker;

impl CitationTracker {
    pub fn extract(result: &SearchResult) -> CitationMeta {
        CitationMeta {
            source_title: result.title.clone(),
            source_path: result.source_path.clone(),
            source_db: result.source_db.clone(),
            item_type: result.item_type.clone(),
            score: result.score,
        }
    }

    /// 生成引用列表
    pub fn format_citations(results: &[SearchResult]) -> String {
        let mut citations = String::from("\n## 引用来源\n\n");
        for (i, r) in results.iter().enumerate() {
            citations.push_str(&format!("{}. ", i + 1));
            citations.push_str(&r.title);
            if !r.source_path.is_empty() {
                citations.push_str(&format!(" (来源: {})", r.source_path));
            }
            if !r.source_db.is_empty() {
                citations.push_str(&format!(" [{}]", r.source_db));
            }
            citations.push_str(&format!(" - 相关度: {:.2}\n", r.score));
        }
        citations
    }

    /// 为 Agent prompt 注入生成简洁引用前缀
    pub fn citation_prefix(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return String::new();
        }
        let mut prefix = String::from("基于以下知识来源:\n");
        for r in results.iter().take(5) {
            let source = if !r.source_path.is_empty() {
                r.source_path.clone()
            } else {
                r.title.clone()
            };
            prefix.push_str(&format!("- {}\n", source));
        }
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_patent_core::SearchSource;

    #[test]
    fn test_citation_format() {
        let results = vec![SearchResult {
            source: SearchSource::KnowledgeGraph,
            title: "专利法第22条第3款".into(),
            content: "创造性...".into(),
            score: 0.95,
            id: "law_22_3".into(),
            item_type: "Law".into(),
            source_path: "专利法-2020#第22条".into(),
            source_db: "laws.db".into(),
        }];
        let citations = CitationTracker::format_citations(&results);
        assert!(citations.contains("专利法第22条第3款"));
        assert!(citations.contains("专利法-2020#第22条"));
        assert!(citations.contains("laws.db"));
    }

    #[test]
    fn test_citation_prefix() {
        let results = vec![SearchResult {
            source: SearchSource::LawDatabase,
            title: "审查指南第二部分第四章".into(),
            content: "...".into(),
            score: 0.8,
            id: "guide_2_4".into(),
            item_type: "Guideline".into(),
            source_path: "创造性审查".into(),
            source_db: "patent_kg.db".into(),
        }];
        let prefix = CitationTracker::citation_prefix(&results);
        assert!(prefix.contains("创造性审查"));
    }

    #[test]
    fn test_empty_results() {
        let prefix = CitationTracker::citation_prefix(&[]);
        assert!(prefix.is_empty());
    }
}
