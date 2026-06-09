//! 结构化说明书生成引擎
//!
//! 从 DisclosureDoc 解析各节内容，生成符合 CNIPA 标准格式的说明书大纲，
//! 并支持展开为标准文本。

use codex_patent_core::{DisclosureDoc, PatentError};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 数据类型
// ---------------------------------------------------------------------------

/// 说明书结构化大纲
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecOutline {
    pub title: String,
    pub tech_field: Section,
    pub background: Section,
    pub summary: Section,
    pub brief_description: Section,
    pub embodiments: Vec<Embodiment>,
    pub abstract_text: String,
    pub sequence_listing: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub heading: String,
    pub content: String,
    pub subsections: Vec<Subsection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsection {
    pub heading: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embodiment {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub examples: Vec<TechExample>,
    pub reference_signs: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechExample {
    pub title: String,
    pub conditions: Vec<String>,
    pub results: Vec<String>,
    pub comparative: Option<String>,
}

// ---------------------------------------------------------------------------
// 核心函数
// ---------------------------------------------------------------------------

/// 从 DisclosureDoc 中解析各节，生成结构化说明书大纲。
///
/// 从 `disclosure.sections` HashMap 中提取以下键：
/// - `技术领域` / `tech_field`
/// - `背景技术` / `background`
/// - `发明内容` / `summary`（含技术问题、技术方案、有益效果三个子节）
/// - `附图说明` / `brief_description`
/// - `具体实施方式` / `embodiments`（自动按段落拆分为多个 Embodiment 并编号）
/// - `摘要` / `abstract`
/// - `序列表` / `sequence_listing`（可选）
pub fn generate_spec_outline(disclosure: &DisclosureDoc) -> Result<SpecOutline, PatentError> {
    let sections = &disclosure.sections;

    let title = extract(sections, &["发明名称", "title"]).unwrap_or_default();

    let tech_field = build_section(sections, &["技术领域", "tech_field"], "技术领域");

    let background = build_section(sections, &["背景技术", "background"], "背景技术");

    let summary = build_summary_section(sections);

    let brief_description = build_section(sections, &["附图说明", "brief_description"], "附图说明");

    let embodiment_raw = extract(sections, &["具体实施方式", "embodiments"]).unwrap_or_default();
    let embodiments = parse_embodiments(&embodiment_raw);

    let abstract_text = extract(sections, &["摘要", "abstract"]).unwrap_or_default();

    let sequence_listing = extract(sections, &["序列表", "sequence_listing"]);

    Ok(SpecOutline {
        title,
        tech_field,
        background,
        summary,
        brief_description,
        embodiments,
        abstract_text,
        sequence_listing,
    })
}

/// 将结构化大纲展开为符合 CNIPA 标准格式的说明书全文。
///
/// 输出顺序：
/// 1. 【技术领域】
/// 2. 【背景技术】
/// 3. 【发明内容】（含技术问题、技术方案、有益效果）
/// 4. 【附图说明】
/// 5. 【具体实施方式】（含实施例编号）
/// 6. 【摘要】
pub fn expand_outline_to_text(outline: &SpecOutline) -> String {
    let mut parts = Vec::new();

    // 技术领域
    parts.push(format_section(&outline.tech_field));

    // 背景技术
    parts.push(format_section(&outline.background));

    // 发明内容
    parts.push(format!("【发明内容】\n{}", outline.summary.content));
    for sub in &outline.summary.subsections {
        parts.push(format!("{}：{}", sub.heading, sub.content));
    }

    // 附图说明
    if !outline.brief_description.content.is_empty() {
        parts.push(format_section(&outline.brief_description));
    }

    // 具体实施方式
    if !outline.embodiments.is_empty() {
        let mut embodiment_text = String::from("【具体实施方式】\n");
        for emb in &outline.embodiments {
            embodiment_text.push_str(&format!(
                "实施例{}：{}\n{}\n",
                emb.number, emb.title, emb.description
            ));
            for ex in &emb.examples {
                embodiment_text.push_str(&format!("（{}）", ex.title));
                if !ex.conditions.is_empty() {
                    embodiment_text.push_str(&format!("条件：{}", ex.conditions.join("；")));
                }
                if !ex.results.is_empty() {
                    embodiment_text.push_str(&format!(" 结果：{}", ex.results.join("；")));
                }
                if let Some(comp) = &ex.comparative {
                    embodiment_text.push_str(&format!(" 对比：{comp}"));
                }
                embodiment_text.push('\n');
            }
            if !emb.reference_signs.is_empty() {
                let signs: Vec<String> = emb
                    .reference_signs
                    .iter()
                    .map(|(k, v)| format!("{k}-{v}"))
                    .collect();
                embodiment_text.push_str(&format!("附图标记：{}\n", signs.join("，")));
            }
        }
        parts.push(embodiment_text);
    }

    // 摘要
    if !outline.abstract_text.is_empty() {
        parts.push(format!("【摘要】\n{}", outline.abstract_text));
    }

    parts.join("\n\n")
}

// ---------------------------------------------------------------------------
// 内部辅助
// ---------------------------------------------------------------------------

/// 从 HashMap 中按候选键列表依次查找，返回第一个匹配值。
fn extract(map: &std::collections::HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(val) = map.get(*key) {
            return Some(val.clone());
        }
    }
    None
}

/// 构建 Section，无子节。
fn build_section(
    map: &std::collections::HashMap<String, String>,
    keys: &[&str],
    heading: &str,
) -> Section {
    let content = extract(map, keys).unwrap_or_default();
    Section {
        heading: heading.to_string(),
        content,
        subsections: Vec::new(),
    }
}

/// 构建发明内容 Section，自动提取技术问题/技术方案/有益效果三个子节。
fn build_summary_section(map: &std::collections::HashMap<String, String>) -> Section {
    let raw = extract(map, &["发明内容", "summary"]).unwrap_or_default();

    let mut subsections = Vec::new();

    let problem = extract(map, &["技术问题", "technical_problem"]);
    let solution = extract(map, &["技术方案", "technical_solution"]);
    let effect = extract(map, &["有益效果", "advantageous_effect"]);

    if let Some(p) = problem {
        subsections.push(Subsection {
            heading: "技术问题".to_string(),
            content: p,
        });
    }
    if let Some(s) = solution {
        subsections.push(Subsection {
            heading: "技术方案".to_string(),
            content: s,
        });
    }
    if let Some(e) = effect {
        subsections.push(Subsection {
            heading: "有益效果".to_string(),
            content: e,
        });
    }

    Section {
        heading: "发明内容".to_string(),
        content: raw,
        subsections,
    }
}

/// 将"具体实施方式"原始文本按段落拆分为多个 Embodiment。
///
/// 拆分策略：
/// 1. 优先按 "实施例N" / "实施方式N" 标记拆分
/// 2. 若无标记，按空行分段，每段一个 Embodiment
fn parse_embodiments(raw: &str) -> Vec<Embodiment> {
    if raw.trim().is_empty() {
        return Vec::new();
    }

    // 尝试按"实施例N"或"实施方式N"标记拆分
    let mut embodiments = Vec::new();
    let mut current_title = String::new();
    let mut current_body = String::new();
    let mut found_marker = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(title) = parse_embodiment_marker(trimmed) {
            if found_marker {
                embodiments.push(Embodiment {
                    number: embodiments.len() as u32 + 1,
                    title: current_title.clone(),
                    description: current_body.trim().to_string(),
                    examples: Vec::new(),
                    reference_signs: Vec::new(),
                });
                current_body.clear();
            }
            current_title = title;
            found_marker = true;
        } else if found_marker {
            if !current_body.is_empty() {
                current_body.push('\n');
            }
            current_body.push_str(trimmed);
        }
    }

    if found_marker && !current_title.is_empty() {
        embodiments.push(Embodiment {
            number: embodiments.len() as u32 + 1,
            title: current_title,
            description: current_body.trim().to_string(),
            examples: Vec::new(),
            reference_signs: Vec::new(),
        });
    }

    // 没有找到标记时按空行分段
    if !found_marker {
        let paragraphs: Vec<&str> = raw
            .split("\n\n")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        for (i, para) in paragraphs.iter().enumerate() {
            let title = format!("实施方式{}", i + 1);
            embodiments.push(Embodiment {
                number: i as u32 + 1,
                title,
                description: (*para).to_string(),
                examples: Vec::new(),
                reference_signs: Vec::new(),
            });
        }
    }

    // 重新编号保证连续
    for (i, emb) in embodiments.iter_mut().enumerate() {
        emb.number = i as u32 + 1;
    }

    embodiments
}

/// 识别 "实施例N" / "实施方式N" 行首标记，返回标题。
fn parse_embodiment_marker(line: &str) -> Option<String> {
    let prefixes = ["实施例", "实施方式"];
    for prefix in &prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            // 跳过数字
            let num_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            if num_end == 0 {
                continue;
            }
            let _num: u32 = rest[..num_end].parse().ok()?;
            // 取 "：" 或 "：" 后的内容作为标题，若无则用整行
            let after = &rest[num_end..];
            let title = after.trim_start_matches(&['：', ':', ' ', '\t'][..]).trim();
            if title.is_empty() {
                return Some(format!("{prefix}{_num}"));
            }
            return Some(title.to_string());
        }
    }
    None
}

