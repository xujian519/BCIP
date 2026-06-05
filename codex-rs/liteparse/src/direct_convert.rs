//! Direct format conversion (bypass PDF pipeline).
//!
//! When the `direct-convert` feature is enabled, structured document formats
//! (DOCX, PPTX, XLSX, HTML, CSV) are converted directly to Markdown via the
//! `anytomd` crate — no LibreOffice, ImageMagick, or PDF conversion needed.
//! Plain-text files are read as-is with zero overhead.
//!
//! If direct conversion fails, callers should fall back to the PDF pipeline
//! (`parse_via_pdf`).

use crate::error::LiteParseError;
use crate::types::InputFormat;

/// Result of a direct (non-PDF) conversion to Markdown.
pub struct DirectConvertResult {
    /// The Markdown text produced by the conversion.
    pub markdown: String,
    /// The detected source format.
    pub source_format: InputFormat,
    /// Number of "pages" (always 1 for direct conversion, as there is no
    /// page-based layout).
    pub page_count: usize,
}

/// Convert a file on disk directly to Markdown.
///
/// Returns `Ok(DirectConvertResult)` on success, or `Err(LiteParseError)` if
/// the format is unsupported or conversion fails. Callers should fall back to
/// the PDF pipeline on error.
#[cfg(feature = "direct-convert")]
pub fn convert_file_direct(
    path: &str,
    format: &InputFormat,
) -> Result<DirectConvertResult, LiteParseError> {
    match format {
        InputFormat::Text => {
            let text = std::fs::read_to_string(path)?;
            Ok(DirectConvertResult {
                markdown: text,
                source_format: format.clone(),
                page_count: 1,
            })
        }
        InputFormat::Docx
        | InputFormat::Pptx
        | InputFormat::Xlsx
        | InputFormat::Html
        | InputFormat::Csv => {
            let options = anytomd::ConversionOptions::default();
            let result = anytomd::convert_file(path, &options).map_err(|e| {
                LiteParseError::Conversion(format!("direct conversion failed: {e}"))
            })?;
            Ok(DirectConvertResult {
                markdown: result.markdown,
                source_format: format.clone(),
                page_count: 1,
            })
        }
        _ => Err(LiteParseError::Conversion(format!(
            "format {:?} does not support direct conversion",
            format
        ))),
    }
}

/// Convert raw bytes directly to Markdown.
///
/// The `ext` parameter provides a hint for format detection (e.g. `"docx"`).
#[cfg(feature = "direct-convert")]
pub fn convert_bytes_direct(
    data: &[u8],
    ext: &str,
    format: &InputFormat,
) -> Result<DirectConvertResult, LiteParseError> {
    match format {
        InputFormat::Text => {
            let text = String::from_utf8_lossy(data).to_string();
            Ok(DirectConvertResult {
                markdown: text,
                source_format: format.clone(),
                page_count: 1,
            })
        }
        InputFormat::Docx
        | InputFormat::Pptx
        | InputFormat::Xlsx
        | InputFormat::Html
        | InputFormat::Csv => {
            let options = anytomd::ConversionOptions::default();
            let result = anytomd::convert_bytes(data, ext, &options).map_err(|e| {
                LiteParseError::Conversion(format!("direct conversion failed: {e}"))
            })?;
            Ok(DirectConvertResult {
                markdown: result.markdown,
                source_format: format.clone(),
                page_count: 1,
            })
        }
        _ => Err(LiteParseError::Conversion(format!(
            "format {:?} does not support direct conversion",
            format
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_format_passthrough() {
        let format = InputFormat::Text;
        assert!(format.supports_direct_markdown());
    }

    #[test]
    fn test_docx_format_supports_direct() {
        let format = InputFormat::from_extension("docx");
        assert_eq!(format, InputFormat::Docx);
        assert!(format.supports_direct_markdown());
    }

    #[test]
    fn test_pdf_format_not_direct() {
        let format = InputFormat::from_extension("pdf");
        assert_eq!(format, InputFormat::Pdf);
        assert!(!format.supports_direct_markdown());
    }

    #[test]
    fn test_doc_format_not_direct() {
        // Legacy .doc must go through LibreOffice → PDF
        let format = InputFormat::from_extension("doc");
        assert_eq!(format, InputFormat::Doc);
        assert!(!format.supports_direct_markdown());
    }

    #[test]
    fn test_image_format_not_direct() {
        let format = InputFormat::from_extension("png");
        assert_eq!(format, InputFormat::Image);
        assert!(!format.supports_direct_markdown());
    }

    #[test]
    fn test_unsupported_format() {
        let format = InputFormat::from_extension("xyz");
        assert!(matches!(format, InputFormat::Unsupported(_)));
        assert!(!format.supports_direct_markdown());
    }

    #[test]
    fn test_pptx_direct_but_ppt_legacy() {
        // Modern OOXML → direct
        assert!(InputFormat::from_extension("pptx").supports_direct_markdown());
        // Legacy binary → LibreOffice
        assert!(!InputFormat::from_extension("ppt").supports_direct_markdown());
        assert!(!InputFormat::from_extension("pot").supports_direct_markdown());
        assert!(!InputFormat::from_extension("odp").supports_direct_markdown());
        assert!(!InputFormat::from_extension("key").supports_direct_markdown());
    }

    #[test]
    fn test_xlsx_direct_but_xls_legacy() {
        // Modern OOXML → direct
        assert!(InputFormat::from_extension("xlsx").supports_direct_markdown());
        // Legacy binary → LibreOffice
        assert!(!InputFormat::from_extension("xls").supports_direct_markdown());
        assert!(!InputFormat::from_extension("ods").supports_direct_markdown());
    }

    #[test]
    fn test_csv_supports_direct() {
        let format = InputFormat::from_extension("csv");
        assert_eq!(format, InputFormat::Csv);
        assert!(format.supports_direct_markdown());
    }

    #[test]
    fn test_html_supports_direct() {
        let format = InputFormat::from_extension("html");
        assert_eq!(format, InputFormat::Html);
        assert!(format.supports_direct_markdown());
    }
}
