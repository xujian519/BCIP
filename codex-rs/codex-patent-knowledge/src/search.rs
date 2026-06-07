//! 统一搜索核心
//!
//! 聚合多源检索能力：专利知识图谱（SQLite FTS5）+ 法规数据库 + 知识卡片 + 语义向量索引。
//! 支持关键词增强和混合搜索两种模式，含查询结果缓存和实时线程池搜索。

use crate::cards::CardIndex;
use crate::embedding_client::EmbeddingClient;
use crate::graph::SqliteKnowledgeGraph;
use crate::keyword_search::KeywordSearch;
use crate::law_db::LawDatabase;
use crate::paths;
use crate::synonym::SynonymDict;
use crate::vector_index::VectorIndex;
use codex_patent_core::SearchResult;
use codex_patent_core::SearchSource;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Instant;

/// 搜索模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// 纯文本匹配
    Text,
    /// 关键词增强（默认）
    #[default]
    KeywordEnhanced,
    /// 传统 FTS 搜索
    Legacy,
    /// 关键词 + 语义混合搜索
    Hybrid,
}

/// 搜索配置
pub struct SearchConfig {
    pub query: String,
    pub limit: usize,
    pub search_kg: bool,
    pub search_law: bool,
    pub search_cards: bool,
    pub min_card_quality: f64,
    pub mode: SearchMode,
    pub prefix_filter: Option<String>,
    /// 语义搜索在前 top_k 条结果中的权重 (0.0-1.0)
    pub semantic_weight: f64,
    pub semantic_top_k: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            search_kg: true,
            search_law: true,
            search_cards: true,
            min_card_quality: 0.0,
            mode: SearchMode::KeywordEnhanced,
            prefix_filter: None,
            semantic_weight: 0.4,
            semantic_top_k: 20,
        }
    }
}

pub struct UnifiedSearch {
    kg: Option<Mutex<SqliteKnowledgeGraph>>,
    law_db: Option<LawDatabase>,
    card_index: Option<Mutex<CardIndex>>,
    synonym_dict: SynonymDict,
    vector_index: Option<VectorIndex>,
    embedding_client: Option<EmbeddingClient>,
    vector_available: bool,
    cache: Mutex<HashMap<String, (Instant, Vec<SearchResult>)>>,
}

const CACHE_TTL_SECS: u64 = 300;
const MAX_CACHE_ENTRIES: usize = 256;

static GLOBAL_SEARCH: OnceLock<UnifiedSearch> = OnceLock::new();

