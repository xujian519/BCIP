//! 专利文档解析工具。
//!
//! 提供审查意见（OA）解析、文档类型识别、附图理解等文档处理能力。

use serde::Deserialize;

/// OA（Office Action）解析输入参数。
#[derive(Debug, Deserialize)]
pub struct OaParseInput {
    /// OA 文本内容。
    pub oa_text: String,
}

/// 专利文档解析输入参数。
#[derive(Debug, Deserialize)]
pub struct DocumentParseInput {
    /// 文档文本内容。
    pub document_text: String,
    /// 文档类型（如 "patent", "oa" 等）。
    pub document_type: Option<String>,
}

/// 附图理解输入参数。
#[derive(Debug, Deserialize)]
pub struct DrawingUnderstandingInput {
    /// 附图文字描述。
    pub description: String,
}

/// 专利文档解析工具集。
pub struct PatentDocumentTools;

impl PatentDocumentTools {
    pub fn oa_parse(input: OaParseInput) -> Result<serde_json::Value, String> {
        let rejection_type =
            codex_patent_domain::examiner_simulator::ExaminerSimulator::detect_rejection_type(
                &input.oa_text,
            );
        let has_comparison = input.oa_text.contains("对比文件");
        let has_claims_analysis = input.oa_text.contains("权利要求");
        let has_conclusion = input.oa_text.contains("驳回") || input.oa_text.contains("授权");

        let re = regex::Regex::new(r"(CN|US|EP)\d{6,12}[A-Z]?\d?").unwrap();
        let cited_patents: Vec<String> = re
            .find_iter(&input.oa_text)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(serde_json::json!({
            "rejection_type": format!("{rejection_type:?}"),
            "sections": {
                "has_comparison": has_comparison,
                "has_claims_analysis": has_claims_analysis,
                "has_conclusion": has_conclusion,
            },
            "cited_patents": cited_patents,
            "word_count": input.oa_text.len(),
        }))
    }

    pub fn document_parse(input: DocumentParseInput) -> Result<serde_json::Value, String> {
        let doc_type = input.document_type.unwrap_or_else(|| "unknown".to_string());
        let has_claims = input.document_text.contains("权利要求");
        let has_description =
            input.document_text.contains("说明书") || input.document_text.contains("技术领域");
        let has_abstract = input.document_text.contains("摘要");

        let sections = [
            ("技术领域", input.document_text.contains("技术领域")),
            ("背景技术", input.document_text.contains("背景技术")),
            ("发明内容", input.document_text.contains("发明内容")),
            ("附图说明", input.document_text.contains("附图说明")),
            ("具体实施方式", input.document_text.contains("具体实施方式")),
        ];
        let section_count = sections.iter().filter(|(_, found)| *found).count();

        Ok(serde_json::json!({
            "document_type": doc_type,
            "has_claims": has_claims,
            "has_description": has_description,
            "has_abstract": has_abstract,
            "sections_found": section_count,
            "total_sections_expected": sections.len(),
            "word_count": input.document_text.len(),
        }))
    }

    pub fn drawing_understanding(
        input: DrawingUnderstandingInput,
    ) -> Result<serde_json::Value, String> {
        let has_numbering = regex::Regex::new(r"图\s*\d+")
            .unwrap()
            .is_match(&input.description);
        let has_components = input.description.contains("包括")
            || input.description.contains("包含")
            || input.description.contains("设有");
        let has_connections = input.description.contains("连接")
            || input.description.contains("固定")
            || input.description.contains("安装");

        let re = regex::Regex::new(r"图\s*(\d+)").unwrap();
        let figures: Vec<String> = re
            .captures_iter(&input.description)
            .map(|c| c.get(1).unwrap().as_str().to_string())
            .collect();

        Ok(serde_json::json!({
            "has_numbering": has_numbering,
            "has_components": has_components,
            "has_connections": has_connections,
            "figures_found": figures.len(),
            "figure_numbers": figures,
        }))
    }
}
