//! Integration tests for codex-patent-core serde roundtrips and Display impls.

use codex_patent_core::*;
use pretty_assertions::assert_eq;
use serde_json;

// ── 1. PatentDocument roundtrip ──

#[test]
fn patent_document_serde_roundtrip() {
    let doc = PatentDocument {
        title: Some("图像识别装置".into()),
        abstract_text: Some("一种基于深度学习的图像识别装置".into()),
        claims: vec!["1. 一种图像识别装置，其特征在于...".into()],
        specification: Some("具体实施方式...".into()),
        drawings: vec!["图1".into()],
    };
    let json = serde_json::to_string(&doc).unwrap();
    let back: PatentDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, doc.title);
    assert_eq!(back.abstract_text, doc.abstract_text);
    assert_eq!(back.claims, doc.claims);
    assert_eq!(back.specification, doc.specification);
    assert_eq!(back.drawings, doc.drawings);
}

// ── 2. ClaimType roundtrip ──

#[test]
fn claim_type_serde_roundtrip() {
    let variants = [ClaimType::Independent, ClaimType::Dependent];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ClaimType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 3. FeatureType roundtrip ──

#[test]
fn feature_type_serde_roundtrip() {
    let variants = [
        FeatureType::Element,
        FeatureType::Action,
        FeatureType::Parameter,
        FeatureType::Condition,
        FeatureType::Result,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: FeatureType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 4. CorrespondenceType roundtrip ──

#[test]
fn correspondence_type_serde() {
    let variants = [
        CorrespondenceType::Exact,
        CorrespondenceType::Equivalent,
        CorrespondenceType::Different,
        CorrespondenceType::Missing,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: CorrespondenceType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 5. OfficeAction roundtrip ──

#[test]
fn office_action_serde() {
    let oa = OfficeAction {
        oa_type: OaType::Novelty,
        citations: vec![CitedReference {
            document_number: "CN102345678A".into(),
            relevancy: "X".into(),
            claims_affected: vec![1, 3],
        }],
        examiner_arguments: "对比文件1公开了...".into(),
        affected_claims: vec![1, 2, 3],
    };
    let json = serde_json::to_string(&oa).unwrap();
    let back: OfficeAction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.oa_type, oa.oa_type);
    assert_eq!(back.citations.len(), 1);
    assert_eq!(back.citations[0].document_number, "CN102345678A");
    assert_eq!(back.examiner_arguments, oa.examiner_arguments);
    assert_eq!(back.affected_claims, oa.affected_claims);
}

// ── 6. ToolDomain exhaustive roundtrip ──

#[test]
fn tool_domain_exhaustive() {
    let all_variants = [
        ToolDomain::Search,
        ToolDomain::WebSearch,
        ToolDomain::Drafting,
        ToolDomain::Oa,
        ToolDomain::Quality,
        ToolDomain::Analysis,
        ToolDomain::Document,
        ToolDomain::Legal,
        ToolDomain::Management,
        ToolDomain::Review,
        ToolDomain::Evaluation,
        ToolDomain::Simulator,
        ToolDomain::Council,
    ];
    for v in &all_variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ToolDomain = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v, "ToolDomain::{v:?} roundtrip failed");
    }
}

// ── 7. SearchResult roundtrip ──

#[test]
fn search_result_serde() {
    let sr = SearchResult {
        source: SearchSource::KnowledgeGraph,
        title: "图像识别技术".into(),
        content: "涉及卷积神经网络...".into(),
        score: 0.92,
        id: "kg-node-001".into(),
        item_type: "concept".into(),
        source_path: "/path/to/node".into(),
        source_db: String::new(),
    };
    let json = serde_json::to_string(&sr).unwrap();
    let back: SearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source, SearchSource::KnowledgeGraph);
    assert_eq!(back.title, sr.title);
    assert_eq!(back.content, sr.content);
    assert_eq!(back.score, sr.score);
    assert_eq!(back.id, sr.id);
}

// ── 8. SearchSource roundtrip ──

#[test]
fn search_source_serde() {
    let variants = [
        SearchSource::KnowledgeGraph,
        SearchSource::LawDatabase,
        SearchSource::KnowledgeCard,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: SearchSource = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 9. OaType roundtrip ──

#[test]
fn oa_type_serde() {
    let variants = [
        OaType::Novelty,
        OaType::InventiveStep,
        OaType::Clarity,
        OaType::Support,
        OaType::Scope,
        OaType::Formal,
        OaType::Other("自定义类型".into()),
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: OaType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 10. ResponseStrategyType roundtrip ──

#[test]
fn response_strategy_type_serde() {
    let variants = [
        ResponseStrategyType::AmendClaims,
        ResponseStrategyType::Argue,
        ResponseStrategyType::Hybrid,
        ResponseStrategyType::Withdraw,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ResponseStrategyType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 11. PatentError Display ──

#[test]
fn patent_error_display() {
    let err = PatentError::NotFound("test".into());
    let msg = err.to_string();
    assert!(
        msg.contains("test"),
        "PatentError::NotFound Display should contain 'test', got: {msg}"
    );
    assert!(
        msg.contains("not found"),
        "Expected 'not found' prefix in: {msg}"
    );

    // Verify is_retryable for a retryable variant
    assert!(PatentError::KnowledgeGraph("timeout".into()).is_retryable());
    // Non-retryable variant
    assert!(!PatentError::NotFound("x".into()).is_retryable());
}

// ── 12. ApiKeyError Display ──

#[test]
fn api_key_error_display() {
    let missing = ApiKeyError::Missing("OPENAI_API_KEY".into());
    let msg = missing.to_string();
    assert!(
        msg.contains("OPENAI_API_KEY"),
        "ApiKeyError::Missing Display should contain env var name, got: {msg}"
    );
    assert!(msg.contains("not set"), "Expected 'not set' in: {msg}");

    let empty = ApiKeyError::Empty("MY_KEY".into());
    let msg = empty.to_string();
    assert!(
        msg.contains("MY_KEY"),
        "ApiKeyError::Empty should contain env var, got: {msg}"
    );
    assert!(msg.contains("empty"), "Expected 'empty' in: {msg}");

    let suspected = ApiKeyError::SuspectedProxyValue {
        env_var: "HTTP_PROXY".into(),
        len: 42,
    };
    let msg = suspected.to_string();
    assert!(
        msg.contains("HTTP_PROXY"),
        "ApiKeyError::SuspectedProxyValue should contain env var, got: {msg}"
    );
    assert!(msg.contains("42"), "Expected length in: {msg}");
}
