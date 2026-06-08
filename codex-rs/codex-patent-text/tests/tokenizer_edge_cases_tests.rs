use codex_patent_text::{TextStats, Token, extract_keywords, text_stats, tokenize};
use pretty_assertions::assert_eq;

#[test]
fn tokenize_empty_string() {
    let tokens = tokenize("");
    assert_eq!(tokens, Vec::<Token>::new());
}

#[test]
fn tokenize_whitespace_only() {
    let tokens = tokenize("   \t\n  \r\n  ");
    assert_eq!(tokens, Vec::<Token>::new());
}

#[test]
fn tokenize_cjk_punctuation() {
    // CJK punctuation should split tokens but not produce tokens themselves
    let tokens = tokenize("，。、；：？！【】（《》）");
    assert_eq!(tokens, Vec::<Token>::new());
}

#[test]
fn is_cjk_char_boundary() {
    // Test CJK boundary code points indirectly via tokenize and text_stats.
    // is_cjk_char is private, so we verify behavior through public APIs.

    // U+4E00 (一) — first char of CJK Unified Ideographs block
    let stats = text_stats("\u{4E00}");
    assert_eq!(stats.cjk_char_count, 1, "U+4E00 should be CJK");
    assert_eq!(stats.char_count, 1);
    assert_eq!(stats.word_count, 1);

    // U+9FFF — last char of CJK Unified Ideographs block
    let stats = text_stats("\u{9FFF}");
    assert_eq!(stats.cjk_char_count, 1, "U+9FFF should be CJK");

    // U+4DBF — last char of CJK Extension A block
    let stats = text_stats("\u{4DBF}");
    assert_eq!(stats.cjk_char_count, 1, "U+4DBF should be CJK");

    // U+3400 — first char of CJK Extension A block
    let stats = text_stats("\u{3400}");
    assert_eq!(stats.cjk_char_count, 1, "U+3400 should be CJK");

    // U+FFEF — last char of Halfwidth and Fullwidth Forms block
    let stats = text_stats("\u{FFEF}");
    assert_eq!(stats.cjk_char_count, 1, "U+FFEF should be CJK");

    // U+3000 — first char of CJK Symbols and Punctuation block
    let stats = text_stats("\u{3000}");
    assert_eq!(stats.cjk_char_count, 1, "U+3000 should be CJK");

    // ASCII 'A' is not CJK
    let stats = text_stats("A");
    assert_eq!(stats.cjk_char_count, 0, "ASCII 'A' should not be CJK");

    // U+4DFF — just before CJK Extension A end boundary (not CJK)
    // Actually U+3400..=U+4DBF is CJK Extension A, so U+4DC0 is outside
    let stats = text_stats("\u{4DC0}");
    assert_eq!(stats.cjk_char_count, 0, "U+4DC0 should not be CJK");
}

#[test]
fn text_stats_empty() {
    let stats = text_stats("");
    assert_eq!(stats.char_count, 0);
    assert_eq!(stats.word_count, 0);
    // "".lines().count() == 0, but max(1) ensures at least 1
    assert_eq!(stats.line_count, 1);
    assert_eq!(stats.cjk_char_count, 0);
    assert_eq!(stats.ascii_word_count, 0);
}

#[test]
fn text_stats_newlines() {
    let stats = text_stats("\n\n\n");
    assert_eq!(stats.char_count, 3);
    assert_eq!(stats.word_count, 0);
    assert_eq!(stats.line_count, 3);
    assert_eq!(stats.cjk_char_count, 0);
    assert_eq!(stats.ascii_word_count, 0);
}

#[test]
fn extract_keywords_zero() {
    let keywords = extract_keywords("专利发明技术创新技术", 0);
    assert!(keywords.is_empty(), "top_n=0 should return empty");
}

#[test]
fn tokenize_mixed_cjk_latin() {
    let tokens = tokenize("专利abc技术");
    // Each CJK char is a separate token, "abc" is one token
    assert_eq!(tokens.len(), 5, "expected 5 tokens: 专, 利, abc, 技, 术");

    assert_eq!(tokens[0].text, "专");
    assert_eq!(tokens[1].text, "利");
    assert_eq!(tokens[2].text, "abc");
    assert_eq!(tokens[3].text, "技");
    assert_eq!(tokens[4].text, "术");

    // Verify byte offsets: "专利" is 6 bytes (3 each), "abc" is 3 bytes
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 3);
    assert_eq!(tokens[1].start, 3);
    assert_eq!(tokens[1].end, 6);
    assert_eq!(tokens[2].start, 6);
    assert_eq!(tokens[2].end, 9);
    assert_eq!(tokens[3].start, 9);
    assert_eq!(tokens[3].end, 12);
    assert_eq!(tokens[4].start, 12);
    assert_eq!(tokens[4].end, 15);
}

#[test]
fn tokenize_long_text() {
    let repeated = "专利技术".repeat(1000);
    let tokens = tokenize(&repeated);
    // Each CJK char is a separate token: 4 chars × 1000 = 4000 tokens
    assert_eq!(tokens.len(), 4000);

    // Verify first and last tokens
    assert_eq!(tokens[0].text, "专");
    assert_eq!(tokens[3999].text, "术");
}

#[test]
fn text_stats_typical() {
    let text = "一种数据处理方法，包括以下步骤：获取输入数据。";
    let stats = text_stats(text);

    // Count characters: 22 visible chars (CJK) + 5 CJK punctuation
    assert!(stats.char_count > 0);
    assert!(stats.cjk_char_count > 0, "should have CJK characters");
    assert_eq!(stats.line_count, 1, "single line text");

    // word_count should equal number of tokens from tokenize
    let tokens = tokenize(text);
    assert_eq!(stats.word_count, tokens.len());

    // All tokens are CJK single chars, no ASCII words
    assert_eq!(stats.ascii_word_count, 0);
}
