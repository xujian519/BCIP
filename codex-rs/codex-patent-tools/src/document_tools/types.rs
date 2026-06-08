use serde::Deserialize;

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
pub struct ReadFileInput {
    /// 文件路径（绝对路径或相对于工作目录的路径）
    pub file_path: String,
    /// 最大字符数。默认 500_000
    pub max_chars: Option<usize>,
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

#[derive(Debug, Deserialize)]
pub struct DocumentParserInput {
    /// 文档文件路径。支持 PDF/DOCX/DOC/ODT/RTF/Pages 等。
    pub file_path: String,
    /// 是否启用 OCR（用于扫描版文档）。默认 false。
    pub ocr_enabled: Option<bool>,
    /// OCR 语言（如 "chi_sim+eng"）。默认 "chi_sim+eng"。
    pub ocr_language: Option<String>,
    /// 页面范围（如 "1-5,10"）。None 表示全部页面。
    pub pages: Option<String>,
    /// Markdown 中是否插入页码分隔符。默认 true。
    pub page_breaks: Option<bool>,
    /// 输出最大字符数。默认 500_000。
    pub max_chars: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ExportInput {
    pub content: serde_json::Value,
    pub export_type: String, // "claims" / "oa_response" / "specification" / "analysis_report"
    pub output_path: Option<String>,
}
