//! 审查意见通知书（Office Action）解析与答复策略推荐。
//!
//! - `OaParser`：解析 OA 文本，提取类型、引用文献、影响的权利要求及审查员意见。
//! - `OaResponder`：根据 OA 类型推荐答复策略（争辩、修改权利要求、混合策略）。

use codex_patent_core::*;
use regex::Regex;
use std::sync::LazyLock;

static RE_CITATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:CN|US|WO|EP|JP|KR)\d{6,}[A-Z]?").unwrap());
static RE_CLAIM_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"权利要求\s*(\d+)").unwrap());
static RE_CLAIM_RANGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"第\s*(\d+)\s*[-至到]\s*(\d+)\s*项").unwrap());

static RE_EXAMINER_ARGS: [LazyLock<Regex>; 3] = [
    LazyLock::new(|| Regex::new(r"(?s)审查意见[：:](.+?)(?:\n答复|$)").unwrap()),
    LazyLock::new(|| Regex::new(r"(?s)驳回理由[：:](.+?)(?:\n答复|$)").unwrap()),
    LazyLock::new(|| Regex::new(r"(?s)认为[：:](.+?)(?:\n|$)").unwrap()),
];

/// 审查意见通知书解析器
///
/// 通过文本模式匹配识别 OA 类型（新颖性/创造性/清楚/支持等），
/// 并提取引用对比文件、影响的权利要求编号及审查员论证内容。
pub struct OaParser;

impl OaParser {
    /// 解析 OA 文本，返回结构化 OfficeAction
    pub fn parse(text: &str) -> OfficeAction {
        let oa_type = Self::detect_oa_type(text);
        let citations = Self::extract_citations(text);
        let affected_claims = Self::extract_affected_claims(text);
        let examiner_arguments = Self::extract_examiner_arguments(text);

        OfficeAction {
            oa_type,
            citations,
            examiner_arguments,
            affected_claims,
        }
    }

    fn detect_oa_type(text: &str) -> OaType {
        let t = text.to_lowercase();
        if t.contains("创造性") || t.contains("显而易见") || t.contains("22条第3款") {
            return OaType::InventiveStep;
        }
        if t.contains("新颖性") || t.contains("不具备新颖性") || t.contains("22条第2款")
        {
            return OaType::Novelty;
        }
        if t.contains("清楚") || t.contains("26条第4款") || t.contains("简明") {
            return OaType::Clarity;
        }
        if t.contains("支持") || t.contains("超范围") {
            return OaType::Support;
        }
        if t.contains("保护范围") || t.contains("33条") {
            return OaType::Scope;
        }
        if t.contains("形式") || t.contains("格式") {
            return OaType::Formal;
        }
        OaType::Other("未知类型".into())
    }

    fn extract_citations(text: &str) -> Vec<CitedReference> {
        let mut citations = Vec::new();
        for m in RE_CITATION.find_iter(text) {
            citations.push(CitedReference {
                document_number: m.as_str().to_string(),
                relevancy: "X".into(),
                claims_affected: vec![1],
            });
        }
        citations.dedup_by(|a, b| a.document_number == b.document_number);
        citations
    }

    fn extract_affected_claims(text: &str) -> Vec<usize> {
        let mut claims: Vec<usize> = Vec::new();

        for cap in RE_CLAIM_NUMBER.captures_iter(text) {
            if let Some(m) = cap.get(1)
                && let Ok(n) = m.as_str().parse::<usize>()
                && !claims.contains(&n)
            {
                claims.push(n);
            }
        }

        for cap in RE_CLAIM_RANGE.captures_iter(text) {
            if let (Some(s), Some(e)) = (cap.get(1), cap.get(2))
                && let (Ok(sn), Ok(en)) = (s.as_str().parse::<usize>(), e.as_str().parse::<usize>())
            {
                for n in sn..=en {
                    if !claims.contains(&n) {
                        claims.push(n);
                    }
                }
            }
        }

        if claims.is_empty() {
            claims.push(1);
        }
        claims.sort();
        claims
    }

