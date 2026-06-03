use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConvertInput {
    pub content: serde_json::Value,
    pub input_format: String,
    pub output_format: String,
    pub patent_office_format: Option<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocxInput {
    pub markdown: String,
    pub output_path: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PdfInput {
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub operation: String,
    #[cfg(feature = "document-pdf")]
    pub pages: Option<String>,
    #[cfg(feature = "document-pdf")]
    pub ocr_enabled: Option<bool>,
    #[cfg(feature = "document-pdf")]
    pub ocr_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OcrInput {
    pub image_path: String,
    pub language: Option<String>,
    pub operation: Option<String>,
    #[cfg(feature = "document-pdf")]
    pub pages: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize)]
pub struct MarkdownInput {
    pub text: String,
    pub format_options: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateInput {
    pub template_id: String,
    pub variables: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[cfg(feature = "document-pdf")]
pub struct ScreenshotInput {
    pub file_path: String,
    pub pages: Option<Vec<u32>>,
    pub dpi: Option<u32>,
}

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

// ── DocumentTools implementation ───────────────────────────────────────

pub struct DocumentTools;

impl DocumentTools {
    pub fn format_converter(input: ConvertInput) -> Result<serde_json::Value, String> {
        let content_str = match &input.content {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let format = input.patent_office_format.as_deref().unwrap_or("CNIPA");
        Ok(serde_json::json!({
            "input_format": input.input_format,
            "output_format": input.output_format,
            "patent_office_format": format,
            "content_length": content_str.len(),
            "output_path": input.output_path,
            "status": "ready",
        }))
    }

    pub fn docx_tools(input: DocxInput) -> Result<serde_json::Value, String> {
        let word_count: usize = input.markdown.chars().count();
        Ok(serde_json::json!({
            "word_count": word_count,
            "page_estimate": (word_count as f64 / 800.0).ceil() as u32,
            "template": input.template.unwrap_or_else(|| "default".into()),
            "output_path": input.output_path.unwrap_or_else(|| "output.docx".into()),
            "market_available": false,
            "markdown_to_docx": "需安装 pandoc 或 python-docx: pip install python-docx",
        }))
    }

    pub fn markdown_parser(input: MarkdownInput) -> Result<serde_json::Value, String> {
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

    pub fn template_library(input: TemplateInput) -> Result<serde_json::Value, String> {
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
}

// ── PDF tools: LiteParse implementation (feature-gated) ───────────────

#[cfg(feature = "document-pdf")]
impl DocumentTools {
    pub async fn pdf_tools(input: PdfInput) -> Result<serde_json::Value, String> {
        let file_path = input
            .file_path
            .as_deref()
            .ok_or_else(|| "file_path is required".to_string())?;

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
            .map_err(|e| format!("PDF parse error: {e}"))?;

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

    pub async fn ocr_bridge(input: OcrInput) -> Result<serde_json::Value, String> {
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
            .map_err(|e| format!("OCR error: {e}"))?;

        Ok(serde_json::json!({
            "operation": input.operation.as_deref().unwrap_or("recognize"),
            "image_path": input.image_path,
            "language": lang,
            "text": result.text,
            "page_count": result.pages.len(),
            "word_count": result.text.chars().count(),
        }))
    }

    pub async fn pdf_screenshot(input: ScreenshotInput) -> Result<serde_json::Value, String> {
        let mut config = shared_liteparse().config().clone();
        if let Some(dpi) = input.dpi {
            config.dpi = dpi as f32;
        }
        let lp = liteparse::LiteParse::new(config);

        let screenshots = lp
            .screenshot(&input.file_path, input.pages)
            .await
            .map_err(|e| format!("Screenshot error: {e}"))?;

        let results: Vec<serde_json::Value> = screenshots
            .into_iter()
            .map(|s| {
                let b64 = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &s.image_bytes,
                );
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
}

// ── PDF tools: stub fallback (no LiteParse) ────────────────────────────

#[cfg(not(feature = "document-pdf"))]
impl DocumentTools {
    pub async fn pdf_tools(input: PdfInput) -> Result<serde_json::Value, String> {
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

    pub async fn ocr_bridge(input: OcrInput) -> Result<serde_json::Value, String> {
        let lang = input.language.unwrap_or_else(|| "chi_sim+eng".into());
        Ok(serde_json::json!({
            "operation": input.operation.as_deref().unwrap_or("recognize"),
            "image_path": input.image_path,
            "language": lang,
            "supported_backends": ["tesseract", "omlx_vision"],
            "hint": "enable feature 'document-pdf' for real OCR via LiteParse",
        }))
    }
}

// ── Tool registration ──────────────────────────────────────────────────

pub fn register_document_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();

    t.insert("FormatConverter".into(), |input| {
        Box::pin(async move {
            let parsed: ConvertInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::format_converter(parsed)
        })
    });

    t.insert("DocxTools".into(), |input| {
        Box::pin(async move {
            let parsed: DocxInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::docx_tools(parsed)
        })
    });

    t.insert("PdfTools".into(), |input| {
        Box::pin(async move {
            let parsed: PdfInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::pdf_tools(parsed).await
        })
    });

    t.insert("OcrBridge".into(), |input| {
        Box::pin(async move {
            let parsed: OcrInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::ocr_bridge(parsed).await
        })
    });

    t.insert("MarkdownParser".into(), |input| {
        Box::pin(async move {
            let parsed: MarkdownInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::markdown_parser(parsed)
        })
    });

    t.insert("TemplateLibrary".into(), |input| {
        Box::pin(async move {
            let parsed: TemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::template_library(parsed)
        })
    });

    #[cfg(feature = "document-pdf")]
    t.insert("PdfScreenshot".into(), |input| {
        Box::pin(async move {
            let parsed: ScreenshotInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::pdf_screenshot(parsed).await
        })
    });

    t
}
