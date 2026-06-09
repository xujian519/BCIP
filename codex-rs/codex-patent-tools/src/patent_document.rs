//! 专利文档解析工具。
//!
//! 提供审查意见（OA）解析、文档类型识别、附图理解等文档处理能力。

use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

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

/// 专利号引用正则：匹配 CN/US/EP 专利号
static PATENT_CITE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(CN|US|EP)\d{6,12}[A-Z]?\d?").expect("PATENT_CITE_RE 正则字面量有效")
});

/// 附图编号正则：匹配"图 数字"模式
static FIGURE_NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"图\s*(\d+)").expect("FIGURE_NUMBER_RE 正则字面量有效"));

/// 附图标记正则：匹配"图 数字"（用于检测是否存在）
static FIGURE_MARK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"图\s*\d+").expect("FIGURE_MARK_RE 正则字面量有效"));

pub fn oa_parse(input: OaParseInput) -> Result<serde_json::Value, String> {
    let rejection_type =
        codex_patent_domain::examiner_simulator::ExaminerSimulator::detect_rejection_type(
            &input.oa_text,
        );
    let has_comparison = input.oa_text.contains("对比文件");
    let has_claims_analysis = input.oa_text.contains("权利要求");
    let has_conclusion = input.oa_text.contains("驳回") || input.oa_text.contains("授权");

    let cited_patents: Vec<String> = PATENT_CITE_RE
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
    let has_numbering = FIGURE_MARK_RE.is_match(&input.description);
    let has_components = input.description.contains("包括")
        || input.description.contains("包含")
        || input.description.contains("设有");
    let has_connections = input.description.contains("连接")
        || input.description.contains("固定")
        || input.description.contains("安装");

    let figures: Vec<String> = FIGURE_NUMBER_RE
        .captures_iter(&input.description)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();

    Ok(serde_json::json!({
        "has_numbering": has_numbering,
        "has_components": has_components,
        "has_connections": has_connections,
        "figures_found": figures.len(),
        "figure_numbers": figures,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn oa_parse_detects_cited_patents_and_sections() {
        let input = OaParseInput {
            oa_text: "审查意见：本申请不具备新颖性。对比文件CN1234567A公开了全部技术特征。权利要求1-3不具备新颖性，驳回。".into(),
        };
        let result = oa_parse(input).unwrap();
        assert!(!result["cited_patents"].as_array().unwrap().is_empty());
        assert!(result["sections"]["has_comparison"].as_bool().unwrap());
        assert!(result["sections"]["has_claims_analysis"].as_bool().unwrap());
        assert!(result["sections"]["has_conclusion"].as_bool().unwrap());
    }

    #[test]
    fn oa_parse_empty_text_no_citations() {
        let input = OaParseInput {
            oa_text: "无实质内容".into(),
        };
        let result = oa_parse(input).unwrap();
        assert!(result["cited_patents"].as_array().unwrap().is_empty());
        assert!(!result["sections"]["has_comparison"].as_bool().unwrap());
    }

    #[test]
    fn document_parse_identifies_sections() {
        let input = DocumentParseInput {
            document_text:
                "权利要求书\n说明书\n技术领域\n背景技术\n发明内容\n附图说明\n具体实施方式\n摘要"
                    .into(),
            document_type: Some("patent".into()),
        };
        let result = document_parse(input).unwrap();
        assert_eq!(result["document_type"], "patent");
        assert!(result["has_claims"].as_bool().unwrap());
        assert!(result["has_abstract"].as_bool().unwrap());
        assert_eq!(result["sections_found"], 5);
    }

    #[test]
    fn document_parse_minimal() {
        let input = DocumentParseInput {
            document_text: "一些文本".into(),
            document_type: None,
        };
        let result = document_parse(input).unwrap();
        assert_eq!(result["document_type"], "unknown");
        assert_eq!(result["sections_found"], 0);
    }

    #[test]
    fn drawing_understanding_detects_figures() {
        let input = DrawingUnderstandingInput {
            description: "图1是本发明整体结构示意图。包括壳体101，设有连接件102。".into(),
        };
        let result = drawing_understanding(input).unwrap();
        assert!(result["has_numbering"].as_bool().unwrap());
        assert!(result["has_components"].as_bool().unwrap());
        assert!(result["has_connections"].as_bool().unwrap());
        assert_eq!(result["figures_found"], 1);
    }
}
