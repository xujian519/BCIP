//! 知识卡片索引。
//!
//! 从 `card-index.json` 加载概念卡片（知识图谱中的原子知识点），
//! 支持关键词搜索、概念关联搜索、质量过滤，以及卡片内容的延迟加载和缓存。

use codex_patent_core::KnowledgeCard;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

/// 知识卡片索引。
///
/// 管理一组 [`KnowledgeCard`]，提供基于关键词/概念的评分搜索，
/// 并支持按质量阈值过滤。卡片内容按需从文件系统加载，使用 `RefCell` 缓存。
pub struct CardIndex {
    cards: Vec<KnowledgeCard>,
    base_dir: String,
    content_cache: RefCell<HashMap<String, String>>,
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
                Err(e) => eprintln!("Warning: skipping invalid card entry: {e}"),
            }
        }

        Ok(Self {
            cards,
            base_dir,
            content_cache: RefCell::new(HashMap::new()),
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
            let cache = self.content_cache.borrow();
            if let Some(cached) = cache.get(&card.id) {
                return Ok(cached.clone());
            }
        }

        let path = format!("{}/{}", self.base_dir, card.file_path);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取卡片内容失败 {}: {e}", card.file_path))?;

        {
            let mut cache = self.content_cache.borrow_mut();
            if cache.len() > 200 {
                cache.clear();
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
