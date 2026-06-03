use std::collections::HashSet;

/// 计算两个向量的余弦相似度。
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|v| v * v).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// 计算两个字符串的 Jaccard 相似度（基于字符集合）。
pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<char> = a.chars().collect();
    let set_b: HashSet<char> = b.chars().collect();

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

/// 计算两个字符串的编辑距离（Levenshtein）。
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// 归一化编辑相似度（1 - 编辑距离/最大长度）。
pub fn normalized_edit_similarity(a: &str, b: &str) -> f64 {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    let max_len = a_len.max(b_len);

    if max_len == 0 {
        return 1.0;
    }

    1.0 - edit_distance(a, b) as f64 / max_len as f64
}

/// 综合文本相似度（Jaccard 与编辑距离的加权组合）。
pub fn text_similarity(a: &str, b: &str) -> f64 {
    let jaccard = jaccard_similarity(a, b);
    let edit_sim = normalized_edit_similarity(a, b);
    0.3 * jaccard + 0.7 * edit_sim
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_identical() {
        let vec = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec, &vec);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let sim = cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]);
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_identical() {
        let sim = jaccard_similarity("hello", "hello");
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_no_overlap() {
        let sim = jaccard_similarity("abc", "xyz");
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_edit_distance_identical() {
        assert_eq!(edit_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_edit_distance_chinese() {
        assert_eq!(edit_distance("专利申请", "专利审查"), 2);
    }

    #[test]
    fn test_normalized_edit_identical() {
        let sim = normalized_edit_similarity("测试", "测试");
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_text_similarity_identical() {
        let sim = text_similarity("专利申请文件", "专利申请文件");
        assert!((sim - 1.0).abs() < 0.001);
    }
}