/// 格式化单个 Section 为 CNIPA 标准文本。
fn format_section(section: &Section) -> String {
    if section.content.is_empty() && section.subsections.is_empty() {
        return String::new();
    }
    let mut out = format!("【{}】\n{}", section.heading, section.content);
    for sub in &section.subsections {
        out.push_str(&format!("\n{}：{}", sub.heading, sub.content));
    }
    out
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sections() -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("发明名称".to_string(), "一种数据处理方法".to_string());
        map.insert(
            "技术领域".to_string(),
            "本发明涉及数据处理技术领域。".to_string(),
        );
        map.insert(
            "背景技术".to_string(),
            "现有技术在处理大规模数据时效率低下。".to_string(),
        );
        map.insert(
            "发明内容".to_string(),
            "本发明提供一种高效的数据处理方法。".to_string(),
        );
        map.insert("技术问题".to_string(), "如何提高数据处理效率。".to_string());
        map.insert(
            "技术方案".to_string(),
            "采用分布式并行计算框架。".to_string(),
        );
        map.insert("有益效果".to_string(), "处理速度提升50%以上。".to_string());
        map.insert(
            "附图说明".to_string(),
            "图1是本发明的方法流程图。".to_string(),
        );
        map.insert(
            "具体实施方式".to_string(),
            "实施例1：采用MapReduce框架\n步骤包括数据分片、映射、归约。\n\n实施例2：采用Spark框架\n步骤包括RDD转换和行动操作。".to_string(),
        );
        map.insert(
            "摘要".to_string(),
            "本发明公开了一种高效的数据处理方法。".to_string(),
        );
        map
    }

    fn make_disclosure(sections: std::collections::HashMap<String, String>) -> DisclosureDoc {
        DisclosureDoc {
            raw_text: String::new(),
            sections,
            confidence: 1.0,
        }
    }

    #[test]
    fn test_generate_outline_from_sections() {
        let disclosure = make_disclosure(make_sections());
        let outline = generate_spec_outline(&disclosure).expect("should succeed");

        assert_eq!(outline.title, "一种数据处理方法");
        assert_eq!(outline.tech_field.heading, "技术领域");
        assert_eq!(outline.tech_field.content, "本发明涉及数据处理技术领域。");
        assert_eq!(outline.background.heading, "背景技术");
        assert_eq!(outline.summary.heading, "发明内容");
        assert_eq!(outline.summary.subsections.len(), 3);
        assert_eq!(outline.summary.subsections[0].heading, "技术问题");
        assert_eq!(outline.summary.subsections[1].heading, "技术方案");
        assert_eq!(outline.summary.subsections[2].heading, "有益效果");
        assert!(outline.abstract_text.contains("高效的数据处理方法"));
    }

    #[test]
    fn test_expand_outline_format() {
        let disclosure = make_disclosure(make_sections());
        let outline = generate_spec_outline(&disclosure).expect("should succeed");
        let text = expand_outline_to_text(&outline);

        assert!(text.contains("【技术领域】"));
        assert!(text.contains("【背景技术】"));
        assert!(text.contains("【发明内容】"));
        assert!(text.contains("【附图说明】"));
        assert!(text.contains("【具体实施方式】"));
        assert!(text.contains("【摘要】"));
        assert!(text.contains("技术问题："));
        assert!(text.contains("技术方案："));
        assert!(text.contains("有益效果："));
    }

    #[test]
    fn test_embodiment_numbering() {
        let disclosure = make_disclosure(make_sections());
        let outline = generate_spec_outline(&disclosure).expect("should succeed");

        assert_eq!(outline.embodiments.len(), 2);
        assert_eq!(outline.embodiments[0].number, 1);
        assert_eq!(outline.embodiments[1].number, 2);

        let text = expand_outline_to_text(&outline);
        assert!(text.contains("实施例1"));
        assert!(text.contains("实施例2"));
    }

    #[test]
    fn test_empty_sections_produce_valid_outline() {
        let empty = std::collections::HashMap::new();
        let disclosure = make_disclosure(empty);
        let outline = generate_spec_outline(&disclosure).expect("should succeed");

        assert!(outline.title.is_empty());
        assert!(outline.embodiments.is_empty());
        assert!(outline.abstract_text.is_empty());

        let text = expand_outline_to_text(&outline);
        assert!(!text.is_empty()); // still produces section headers
    }

    #[test]
    fn test_fallback_paragraph_splitting() {
        let mut sections = std::collections::HashMap::new();
        sections.insert(
            "具体实施方式".to_string(),
            "第一种方案是使用卷积神经网络。\n详细步骤包括数据预处理和模型训练。\n\n第二种方案是使用循环神经网络。\n详细步骤包括序列编码和解码。".to_string(),
        );
        let disclosure = make_disclosure(sections);
        let outline = generate_spec_outline(&disclosure).expect("should succeed");

        assert_eq!(outline.embodiments.len(), 2);
        assert_eq!(outline.embodiments[0].number, 1);
        assert_eq!(outline.embodiments[1].number, 2);
    }
}
