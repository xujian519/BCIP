pub mod types;

use codex_patent_core::error::PatentError;
pub use types::*;

// ── LiteParse singleton (feature-gated) ────────────────────────────────

#[cfg(feature = "document-pdf")]
fn shared_liteparse() -> &'static liteparse::LiteParse {
    use std::sync::OnceLock;

    static INSTANCE: OnceLock<liteparse::LiteParse> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let ocr_enabled = std::env::var("LITEPARSE_OCR_ENABLED")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(false);
        let ocr_language =
            std::env::var("LITEPARSE_OCR_LANGUAGE").unwrap_or_else(|_| "chi_sim+eng".to_string());
        let max_pages = std::env::var("LITEPARSE_MAX_PAGES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);
        let config = liteparse::LiteParseConfig {
            ocr_enabled,
            ocr_language,
            max_pages,
            dpi: 200.0,
            ..Default::default()
        };
        liteparse::LiteParse::new(config)
    })
}

// ── Document tool functions ────────────────────────────────────────────

pub fn docx_tools(input: DocxInput) -> Result<serde_json::Value, PatentError> {
    let markdown = &input.markdown;
    if markdown.is_empty() {
        return Err(PatentError::Validation("markdown 内容不能为空".into()));
    }

    // 将 Markdown 段落转为简单文本段落
    let paragraphs: Vec<&str> = markdown.lines().filter(|l| !l.trim().is_empty()).collect();

    if paragraphs.is_empty() {
        return Err(PatentError::Validation("解析后无有效段落".into()));
    }

    let output_path = input
        .output_path
        .clone()
        .unwrap_or_else(|| "output.docx".into());

    // 生成简化的 DOCX XML 内容
    let mut body_xml = String::new();
    for para in &paragraphs {
        let is_heading = para.starts_with("# ");
        let text = if is_heading { &para[2..] } else { para };
        let text_escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        let ppr = if is_heading {
            r#"<w:pPr><w:pStyle w:val="Heading1"/><w:jc w:val="center"/></w:pPr>"#
        } else {
            r#"<w:pPr><w:pStyle w:val="Normal"/></w:pPr>"#
        };

        let font_size = if is_heading { "28" } else { "21" };
        body_xml.push_str(&format!(
            r#"<w:p>{}<w:r><w:rPr><w:sz w:val="{}"/><w:szCs w:val="{}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
            ppr, font_size, font_size, text_escaped
        ));
    }

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{}</w:body>
</w:document>"#,
        body_xml
    );

    let docx_bytes = build_minimal_docx(content_types, rels, &document_xml)?;

    // 写入文件
    let dir = std::path::Path::new(&output_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(dir).ok();
    std::fs::write(&output_path, &docx_bytes)?;

    let word_count = markdown.chars().count();
    Ok(serde_json::json!({
        "output_path": output_path,
        "word_count": word_count,
        "paragraph_count": paragraphs.len(),
        "template": input.template.unwrap_or_else(|| "default".into()),
    }))
}

pub fn markdown_parser(input: MarkdownInput) -> Result<serde_json::Value, PatentError> {
    let stats = codex_patent_text::text_stats(&input.text);
    Ok(serde_json::json!({
        "stats": {
            "chars": stats.char_count,
            "words": stats.word_count,
            "cjk_chars": stats.cjk_char_count,
            "lines": stats.line_count,
        },
        "format_options": input.format_options,
    }))
}

