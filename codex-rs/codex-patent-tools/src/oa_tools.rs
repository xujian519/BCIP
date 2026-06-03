//! 审查意见（OA）答复工具集。
//!
//! 提供 OA 解析、策略分析、答复生成、论证构建等功能，
//! 覆盖从接收审查意见到生成答复文书的完整流程。

use codex_patent_core::OfficeAction;
use codex_patent_domain::oa::OaParser;
use codex_patent_domain::oa::OaResponder;
use serde::Deserialize;

/// OA 解析输入参数。
#[derive(Debug, Deserialize)]
pub struct OaParseInput {
    /// OA 原始文本内容。
    pub content: String,
    /// 申请号。
    pub application_number: Option<String>,
    /// 专利标题。
    pub patent_title: Option<String>,
    /// 文档类型。
    pub document_type: Option<String>,
}

/// OA 策略分析输入参数。
#[derive(Debug, Deserialize)]
pub struct OaStrategyInput {
    /// OA 类型（novelty / inventive / clarity / support 等）。
    pub oa_type: String,
    /// 审查员论证理由。
    pub examiner_arguments: String,
    /// 受影响的权利要求索引。
    pub affected_claims: Vec<usize>,
    /// 引用对比文件列表。
    pub citations: Vec<CitationInput>,
}

/// 对比文件引用信息。
#[derive(Debug, Deserialize)]
pub struct CitationInput {
    /// 对比文件号码。
    pub document_number: String,
    /// 相关度（如 "X", "Y", "A"）。
    pub relevancy: Option<String>,
    /// 受影响的权利要求索引。
    pub claims_affected: Option<Vec<usize>>,
}

/// 答复生成器输入参数。
#[derive(Debug, Deserialize)]
pub struct ResponderInput {
    /// OA 原始内容。
    pub oa_content: String,
    /// 答复策略（amend / argue / hybrid / withdraw）。
    pub strategy: Option<String>,
    /// 专利信息。
    pub patent_info: Option<String>,
}

/// 论证构建输入参数。
#[derive(Debug, Deserialize)]
pub struct ArgumentInput {
    /// OA 类型。
    pub oa_type: String,
    /// 区别技术特征列表。
    pub differences: Vec<String>,
    /// 技术效果列表。
    pub technical_effects: Vec<String>,
    /// 法律依据（如 "专利法第22条第3款"）。
    pub legal_basis: Option<String>,
}

/// 答复模板输入参数。
#[derive(Debug, Deserialize)]
pub struct TemplateInput {
    /// OA 类型。
    pub oa_type: String,
    /// 输出格式（如 "cnipa"）。
    pub format: Option<String>,
}

/// OA 审查意见答复工具集。
pub struct OaTools;

impl OaTools {
    pub fn oa_parser(input: OaParseInput) -> Result<serde_json::Value, String> {
        let oa = OaParser::parse(&input.content);
        serde_json::to_value(&oa).map_err(|e| format!("{e}"))
    }

    pub fn oa_strategist(input: OaStrategyInput) -> Result<serde_json::Value, String> {
        let oa_type = match input.oa_type.as_str() {
            "novelty" | "新颖性" => codex_patent_core::OaType::Novelty,
            "inventive" | "创造性" => codex_patent_core::OaType::InventiveStep,
            "clarity" | "清楚" => codex_patent_core::OaType::Clarity,
            "support" | "支持" => codex_patent_core::OaType::Support,
            "scope" | "范围" => codex_patent_core::OaType::Scope,
            "formal" | "形式" => codex_patent_core::OaType::Formal,
            _ => codex_patent_core::OaType::Other(input.oa_type.clone()),
        };
        let citations = input
            .citations
            .iter()
            .map(|c| codex_patent_core::CitedReference {
                document_number: c.document_number.clone(),
                relevancy: c.relevancy.clone().unwrap_or_else(|| "X".into()),
                claims_affected: c.claims_affected.clone().unwrap_or_default(),
            })
            .collect();
        let oa = OfficeAction {
            oa_type,
            citations,
            examiner_arguments: input.examiner_arguments,
            affected_claims: input.affected_claims,
        };
        let strategies = OaResponder::analyze_and_recommend(&oa);
        serde_json::to_value(&strategies).map_err(|e| format!("{e}"))
    }

    pub fn patent_responder(input: ResponderInput) -> Result<serde_json::Value, String> {
        let oa = OaParser::parse(&input.oa_content);
        let strategies = OaResponder::analyze_and_recommend(&oa);
        let best = strategies
            .first()
            .map_or("argue", |s| match s.strategy_type {
                codex_patent_core::ResponseStrategyType::AmendClaims => "amend",
                codex_patent_core::ResponseStrategyType::Argue => "argue",
                codex_patent_core::ResponseStrategyType::Hybrid => "hybrid",
                codex_patent_core::ResponseStrategyType::Withdraw => "withdraw",
            });
        let template = Self::get_response_template(&oa.oa_type, best);
        Ok(serde_json::json!({
            "strategy": best,
            "confidence": strategies.first().map_or(0.0, |s| s.confidence),
            "template": template,
            "oa_type": format!("{:?}", oa.oa_type),
            "affected_claims": oa.affected_claims,
            "citation_count": oa.citations.len(),
        }))
    }