    fn extract_examiner_arguments(text: &str) -> String {
        for re in &RE_EXAMINER_ARGS {
            if let Some(cap) = re.captures(text)
                && let Some(m) = cap.get(1)
            {
                let content = m.as_str().trim().to_string();
                if !content.is_empty() {
                    return content;
                }
            }
        }
        text.chars().take(500).collect()
    }
}

/// OA 答复策略推荐器
///
/// 根据 OA 类型（新颖性、创造性、清楚、支持等）推荐相应的答复策略，
/// 包括争辩、修改权利要求或混合策略，并给出置信度评分。
pub struct OaResponder;

impl OaResponder {
    /// 分析 OA 并推荐答复策略列表（按置信度降序排列）
    pub fn analyze_and_recommend(oa: &OfficeAction) -> Vec<ResponseStrategy> {
        let mut strategies = Vec::new();

        match &oa.oa_type {
            OaType::Novelty => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::Argue,
                    reasoning: "审查意见基于新颖性，可争辩存在区别技术特征未被对比文件公开".into(),
                    confidence: 0.6,
                });
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::AmendClaims,
                    reasoning: "将区别特征写入独立权利要求，增强新颖性".into(),
                    confidence: 0.8,
                });
            }
            OaType::InventiveStep => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::Hybrid,
                    reasoning: "创造性争辩需证明区别特征非显而易见，同时修改权利要求增加限定"
                        .into(),
                    confidence: 0.5,
                });
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::AmendClaims,
                    reasoning: "缩小保护范围，增加从属权利要求中的特征到独立权利要求".into(),
                    confidence: 0.7,
                });
            }
            OaType::Clarity => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::AmendClaims,
                    reasoning: "澄清模糊表述，使权利要求清楚明确".into(),
                    confidence: 0.9,
                });
            }
            OaType::Support => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::Hybrid,
                    reasoning: "修改说明书增加支持内容，同时调整权利要求范围".into(),
                    confidence: 0.7,
                });
            }
            OaType::Scope => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::AmendClaims,
                    reasoning: "调整保护范围，确保不超范围修改".into(),
                    confidence: 0.8,
                });
            }
            _ => {
                strategies.push(ResponseStrategy {
                    strategy_type: ResponseStrategyType::Argue,
                    reasoning: "一般性争辩".into(),
                    confidence: 0.3,
                });
            }
        }

        strategies.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        strategies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_oa_novelty() {
        let text = "审查意见：权利要求1-3不具备新颖性。对比文件CN1234567A公开了权利要求1的全部技术特征。根据专利法第22条第2款，权利要求1不具备新颖性。";
        let oa = OaParser::parse(text);
        assert_eq!(oa.oa_type, OaType::Novelty);
        assert!(oa.affected_claims.contains(&1));
        assert!(!oa.citations.is_empty());
    }

    #[test]
    fn test_parse_oa_inventive() {
        let text = "审查意见：权利要求1不具备创造性。对比文件1公开了特征，且该区别特征是显而易见的。根据专利法第22条第3款，权利要求1-2不具备创造性。";
        let oa = OaParser::parse(text);
        assert_eq!(oa.oa_type, OaType::InventiveStep);
    }

    #[test]
    fn test_responder_novelty() {
        let oa = OfficeAction {
            oa_type: OaType::Novelty,
            citations: vec![CitedReference {
                document_number: "CN1234567A".into(),
                relevancy: "X".into(),
                claims_affected: vec![1],
            }],
            examiner_arguments: "不具备新颖性".into(),
            affected_claims: vec![1],
        };
        let strategies = OaResponder::analyze_and_recommend(&oa);
        assert!(!strategies.is_empty());
        assert!(
            strategies
                .iter()
                .any(|s| s.strategy_type == ResponseStrategyType::AmendClaims)
        );
    }

    #[test]
    fn test_responder_clarity() {
        let oa = OfficeAction {
            oa_type: OaType::Clarity,
            citations: vec![],
            examiner_arguments: "不清楚".into(),
            affected_claims: vec![1],
        };
        let strategies = OaResponder::analyze_and_recommend(&oa);
        assert_eq!(
            strategies[0].strategy_type,
            ResponseStrategyType::AmendClaims
        );
    }
}
