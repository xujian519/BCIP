//! Document tools integration tests.
//!
//! Tests in the `#[cfg(feature = "document-pdf")]` section require the
//! `document-pdf` feature (LiteParse + PDFium runtime). Run with:
//!
//!     cargo test -p codex-patent-tools --features document-pdf -- --test-threads=1
//!
//! PDFium is not thread-safe when multiple instances are created concurrently,
//! so single-threaded test execution is required.

use std::path::PathBuf;

#[allow(dead_code)]
fn fixtures_dir() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[allow(dead_code)]
fn minimal_pdf_path() -> PathBuf {
    fixtures_dir().join("minimal.pdf")
}

// ── Feature-gated tests (require LiteParse) ────────────────────────────

#[cfg(feature = "document-pdf")]
mod liteparse_tests {
    use super::*;
    use codex_patent_tools::register_document_tools;

    #[tokio::test]
    async fn test_pdf_extract_text() {
        let tools = register_document_tools();
        let handler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "operation": "extract_text",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["operation"], "extract_text");
        assert!(result["page_count"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_pdf_parse_with_layout() {
        let tools = register_document_tools();
        let handler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "operation": "parse_with_layout",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["operation"], "parse_with_layout");
        assert_eq!(result["has_layout"], true);
        let pages = result["pages"].as_array().unwrap();
        assert!(!pages.is_empty());
        let first_page = &pages[0];
        assert!(first_page["text_items"].is_array());
    }

    #[tokio::test]
    async fn test_pdf_parse_full() {
        let tools = register_document_tools();
        let handler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "operation": "parse_full",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["operation"], "parse_full");
        assert!(result["text"].is_string());
        assert!(result["page_count"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_pdf_tools_missing_file_path() {
        let tools = register_document_tools();
        let handler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "operation": "extract_text",
        });
        let result = handler(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file_path is required"));
    }

    #[tokio::test]
    async fn test_pdf_tools_unsupported_operation() {
        let tools = register_document_tools();
        let handler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "operation": "unknown_op",
        });
        let result = handler(input).await.unwrap();
        let supported = result["supported_operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(supported.contains(&"extract_text"));
        assert!(supported.contains(&"parse_with_layout"));
        assert!(supported.contains(&"parse_full"));
    }

    #[tokio::test]
    async fn test_pdf_screenshot() {
        let tools = register_document_tools();
        let handler = tools.get("PdfScreenshot").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "pages": [1],
            "dpi": 72,
        });
        let result = handler(input).await.unwrap();
        let screenshots = result["screenshots"].as_array().unwrap();
        assert_eq!(screenshots.len(), 1);
        assert!(screenshots[0]["image_base64"].is_string());
        assert!(screenshots[0]["width"].as_u64().unwrap() > 0);
        assert!(screenshots[0]["height"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    #[ignore = "requires tesseract runtime or OCR server URL"]
    async fn test_ocr_bridge_on_pdf() {
        let tools = register_document_tools();
        let handler = tools.get("OcrBridge").unwrap();

        let input = serde_json::json!({
            "image_path": minimal_pdf_path().to_str().unwrap(),
            "language": "eng",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["language"], "eng");
        assert!(result["text"].is_string());
    }

    #[tokio::test]
    async fn test_document_parser_pdf_file() {
        let tools = register_document_tools();
        let handler = tools.get("DocumentParser").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["tool"], "DocumentParser");
        assert_eq!(result["source_format"], "pdf");
        assert!(result["markdown"].is_string());
        assert!(result["page_count"].as_u64().unwrap() >= 1);
        assert!(result["char_count"].as_u64().unwrap() > 0);
        assert_eq!(result["ocr_enabled"], false);
        assert_eq!(result["truncated"], false);
    }

    #[tokio::test]
    async fn test_document_parser_page_breaks_false() {
        let tools = register_document_tools();
        let handler = tools.get("DocumentParser").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "page_breaks": false,
        });
        let result = handler(input).await.unwrap();
        let markdown = result["markdown"].as_str().unwrap();
        assert!(!markdown.contains("<!-- Page"));
        assert!(!markdown.contains("---"));
    }

    #[tokio::test]
    async fn test_document_parser_nonexistent_file() {
        let tools = register_document_tools();
        let handler = tools.get("DocumentParser").unwrap();

        let input = serde_json::json!({
            "file_path": "/nonexistent/path/to/file.pdf",
        });
        let result = handler(input).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("文件不存在"));
    }

    #[tokio::test]
    async fn test_document_parser_max_chars_truncation() {
        let tools = register_document_tools();
        let handler = tools.get("DocumentParser").unwrap();

        let input = serde_json::json!({
            "file_path": minimal_pdf_path().to_str().unwrap(),
            "max_chars": 10,
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["truncated"], true);
        let markdown = result["markdown"].as_str().unwrap();
        assert!(markdown.contains("内容已截断"));
    }
}

// ── Non-feature tests (always run) ─────────────────────────────────────

#[cfg(not(feature = "document-pdf"))]
mod stub_tests {
    use codex_patent_tools::ToolHandler;
    use codex_patent_tools::register_document_tools;

    #[tokio::test]
    async fn test_stub_pdf_tools() {
        let tools = register_document_tools();
        let handler: &ToolHandler = tools.get("PdfTools").unwrap();

        let input = serde_json::json!({
            "operation": "extract_text",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["operation"], "extract_text");
        assert!(result["pages"].is_u64());
    }

    #[tokio::test]
    async fn test_stub_ocr_bridge() {
        let tools = register_document_tools();
        let handler: &ToolHandler = tools.get("OcrBridge").unwrap();

        let input = serde_json::json!({
            "image_path": "/tmp/test.png",
        });
        let result = handler(input).await.unwrap();
        assert!(result["hint"].is_string());
    }

    #[test]
    fn test_pdf_screenshot_not_registered_without_feature() {
        let tools = register_document_tools();
        assert!(
            !tools.contains_key("PdfScreenshot"),
            "PdfScreenshot should not be registered without document-pdf feature"
        );
    }

    #[test]
    fn test_format_converter_removed() {
        let tools = register_document_tools();
        assert!(
            !tools.contains_key("FormatConverter"),
            "FormatConverter should have been removed"
        );
    }

    #[tokio::test]
    async fn test_markdown_parser() {
        let tools = register_document_tools();
        let handler: &ToolHandler = tools.get("MarkdownParser").unwrap();

        let input = serde_json::json!({
            "text": "这是一个测试文本",
        });
        let result = handler(input).await.unwrap();
        let stats = &result["stats"];
        assert!(stats["cjk_chars"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_document_parser_registered() {
        let tools = register_document_tools();
        assert!(
            tools.contains_key("DocumentParser"),
            "DocumentParser should always be registered"
        );
    }

    #[tokio::test]
    async fn test_document_parser_stub() {
        let tools = register_document_tools();
        let handler: &ToolHandler = tools.get("DocumentParser").unwrap();

        let input = serde_json::json!({
            "file_path": "/tmp/test.pdf",
        });
        let result = handler(input).await.unwrap();
        assert_eq!(result["tool"], "DocumentParser");
        assert!(result["hint"].is_string());
    }
}
