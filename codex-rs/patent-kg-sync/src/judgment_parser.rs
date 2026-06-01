use anyhow::Result;
use regex::Regex;
use std::path::Path;

use crate::models::JudgmentEntry;
use crate::utils::{chinese_to_number, truncate};

/// 解析指导性判决文书目录
pub fn parse_guiding_judgments(dir: &Path) -> Result<Vec<JudgmentEntry>> {
    let mut judgments = Vec::new();
    let mut errors = 0;

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        let path = entry.path();
        match parse_single_guiding_judgment(path) {
            Ok(j) => judgments.push(j),
            Err(e) => {
                errors += 1;
                if errors <= 5 {
                    eprintln!("      警告: 解析失败 {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("      指导性判决: {} 份, 失败: {}", judgments.len(), errors);
    Ok(judgments)
}

/// 解析一般专利判决目录
pub fn parse_general_judgments(dir: &Path) -> Result<Vec<JudgmentEntry>> {
    let mut judgments = Vec::new();
    let mut errors = 0;

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        let path = entry.path();
        match parse_single_general_judgment(path) {
            Ok(j) => judgments.push(j),
            Err(e) => {
                errors += 1;
                if errors <= 5 {
                    eprintln!("      警告: 解析失败 {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("      一般判决: {} 份, 失败: {}", judgments.len(), errors);
    Ok(judgments)
}

fn parse_single_guiding_judgment(path: &Path) -> Result<JudgmentEntry> {
    let content = std::fs::read_to_string(path)?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 案号: （2023）最高法知行终475号
    let case_re = Regex::new(r"[（(](\d{4})[）)][^\n【]+号")?;
    let case_number = case_re
        .captures(&content)
        .map(|c| c[0].replace('（', "(").replace('）', ")"))
        .unwrap_or_default();

    // 裁判要旨
    let key_points = extract_section(&content, "裁判要旨");

    // 关键词
    let keywords = extract_keywords(&content);

    // 法条引用
    let law_articles = extract_law_articles(&content);

    // 审理法院
    let court = if content.contains("最高法") || content.contains("最高人民法院") {
        "最高人民法院".to_string()
    } else {
        extract_court(&content)
    };

    // 裁判日期
    let date = extract_date(&content);

    // 摘要
    let summary = if !key_points.is_empty() {
        key_points.clone()
    } else {
        truncate(
            &content
                .replace('#', " ")
                .replace('*', " ")
                .replace('|', " "),
            500,
        )
    };

    Ok(JudgmentEntry {
        case_number,
        court,
        date,
        cause: String::new(),
        law_articles,
        keywords,
        key_points,
        summary,
        source_file: file_name,
        is_guiding: true,
    })
}

fn parse_single_general_judgment(path: &Path) -> Result<JudgmentEntry> {
    let content = std::fs::read_to_string(path)?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 案号
    let case_re = Regex::new(r"[（(](\d{4})[）)][^\n(（]*号")?;
    let case_number = case_re
        .captures(&content)
        .map(|c| c[0].replace('（', "(").replace('）', ")"))
        .unwrap_or_default();

    // 审理法院
    let court = extract_court(&content);

    // 裁判日期
    let date = extract_date(&content);

    // 案由
    let cause = extract_section_label(&content, "案由");

    // 法条引用
    let law_articles = extract_law_articles(&content);

    let summary = truncate(
        &content
            .replace('#', " ")
            .replace('*', " ")
            .replace('|', " "),
        500,
    );

    Ok(JudgmentEntry {
        case_number,
        court,
        date,
        cause,
        law_articles,
        keywords: Vec::new(),
        key_points: String::new(),
        summary,
        source_file: file_name,
        is_guiding: false,
    })
}

fn extract_section(content: &str, section_name: &str) -> String {
    let start_tag = format!("【{}】", section_name);
    if let Some(start) = content.find(&start_tag) {
        let after = &content[start + start_tag.len()..];
        let text = if let Some(end) = after.find("【") {
            &after[..end]
        } else {
            after
        };
        return text.trim().to_string();
    }
    String::new()
}

fn extract_section_label(content: &str, label: &str) -> String {
    let re = Regex::new(&format!(r"{}\s*[：:]\s*(.+)", label)).unwrap();
    if let Some(cap) = re.captures(content) {
        return cap[1].trim().to_string();
    }
    String::new()
}

fn extract_keywords(content: &str) -> Vec<String> {
    let section = extract_section(content, "关键词");
    if section.is_empty() {
        return Vec::new();
    }
    section
        .split(|c: char| c == ' ' || c == '\t' || c == '、' || c == ',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn extract_law_articles(content: &str) -> Vec<String> {
    let mut articles = Vec::new();

    let re = Regex::new(r"专利法第(\d+)条").unwrap();
    for cap in re.captures_iter(content) {
        articles.push(format!("A{}", &cap[1]));
    }

    let re2 = Regex::new(r"专利法实施细则第(\d+)条").unwrap();
    for cap in re2.captures_iter(content) {
        articles.push(format!("R{}", &cap[1]));
    }

    // 反不正当竞争法 etc.
    let re3 = Regex::new(r"第([一二三四五六七八九十百千\d]+)条").unwrap();
    for cap in re3.captures_iter(content) {
        // only add if not already captured as patent law
        let num = chinese_to_number(&cap[1]);
        let article = format!("G{}", num);
        if !articles.iter().any(|a| a.contains(&cap[1])) {
            articles.push(article);
        }
    }

    articles.sort_unstable();
    articles.dedup();
    articles.truncate(10); // 限制数量
    articles
}

fn extract_court(content: &str) -> String {
    let re = Regex::new(r"审理法院\s*[：:]\s*(.+)").unwrap();
    if let Some(cap) = re.captures(content) {
        return cap[1].trim().to_string();
    }
    if content.contains("最高人民法院") {
        return "最高人民法院".to_string();
    }
    if content.contains("北京知识产权法院") {
        return "北京知识产权法院".to_string();
    }
    String::new()
}

fn extract_date(content: &str) -> String {
    let re = Regex::new(r"裁判日期\s*[：:]\s*(\d{4}[.\-/年]\d{1,2}[.\-/月]\d{1,2})").unwrap();
    if let Some(cap) = re.captures(content) {
        return cap[1].to_string();
    }
    let re2 = Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日").unwrap();
    if let Some(cap) = re2.captures(content) {
        return format!("{}年{}月{}日", &cap[1], &cap[2], &cap[3]);
    }
    String::new()
}