/// Read a local text file and return its content.
pub fn read_file(input: ReadFileInput) -> Result<serde_json::Value, PatentError> {
    let path = std::path::Path::new(&input.file_path);
    if !path.exists() {
        return Err(PatentError::NotFound(format!(
            "文件不存在: {}",
            input.file_path
        )));
    }
    if !path.is_file() {
        return Err(PatentError::Validation(format!(
            "路径不是文件: {}",
            input.file_path
        )));
    }

    // Security: block sensitive paths
    check_path_security(path)?;

    // File size pre-check to avoid OOM
    const MAX_FILE_BYTES: u64 = 100 * 1024 * 1024; // 100 MB
    let file_size = std::fs::metadata(path).map_err(PatentError::Io)?.len();
    if file_size > MAX_FILE_BYTES {
        return Err(PatentError::Validation(format!(
            "文件过大（{} MB），超过最大限制（100 MB）",
            file_size / (1024 * 1024)
        )));
    }

    // Determine if we need binary detection based on extension
    const TEXT_EXTENSIONS: &[&str] = &[
        "md",
        "txt",
        "csv",
        "tsv",
        "json",
        "xml",
        "yaml",
        "yml",
        "toml",
        "ini",
        "cfg",
        "conf",
        "log",
        "rs",
        "py",
        "js",
        "ts",
        "tsx",
        "jsx",
        "html",
        "htm",
        "css",
        "scss",
        "less",
        "sh",
        "bash",
        "zsh",
        "sql",
        "gitignore",
        "env",
        "properties",
        "gradle",
        "cmake",
        "makefile",
        "dockerfile",
        "r",
        "go",
        "java",
        "kt",
        "swift",
        "c",
        "cpp",
        "h",
        "hpp",
        "rb",
        "php",
        "lua",
        "pl",
        "ex",
        "exs",
        "erl",
        "clj",
        "vue",
        "svelte",
        "graphql",
        "proto",
        "tf",
        "hcl",
    ];
    // Common filenames without extension that are always text
    const TEXT_FILENAMES: &[&str] = &[
        "Makefile",
        "Dockerfile",
        "Vagrantfile",
        "Gemfile",
        "Rakefile",
        "Cargo.toml",
        "Cargo.lock",
        "go.mod",
        "go.sum",
        "requirements.txt",
    ];

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let is_text_ext = TEXT_EXTENSIONS.contains(&ext.as_str());
    let is_text_filename = TEXT_FILENAMES.contains(&filename);
    // Files with no extension (not dotfiles like .gitignore which have extension "gitignore")
    let no_ext = path.extension().is_none() && !filename.starts_with('.');
    // Dotfiles like .gitignore — extension() returns Some("gitignore") which is in TEXT_EXTENSIONS
    let is_dotfile_with_known_ext = filename.starts_with('.') && is_text_ext;

    let needs_binary_check =
        !is_text_ext && !is_text_filename && !no_ext && !is_dotfile_with_known_ext;

    if needs_binary_check {
        // Read only the first 8KB for binary detection (not the whole file)
        let file = std::fs::File::open(path).map_err(PatentError::Io)?;
        let mut buf = [0u8; 8192];
        let bytes_read = std::io::Read::read(&mut std::io::BufReader::new(file), &mut buf)
            .map_err(PatentError::Io)?;
        if buf[..bytes_read].contains(&0) {
            return Err(PatentError::Validation(format!(
                "文件 {} 看起来是二进制文件，请使用 DocumentParser 工具处理",
                input.file_path
            )));
        }
    }

    let content = std::fs::read_to_string(path).map_err(PatentError::Io)?;

    let max_chars = input.max_chars.unwrap_or(500_000);
    let original_char_count = content.chars().count();
    let truncated = original_char_count > max_chars;
    let content = if truncated {
        let mut s: String = content.chars().take(max_chars).collect();
        s.push_str(&format!(
            "\n\n[... 内容已截断，原始文件共 {} 字符 ...]",
            original_char_count
        ));
        s
    } else {
        content
    };

    let stats = codex_patent_text::text_stats(&content);
    Ok(serde_json::json!({
        "tool": "ReadFile",
        "file_path": input.file_path,
        "content": content,
        "stats": {
            "chars": stats.char_count,
            "words": stats.word_count,
            "cjk_chars": stats.cjk_char_count,
            "lines": stats.line_count,
        },
        "truncated": truncated,
    }))
}