impl UnifiedSearch {
    /// 创建基础版统一搜索引擎（不含语义索引）
    pub fn new(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
    ) -> Self {
        let kg = kg_path
            .and_then(|p| {
                SqliteKnowledgeGraph::open(p)
                    .map_err(|e| {
                        tracing::warn!("打开知识图谱失败 (路径={p:?}): {e}");
                        e
                    })
                    .ok()
            })
            .map(Mutex::new);
        let law_db = law_db_path.and_then(|p| {
            LawDatabase::open(p)
                .map_err(|e| {
                    tracing::warn!("打开法规数据库失败 (路径={p:?}): {e}");
                    e
                })
                .ok()
        });
        let card_index = card_index_path
            .and_then(|p| {
                CardIndex::load(p)
                    .map_err(|e| {
                        tracing::warn!("加载知识卡片失败 (路径={p:?}): {e}");
                        e
                    })
                    .ok()
            })
            .map(Mutex::new);
        Self {
            kg,
            law_db,
            card_index,
            synonym_dict: SynonymDict::new(),
            vector_index: None,
            embedding_client: None,
            vector_available: false,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// 构造完整版：包含向量语义索引
    ///
    /// 使用 `std::thread::scope` 并行打开 4 个数据库，显著减少首次初始化延迟。
    pub fn with_vector(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
        semantic_index_path: Option<&str>,
        mlx_base_url: Option<&str>,
        mlx_api_key: Option<&str>,
        mlx_model: Option<&str>,
    ) -> Self {
        let kg_path_owned = kg_path.map(String::from);
        let law_db_path_owned = law_db_path.map(String::from);
        let card_index_path_owned = card_index_path.map(String::from);
        let semantic_index_path_owned = semantic_index_path.map(String::from);

        let (kg, law_db, card_index, vector_index) = std::thread::scope(|s| {
            let kg_h = s.spawn(move || {
                kg_path_owned.and_then(|p| {
                    SqliteKnowledgeGraph::open(&p)
                        .map_err(|e| {
                            tracing::warn!("打开知识图谱失败 (路径={p:?}): {e}");
                            e
                        })
                        .ok()
                })
            });
            let law_h = s.spawn(move || {
                law_db_path_owned.and_then(|p| {
                    LawDatabase::open(&p)
                        .map_err(|e| {
                            tracing::warn!("打开法规数据库失败 (路径={p:?}): {e}");
                            e
                        })
                        .ok()
                })
            });
            let card_h = s.spawn(move || {
                card_index_path_owned.and_then(|p| {
                    CardIndex::load(&p)
                        .map_err(|e| {
                            tracing::warn!("加载知识卡片失败 (路径={p:?}): {e}");
                            e
                        })
                        .ok()
                })
            });
            let vec_h = s.spawn(move || {
                semantic_index_path_owned.and_then(|p| {
                    VectorIndex::open(&p)
                        .map_err(|e| {
                            tracing::warn!("打开语义向量索引失败 (路径={p:?}): {e}");
                            e
                        })
                        .ok()
                })
            });

            (
                kg_h.join().unwrap_or(None),
                law_h.join().unwrap_or(None),
                card_h.join().unwrap_or(None),
                vec_h.join().unwrap_or(None),
            )
        });

        let kg = kg.map(Mutex::new);
        let card_index = card_index.map(Mutex::new);
        let embedding_client = mlx_base_url.map(|url| {
            EmbeddingClient::new(
                url,
                mlx_api_key.map(|k| k.to_string()),
                mlx_model.unwrap_or("bge-m3-mlx-8bit"),
            )
        });
        let vector_available =
            vector_index.is_some() && embedding_client.as_ref().is_some_and(|c| c.health_check());
        Self {
            kg,
            law_db,
            card_index,
            synonym_dict: SynonymDict::new(),
            vector_index,
            embedding_client,
            vector_available,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn build_global() -> Self {
        let mlx_key = paths::mlx_api_key();
        Self::with_vector(
            Some(&paths::kg_db_path()),
            Some(&paths::law_db_path()),
            Some(&paths::card_index_path()),
            Some(&paths::semantic_index_path()),
            Some(&paths::mlx_url()),
            mlx_key.as_deref(),
            Some(&paths::mlx_model()),
        )
    }

    /// 获取全局单例（带语义搜索），工具函数应优先使用此方法避免重复构建
    pub fn global() -> &'static Self {
        GLOBAL_SEARCH.get_or_init(Self::build_global)
    }

    /// 从默认路径构建不含语义搜索的版本
    pub fn new_from_defaults() -> Self {
        Self::new(
            Some(&paths::kg_db_path()),
            Some(&paths::law_db_path()),
            Some(&paths::card_index_path()),
        )
    }

    /// 直接访问内部 KG（供工具函数使用，避免重复 open_kg）
    pub fn kg(&self) -> Option<&Mutex<SqliteKnowledgeGraph>> {
        self.kg.as_ref()
    }

    /// 直接访问内部 CardIndex
    pub fn card_index(&self) -> Option<&Mutex<CardIndex>> {
        self.card_index.as_ref()
    }

    /// 执行多源搜索，按 `SearchConfig` 配置检索知识图谱/法规库/知识卡片/语义索引
    ///
    /// 使用 `std::thread::scope` 并行发起多个数据源的搜索，结果合并去重后按
    /// 相关性分数排序。支持 300 秒 TTL 的结果缓存。
    pub fn search(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let cache_key = format!(
            "{}|{}|{}|{}|{}|{}",
            config.query,
            config.limit,
            config.search_kg as u8,
            config.search_law as u8,
            config.search_cards as u8,
            config.mode as u8
        );

        {
            let mut cache = self.cache.lock().unwrap();
            if let Some((timestamp, cached)) = cache.get(&cache_key)
                && timestamp.elapsed().as_secs() < CACHE_TTL_SECS
            {
                tracing::debug!("知识检索缓存命中: query={}", config.query);
                return cached.clone();
            }
            if cache.len() >= MAX_CACHE_ENTRIES {
                cache.clear();
            }
        }
        let synonyms = self.synonym_dict.expand(&config.query);
        let mut all_terms: Vec<&str> = synonyms.to_vec();
        all_terms.push(&config.query);
        all_terms.sort();
        all_terms.dedup();

        let effective_limit = config.limit.min(50);
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::scope(|s| {
            if config.search_kg
                && let Some(ref kg_mutex) = self.kg
            {
                let tx = tx.clone();
                let all_terms = all_terms.clone();
                let config_query = config.query.clone();
                s.spawn(move || {
                    let kg = kg_mutex.lock().unwrap();
                    let mut results = Vec::new();
                    for term in all_terms.iter().take(5) {
                        if results.len() >= effective_limit {
                            break;
                        }
                        if let Ok(nodes) = kg.search_nodes(term, None, effective_limit) {
                            for node in nodes {
                                if results.len() >= effective_limit {
                                    break;
                                }
                                let content = node.content.clone().unwrap_or_default();
                                let score = compute_score(
                                    &config_query,
                                    &node.title,
                                    &content,
                                    &node.node_type,
                                );
                                results.push(SearchResult {
                                    source: SearchSource::KnowledgeGraph,
                                    title: if node.name.is_empty() {
                                        node.title.clone()
                                    } else {
                                        node.name.clone()
                                    },
                                    content,
                                    score,
                                    id: node.id.clone(),
                                    item_type: node.node_type.clone(),
                                    source_path: node.full_ref.clone().unwrap_or_default(),
                                    source_db: String::new(),
                                });
                            }
                        }
                    }
                    let _ = tx.send(results);
                });
            }

            if config.search_law
                && let Some(ref db) = self.law_db
            {
                let tx = tx.clone();
                let all_terms = all_terms.clone();
                let config_query = config.query.clone();
                s.spawn(move || {
                    let mut results = Vec::new();
                    for term in all_terms.iter().take(5) {
                        if results.len() >= effective_limit {
                            break;
                        }
                        if let Ok(laws) = db.search_by_content(term, effective_limit) {
                            for law in laws {
                                if results.len() >= effective_limit {
                                    break;
                                }
                                let score = compute_score(
                                    &config_query,
                                    &law.name,
                                    &law.content,
                                    &law.level,
                                );
                                let source_path = format!("{} ({})", law.name, law.level);
                                results.push(SearchResult {
                                    source: SearchSource::LawDatabase,
                                    title: law.name.clone(),
                                    content: law.content.clone(),
                                    score,
                                    id: law.id.clone(),
                                    item_type: law.level.clone(),
                                    source_path,
                                    source_db: "laws.db".into(),
                                });
                            }
                        }
                    }
                    let _ = tx.send(results);
                });
            }

            if config.search_cards
                && let Some(ref card_mutex) = self.card_index
            {
                let tx = tx.clone();
                let config_query = config.query.clone();
                let min_card_quality = config.min_card_quality;
                s.spawn(move || {
                    let index = card_mutex.lock().unwrap();
                    let cards = index.search_by_keyword(&config_query, effective_limit.min(10));
                    let mut results = Vec::new();
                    for card in cards {
                        if results.len() >= effective_limit {
                            break;
                        }
                        if card.quality < min_card_quality {
                            continue;
                        }
                        if let Ok(content) = index.load_content(card) {
                            let score =
                                compute_score(&config_query, &card.title, &content, &card.concept);
                            let source_path = format!("{} ({})", card.concept, card.domain);
                            results.push(SearchResult {
                                source: SearchSource::KnowledgeCard,
                                title: card.title.clone(),
                                content,
                                score,
                                id: card.id.clone(),
                                item_type: card.concept.clone(),
                                source_path,
                                source_db: "card-index.json".into(),
                            });
                        }
                    }
                    let _ = tx.send(results);
                });
            }

            if config.mode == SearchMode::Hybrid {
                if self.vector_available
                    && let (Some(client), Some(index)) =
                        (&self.embedding_client, &self.vector_index)
                {
                    let tx = tx.clone();
                    let config_query = config.query.clone();
                    let semantic_top_k = config.semantic_top_k;
                    let semantic_weight = config.semantic_weight;
                    s.spawn(move || {
                        let mut results = Vec::new();
                        if let Ok(query_embedding) = client.embed(&config_query) {
                            let semantic_results = index.search(&query_embedding, semantic_top_k);
                            for scored in &semantic_results {
                                if results.len() >= effective_limit {
                                    break;
                                }
                                let semantic_score = scored.score;
                                let kw_score = KeywordSearch::score_text_with_query(
                                    &config_query,
                                    &format!("{} {}", scored.chunk.title, scored.chunk.content),
                                );
                                let combined = kw_score * (1.0 - semantic_weight)
                                    + semantic_score * semantic_weight;
                                results.push(SearchResult {
                                    source: SearchSource::KnowledgeGraph,
                                    title: scored.chunk.title.clone(),
                                    content: scored.chunk.content.clone(),
                                    score: combined,
                                    id: scored.chunk.chunk_id.clone(),
                                    item_type: "semantic_chunk".into(),
                                    source_path: scored.chunk.file_path.clone(),
                                    source_db: "semantic-index.sqlite".into(),
                                });
                            }
                        }
                        let _ = tx.send(results);
                    });
                } else if let Some(ref kg_mutex) = self.kg {
                    // BM25 降级：MLX 服务不可用时，用 KG FTS5 扩展搜索替代语义搜索
                    let tx = tx.clone();
                    let config_query = config.query.clone();
                    let all_terms = all_terms.clone();
                    s.spawn(move || {
                        let kg = kg_mutex.lock().unwrap();
                        let mut results = Vec::new();
                        for term in all_terms.iter().take(3) {
                            if results.len() >= effective_limit / 2 {
                                break;
                            }
                            if let Ok(nodes) = kg.search_nodes(term, None, effective_limit / 2) {
                                for node in nodes {
                                    if results.len() >= effective_limit {
                                        break;
                                    }
                                    let content = node.content.clone().unwrap_or_default();
                                    let score = KeywordSearch::score_text_with_query(
                                        &config_query,
                                        &format!("{} {}", node.title, content),
                                    );
                                    if score > 0.1 {
                                        results.push(SearchResult {
                                            source: SearchSource::KnowledgeGraph,
                                            title: node.name.clone(),
                                            content,
                                            score: score * 0.8,
                                            id: format!("bm25_{}", node.id),
                                            item_type: "bm25_fallback".into(),
                                            source_path: node.full_ref.clone().unwrap_or_default(),
                                            source_db: "patent_kg.db (bm25)".into(),
                                        });
                                    }
                                }
                            }
                        }
                        let _ = tx.send(results);
                    });
                }
            }
        });

        drop(tx);

        let mut results: Vec<SearchResult> = Vec::new();
        let mut seen_ids = HashSet::new();
        for thread_results in rx {
            for result in thread_results {
                if seen_ids.insert(result.id.clone()) {
                    results.push(result);
                }
            }
        }

        if let Some(ref prefix) = config.prefix_filter {
            results.retain(|r| r.title.starts_with(prefix) || r.content.starts_with(prefix));
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(effective_limit);

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, (Instant::now(), results.clone()));
        }

        results
    }
}

fn compute_score(query: &str, title: &str, content: &str, _item_type: &str) -> f64 {
    let title_score = KeywordSearch::score_text_with_query(query, title);
    let content_score = KeywordSearch::score_text_with_query(query, content);
    let boost = if title.contains(query) { 0.2 } else { 0.0 };
    (title_score * 0.4 + content_score * 0.6 + boost).clamp(0.0, 1.0)
}

impl UnifiedSearch {
    /// 返回搜索引擎各组件可用性状态（JSON）
    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "knowledge_graph": self.kg.as_ref().and_then(|kg| kg.lock().ok().and_then(|g| g.stats().ok().map(|s| serde_json::json!({
                "available": true,
                "node_count": s.node_count,
                "edge_count": s.edge_count
            })))).unwrap_or(serde_json::json!({"available": false})),
            "law_database": self.law_db.as_ref().and_then(|db| db.count().ok().map(|c| serde_json::json!({
                "available": true,
                "count": c
            }))).unwrap_or(serde_json::json!({"available": false})),
            "knowledge_cards": self.card_index.as_ref().and_then(|idx| idx.lock().ok().map(|c| serde_json::json!({
                "available": true,
                "count": c.len()
            }))).unwrap_or(serde_json::json!({"available": false})),
            "vector_index": self.vector_index.as_ref().map(|vi| serde_json::json!({
                "available": true,
                "chunk_count": vi.len(),
                "dimension": vi.dimension(),
            })).unwrap_or(serde_json::json!({"available": false})),
            "mlx_service": self.vector_available,
            "search_mode": if self.vector_available { "hybrid" } else { "keyword_enhanced" },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_score_prefers_relevant() {
        let s1 = compute_score(
            "图像识别",
            "图像识别装置",
            "一种图像识别方法和装置，包括摄像头和处理器",
            "patent",
        );
        let s2 = compute_score("图像识别", "化工材料", "一种化工材料的制备方法", "patent");
        assert!(s1 > s2, "relevant should score higher: {s1} vs {s2}");
    }

    #[test]
    fn test_compute_score_title_boost() {
        let s = compute_score("图像识别", "图像识别装置", "其他技术内容", "patent");
        assert!(s > 0.0, "title match should give some score");
    }
}
