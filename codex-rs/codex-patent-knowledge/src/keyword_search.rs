//! 关键词搜索与 CJK n-gram 提取
//!
//! 提供停用词过滤、CJK n-gram 提取和相关性评分。
//! 适用于专利文本的关键词匹配搜索场景。

use std::collections::HashSet;

const STOP_WORDS: &[&str] = &[
    "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也",
    "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "那", "与",
    "及", "或", "而", "但", "且", "被", "把", "从", "对", "the", "a", "an", "is", "are", "was",
    "were", "be", "been", "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "shall", "can", "of", "in", "to", "for", "with", "on", "at", "from",
    "by", "as",
];

const NGRAM_MIN: usize = 2;
const NGRAM_MAX: usize = 4;

/// CJK 关键词搜索引擎
///
/// 对中文专利文本提取 2-4 gram 并计算相关性分数。
/// 通过 n-gram 匹配覆盖率和密度双重指标评分。
pub struct KeywordSearch;

impl KeywordSearch {
    /// 从查询文本中提取 CJK n-gram
    pub fn extract_ngrams(query: &str) -> Vec<String> {
        let cjk_chars: Vec<char> = query
            .chars()
            .filter(|c| {
                matches!(c,
                    '\u{4E00}'..='\u{9FFF}' |
                    '\u{3400}'..='\u{4DBF}' |
                    '\u{3000}'..='\u{303F}'
                )
            })
            .collect();

        if cjk_chars.len() < NGRAM_MIN {
            return Vec::new();
        }

        let mut ngrams = Vec::new();
        for win_size in NGRAM_MIN..=NGRAM_MAX.min(cjk_chars.len()) {
            for win in cjk_chars.windows(win_size) {
                let ngram: String = win.iter().collect();
                if !is_stop_ngram(&ngram) {
                    ngrams.push(ngram);
                }
            }
        }

        ngrams.sort();
        ngrams.dedup();
        ngrams
    }

    /// 计算文本的相关性分数
    /// score = coverage × density
    ///   coverage: 命中的 n-gram 数 ÷ 总 n-gram 数
    ///   density: 命中的总字符 ÷ 文本长度
    pub fn score_text(query_ngrams: &[String], text: &str) -> f64 {
        if query_ngrams.is_empty() || text.is_empty() {
            return 0.0;
        }

        let mut hit_count = 0;
        let mut hit_chars = HashSet::new();

        for ngram in query_ngrams {
            if let Some(pos) = text.find(ngram) {
                hit_count += 1;
                for i in pos..pos + ngram.len() {
                    hit_chars.insert(i);
                }
            }
        }

        if hit_count == 0 {
            return 0.0;
        }

        let coverage = hit_count as f64 / query_ngrams.len() as f64;
        let density = hit_chars.len() as f64 / text.len().max(1) as f64;
        coverage * (0.3 + 0.7 * density)
    }

    /// 同时考虑 n-gram 和原始关键词的评分
    pub fn score_text_with_query(query: &str, text: &str) -> f64 {
        let ngrams = Self::extract_ngrams(query);
        let ngram_score = Self::score_text(&ngrams, text);

        let raw_score = if text.contains(query) {
            1.0
        } else {
            let query_terms: Vec<&str> = query
                .split(|c: char| c.is_whitespace() || c == '，' || c == '。')
                .filter(|t| t.len() >= 2)
                .collect();
            if query_terms.is_empty() {
                return ngram_score;
            }
            let hits = query_terms.iter().filter(|t| text.contains(*t)).count();
            hits as f64 / query_terms.len() as f64
        };

        0.4 * ngram_score + 0.6 * raw_score
    }
}

fn is_stop_ngram(ngram: &str) -> bool {
    if ngram.len() <= 2 {
        STOP_WORDS.contains(&ngram)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ngrams() {
        let ngrams = KeywordSearch::extract_ngrams("图像识别装置");
        assert!(!ngrams.is_empty(), "should extract CJK n-grams");
        assert!(
            ngrams.contains(&"图像".to_string()),
            "should contain 2-gram '图像'"
        );
        assert!(
            ngrams.contains(&"识别装置".to_string()),
            "should contain 4-gram '识别装置'"
        );
    }

    #[test]
    fn test_extract_ngrams_english_only() {
        let ngrams = KeywordSearch::extract_ngrams("machine learning");
        assert!(ngrams.is_empty(), "no CJK = no n-grams");
    }

    #[test]
    fn test_score_text_high_relevance() {
        let ngrams = KeywordSearch::extract_ngrams("图像识别装置");
        let text = "本发明涉及一种图像识别装置，包括图像采集模块和识别处理模块";
        let score = KeywordSearch::score_text(&ngrams, text);
        assert!(
            score > 0.3,
            "highly relevant text should score > 0.3, got {score}"
        );
    }

    #[test]
    fn test_score_text_low_relevance() {
        let ngrams = KeywordSearch::extract_ngrams("图像识别装置");
        let text = "本发明涉及一种化工材料的制备方法";
        let score = KeywordSearch::score_text(&ngrams, text);
        assert!(
            score < 0.3,
            "unrelated text should score < 0.3, got {score}"
        );
    }

    #[test]
    fn test_score_text_with_query() {
        let s1 = KeywordSearch::score_text_with_query("图像识别", "一种图像识别方法和装置");
        let s2 = KeywordSearch::score_text_with_query("图像识别", "一种化工材料的制备方法");
        assert!(s1 > s2, "relevant text should score higher");
    }

    #[test]
    fn test_score_text_with_query_chinese_mixed() {
        let s = KeywordSearch::score_text_with_query(
            "深度学习 神经网络",
            "一种基于深度卷积神经网络的图像识别方法",
        );
        assert!(s > 0.0, "should find some matches");
    }
}