/// Check if the path is safe to read. Blocks sensitive system/user paths.
fn check_path_security(path: &std::path::Path) -> Result<(), PatentError> {
    // Block well-known sensitive directories
    const BLOCKED_PREFIXES: &[&str] = &["/etc/passwd", "/etc/shadow", "/etc/ssh", "/.ssh/"];
    let path_str = path.to_string_lossy();
    // Also check the canonical (resolved) path to catch symlinks
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let canonical_str = canonical.to_string_lossy();

    for blocked in BLOCKED_PREFIXES {
        if path_str.contains(blocked) || canonical_str.contains(blocked) {
            return Err(PatentError::Validation(format!(
                "出于安全考虑，不允许读取该路径: {}",
                path.display()
            )));
        }
    }
    // Block home-directory sensitive folders
    if let Some(home) = dirs::home_dir() {
        let sensitive = [".ssh", ".gnupg", ".aws", ".kube"];
        for s in &sensitive {
            let sensitive_path = home.join(s);
            if canonical.starts_with(&sensitive_path) {
                return Err(PatentError::Validation(format!(
                    "出于安全考虑，不允许读取 {} 目录",
                    sensitive_path.display()
                )));
            }
        }
    }
    Ok(())
}

pub fn template_library(input: TemplateInput) -> Result<serde_json::Value, PatentError> {
    let templates: std::collections::HashMap<&str, (&str, &str)> = [
        (
            "oa_response",
            ("审查意见答复模板", "一、修改说明\n二、意见陈述\n三、结论"),
        ),
        (
            "claims",
            (
                "权利要求书模板",
                "1. 一种……，其特征在于，……\n2. 根据权利要求1所述的……",
            ),
        ),
        (
            "specification",
            (
                "说明书模板",
                "技术领域\n背景技术\n发明内容\n附图说明\n具体实施方式",
            ),
        ),
        ("abstract", ("摘要模板", "本发明公开了一种……")),
        (
            "invalidation",
            (
                "无效宣告请求书模板",
                "一、请求人信息\n二、事实与理由\n三、证据清单",
            ),
        ),
    ]
    .into();
    let (name, structure) = templates
        .get(input.template_id.as_str())
        .copied()
        .unwrap_or(("通用模板", ""));
    let rendered = if let Some(ref vars) = input.variables {
        let mut result = structure.to_string();
        for (k, v) in vars {
            result = result.replace(&format!("{{{{{}}}}}", k), v);
        }
        result
    } else {
        structure.to_string()
    };
    Ok(serde_json::json!({"template_name": name, "structure": rendered}))
}

pub fn export_tool(input: ExportInput) -> Result<serde_json::Value, PatentError> {
    let content_str = match &input.content {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    if content_str.trim().is_empty() {
        return Err(PatentError::Validation("导出内容不能为空".into()));
    }

    let rendered = match input.export_type.as_str() {
        "claims" => format_claims_document(&input.content),
        "oa_response" => format_oa_response(&input.content),
        "specification" => format_specification(&input.content),
        "analysis_report" => format_analysis_report(&input.content),
        _ => {
            return Err(PatentError::Validation(format!(
                "未知导出类型: {}，支持 claims/oa_response/specification/analysis_report",
                input.export_type
            )));
        }
    };

    // 如果指定了 output_path，写入文件
    if let Some(ref path) = input.output_path {
        let dir = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        std::fs::create_dir_all(dir).ok();
        std::fs::write(path, &rendered)?;
    }

    Ok(serde_json::json!({
        "export_type": input.export_type,
        "output_path": input.output_path,
        "content_length": rendered.len(),
        "content": rendered.chars().take(2000).collect::<String>(),
    }))
}

// ── Export format helpers ──────────────────────────────────────────────

fn format_claims_document(content: &serde_json::Value) -> String {
    let claims = content
        .get("claims")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    let mut doc = String::from("权 利 要 求 书\n\n");
    for (i, claim) in claims.iter().enumerate() {
        let text: String = claim
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| claim.to_string());
        doc.push_str(&format!("{}. {}\n\n", i + 1, text));
    }
    if claims.is_empty() {
        doc.push_str(&content.to_string());
    }
    doc
}

