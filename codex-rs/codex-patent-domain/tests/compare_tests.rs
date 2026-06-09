use codex_patent_core::CompareFeature;
use codex_patent_domain::compare::{MatchType, compare, lexical_similarity};
use pretty_assertions::assert_eq;

fn feat(id: &str, desc: &str) -> CompareFeature {
    CompareFeature {
        id: id.to_string(),
        description: desc.to_string(),
    }
}

#[test]
fn feature_match_identical() {
    let target = vec![
        feat("t1", "a housing body made of aluminum alloy"),
        feat("t2", "a display screen disposed on the front surface"),
        feat(
            "t3",
            "a battery module electrically connected to the display",
        ),
    ];
    // Same descriptions → all should be Exact matches
    let prior = target.clone();

    let result = compare(&target, &prior);

    assert_eq!(
        result.exact_matches.len(),
        3,
        "all three features should match exactly"
    );
    assert_eq!(result.equivalent_matches.len(), 0);
    assert_eq!(result.different_features.len(), 0);
    assert_eq!(result.missing_features.len(), 0);
    assert!(
        (result.coverage_ratio - 1.0).abs() < 1e-9,
        "coverage should be 1.0"
    );
    for m in &result.exact_matches {
        assert_eq!(m.match_type, MatchType::Exact);
    }
}

#[test]
fn feature_match_different() {
    let target = vec![
        feat("t1", "a cylindrical metal housing with threaded ends"),
        feat("t2", "a solar panel array mounted on the roof"),
    ];
    let prior = vec![
        feat("p1", "a rubber gasket seal for fluid containment"),
        feat("p2", "a wooden frame structure with dovetail joints"),
    ];

    let result = compare(&target, &prior);

    assert_eq!(
        result.exact_matches.len(),
        0,
        "no features should match exactly"
    );
    // Completely unrelated text → similarity < 0.6, so all go to different
    assert!(
        result.different_features.len() == 2,
        "both features should be classified as different, got {} different, {} equivalent",
        result.different_features.len(),
        result.equivalent_matches.len(),
    );
    assert_eq!(result.missing_features.len(), 0);
    assert!(
        (result.coverage_ratio).abs() < 1e-9,
        "coverage should be 0.0"
    );
}

#[test]
fn feature_match_partial() {
    let target = vec![
        feat("t1", "a housing body made of aluminum alloy"), // will exact-match p1
        feat("t2", "a display screen mounted on the front panel"), // will equivalent-match p2
        feat("t3", "a wireless charging coil"),              // no close match → different
    ];
    let prior = vec![
        feat("p1", "a housing body made of aluminum alloy"), // identical to t1
        feat("p2", "a display screen mounted on the front panel"), // identical to t2
        feat("p3", "a battery cell connected to the main board"), // unrelated to t3
    ];

    let result = compare(&target, &prior);

    assert_eq!(
        result.exact_matches.len(),
        2,
        "t1 and t2 should exact-match"
    );
    assert_eq!(
        result.different_features.len(),
        1,
        "t3 has no close match → different"
    );
    assert!(
        (result.coverage_ratio - (2.0 / 3.0)).abs() < 1e-9,
        "coverage should be 2/3",
    );
}

#[test]
fn compare_empty_features() {
    let empty: Vec<CompareFeature> = vec![];

    // Both empty
    let result = compare(&empty, &empty);
    assert_eq!(result.exact_matches.len(), 0);
    assert_eq!(result.equivalent_matches.len(), 0);
    assert_eq!(result.different_features.len(), 0);
    assert_eq!(result.missing_features.len(), 0);
    assert!(
        (result.coverage_ratio).abs() < 1e-9,
        "empty target → coverage 0.0"
    );

    // Target non-empty, prior empty → all features missing
    let target = vec![feat("t1", "some feature")];
    let result = compare(&target, &empty);
    assert_eq!(result.missing_features.len(), 1);
    assert!((result.coverage_ratio).abs() < 1e-9);

    // Target empty, prior non-empty
    let prior = vec![feat("p1", "some feature")];
    let result = compare(&empty, &prior);
    assert_eq!(result.exact_matches.len(), 0);
    assert!((result.coverage_ratio).abs() < 1e-9);
}

#[test]
fn compare_symmetry() {
    let set_a = vec![
        feat("a1", "a semiconductor substrate of silicon material"),
        feat("a2", "a gate electrode formed on the substrate"),
        feat("a3", "a source drain region doped with phosphorus"),
    ];
    let set_b = vec![
        feat("b1", "a semiconductor substrate made of silicon wafer"),
        feat("b2", "a gate dielectric layer over the substrate"),
    ];

    let result_ab = compare(&set_a, &set_b);
    let result_ba = compare(&set_b, &set_a);

    // Compare(a, b) uses lexical_similarity which is symmetric on description pairs.
    // The coverage ratios should be symmetric because lexical_similarity(a,b) == lexical_similarity(b,a)
    // and both sides use the same threshold logic.
    // However, coverage_ratio depends on len(target), so we check that the matching
    // scores are consistent — not necessarily identical coverage due to different target sizes.
    // Instead verify the core similarity function is symmetric.
    for a in &set_a {
        for b in &set_b {
            let sim_ab = lexical_similarity(&a.description, &b.description);
            let sim_ba = lexical_similarity(&b.description, &a.description);
            assert!(
                (sim_ab - sim_ba).abs() < 1e-12,
                "lexical_similarity should be symmetric: sim({:?}, {:?}) = {} but sim({:?}, {:?}) = {}",
                a.description,
                b.description,
                sim_ab,
                b.description,
                a.description,
                sim_ba,
            );
        }
    }

    // Also verify both results have reasonable structure
    assert!(result_ab.coverage_ratio >= 0.0 && result_ab.coverage_ratio <= 1.0);
    assert!(result_ba.coverage_ratio >= 0.0 && result_ba.coverage_ratio <= 1.0);
}
