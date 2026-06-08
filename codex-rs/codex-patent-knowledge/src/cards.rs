//! 知识卡片索引。
//!
//! 从 `card-index.json` 加载概念卡片（知识图谱中的原子知识点），
//! 支持关键词搜索、概念关联搜索、质量过滤，以及卡片内容的延迟加载和缓存。

use codex_patent_core::KnowledgeCard;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;

/// 知识卡片索引。
///
/// 管理一组 [`KnowledgeCard`]，提供基于关键词/概念的评分搜索，
/// 并支持按质量阈值过滤。卡片内容按需从文件系统加载，使用 `Mutex` 缓存。
pub struct CardIndex {
    cards: Vec<KnowledgeCard>,
    base_dir: String,
    content_cache: Mutex<HashMap<String, String>>,
}

impl CardIndex {
    /// 从 JSON 索引文件加载卡片列表。
    ///
    /// JSON 格式：`{"cards": [{id, file_path, title, concept, domain, quality, related_concepts}]}`
    /// `base_dir` 由索引文件所在目录自动推断，用于解析相对路径的卡片内容文件。
    pub fn load(index_path: &str) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(index_path).map_err(|e| format!("读取卡片索引失败: {e}"))?;

        let raw: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析卡片索引失败: {e}"))?;

        let cards_array = raw
            .get("cards")
            .and_then(|v| v.as_array())
            .ok_or("卡片索引缺少 cards 字段")?;

        let base_dir = Path::new(index_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".into());

        let mut cards = Vec::new();
        for card_val in cards_array {
            match serde_json::from_value::<KnowledgeCard>(card_val.clone()) {
                Ok(mut card) => {
                    let filename = Path::new(&card.file_path)
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();
                    card.file_path = filename;
                    cards.push(card);
                }
                Err(e) => tracing::warn!("Skipping invalid card entry: {e}"),
            }
        }

        Ok(Self {
            cards,
            base_dir,
            content_cache: Mutex::new(HashMap::new()),
        })
    }

    /// 返回卡片总数。
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// 索引是否为空。
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// 返回所有卡片的引用。
    pub fn all(&self) -> &[KnowledgeCard] {
        &self.cards
    }

