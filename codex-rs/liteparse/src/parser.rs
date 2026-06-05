use crate::config::LiteParseConfig;
use crate::config::parse_target_pages;
#[cfg(not(target_arch = "wasm32"))]
use crate::conversion;
#[cfg(feature = "direct-convert")]
use crate::direct_convert;
use crate::error::LiteParseError;
use crate::extract;
use crate::ocr::OcrEngine;
#[cfg(not(target_arch = "wasm32"))]
use crate::ocr::http_simple::HttpOcrEngine;
#[cfg(feature = "tesseract")]
use crate::ocr::tesseract::TesseractOcrEngine;
use crate::ocr_merge;
use crate::projection;
#[cfg(not(target_arch = "wasm32"))]
use crate::render;
use crate::types::InputFormat;
use crate::types::ParsedPage;
use crate::types::PdfInput;

/// Result of parsing a document.
pub struct ParseResult {
    /// Parsed pages with projected text layout.
    pub pages: Vec<ParsedPage>,
    /// Full document text, concatenated from all pages.
    pub text: String,
}

/// Result of rendering a single page screenshot.
#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    pub page_num: u32,
    pub width: u32,
    pub height: u32,
    pub image_bytes: Vec<u8>,
}

/// Main LiteParse orchestrator.
pub struct LiteParse {
    config: LiteParseConfig,
    /// Optional caller-provided OCR engine. When set, this overrides the
    /// built-in selection logic (HTTP OCR / Tesseract). This is the primary
    /// mechanism for plugging an OCR engine in environments without the
    /// built-ins (e.g. WASM, where the JS side supplies a callback engine).
    ocr_engine_override: Option<std::sync::Arc<dyn OcrEngine>>,
}

impl LiteParse {
    pub fn new(config: LiteParseConfig) -> Self {
        Self {
            config,
            ocr_engine_override: None,
        }
    }

    /// Override the OCR engine. When set, the engine is used regardless of
    /// `ocr_server_url` / built-in Tesseract availability.
    pub fn with_ocr_engine(mut self, engine: std::sync::Arc<dyn OcrEngine>) -> Self {
        self.ocr_engine_override = Some(engine);
        self
    }