fn format_oa_response(content: &serde_json::Value) -> String {
    let strategy = content
        .get("strategy")
        .and_then(|s| s.as_str())
        .unwrap_or("argue");
    let oa_type = content
        .get("oa_type")
        .and_then(|s| s.as_str())
        .unwrap_or("");

    format!(
        "意 见 陈 述 书\n\n\
         申请人：\n\
         申请号：\n\
         发明名称：\n\n\
         尊敬的审查员：\n\n\
         申请人仔细研究了贵局发出的审查意见通知书，现针对通知书中指出的{}问题，陈述意见如下：\n\n\
         {}\n\n\
         综上所述，申请人认为修改后的权利要求书已克服审查意见中指出的缺陷，\
         符合专利法及实施细则的相关规定，恳请审查员予以审查并早日授权。\n\n\
         申请人：\n\
         日期：{}",
        oa_type,
        match strategy {
            "amend" =>
                "申请人根据审查意见对权利要求书进行了修改。修改未超出原说明书和权利要求书记载的范围。",
            "argue" => "申请人经仔细对比分析后认为，本申请与对比文件存在区别技术特征。",
            _ => "申请人结合审查意见进行了认真分析。",
        },
        chrono::Local::now().format("%Y年%m月%d日"),
    )
}

fn format_specification(content: &serde_json::Value) -> String {
    let title = content.get("title").and_then(|t| t.as_str()).unwrap_or("");
    let field = content
        .get("technical_field")
        .and_then(|f| f.as_str())
        .unwrap_or("");
    let background = content
        .get("background")
        .and_then(|b| b.as_str())
        .unwrap_or("");
    let invention = content
        .get("invention_content")
        .and_then(|i| i.as_str())
        .unwrap_or("");
    let embodiment = content
        .get("embodiments")
        .and_then(|e| e.as_str())
        .unwrap_or("");

    format!(
        "说 明 书\n\n\
         {}\n\n\
         技术领域\n{}\n\n\
         背景技术\n{}\n\n\
         发明内容\n{}\n\n\
         具体实施方式\n{}",
        title, field, background, invention, embodiment,
    )
}

fn format_analysis_report(content: &serde_json::Value) -> String {
    let mut report = String::from("专 利 分 析 报 告\n");
    report.push_str(&format!(
        "生成时间: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    report.push_str(&format!(
        "{}\n",
        serde_json::to_string_pretty(content).unwrap_or_default()
    ));
    report
}

// ── PDF tools: LiteParse implementation (feature-gated) ───────────────

#[cfg(feature = "document-pdf")]
fn format_as_markdown(
    pages: &[liteparse::ParsedPage],
    page_breaks: bool,
    max_chars: usize,
) -> String {
    let mut md = String::new();
    for (i, page) in pages.iter().enumerate() {
        if page_breaks && i > 0 {
            md.push_str("\n---\n\n");
        }
        if page_breaks {
            md.push_str(&format!("<!-- Page {} -->\n\n", page.page_number));
        }
        md.push_str(&page.text);
        md.push('\n');
    }
    if md.len() > max_chars {
        md.truncate(max_chars);
        md.push_str("\n\n[... 内容已截断 ...]");
    }
    md
}

