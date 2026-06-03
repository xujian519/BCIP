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
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub operation: String,
}

#[derive(Debug, Deserialize)]
pub struct OcrInput {
    pub image_path: String,
    pub language: Option<String>,
    pub operation: Option<String>,
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

    pub fn pdf_tools(input: PdfInput) -> Result<serde_json::Value, String> {
        match input.operation.as_str() {
            "extract_text" => Ok(
                serde_json::json!({"operation": "extract_text", "text_length": input.content.as_ref().map_or(0, |c| c.len()), "pages": 1}),
            ),
            "parse" => Ok(
                serde_json::json!({"operation": "parse", "file": input.file_path, "text": input.content}),
            ),
            _ => Ok(
                serde_json::json!({"operation": input.operation, "supported_operations": ["extract_text", "parse"]}),
            ),
        }
    }

    pub fn ocr_bridge(input: OcrInput) -> Result<serde_json::Value, String> {
        let lang = input.language.unwrap_or_else(|| "chi_sim+eng".into());
        Ok(serde_json::json!({
            "operation": input.operation.as_deref().unwrap_or("recognize"),
            "image_path": input.image_path,
            "language": lang,
            "supported_backends": ["tesseract", "omlx_vision"],
            "setup_tesseract": "brew install tesseract tesseract-lang",
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
            DocumentTools::pdf_tools(parsed)
        })
    });
    t.insert("OcrBridge".into(), |input| {
        Box::pin(async move {
            let parsed: OcrInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DocumentTools::ocr_bridge(parsed)
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
    t
}
