use anyhow::Result;
use regex::Regex;
use std::path::Path;

use crate::models::IpcEntry;

/// 解析 IPC 2026.01 原始文本文件，返回结构化条目
pub fn parse_ipc_files(dir: &Path) -> Result<Vec<IpcEntry>> {
    let mut all_entries = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "txt") {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if !file_name.contains("2026.01") {
                continue;
            }
            let entries = parse_single_ipc_file(&path)?;
            println!("      {} -> {} 条", file_name, entries.len());
            all_entries.extend(entries);
        }
    }

    println!("      IPC 总计: {} 条", all_entries.len());
    Ok(all_entries)
}

fn parse_single_ipc_file(path: &Path) -> Result<Vec<IpcEntry>> {
    let content = std::fs::read_to_string(path)?;
    let source = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 匹配行首的 IPC 编码
    // Section: A, B, C, ...
    // Class: A01, B60, ...
    // Subclass: A01B, B60K, ...
    // Group/Subgroup: A01B1/00, A01B1/02, ...
    let ipc_line_re = Regex::new(
        r"^([A-H]\d{2}[A-Z]\d+/\d{2,})\s+((?:\.+\s+)?(.+?))\s*(?:\[(\d{4}\.\d{2})\])?\s*$",
    )?;
    let class_re = Regex::new(r"^([A-H]\d{2}[A-Z])\s+(.+?)(?:\s*\[(\d{4}\.\d{2})\])?\s*$")?;
    let top_class_re = Regex::new(r"^([A-H]\d{2})\s+(.+?)(?:\s*\[(\d{4}\.\d{2})\])?\s*$")?;
    let section_re = Regex::new(r"^([A-H])\s+([A-H]部.+?)(?:\s*)$")?;

    // 页眉/页脚过滤
    let page_header_re = Regex::new(r"^\s*\d{4}\.\d{2}版IPC分类表")?;
    let page_footer_re = Regex::new(r"^\s*第\s*\d+\s*页")?;
    let version_tag_re = Regex::new(r"\[\d{4}\.\d{2}\]")?;

    let mut entries = Vec::new();
    let mut pending_class_code: Option<String> = None;
    let mut pending_class_desc = String::new();

    for line in content.lines() {
        let line = line.trim();

        // 跳过空行、页眉页脚
        if line.is_empty()
            || page_header_re.is_match(line)
            || page_footer_re.is_match(line)
            || line.starts_with("分部")
            || line.starts_with("附注")
        {
            continue;
        }

        // 1. 尝试匹配 Group/Subgroup: A01B1/00 或 A01B1/02
        if let Some(caps) = ipc_line_re.captures(line) {
            let code = caps[1].to_string();
            let desc_raw = caps.get(3).map(|m| m.as_str()).unwrap_or(&caps[2]);
            let desc = desc_raw.trim().to_string();
            let version = caps
                .get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "2026.01".into());

            // 先 flush pending class description
            if pending_class_code.is_some() && pending_class_desc.is_empty() {
                // class desc was on the next line after code
            }

            let entry = build_ipc_entry(&code, &desc, &version, &source);
            entries.push(entry);
            continue;
        }

        // 2. 尝试匹配 Subclass: A01B
        if let Some(caps) = class_re.captures(line) {
            // flush any pending class
            if let Some(cc) = pending_class_code.take() {
                let desc = pending_class_desc.trim().to_string();
                if !desc.is_empty() {
                    entries.push(build_ipc_entry(&cc, &desc, "2026.01", &source));
                }
                pending_class_desc.clear();
            }

            let code = caps[1].to_string();
            let desc = caps[2].trim().to_string();

            // subclass 描述可能跨行
            pending_class_code = Some(code);
            pending_class_desc = desc;
            continue;
        }

        // 3. 尝试匹配 Class: A01
        if let Some(caps) = top_class_re.captures(line) {
            // flush any pending class
            if let Some(cc) = pending_class_code.take() {
                let desc = pending_class_desc.trim().to_string();
                if !desc.is_empty() {
                    entries.push(build_ipc_entry(&cc, &desc, "2026.01", &source));
                }
                pending_class_desc.clear();
            }

            let code = caps[1].to_string();
            let desc = caps[2].trim().to_string();

            pending_class_code = Some(code);
            pending_class_desc = desc;
            continue;
        }

        // 4. 尝试匹配 Section: A
        if let Some(caps) = section_re.captures(line) {
            // flush pending
            if let Some(cc) = pending_class_code.take() {
                let desc = pending_class_desc.trim().to_string();
                if !desc.is_empty() {
                    entries.push(build_ipc_entry(&cc, &desc, "2026.01", &source));
                }
                pending_class_desc.clear();
            }

            let code = caps[1].to_string();
            let desc = caps[2].trim().to_string();
            entries.push(build_ipc_entry(&code, &desc, "2026.01", &source));
            continue;
        }

        // 5. 续行内容 — 追加到 pending class desc
        if pending_class_code.is_some() && !line.starts_with('[') {
            if !pending_class_desc.is_empty() {
                pending_class_desc.push(' ');
            }
            // 去掉版本标签
            let cleaned = version_tag_re.replace_all(line, "").trim().to_string();
            pending_class_desc.push_str(&cleaned);
        }
    }

    // flush last pending
    if let Some(cc) = pending_class_code.take() {
        let desc = pending_class_desc.trim().to_string();
        if !desc.is_empty() {
            entries.push(build_ipc_entry(&cc, &desc, "2026.01", &source));
        }
    }

    Ok(entries)
}