#[cfg(feature = "document-pdf")]
pub async fn pdf_tools(input: PdfInput) -> Result<serde_json::Value, PatentError> {
    let file_path = input
        .file_path
        .as_deref()
        .ok_or_else(|| PatentError::Validation("file_path is required".into()))?;

    let mut config = shared_liteparse().config().clone();
    if let Some(ref pages) = input.pages {
        config.target_pages = Some(pages.clone());
    }
    if let Some(ocr) = input.ocr_enabled {
        config.ocr_enabled = ocr;
    }
    if let Some(ref lang) = input.ocr_language {
        config.ocr_language = lang.clone();
    }

    let lp = liteparse::LiteParse::new(config);
    let result = lp
        .parse(file_path)
        .await
        .map_err(|e| PatentError::DocumentParse(format!("pdf_tools: {e}")))?;

    match input.operation.as_str() {
        "extract_text" => Ok(serde_json::json!({
            "operation": "extract_text",
            "text": result.text,
            "page_count": result.pages.len(),
        })),
        "parse_with_layout" => {
            let pages: Vec<serde_json::Value> = result
                .pages
                .iter()
                .map(|p| {
                    let items: Vec<serde_json::Value> = p
                        .text_items
                        .iter()
                        .map(|ti| {
                            serde_json::json!({
                                "text": ti.text,
                                "x": ti.x,
                                "y": ti.y,
                                "width": ti.width,
                                "height": ti.height,
                                "font_size": ti.font_size,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "page_number": p.page_number,
                        "page_width": p.page_width,
                        "page_height": p.page_height,
                        "text": p.text,
                        "text_items": items,
                    })
                })
                .collect();
            Ok(serde_json::json!({
                "operation": "parse_with_layout",
                "pages": pages,
                "page_count": result.pages.len(),
                "has_layout": true,
            }))
        }
        "parse_full" => {
            let pages: Vec<serde_json::Value> = result
                .pages
                .iter()
                .map(|p| {
                    let items: Vec<serde_json::Value> = p
                        .text_items
                        .iter()
                        .map(|ti| {
                            serde_json::json!({
                                "text": ti.text,
                                "x": ti.x,
                                "y": ti.y,
                                "width": ti.width,
                                "height": ti.height,
                                "rotation": ti.rotation,
                                "font_name": ti.font_name,
                                "font_size": ti.font_size,
                                "confidence": ti.confidence,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "page_number": p.page_number,
                        "page_width": p.page_width,
                        "page_height": p.page_height,
                        "text": p.text,
                        "text_items": items,
                    })
                })
                .collect();
            Ok(serde_json::json!({
                "operation": "parse_full",
                "pages": pages,
                "text": result.text,
                "page_count": result.pages.len(),
                "has_layout": true,
            }))
        }
        _ => Ok(serde_json::json!({
            "operation": input.operation,
            "supported_operations": ["extract_text", "parse_with_layout", "parse_full"],
        })),
    }
}

#[cfg(feature = "document-pdf")]
pub async fn ocr_bridge(input: OcrInput) -> Result<serde_json::Value, PatentError> {
    let lang = input.language.unwrap_or_else(|| "chi_sim+eng".into());

    let config = liteparse::LiteParseConfig {
        ocr_enabled: true,
        ocr_language: lang.clone(),
        max_pages: 100,
        dpi: 200.0,
        ..Default::default()
    };
    let lp = liteparse::LiteParse::new(config);

    let result = lp
        .parse(&input.image_path)
        .await
        .map_err(|e| PatentError::DocumentParse(format!("ocr_bridge: {e}")))?;

    Ok(serde_json::json!({
        "operation": input.operation.as_deref().unwrap_or("recognize"),
        "image_path": input.image_path,
        "language": lang,
        "text": result.text,
        "page_count": result.pages.len(),
        "word_count": result.text.chars().count(),
    }))
}

#[cfg(feature = "document-pdf")]
pub async fn document_parser(input: DocumentParserInput) -> Result<serde_json::Value, PatentError> {
    let path = std::path::Path::new(&input.file_path);
    if !path.exists() {
        return Err(PatentError::NotFound(format!(
            "文件不存在: {}",
            input.file_path
        )));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut config = shared_liteparse().config().clone();
    if let Some(ocr) = input.ocr_enabled {
        config.ocr_enabled = ocr;
    }
    if let Some(ref lang) = input.ocr_language {
        config.ocr_language = lang.clone();
    }
    if let Some(ref pages) = input.pages {
        config.target_pages = Some(pages.clone());
    }
    config.quiet = true;

    let lp = liteparse::LiteParse::new(config);
    let result = lp
        .parse(&input.file_path)
        .await
        .map_err(|e| PatentError::DocumentParse(format!("document_parser: {e}")))?;

    let page_breaks = input.page_breaks.unwrap_or(true);
    let max_chars = input.max_chars.unwrap_or(500_000);
    let markdown = format_as_markdown(&result.pages, page_breaks, max_chars);
    let truncated = markdown.len() >= max_chars;

    Ok(serde_json::json!({
        "tool": "DocumentParser",
        "file_path": input.file_path,
        "source_format": ext,
        "markdown": markdown,
        "page_count": result.pages.len(),
        "char_count": markdown.chars().count(),
        "ocr_enabled": input.ocr_enabled.unwrap_or(false),
        "truncated": truncated,
    }))
}

#[cfg(feature = "document-pdf")]
pub async fn pdf_screenshot(input: ScreenshotInput) -> Result<serde_json::Value, PatentError> {
    let mut config = shared_liteparse().config().clone();
    if let Some(dpi) = input.dpi {
        config.dpi = dpi as f32;
    }
    let lp = liteparse::LiteParse::new(config);

    let screenshots = lp
        .screenshot(&input.file_path, input.pages)
        .await
        .map_err(|e| PatentError::DocumentParse(format!("pdf_screenshot: {e}")))?;

    let results: Vec<serde_json::Value> = screenshots
        .into_iter()
        .map(|s| {
            let b64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &s.image_bytes);
            serde_json::json!({
                "page": s.page_num,
                "image_base64": b64,
                "width": s.width,
                "height": s.height,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "file_path": input.file_path,
        "screenshots": results,
        "count": results.len(),
    }))
}

// ── PDF tools: stub fallback (no LiteParse) ────────────────────────────

#[cfg(not(feature = "document-pdf"))]
pub async fn pdf_tools(input: PdfInput) -> Result<serde_json::Value, PatentError> {
    match input.operation.as_str() {
        "extract_text" => Ok(serde_json::json!({
            "operation": "extract_text",
            "text_length": input.content.as_ref().map_or(0, |c| c.len()),
            "pages": 1,
        })),
        "parse" => Ok(serde_json::json!({
            "operation": "parse",
            "file": input.file_path,
            "text": input.content,
        })),
        _ => Ok(serde_json::json!({
            "operation": input.operation,
            "supported_operations": ["extract_text", "parse"],
            "hint": "enable feature 'document-pdf' for real PDF parsing",
        })),
    }
}

#[cfg(not(feature = "document-pdf"))]
pub async fn ocr_bridge(input: OcrInput) -> Result<serde_json::Value, PatentError> {
    let lang = input.language.unwrap_or_else(|| "chi_sim+eng".into());
    Ok(serde_json::json!({
        "operation": input.operation.as_deref().unwrap_or("recognize"),
        "image_path": input.image_path,
        "language": lang,
        "supported_backends": ["tesseract", "omlx_vision"],
        "hint": "enable feature 'document-pdf' for real OCR via LiteParse",
    }))
}

#[cfg(not(feature = "document-pdf"))]
pub async fn document_parser(input: DocumentParserInput) -> Result<serde_json::Value, PatentError> {
    Ok(serde_json::json!({
        "tool": "DocumentParser",
        "file_path": input.file_path,
        "supported_formats": ["pdf", "docx", "doc", "odt", "rtf", "pages"],
        "hint": "enable feature 'document-pdf' for real document parsing",
    }))
}

// ── Tool registration ──────────────────────────────────────────────────

pub fn register_document_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();

    t.insert("DocxTools".into(), |input| {
        Box::pin(async move {
            let parsed: DocxInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            docx_tools(parsed).map_err(|e| e.to_string())
        })
    });

    t.insert("PdfTools".into(), |input| {
        Box::pin(async move {
            let parsed: PdfInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            pdf_tools(parsed).await.map_err(|e| e.to_string())
        })
    });

    t.insert("OcrBridge".into(), |input| {
        Box::pin(async move {
            let parsed: OcrInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            ocr_bridge(parsed).await.map_err(|e| e.to_string())
        })
    });

    t.insert("MarkdownParser".into(), |input| {
        Box::pin(async move {
            let parsed: MarkdownInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            markdown_parser(parsed).map_err(|e| e.to_string())
        })
    });

    t.insert("ReadFile".into(), |input| {
        Box::pin(async move {
            let parsed: ReadFileInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            read_file(parsed).map_err(|e| e.to_string())
        })
    });

    t.insert("TemplateLibrary".into(), |input| {
        Box::pin(async move {
            let parsed: TemplateInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            template_library(parsed).map_err(|e| e.to_string())
        })
    });

    t.insert("ExportTool".into(), |input| {
        Box::pin(async move {
            let parsed: ExportInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
            export_tool(parsed).map_err(|e| e.to_string())
        })
    });

    t.insert("DocumentParser".into(), |input| {
        Box::pin(async move {
            let parsed: DocumentParserInput =
                serde_json::from_value(input).map_err(|e| e.to_string())?;
            document_parser(parsed).await.map_err(|e| e.to_string())
        })
    });

    #[cfg(feature = "document-pdf")]
    t.insert("PdfScreenshot".into(), |input| {
        Box::pin(async move {
            let parsed: ScreenshotInput =
                serde_json::from_value(input).map_err(|e| e.to_string())?;
            pdf_screenshot(parsed).await.map_err(|e| e.to_string())
        })
    });

    t
}

// ── Minimal DOCX ZIP builder (no external deps) ────────────────────────

fn build_minimal_docx(
    content_types: &str,
    rels: &str,
    document_xml: &str,
) -> Result<Vec<u8>, PatentError> {
    let files: [(&[u8], &[u8]); 3] = [
        (b"[Content_Types].xml", content_types.as_bytes()),
        (b"_rels/.rels", rels.as_bytes()),
        (b"word/document.xml", document_xml.as_bytes()),
    ];

    let mut result = Vec::new();
    let mut central_dir = Vec::new();
    let mut offset: u32 = 0;

    for (name, data) in &files {
        let crc = crc32(data);

        // Local file header (30 + name_len + data_len bytes)
        result.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // PK signature
        result.extend_from_slice(&20u16.to_le_bytes()); // version needed
        result.extend_from_slice(&0u16.to_le_bytes()); // flags
        result.extend_from_slice(&0u16.to_le_bytes()); // compression (stored)
        result.extend_from_slice(&0u16.to_le_bytes()); // mod time
        result.extend_from_slice(&0u16.to_le_bytes()); // mod date
        result.extend_from_slice(&crc.to_le_bytes()); // crc32
        result.extend_from_slice(&(data.len() as u32).to_le_bytes()); // compressed size
        result.extend_from_slice(&(data.len() as u32).to_le_bytes()); // uncompressed size
        result.extend_from_slice(&(name.len() as u16).to_le_bytes()); // name length
        result.extend_from_slice(&0u16.to_le_bytes()); // extra length
        result.extend_from_slice(name); // file name
        result.extend_from_slice(data); // file data

        // Central directory entry
        central_dir.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]); // PK signature
        central_dir.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central_dir.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // flags
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // compression
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // mod time
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // mod date
        central_dir.extend_from_slice(&crc.to_le_bytes()); // crc32
        central_dir.extend_from_slice(&(data.len() as u32).to_le_bytes()); // compressed
        central_dir.extend_from_slice(&(data.len() as u32).to_le_bytes()); // uncompressed
        central_dir.extend_from_slice(&(name.len() as u16).to_le_bytes()); // name length
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // extra length
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // comment length
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // disk number
        central_dir.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central_dir.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central_dir.extend_from_slice(&offset.to_le_bytes()); // local header offset
        central_dir.extend_from_slice(name);

        offset += 30 + name.len() as u32 + data.len() as u32;
    }

    let cd_start = result.len() as u32;
    result.extend_from_slice(&central_dir);
    let cd_size = central_dir.len() as u32;

    // End of central directory record
    result.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]); // PK signature
    result.extend_from_slice(&0u16.to_le_bytes()); // disk number
    result.extend_from_slice(&0u16.to_le_bytes()); // disk with cd
    result.extend_from_slice(&(files.len() as u16).to_le_bytes()); // entries on disk
    result.extend_from_slice(&(files.len() as u16).to_le_bytes()); // total entries
    result.extend_from_slice(&cd_size.to_le_bytes()); // cd size
    result.extend_from_slice(&cd_start.to_le_bytes()); // cd offset
    result.extend_from_slice(&0u16.to_le_bytes()); // comment length

    Ok(result)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn docx_tools_empty_markdown_error() {
        let input = DocxInput {
            markdown: "".into(),
            output_path: None,
            template: None,
        };
        assert!(docx_tools(input).is_err());
    }

    #[test]
    fn docx_tools_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.docx");
        let input = DocxInput {
            markdown: "# 测试标题\n这是正文内容".into(),
            output_path: Some(path.to_str().unwrap().into()),
            template: None,
        };
        let result = docx_tools(input).unwrap();
        assert!(path.exists());
        assert!(result["word_count"].as_u64().unwrap() > 0);
        assert_eq!(result["paragraph_count"], 2);
    }

    #[test]
    fn markdown_parser_returns_stats() {
        let input = MarkdownInput {
            text: "这是一段测试文本\n第二行内容".into(),
            format_options: None,
        };
        let result = markdown_parser(input).unwrap();
        let stats = &result["stats"];
        assert!(stats["chars"].as_u64().unwrap() > 0);
        assert!(stats["lines"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn template_library_known_id() {
        let input = TemplateInput {
            template_id: "oa_response".into(),
            variables: None,
        };
        let result = template_library(input).unwrap();
        assert_eq!(result["template_name"], "审查意见答复模板");
    }

    #[test]
    fn template_library_unknown_id() {
        let input = TemplateInput {
            template_id: "nonexistent".into(),
            variables: None,
        };
        let result = template_library(input).unwrap();
        assert_eq!(result["template_name"], "通用模板");
    }

    #[test]
    fn template_library_variable_replacement_no_effect_without_placeholders() {
        let mut vars = HashMap::new();
        vars.insert("key".into(), "value".into());
        let input = TemplateInput {
            template_id: "oa_response".into(),
            variables: Some(vars),
        };
        let result = template_library(input).unwrap();
        assert_eq!(result["template_name"], "审查意见答复模板");
        assert!(!result["structure"].as_str().unwrap().contains("value"));
    }

    #[test]
    fn export_tool_empty_content_error() {
        let input = ExportInput {
            content: serde_json::json!(""),
            export_type: "claims".into(),
            output_path: None,
        };
        assert!(export_tool(input).is_err());
    }

    #[test]
    fn export_tool_claims_format() {
        let input = ExportInput {
            content: serde_json::json!({"claims": ["一种装置", "根据权利要求1所述的装置"]}),
            export_type: "claims".into(),
            output_path: None,
        };
        let result = export_tool(input).unwrap();
        assert!(
            result["content"]
                .as_str()
                .unwrap()
                .contains("权 利 要 求 书")
        );
    }

    #[test]
    fn export_tool_invalid_type_error() {
        let input = ExportInput {
            content: serde_json::json!("some content"),
            export_type: "invalid_type".into(),
            output_path: None,
        };
        assert!(export_tool(input).is_err());
    }

    #[test]
    fn export_tool_oa_response_format() {
        let input = ExportInput {
            content: serde_json::json!({"strategy": "argue", "oa_type": "新颖性"}),
            export_type: "oa_response".into(),
            output_path: None,
        };
        let result = export_tool(input).unwrap();
        assert!(
            result["content"]
                .as_str()
                .unwrap()
                .contains("意 见 陈 述 书")
        );
    }
}
