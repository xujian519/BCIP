//! 专利知识检索系统
//!
//! 集成多种知识来源的统一检索 crate：
//! - **知识图谱**：基于 SQLite FTS5 的专利/法律知识图谱（`graph.rs`）
//! - **法规数据库**：法律法规全文检索（`law_db.rs`）
//! - **知识卡片**：JSON 索引的概念卡片检索（`cards.rs`）
//! - **语义搜索**：BGE-M3 向量索引的混合搜索（`vector_index.rs` 和 `embedding_client.rs`）
//! - **CNIPA 公布公告**：构建 CNIPA 搜索 URL 并解析 HTML 结果（`cnipa.rs`）
//! - **双链图**：知识库 Wiki 双链引用图（`link_graph.rs`）
//! - **同义词扩展**：专利领域的同义词词典（`synonym.rs`）
//! - **搜索评估**：基于标注数据的检索精度评估（`search_eval.rs`）
//! - **增量刷新**：检测知识库文件变更的刷新流水线（`refresh_pipeline.rs`）
//!
//! 核心入口是 [`UnifiedSearch`]，支持多源并行检索和结果融合。

pub mod cards;
pub mod citation_tracker;
pub mod embedding_client;
pub mod graph;
pub mod keyword_search;
pub mod law_db;
pub mod link_graph;
pub mod paths;
pub mod refresh_pipeline;
pub mod search;
pub mod search_eval;
pub mod synonym;
pub mod vector_index;

pub use cards::CardIndex;
pub use citation_tracker::CitationTracker;
pub use graph::IpcSearchResult;
pub use graph::SqliteKnowledgeGraph;
pub use keyword_search::KeywordSearch;
pub use law_db::LawDatabase;
pub use link_graph::LinkGraph;
pub use refresh_pipeline::RefreshPipeline;
pub use search::SearchConfig;
pub use search::SearchMode;
pub use search::UnifiedSearch;
pub use search_eval::SearchEval;
pub use vector_index::VectorIndex;

pub mod cnipa;
pub mod semantic_memory;
pub use cnipa::CnipaParser;
pub use cnipa::CnipaSearchBuilder;
pub use semantic_memory::MemoryEntry;
pub use semantic_memory::SemanticMemoryStore;
