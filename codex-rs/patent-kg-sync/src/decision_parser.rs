use anyhow::Result;
use regex::Regex;
use std::path::Path;

use crate::models::InvalidDecision;
use crate::utils::{chinese_to_number, truncate};

/// 解析目录下所有复审决定 MD 文件
pub fn parse_decisions(dir: &Path) -> Result<Vec<InvalidDecision>> {
    let mut decisions = Vec::new();
    let mut errors = 0;

    let entries: Vec<_> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "md")
        })
        .collect();

    let total = entries.len();
    println!("      发现 {} 份复审决定文件", total);

    for entry in entries {
        let path = entry.path();
        match parse_single_decision(path) {
            Ok(d) => decisions.push(d),
            Err(e) => {
                errors += 1;
                if errors <= 5 {
                    eprintln!("      警告: 解析失败 {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("      解析成功: {}, 失败: {}", decisions.len(), errors);
    Ok(decisions)
}

fn parse_single_decision(path: &Path) -> Result<InvalidDecision> {
    let content = std::fs::read_to_string(path)?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 决定号: （第561695号）或 (第561695号)
    let decision_re = Regex::new(r"[（(]第(\d+)号[）)]")?;
    let decision_number = decision_re
        .captures(&content)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    // 专利号: 从文件标题 "# 专利无效复审决定 {patent_number}" 提取
    let title_re = Regex::new(r"#\s*专利无效复审决定\s+(\d[\d.]+)")?;
    let patent_number = title_re
        .captures(&content)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    // 结论判断: 三个选项按出现顺序判断
    // 在原始 PDF 中第一个选项前有 ✓，但文本提取后无法区分
    // 使用策略: 寻找独立的结论行
    let conclusion = extract_conclusion(&content);

    // 法条引用
    let law_articles = extract_law_articles(&content);

    // 无效理由（关键词匹配）
    let reasons = extract_reasons(&content);

    // IPC 分类号: 从结尾表格提取 "国际分类号 | H02G 3/00"
    let ipc_code = extract_ipc_from_table(&content);

    // 摘要: 提取"决定要点"或"案由"段前500字
    let summary = extract_summary(&content);

    Ok(InvalidDecision {
        decision_number,
        patent_number,
        conclusion,
        law_articles,
        reasons,
        ipc_code,
        summary,
        source_file: file_name,
    })
}

fn extract_conclusion(content: &str) -> String {
    // 寻找"现决定如下"后面的结论
    // 策略: 查找第一个出现的结论行
    let candidates = ["宣告专利权全部无效", "宣告专利权部分无效", "维持专利权有效"];

    // 在"现决定如下"段之后、"根据专利法第46条第2款"之前
    if let Some(start) = content.find("现决定如下") {
        if let Some(end) = content[start..].find("根据专利法第46条第2款") {
            let section = &content[start..start + end];
            for candidate in &candidates {
                if section.contains(candidate) {
                    return candidate.to_string();
                }
            }
        }
    }

    // fallback: 在全文中找
    for candidate in &candidates {
        if content.contains(candidate) {
            return candidate.to_string();
        }
    }

    "未知".to_string()
}

fn extract_law_articles(content: &str) -> Vec<String> {
    let mut articles = Vec::new();

    // 专利法第X条第Y款
    let re1 = Regex::new(
        r"专利法第([一二三四五六七八九十百千\d]+)条(?:第([一二三四五六七八九十\d]+)款)?",
    )
    .unwrap();
    for cap in re1.captures_iter(content) {
        let article = &cap[1];
        let article_num = chinese_to_number(article);
        if let Some(clause) = cap.get(2) {
            let clause_num = chinese_to_number(clause.as_str());
            articles.push(format!("A{}.{}", article_num, clause_num));
        } else {
            articles.push(format!("A{}", article_num));
        }
    }

    // 专利法实施细则第X条第Y款
    let re2 = Regex::new(
        r"专利法实施细则第([一二三四五六七八九十百千\d]+)条(?:第([一二三四五六七八九十\d]+)款)?",
    )
    .unwrap();
    for cap in re2.captures_iter(content) {
        let article = &cap[1];
        let article_num = chinese_to_number(article);
        if let Some(clause) = cap.get(2) {
            let clause_num = chinese_to_number(clause.as_str());
            articles.push(format!("R{}.{}", article_num, clause_num));
        } else {
            articles.push(format!("R{}", article_num));
        }
    }

    articles.sort_unstable();
    articles.dedup();
    articles
}

fn extract_reasons(content: &str) -> Vec<String> {
    let mut reasons = Vec::new();
    let keywords = [
        ("创造性", "创造性"),
        ("新颖性", "新颖性"),
        ("实用性", "实用性"),
        ("充分公开", "公开不充分"),
        ("不清楚", "保护范围不清楚"),
        ("得不到说明书支持", "得不到说明书支持"),
        ("修改超范围", "修改超范围"),
    ];

    for (keyword, label) in &keywords {
        if content.contains(keyword) {
            reasons.push(label.to_string());
        }
    }

    reasons
}

fn extract_ipc_from_table(content: &str) -> Option<String> {
    // 从"国际分类号"表格行提取
    let re = Regex::new(r"国际分类号\s*\|\s*([A-H]\d{2}[A-Z]\s*\d+/\d+)").ok()?;
    if let Some(cap) = re.captures(content) {
        let ipc = cap[1].replace(' ', "");
        return Some(ipc);
    }

    // 备选: "分类号" 后的 IPC 编码
    let re2 = Regex::new(r"分类号[：:]\s*([A-H]\d{2}[A-Z]\s*\d+/\d+)").ok()?;
    re2.captures(content).map(|cap| cap[1].replace(' ', ""))
}

fn extract_summary(content: &str) -> String {
    // 优先提取"决定要点"
    if let Some(start) = content.find("决定要点") {
        let section = &content[start..];
        let text = section
            .lines()
            .skip(1)
            .take(5)
            .collect::<Vec<_>>()
            .join(" ")
            .replace('|', " ")
            .trim()
            .to_string();
        if !text.is_empty() {
            return truncate(&text, 500);
        }
    }

    // fallback: "案由"段前500字
    if let Some(start) = content.find("一、案由") {
        let section = &content[start..];
        return truncate(section, 500);
    }

    truncate(content, 500)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_law_articles() {
        let content = "根据专利法第46条第1款的规定，不符合专利法第22条第3款的规定";
        let articles = extract_law_articles(content);
        assert!(articles.contains(&"A22.3".to_string()));
        assert!(articles.contains(&"A46.1".to_string()));
    }

    #[test]
    fn test_extract_reasons() {
        let content = "权利要求1-14不具有创造性，不符合专利法第22条第3款的规定";
        let reasons = extract_reasons(content);
        assert!(reasons.contains(&"创造性".to_string()));
    }
}
