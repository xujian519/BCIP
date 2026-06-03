use std::collections::HashMap;

/// 分词结果中的一个词元。
pub struct Token {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// 文本统计信息。
pub struct TextStats {
    pub char_count: usize,
    pub word_count: usize,
    pub line_count: usize,
    pub cjk_char_count: usize,
    pub ascii_word_count: usize,
}

/// 对文本进行分词（支持中英文混合）。
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_start = 0;
    let mut current_token = String::new();

    for (i, ch) in text.char_indices() {
        if ch.is_whitespace() || is_cjk_punctuation(ch) {
            if !current_token.is_empty() {
                tokens.push(Token {
                    text: current_token.clone(),
                    start: current_start,
                    end: i,
                });
                current_token.clear();
            }
            current_start = i + ch.len_utf8();
        } else if is_cjk_char(ch) {
            if !current_token.is_empty() {
                tokens.push(Token {
                    text: current_token.clone(),
                    start: current_start,
                    end: i,
                });
                current_token.clear();
            }
            tokens.push(Token {
                text: ch.to_string(),
                start: i,
                end: i + ch.len_utf8(),
            });
            current_start = i + ch.len_utf8();
        } else {
            if current_token.is_empty() {
                current_start = i;
            }
            current_token.push(ch);
        }
    }

    if !current_token.is_empty() {
        tokens.push(Token {
            text: current_token,
            start: current_start,
            end: text.len(),
        });
    }

    tokens
}

/// 统计文本基本信息（字符数、词数、中文字符数等）。
pub fn text_stats(text: &str) -> TextStats {
    let char_count = text.chars().count();
    let cjk_char_count = text.chars().filter(|c| is_cjk_char(*c)).count();
    let line_count = text.lines().count().max(1);
    let words = tokenize(text);
    let ascii_word_count = words
        .iter()
        .filter(|t| t.text.chars().all(|c| !is_cjk_char(c)))
        .count();

    TextStats {
        char_count,
        word_count: words.len(),
        line_count,
        cjk_char_count,
        ascii_word_count,
    }
}

/// 基于词频提取关键词（支持中英文）。
pub fn extract_keywords(text: &str, top_n: usize) -> Vec<(String, usize)> {
    let tokens = tokenize(text);
    let mut freq: HashMap<String, usize> = HashMap::new();

    for token in &tokens {
        let word = token.text.to_lowercase();
        if word.len() < 2 && !is_cjk_char(word.chars().next().unwrap_or(' ')) {
            continue;
        }
        if is_stop_word(&word) {
            continue;
        }
        *freq.entry(word).or_insert(0) += 1;
    }

    let mut pairs: Vec<(String, usize)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs.truncate(top_n);
    pairs
}

fn is_cjk_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{3000}'..='\u{303F}' |
        '\u{FF00}'..='\u{FFEF}'
    )
}

fn is_cjk_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '，' | '。'
            | '、'
            | '；'
            | '：'
            | '？'
            | '！'
            | '\u{201C}'
            | '\u{201D}'
            | '\''
            | '【'
            | '】'
            | '（'
            | '）'
            | '《'
            | '》'
    )
}

fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也",
        "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "the",
        "a", "an", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
        "did", "will", "would", "could", "should", "may", "might", "shall", "can", "of", "in",
        "to", "for", "with", "on", "at", "from", "by", "as",
    ];
    STOP_WORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_english() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
    }

    #[test]
    fn test_tokenize_chinese() {
        let tokens = tokenize("专利申请");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].text, "专");
    }

    #[test]
    fn test_tokenize_mixed() {
        let tokens = tokenize("IPC分类号 G06F");
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"IPC"));
        assert!(texts.contains(&"G06F"));
    }

    #[test]
    fn test_extract_keywords() {
        let text = "patent application patent review patent invention";
        let keywords = extract_keywords(text, 3);
        assert!(keywords.iter().any(|(w, _)| w == "patent"));
    }
}
