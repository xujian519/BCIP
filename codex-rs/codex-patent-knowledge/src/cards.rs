use codex_patent_core::KnowledgeCard;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

pub struct CardIndex {
    cards: Vec<KnowledgeCard>,
    base_dir: String,
    content_cache: RefCell<HashMap<String, String>>,
}

impl CardIndex {
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

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn all(&self) -> &[KnowledgeCard] {
        &self.cards
    }

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