    pub fn strategy_argument_generator(input: ArgumentInput) -> Result<serde_json::Value, String> {
        let legal_basis = input.legal_basis.unwrap_or_else(|| {
            match input.oa_type.as_str() {
                "novelty" => "专利法第22条第2款",
                "inventive" => "专利法第22条第3款",
                _ => "",
            }
            .into()
        });
        Ok(serde_json::json!({
            "oa_type": input.oa_type,
            "legal_basis": legal_basis,
            "differences": input.differences,
            "effects": input.technical_effects,
            "argument": Self::build_argument(&input.oa_type, &input.differences, &input.technical_effects),
        }))
    }

    pub fn response_template(input: TemplateInput) -> Result<serde_json::Value, String> {
        let template = Self::get_response_template(
            &match input.oa_type.as_str() {
                "novelty" | "新颖性" => codex_patent_core::OaType::Novelty,
                "inventive" | "创造性" => codex_patent_core::OaType::InventiveStep,
                _ => codex_patent_core::OaType::Other(input.oa_type.clone()),
            },
            "argue",
        );
        Ok(
            serde_json::json!({"oa_type": input.oa_type, "template": template, "format": input.format.unwrap_or_else(|| "cnipa".into())}),
        )
    }

    fn oa_type_str(oa_type: &codex_patent_core::OaType) -> &str {
        match oa_type {
            codex_patent_core::OaType::Novelty => "新颖性",
            codex_patent_core::OaType::InventiveStep => "创造性",
            codex_patent_core::OaType::Clarity => "清楚",
            codex_patent_core::OaType::Support => "支持",
            codex_patent_core::OaType::Scope => "超范围",
            codex_patent_core::OaType::Formal => "形式缺陷",
            codex_patent_core::OaType::Other(_) => "其他",
        }
    }

    fn get_response_template(oa_type: &codex_patent_core::OaType, strategy: &str) -> String {
        let t = Self::oa_type_str(oa_type);
        format!(
            "意见陈述书\n\n尊敬的审查员：\n\n申请人仔细研究了贵局于____年__月__日发出的审查意见通知书，现针对通知书中指出的{t}问题，陈述意见如下：\n\n{}\n\n综上所述，申请人认为修改后的权利要求书已克服审查意见中指出的缺陷，符合专利法及实施细则的相关规定，恳请审查员予以审查并早日授权。\n\n申请人：\n日期：",
            match strategy {
                "amend" =>
                    "申请人根据审查意见对权利要求书进行了修改，具体修改内容见修改后的权利要求书。上述修改未超出原说明书和权利要求书记载的范围，符合专利法第三十三条的规定。",
                "argue" =>
                    "申请人经仔细对比分析后认为，本申请与对比文件存在如下区别技术特征：\n\n（此处列出区别特征）\n\n上述区别特征具有如下技术效果：\n\n（此处列举技术效果）\n\n因此，本申请具备专利法第22条规定的新颖性和创造性。",
                _ =>
                    "申请人结合审查意见进行了认真分析，并据此对申请文件进行了适应性修改。恳请审查员重新审查。",
            }
        )
    }

    fn build_argument(oa_type: &str, differences: &[String], effects: &[String]) -> String {
        let diff_text = if differences.is_empty() {
            "区别技术特征"
        } else {
            &differences.join("、")
        };
        let eff_text = if effects.is_empty() {
            "非显而易见的技术效果"
        } else {
            &effects.join("、")
        };
        match oa_type {
            "novelty" => format!(
                "对比文件未公开{}这一区别技术特征，因此本申请具备新颖性。",
                diff_text
            ),
            "inventive" => format!(
                "{}未被任何对比文件公开，且产生了{}，对本领域技术人员而言并非显而易见。",
                diff_text, eff_text
            ),
            _ => "本申请符合专利法相关规定。".into(),
        }
    }
}

pub fn register_oa_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("OaParser".into(), |input| {
        Box::pin(async move {
            let parsed: OaParseInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            OaTools::oa_parser(parsed)
        })
    });
    t.insert("OaStrategist".into(), |input| {
        Box::pin(async move {
            let parsed: OaStrategyInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            OaTools::oa_strategist(parsed)
        })
    });
    t.insert("PatentResponder".into(), |input| {
        Box::pin(async move {
            let parsed: ResponderInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            OaTools::patent_responder(parsed)
        })
    });
    t.insert("StrategyArgumentGenerator".into(), |input| {
        Box::pin(async move {
            let parsed: ArgumentInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            OaTools::strategy_argument_generator(parsed)
        })
    });
    t.insert("ResponseTemplate".into(), |input| {
        Box::pin(async move {
            let parsed: TemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            OaTools::response_template(parsed)
        })
    });
    t
}