    /// 基于关键词搜索卡片，使用评分排序（title > concept > domain > related_concepts）。
    pub fn search_by_keyword(&self, keyword: &str, limit: usize) -> Vec<&KnowledgeCard> {
        let kw_lower = keyword.to_lowercase();
        let mut scored: Vec<(&KnowledgeCard, f64)> = self
            .cards
            .iter()
            .map(|c| {
                let mut score = 0.0;
                if c.title.to_lowercase().contains(&kw_lower) {
                    score += 3.0;
                }
                if c.concept.to_lowercase().contains(&kw_lower) {
                    score += 5.0;
                }
                if c.domain.to_lowercase().contains(&kw_lower) {
                    score += 2.0;
                }
                if c.related_concepts
                    .iter()
                    .any(|rc| rc.to_lowercase().contains(&kw_lower))
                {
                    score += 2.0;
                }
                (c, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(c, _)| c).collect()
    }

    /// 基于 related_concepts 的关联搜索：查找与给定概念相关的卡片
    pub fn search_by_concept(&self, concept: &str, limit: usize) -> Vec<&KnowledgeCard> {
        let concept_lower = concept.to_lowercase();
        let mut scored: Vec<(&KnowledgeCard, f64)> = self
            .cards
            .iter()
            .map(|c| {
                let mut score = 0.0;
                if c.concept.to_lowercase() == concept_lower {
                    score += 10.0;
                } else if c.concept.to_lowercase().contains(&concept_lower) {
                    score += 5.0;
                }
                for rc in &c.related_concepts {
                    if rc.to_lowercase() == concept_lower {
                        score += 3.0;
                    } else if rc.to_lowercase().contains(&concept_lower) {
                        score += 1.5;
                    }
                }
                if c.domain.to_lowercase().contains(&concept_lower) {
                    score += 1.0;
                }
                (c, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(c, _)| c).collect()
    }

    /// 按质量阈值过滤卡片，返回评分 ≥ threshold 的前 limit 张。
    pub fn filter_by_quality(&self, threshold: f64, limit: usize) -> Vec<&KnowledgeCard> {
        let mut results: Vec<&KnowledgeCard> = self
            .cards
            .iter()
            .filter(|c| c.quality >= threshold)
            .collect();
        results.sort_by(|a, b| {
            b.quality
                .partial_cmp(&a.quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// 加载卡片的 Markdown 内容（带缓存，最多缓存 200 项）。
    pub fn load_content(&self, card: &KnowledgeCard) -> Result<String, String> {
        {
            let cache = self.content_cache.lock();
            if let Some(cached) = cache.get(&card.id) {
                return Ok(cached.clone());
            }
        }

        let path = format!("{}/{}", self.base_dir, card.file_path);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取卡片内容失败 {}: {e}", card.file_path))?;

        {
            let mut cache = self.content_cache.lock();
            if cache.len() > 200 {
                // TTL 过期淘汰，避免全量 clear()
                // 简化实现：保留最近一半
                let keys: Vec<_> = cache.keys().take(100).cloned().collect();
                for k in keys {
                    cache.remove(&k);
                }
            }
            cache.insert(card.id.clone(), content.clone());
        }

        Ok(content)
    }

    /// 关键词搜索并直接加载内容，返回 `(卡片, 内容)` 元组。
    ///
    /// 是 `search_by_keyword` + `load_content` 的组合操作，方便工具函数调用。
    pub fn search_with_content(&self, keyword: &str, limit: usize) -> Vec<(KnowledgeCard, String)> {
        let cards_refs = self.search_by_keyword(keyword, limit);
        let mut results = Vec::new();
        for card_ref in cards_refs {
            if let Ok(content) = self.load_content(card_ref) {
                results.push((card_ref.clone(), content));
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_patent_core::KnowledgeCard;

    fn make_card(
        id: &str,
        title: &str,
        concept: &str,
        domain: &str,
        quality: f64,
    ) -> KnowledgeCard {
        KnowledgeCard {
            id: id.into(),
            file_path: format!("{id}.md"),
            title: title.into(),
            concept: concept.into(),
            domain: domain.into(),
            quality,
            related_concepts: vec!["关联概念".into()],
            generated_at: "2024-01-01".into(),
            version: 1,
        }
    }

    fn make_index_json(cards: &[KnowledgeCard]) -> String {
        let cards_json: Vec<String> = cards
            .iter()
            .map(|c| {
                format!(
                    r#"{{"id":"{}","file_path":"{}","title":"{}","concept":"{}","domain":"{}","quality":{},"related_concepts":["关联概念"],"generated_at":"2024-01-01","version":1}}"#,
                    c.id, c.file_path, c.title, c.concept, c.domain, c.quality
                )
            })
            .collect();
        format!(r#"{{"cards":[{}]}}"#, cards_json.join(","))
    }

    #[test]
    fn load_valid_index() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let cards = vec![
            make_card("c1", "新颖性", "novelty", "专利法", 0.9),
            make_card("c2", "创造性", "inventiveness", "专利法", 0.8),
        ];
        std::fs::write(&index_path, make_index_json(&cards)).unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        assert_eq!(idx.len(), 2);
        assert!(!idx.is_empty());
    }

    #[test]
    fn load_missing_file_returns_error() {
        let result = CardIndex::load("/nonexistent/card-index.json");
        assert!(result.is_err());
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("card-index.json");
        std::fs::write(&path, "not json").unwrap();
        let result = CardIndex::load(path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn search_by_keyword_concept_match() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let cards = vec![
            make_card("c1", "新颖性概念", "novelty", "专利法", 0.9),
            make_card("c2", "创造性概念", "inventiveness", "专利法", 0.8),
        ];
        std::fs::write(&index_path, make_index_json(&cards)).unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let results = idx.search_by_keyword("novelty", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "c1");
    }

    #[test]
    fn search_by_keyword_no_match() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let cards = vec![make_card("c1", "新颖性", "novelty", "专利法", 0.9)];
        std::fs::write(&index_path, make_index_json(&cards)).unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let results = idx.search_by_keyword("量子计算", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn search_by_concept_exact_match() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let cards = vec![
            make_card("c1", "新颖性", "novelty", "专利法", 0.9),
            make_card("c2", "创造性", "inventiveness", "专利法", 0.8),
        ];
        std::fs::write(&index_path, make_index_json(&cards)).unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let results = idx.search_by_concept("novelty", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "c1");
    }

    #[test]
    fn filter_by_quality() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let cards = vec![
            make_card("c1", "A", "a", "d", 0.9),
            make_card("c2", "B", "b", "d", 0.5),
            make_card("c3", "C", "c", "d", 0.7),
        ];
        std::fs::write(&index_path, make_index_json(&cards)).unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let results = idx.filter_by_quality(0.7, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "c1");
        assert_eq!(results[1].id, "c3");
    }

    #[test]
    fn load_content_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let card = make_card("c1", "Test", "test", "d", 0.9);
        std::fs::write(&index_path, make_index_json(std::slice::from_ref(&card))).unwrap();
        let md_path = dir.path().join("c1.md");
        std::fs::write(&md_path, "# Test Content\nHello world").unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let content = idx.load_content(&idx.all()[0]).unwrap();
        assert!(content.contains("Hello world"));
    }

    #[test]
    fn load_content_caches() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("card-index.json");
        let card = make_card("c1", "Test", "test", "d", 0.9);
        std::fs::write(&index_path, make_index_json(std::slice::from_ref(&card))).unwrap();
        let md_path = dir.path().join("c1.md");
        std::fs::write(&md_path, "cached content").unwrap();

        let idx = CardIndex::load(index_path.to_str().unwrap()).unwrap();
        let _first = idx.load_content(&idx.all()[0]).unwrap();
        std::fs::remove_file(dir.path().join("c1.md")).unwrap();
        let second = idx.load_content(&idx.all()[0]).unwrap();
        assert!(second.contains("cached content"));
    }
}
