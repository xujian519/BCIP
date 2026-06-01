use codex_patent_knowledge::CardIndex;
use codex_patent_knowledge::LawDatabase;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SqliteKnowledgeGraph;
use codex_patent_knowledge::UnifiedSearch;
use codex_patent_knowledge::VectorIndex;

#[test]
#[ignore = "requires local asset files"]
fn test_open_knowledge_graph() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db");
    assert!(kg.is_ok(), "Should open patent_kg.db");
    let kg = kg.unwrap();
    let stats = kg.stats().unwrap();
    assert!(stats.node_count > 0);
    assert!(stats.edge_count > 0);
}

#[test]
#[ignore = "requires local asset files"]
fn test_search_knowledge_graph() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();
    let results = kg.search_nodes("新颖性", None, 5).unwrap();
    assert!(!results.is_empty(), "Should find results for '新颖性'");
}

#[test]
#[ignore = "requires local asset files"]
fn test_open_law_database() {
    let db = LawDatabase::open("../codex-patent-assets/laws.db");
    assert!(db.is_ok(), "Should open laws.db");
    let db = db.unwrap();
    let count = db.count().unwrap();
    assert!(count > 0);
}

#[test]
#[ignore = "requires local asset files"]
fn test_search_law_database() {
    let db = LawDatabase::open("../codex-patent-assets/laws.db").unwrap();
    let results = db.search_by_name("专利法", 10).unwrap();
    assert!(!results.is_empty(), "Should find '专利法'");
}

#[test]
#[ignore = "requires local asset files"]
fn test_load_card_index() {
    let idx = CardIndex::load("../codex-patent-assets/card-index.json");
    assert!(idx.is_ok(), "Should load card-index.json");
    let idx = idx.unwrap();
    assert!(!idx.is_empty(), "Should have cards");
    assert!(idx.len() >= 100, "Expected >=100 cards, got {}", idx.len());
}

#[test]
#[ignore = "requires local asset files"]
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
#[ignore = "requires local asset files"]
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

#[test]
#[ignore = "requires local asset files"]
fn test_vector_index_open() {
    let idx = VectorIndex::open("../codex-patent-assets/.yunpat-semantic-index.sqlite");
    assert!(idx.is_ok(), "Should open semantic index");
    let idx = idx.unwrap();
    assert!(idx.len() >= 100, "Expected >=100 chunks, got {}", idx.len());
    assert_eq!(idx.dimension(), 1024, "BGE-M3 should have 1024 dim");
}

#[test]
#[ignore = "requires local asset files"]
fn test_vector_index_search() {
    let idx = VectorIndex::open("../codex-patent-assets/.yunpat-semantic-index.sqlite").unwrap();
    let dummy_embedding = vec![0.0f32; 1024];
    let results = idx.search(&dummy_embedding, 5);
    assert!(results.is_empty(), "Zero-vector query should return empty");
}

#[test]
#[ignore = "requires local asset files"]
fn test_vector_index_search_relevant() {
    let idx = VectorIndex::open("../codex-patent-assets/.yunpat-semantic-index.sqlite").unwrap();
    let mut query = vec![0.0f32; 1024];
    query[0] = 1.0;
    let results = idx.search(&query, 3);
    assert!(!results.is_empty());
    if results.len() > 1 {
        assert!(results[0].score >= results[1].score);
    }
}

#[test]
#[ignore = "requires local asset files"]
fn test_unified_search_status_with_vector() {
    let search = UnifiedSearch::with_vector(
        Some("../codex-patent-assets/patent_kg.db"),
        Some("../codex-patent-assets/laws.db"),
        Some("../codex-patent-assets/card-index.json"),
        Some("../codex-patent-assets/.yunpat-semantic-index.sqlite"),
        None,
        None,
        None,
    );
    let status = search.status();
    let vi = status.get("vector_index").unwrap();
    assert_eq!(vi["available"], true);
    assert!(vi["chunk_count"].as_u64().unwrap_or(0) >= 100);
    assert_eq!(vi["dimension"].as_u64().unwrap_or(0), 1024);
}

#[test]
#[ignore = "requires local asset files"]
fn test_graph_traverse() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();
    let nodes = kg.search_nodes("新颖性", None, 1).unwrap();
    if nodes.is_empty() {
        return;
    }
    let start_id = &nodes[0].id;
    let edges = kg.traverse(start_id, None, 1).unwrap();
    assert!(
        !edges.is_empty(),
        "Should have at least 1 edge from the node"
    );
    let (edge, depth) = &edges[0];
    assert!(*depth >= 1);
    assert!(!edge.source.is_empty());
    assert!(!edge.target.is_empty());
}

#[test]
#[ignore = "requires local asset files"]
fn test_graph_traverse_with_filter() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();
    let nodes = kg.search_nodes("创造性", None, 1).unwrap();
    if nodes.is_empty() {
        return;
    }
    let start_id = &nodes[0].id;
    let filter = ["RELATED_TO", "CITES"];
    let edges = kg.traverse(start_id, Some(&filter), 2).unwrap();
    for (edge, _depth) in &edges {
        assert!(
            edge.relation == "RELATED_TO" || edge.relation == "CITES",
            "Unexpected relation: {}",
            edge.relation
        );
    }
}

#[test]
#[ignore = "requires local asset files"]
fn test_graph_find_path() {
    let kg = SqliteKnowledgeGraph::open("../codex-patent-assets/patent_kg.db").unwrap();
    let nodes_a = kg.search_nodes("新颖性", None, 1).unwrap();
    let nodes_b = kg.search_nodes("创造性", None, 1).unwrap();
    if nodes_a.is_empty() || nodes_b.is_empty() {
        return;
    }
    let paths = kg.find_path(&nodes_a[0].id, &nodes_b[0].id, 3).unwrap();
    if !paths.is_empty() {
        for path in &paths {
            assert!(!path.is_empty(), "Each path should have edges");
        }
    }
}