/// 根据 IPC 编码自动推断各字段
fn build_ipc_entry(code: &str, description: &str, version: &str, source_file: &str) -> IpcEntry {
    let (section, class, subclass, group_code, level, parent_code) = parse_ipc_code(code);

    IpcEntry {
        code: code.to_string(),
        section,
        class,
        subclass,
        group_code,
        level,
        parent_code,
        description: description.to_string(),
        version: version.to_string(),
        source_file: source_file.to_string(),
    }
}

/// 解析 IPC 编码，拆分为各层级组件并推断层级和父编码
fn parse_ipc_code(code: &str) -> (String, String, String, String, i32, Option<String>) {
    let section = code.chars().next().unwrap_or('A').to_string();
    let class = if code.len() >= 3 {
        format!("{}{}", &section, &code[1..3])
    } else {
        section.clone()
    };
    let subclass = if code.len() >= 4 {
        format!("{}{}", &class, &code[3..4])
    } else {
        class.clone()
    };

    // 判断层级
    if code.len() == 1 {
        // Section: A
        return (section.clone(), class, subclass, String::new(), -3, None);
    }
    if code.len() == 3 {
        // Class: A01
        return (
            section.clone(),
            class.clone(),
            subclass,
            String::new(),
            -2,
            Some(section),
        );
    }
    if code.len() == 4 {
        // Subclass: A01B
        return (
            section.clone(),
            class.clone(),
            subclass.clone(),
            String::new(),
            -1,
            Some(class),
        );
    }

    // Group/Subgroup: A01B1/00
    let group_code = code[4..].to_string();
    let slash_pos = group_code.find('/').unwrap_or(group_code.len());
    let main_group = group_code[..slash_pos].to_string();
    let sub_part = if slash_pos < group_code.len() - 1 {
        &group_code[slash_pos + 1..]
    } else {
        ""
    };

    if sub_part == "00" {
        // Main group: A01B1/00
        let parent = Some(subclass.clone());
        (section, class, subclass, group_code, 0, parent)
    } else {
        // Subgroup: determine depth by dot count in original text
        // We'll set level based on sub_part value; precise depth needs dot count
        // For now, use 1 as default — will be refined during parsing
        let parent = {
            // Try to find parent by stripping last digit
            if sub_part.len() > 2 {
                let parent_sub = format!("{}/{}", main_group, &sub_part[..sub_part.len() - 1]);
                Some(format!("{}{}", subclass, parent_sub))
            } else {
                Some(format!("{}/{:02}", main_group, 0)) // parent is main group
            }
        };
        (section, class, subclass, group_code, 1, parent)
    }
}
