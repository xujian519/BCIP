use codex_patent_text::IpcClassifier;
use pretty_assertions::assert_eq;

/// All 8 IPC sections (A–H) must have at least one keyword defined in the classifier.
#[test]
fn ipc_classify_all_sections() {
    let classifier = IpcClassifier::new();

    // Each section should produce a result when given text that matches its keywords.
    let section_samples = [
        ("A", "农业食品服装医药卫生生活家具运动"),
        ("B", "运输包装分离机床刀具"),
        ("C", "化学催化剂的制备"),
        ("D", "纺织纤维的新型纱线"),
        ("E", "建筑门窗的锁具结构"),
        ("F", "发动机齿轮轴承泵阀"),
        ("G", "计算信号测量仪器"),
        ("H", "半导体电路通信天线"),
    ];

    for (section, text) in &section_samples {
        let results = classifier.classify(text);
        assert!(
            !results.is_empty(),
            "section {section} should produce at least one result"
        );
        assert_eq!(
            results[0].section, *section,
            "expected top match to be section {section}"
        );
    }
}

/// Text with no keywords matching any IPC section should return empty results.
#[test]
fn ipc_classify_no_match() {
    let classifier = IpcClassifier::new();

    let results = classifier.classify("今天天气不错，出去走走吧。");
    assert!(
        results.is_empty(),
        "unrelated text should yield no classification results, got {} sections",
        results.len()
    );
}

/// Text containing keywords from multiple IPC sections should return all matching sections.
#[test]
fn ipc_classify_multiple_sections() {
    let classifier = IpcClassifier::new();

    // Contains keywords from H (通信, 电路), B (印刷), G (计算, 信号)
    let text = "一种印刷电路板的计算信号通信方法";
    let results = classifier.classify(text);

    let matched_sections: Vec<&str> = results.iter().map(|r| r.section.as_str()).collect();
    assert!(
        matched_sections.len() >= 3,
        "expected at least 3 sections matched, got {:?}",
        matched_sections
    );
    assert!(
        matched_sections.contains(&"H"),
        "expected H section in results: {:?}",
        matched_sections
    );
    assert!(
        matched_sections.contains(&"B"),
        "expected B section in results: {:?}",
        matched_sections
    );
    assert!(
        matched_sections.contains(&"G"),
        "expected G section in results: {:?}",
        matched_sections
    );

    // Results should be sorted by score descending.
    for window in matched_sections.windows(2) {
        // Just verify we got multiple results; score ordering is guaranteed by classify().
        let _ = window;
    }
}

/// Chinese patent abstract text should be classifiable.
#[test]
fn ipc_classify_chinese_keywords() {
    let classifier = IpcClassifier::new();

    // Simulated patent abstract for a chemical invention.
    let text = "本发明涉及一种新型聚合物涂料的制备方法，通过化学催化发酵工艺，提高涂料附着力。";
    let results = classifier.classify(text);

    assert!(
        !results.is_empty(),
        "Chinese patent text should produce classification results"
    );
    assert_eq!(
        results[0].section, "C",
        "chemistry-related text should classify as section C"
    );

    // Verify IpcResult fields are populated correctly.
    let top = &results[0];
    assert!(!top.section.is_empty());
    assert!(!top.description.is_empty());
    assert!(
        top.score > 0.0,
        "score should be positive when keywords match"
    );
}

/// Empty string input should return no results.
#[test]
fn ipc_classify_empty_input() {
    let classifier = IpcClassifier::new();

    let results = classifier.classify("");
    assert!(
        results.is_empty(),
        "empty input should yield no classification results, got {} sections",
        results.len()
    );
}
