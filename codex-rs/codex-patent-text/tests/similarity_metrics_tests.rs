use pretty_assertions::assert_eq;

/// `text_similarity` is the only public similarity function.
/// It combines Jaccard (0.3) + normalized edit similarity (0.7).
/// These tests verify behavior indirectly through that public API.

#[test]
fn text_similarity_empty_empty() {
    // Both empty → Jaccard(1.0) + norm_edit(1.0) = 1.0
    let sim = codex_patent_text::text_similarity("", "");
    assert_eq!(sim, 1.0);
}

#[test]
fn text_similarity_empty_nonempty() {
    // "" vs "abc" → Jaccard(0.0) + norm_edit(0.0) = 0.0
    let sim = codex_patent_text::text_similarity("", "abc");
    assert_eq!(sim, 0.0);
}

#[test]
fn text_similarity_single_char_insert() {
    // "abc" vs "abxc" — one insertion, edit distance 1 out of max_len 4
    // norm_edit = 1.0 - 1/4 = 0.75
    // Jaccard: {a,b,c} ∩ {a,b,x,c} / {a,b,c,x} = 3/4 = 0.75
    // total = 0.3*0.75 + 0.7*0.75 = 0.75
    let sim = codex_patent_text::text_similarity("abc", "abxc");
    assert!((sim - 0.75).abs() < 1e-10, "expected 0.75, got {sim}");
}

#[test]
fn text_similarity_single_char_delete() {
    // "abcd" vs "acd" — one deletion, edit distance 1 out of max_len 4
    // norm_edit = 1.0 - 1/4 = 0.75
    // Jaccard: {a,b,c,d} ∩ {a,c,d} / {a,b,c,d} = 3/4 = 0.75
    // total = 0.3*0.75 + 0.7*0.75 = 0.75
    let sim = codex_patent_text::text_similarity("abcd", "acd");
    assert!((sim - 0.75).abs() < 1e-10, "expected 0.75, got {sim}");
}

#[test]
fn text_similarity_single_char_substitute() {
    // "abc" vs "axc" — one substitution, edit distance 1 out of max_len 3
    // norm_edit = 1.0 - 1/3 ≈ 0.6667
    // Jaccard: {a,b,c} ∩ {a,x,c} / {a,b,c,x} = 2/4 = 0.5
    // total = 0.3*0.5 + 0.7*(2/3)
    let sim = codex_patent_text::text_similarity("abc", "axc");
    let expected = 0.3 * 0.5 + 0.7 * (2.0 / 3.0);
    assert!(
        (sim - expected).abs() < 1e-10,
        "expected {expected}, got {sim}"
    );
}

#[test]
fn text_similarity_identical() {
    let sim = codex_patent_text::text_similarity("abc", "abc");
    assert_eq!(sim, 1.0);
}

#[test]
fn text_similarity_range() {
    let cases = [
        ("hello", "world"),
        ("", ""),
        ("", "long string here"),
        ("same", "same"),
        ("a", "b"),
        ("abcdef", "ghijkl"),
        ("专利", "专利"),
        ("专利", "商标"),
    ];
    for (a, b) in cases {
        let sim = codex_patent_text::text_similarity(a, b);
        assert!(
            (0.0..=1.0).contains(&sim),
            "text_similarity({a:?}, {b:?}) = {sim}, not in [0.0, 1.0]"
        );
    }
}

#[test]
fn text_similarity_completely_different() {
    // No shared chars → Jaccard = 0, all edits → norm_edit near 0
    let sim = codex_patent_text::text_similarity("abc", "xyz");
    assert!(sim < 0.1, "expected near 0.0, got {sim}");
}

#[test]
fn text_similarity_unicode_cjk() {
    // CJK characters treated as single chars
    let sim_same = codex_patent_text::text_similarity("专利申请", "专利申请");
    assert!((sim_same - 1.0).abs() < 1e-10);

    // Partial overlap: "专利申请" vs "专利公告" — share "专利"
    // edit distance 2 out of max_len 4 → norm_edit = 0.5
    // Jaccard: {专,利,申,请} ∩ {专,利,公,告} / {专,利,申,请,公,告} = 2/6
    let sim_diff = codex_patent_text::text_similarity("专利申请", "专利公告");
    assert!(sim_diff < 1.0);
    assert!(
        sim_diff > 0.0,
        "partial overlap should yield > 0, got {sim_diff}"
    );
}

#[test]
fn text_similarity_case_sensitivity() {
    // Case matters: 'A' ≠ 'a'. Use strings with partial overlap to get > 0.
    // "Abc" vs "abc" → Jaccard {A,b,c} ∩ {a,b,c} / {A,a,b,c} = 2/4 = 0.5
    // edit distance 1 out of max_len 3 → norm_edit = 2/3
    let sim = codex_patent_text::text_similarity("Abc", "abc");
    assert!(sim < 1.0, "case should matter: expected < 1.0, got {sim}");
    assert!(sim > 0.0, "partial overlap should yield > 0, got {sim}");
}
#[test]
fn text_similarity_symmetry() {
    let a = "专利权";
    let b = "专利申请权";
    let sim_ab = codex_patent_text::text_similarity(a, b);
    let sim_ba = codex_patent_text::text_similarity(b, a);
    assert!(
        (sim_ab - sim_ba).abs() < 1e-10,
        "similarity should be symmetric"
    );
}
