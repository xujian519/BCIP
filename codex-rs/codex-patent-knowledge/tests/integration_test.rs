use codex_patent_knowledge::CardIndex;
use codex_patent_knowledge::LawDatabase;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SqliteKnowledgeGraph;
use codex_patent_knowledge::UnifiedSearch;

#[test]
fn test_open_knowledge_graph() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db");
    assert!(kg.is_ok(), "Should open patent_kg.db");
    let kg = kg.unwrap();
    let stats = kg.stats().unwrap();
    assert!(stats.node_count > 0);
    assert!(stats.edge_count > 0);
}

#[test]
fn test_search_knowledge_graph() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();
    let results = kg.search_nodes("新颖性", None, 5).unwrap();
    assert!(!results.is_empty(), "Should find results for '新颖性'");
}

#[test]
fn test_open_law_database() {
    let db = LawDatabase::open("../codex-patent-assets/laws.db");
    assert!(db.is_ok(), "Should open laws.db");
    let db = db.unwrap();
    let count = db.count().unwrap();
    assert!(count > 0);
}

#[test]
fn test_search_law_database() {
    let db = LawDatabase::open("../codex-patent-assets/laws.db").unwrap();
    let results = db.search_by_name("专利法", 10).unwrap();
    assert!(!results.is_empty(), "Should find '专利法'");
}

#[test]
fn test_load_card_index() {
    let idx = CardIndex::load("../codex-patent-assets/card-index.json");
    assert!(idx.is_ok(), "Should load card-index.json");
    let idx = idx.unwrap();
    assert!(idx.len() > 0, "Should have cards");
    assert!(idx.len() >= 100, "Expected >=100 cards, got {}", idx.len());
}

#[test]
fn test_unified_search() {
    let search = UnifiedSearch::new(
        Some("../codex-patent-assets/patent_kg.db"),
        Some("../codex-patent-assets/laws.db"),
        Some("../codex-patent-assets/card-index.json"),
    );
    let config = SearchConfig {
        query: "新颖性".to_string(),
        limit: 10,
        ..Default::default()
    };
    let results = search.search(&config);
    assert!(!results.is_empty(), "Should find results for '新颖性'");
}

#[test]
fn test_knowledge_status() {
    let search = UnifiedSearch::new(
        Some("../codex-patent-assets/patent_kg.db"),
        Some("../codex-patent-assets/laws.db"),
        Some("../codex-patent-assets/card-index.json"),
    );
    let status = search.status();
    assert!(status.get("knowledge_graph").is_some());
    assert!(status.get("law_database").is_some());
    assert!(status.get("knowledge_cards").is_some());
}