    /// Parse a document from a file path, returning structured results.
    ///
    /// Non-PDF files are automatically converted to PDF first (requires
    /// LibreOffice/ImageMagick on the system).
    ///
    /// Not available on `wasm32` — the browser has no filesystem. Use
    /// [`LiteParse::parse_input`] with [`PdfInput::Bytes`] instead.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn parse(&self, input: &str) -> Result<ParseResult, LiteParseError> {
        self.parse_input(PdfInput::Path(input.to_string())).await
    }

    /// Parse a document from either a file path or raw bytes.
    ///
    /// Use `PdfInput::Path` for files on disk or `PdfInput::Bytes` for
    /// in-memory PDF data (e.g. from a network response or Node.js Buffer).
    ///
    /// When the `direct-convert` feature is enabled, structured formats
    /// (DOCX, PPTX, XLSX, HTML, CSV, plain text) are converted directly to
    /// Markdown without going through the PDF pipeline.
    pub async fn parse_input(&self, input: PdfInput) -> Result<ParseResult, LiteParseError> {
        // ── Direct conversion fast-path ──────────────────────────────────
        #[cfg(feature = "direct-convert")]
        {
            let format = Self::detect_format(&input);
            if format.supports_direct_markdown() {
                return self.parse_direct(input, &format).await;
            }
        }
        // ── PDF pipeline (existing behaviour) ────────────────────────────
        self.parse_via_pdf(input).await
    }

    /// Detect the input format from a `PdfInput`.
    #[allow(dead_code)]
    fn detect_format(input: &PdfInput) -> InputFormat {
        match input {
            PdfInput::Path(p) => {
                let ext = std::path::Path::new(p)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                InputFormat::from_extension(ext)
            }
            PdfInput::Bytes(_) => InputFormat::Pdf, // byte inputs go through PDF pipeline
        }
    }

    /// Direct conversion path: pure Rust → Markdown, no PDF/LibreOffice.
    ///
    /// Falls back to `parse_via_pdf()` if direct conversion fails.
    #[cfg(feature = "direct-convert")]
    async fn parse_direct(
        &self,
        input: PdfInput,
        format: &InputFormat,
    ) -> Result<ParseResult, LiteParseError> {
        let result = match &input {
            PdfInput::Path(p) => {
                match direct_convert::convert_file_direct(p, format) {
                    Ok(r) => r,
                    Err(e) => {
                        // Fall back to PDF pipeline on direct conversion failure
                        let log = |msg: &str| {
                            if !self.config.quiet {
                                eprintln!("{}", msg);
                            }
                        };
                        log(&format!(
                            "[liteparse] direct convert failed ({}), falling back to PDF pipeline: {}",
                            format!("{:?}", format),
                            e
                        ));
                        return self.parse_via_pdf(input).await;
                    }
                }
            }
            PdfInput::Bytes(_) => {
                // Byte inputs always go through PDF pipeline for now
                return self.parse_via_pdf(input).await;
            }
        };

        Ok(ParseResult {
            pages: vec![ParsedPage {
                page_number: 1,
                page_width: 612.0,
                page_height: 792.0,
                text: result.markdown.clone(),
                text_items: vec![],
            }],
            text: result.markdown,
        })
    }

    /// PDF pipeline: convert to PDF if needed, then extract via pdfium + OCR.
    ///
    /// This is the original `parse_input` logic, unchanged.
    async fn parse_via_pdf(&self, input: PdfInput) -> Result<ParseResult, LiteParseError> {
        let log = |msg: &str| {
            if !self.config.quiet {
                eprintln!("{}", msg);
            }
        };

        let t0 = web_time::Instant::now();

        #[cfg(not(target_arch = "wasm32"))]
        let (validated_input, _guard) =
            conversion::resolve_pdf_input(input, self.config.password.as_deref(), false).await?;

        #[cfg(target_arch = "wasm32")]
        let validated_input = input;

        // Determine which pages to extract
        let target_pages = self
            .config
            .target_pages
            .as_ref()
            .map(|s| parse_target_pages(s))
            .transpose()
            .map_err(|e| format!("invalid --target-pages: {}", e))?;

        // Extract text (and pre-render OCR pages in one PDF load when OCR is on).
        let password = self.config.password.as_deref();
        let (mut pages, ocr_rendered) = if self.config.ocr_enabled {
            let document = extract::load_document_from_input(&validated_input, password)?;
            let pages = extract::extract_pages_from_document(
                &document,
                target_pages.as_deref(),
                self.config.max_pages,
            )?;
            let t_extract = web_time::Instant::now();
            log(&format!(
                "[liteparse] extract: {:.1}ms ({} pages)",
                t_extract.duration_since(t0).as_secs_f64() * 1000.0,
                pages.len()
            ));
            let rendered = ocr_merge::render_pages_for_ocr(&document, &pages, self.config.dpi)?;
            log(&format!(
                "[liteparse] ocr render: {:.1}ms ({} pages)",
                web_time::Instant::now()
                    .duration_since(t_extract)
                    .as_secs_f64()
                    * 1000.0,
                rendered.len()
            ));
            (pages, rendered)
        } else {
            let pages = extract::extract_pages_from_input(
                &validated_input,
                target_pages.as_deref(),
                self.config.max_pages,
                password,
            )?;
            log(&format!(
                "[liteparse] extract: {:.1}ms ({} pages)",
                web_time::Instant::now().duration_since(t0).as_secs_f64() * 1000.0,
                pages.len()
            ));
            (pages, Vec::new())
        };
        let t1 = web_time::Instant::now();

        // OCR pass
        if self.config.ocr_enabled {
            let engine: std::sync::Arc<dyn OcrEngine> = if let Some(e) =
                self.ocr_engine_override.clone()
            {
                e
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(ref url) = self.config.ocr_server_url {
                        std::sync::Arc::new(HttpOcrEngine::new(url.clone()))
                    } else {
                        #[cfg(feature = "tesseract")]
                        {
                            std::sync::Arc::new(TesseractOcrEngine::new(
                                self.config.tessdata_path.clone(),
                            ))
                        }
                        #[cfg(not(feature = "tesseract"))]
                        {
                            return Err("OCR enabled but no --ocr-server-url provided and tesseract feature is disabled".into());
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    return Err(
                        "OCR enabled but no `ocrEngine` callback was provided (WASM builds have no built-in OCR engine)".into(),
                    );
                }
            };
            ocr_merge::ocr_and_merge_rendered(
                &mut pages,
                ocr_rendered,
                self.config.dpi,
                engine,
                &self.config.ocr_language,
                self.config.num_workers,
            )
            .await?;
        }
        let t_ocr = web_time::Instant::now();
        log(&format!(
            "[liteparse] ocr: {:.1}ms",
            t_ocr.duration_since(t1).as_secs_f64() * 1000.0
        ));

        // Grid projection
        let parsed_pages = projection::project_pages_to_grid(pages);
        let t2 = web_time::Instant::now();
        log(&format!(
            "[liteparse] project: {:.1}ms",
            t2.duration_since(t_ocr).as_secs_f64() * 1000.0
        ));

        let full_text = parsed_pages
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let total = t2.duration_since(t0).as_secs_f64() * 1000.0;
        log(&format!("[liteparse] total: {:.1}ms", total));

        Ok(ParseResult {
            pages: parsed_pages,
            text: full_text,
        })
    }

    /// Generate screenshots of document pages as PNG bytes.
    ///
    /// Non-PDF files are automatically converted to PDF first (requires
    /// LibreOffice/ImageMagick on the system). Plain-text formats cannot be
    /// rendered and return a clear error.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn screenshot(
        &self,
        input: &str,
        page_numbers: Option<Vec<u32>>,
    ) -> Result<Vec<ScreenshotResult>, LiteParseError> {
        self.screenshot_input(PdfInput::Path(input.to_string()), page_numbers)
            .await
    }

    /// Generate screenshots from a file path or raw bytes.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn screenshot_input(
        &self,
        input: PdfInput,
        page_numbers: Option<Vec<u32>>,
    ) -> Result<Vec<ScreenshotResult>, LiteParseError> {
        let log = |msg: &str| {
            if !self.config.quiet {
                eprintln!("{}", msg);
            }
        };

        let (validated_input, _guard) =
            conversion::resolve_pdf_input(input, self.config.password.as_deref(), true).await?;

        if let PdfInput::Path(ref path) = validated_input
            && !conversion::is_pdf(path)
        {
            log("[liteparse] converted input to PDF for screenshot rendering");
        }

        let rendered = render::render_pages_to_png(
            &validated_input,
            page_numbers.as_deref(),
            self.config.dpi,
            self.config.password.as_deref(),
        )?;

        Ok(rendered
            .into_iter()
            .map(|page| ScreenshotResult {
                page_num: page.page_num,
                width: page.width,
                height: page.height,
                image_bytes: page.png_bytes,
            })
            .collect())
    }

    pub fn config(&self) -> &LiteParseConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_new_stores_config() {
        let mut cfg = LiteParseConfig::default();
        cfg.ocr_enabled = false;
        cfg.max_pages = 7;
        let lp = LiteParse::new(cfg);
        assert!(!lp.config().ocr_enabled);
        assert_eq!(lp.config().max_pages, 7);
    }

    #[test]
    fn test_detect_format_pdf_path() {
        let input = PdfInput::Path("test.pdf".to_string());
        assert_eq!(LiteParse::detect_format(&input), InputFormat::Pdf);
    }

    #[test]
    fn test_detect_format_docx_path() {
        let input = PdfInput::Path("document.docx".to_string());
        assert_eq!(LiteParse::detect_format(&input), InputFormat::Docx);
    }

    #[test]
    fn test_detect_format_bytes_always_pdf() {
        let input = PdfInput::Bytes(vec![1, 2, 3]);
        assert_eq!(LiteParse::detect_format(&input), InputFormat::Pdf);
    }
}
