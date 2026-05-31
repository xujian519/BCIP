//! 无效宣告流水线
//!
//! 全流程：无效理由分析 → 证据收集 → 无效宣告请求书。

use serde::Deserialize;
use serde::Serialize;

/// 无效理由
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidityGround {
    LackOfNovelty,
    LackOfInventiveness,
    InsufficientDisclosure,
    LackOfClarity,
    UnpatentableSubject,
    AmendmentExceedsScope,
}

impl InvalidityGround {
    pub fn legal_basis(&self) -> &'static str {
        match self {
            Self::LackOfNovelty => "专利法第22条第2款（新颖性）",
            Self::LackOfInventiveness => "专利法第22条第3款（创造性）",
            Self::InsufficientDisclosure => "专利法第26条第3款（公开不充分）",
            Self::LackOfClarity => "专利法第26条第4款（不清楚/不支持）",
            Self::UnpatentableSubject => "专利法第2条（不属于保护客体）",
            Self::AmendmentExceedsScope => "专利法第33条（修改超范围）",
        }
    }
}

/// 证据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub evidence_type: String,
    pub document_number: String,
    pub title: String,
    pub publication_date: String,
    pub relevance: String,
}

/// 无效宣告请求书
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidityPetition {
    pub target_patent: String,
    pub petitioner: Option<String>,
    pub grounds: Vec<InvalidityGround>,
    pub evidence_list: Vec<EvidenceItem>,
    pub claim_by_claim_analysis: Vec<ClaimInvalidityAnalysis>,
    pub conclusion: String,
}

/// 逐权利要求无效分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimInvalidityAnalysis {
    pub claim_number: u32,
    pub grounds: Vec<InvalidityGround>,
    pub evidence_mapping: Vec<String>,
    pub feature_by_feature_comparison: String,
}

/// 无效宣告流水线
pub struct InvalidityPipeline;

impl InvalidityPipeline {
    pub fn new() -> Self {
        Self
    }

    /// 分析潜在无效理由
    pub fn analyze_grounds(
        novelty_result: bool,
        inventiveness_result: bool,
        has_clarity_issue: bool,
        has_support_issue: bool,
    ) -> Vec<InvalidityGround> {
        let mut grounds = Vec::new();
        if novelty_result {
            grounds.push(InvalidityGround::LackOfNovelty);
        }
        if inventiveness_result {
            grounds.push(InvalidityGround::LackOfInventiveness);
        }
        if has_clarity_issue {
            grounds.push(InvalidityGround::LackOfClarity);
        }
        if has_support_issue {
            grounds.push(InvalidityGround::InsufficientDisclosure);
        }
        grounds
    }

    /// 生成无效宣告请求书摘要
    pub fn generate_petition(
        target_patent: &str,
        grounds: &[InvalidityGround],
        evidence: Vec<EvidenceItem>,
    ) -> InvalidityPetition {
        let conclusion = if grounds.is_empty() {
            "未发现可用的无效理由".into()
        } else {
            let reasons: Vec<String> = grounds
                .iter()
                .map(|g| g.legal_basis().to_string())
                .collect();
            format!("依据{}，请求宣告专利权全部无效", reasons.join("、"))
        };

        InvalidityPetition {
            target_patent: target_patent.into(),
            petitioner: None,
            grounds: grounds.to_vec(),
            evidence_list: evidence,
            claim_by_claim_analysis: Vec::new(),
            conclusion,
        }
    }

    /// 添加逐权利要求分析
    pub fn add_claim_analysis(
        petition: &mut InvalidityPetition,
        claim_number: u32,
        grounds: Vec<InvalidityGround>,
        evidence_ids: Vec<String>,
        comparison: String,
    ) {
        petition
            .claim_by_claim_analysis
            .push(ClaimInvalidityAnalysis {
                claim_number,
                grounds,
                evidence_mapping: evidence_ids,
                feature_by_feature_comparison: comparison,
            });
    }
}

impl Default for InvalidityPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_grounds_when_all_checks_pass() {
        let grounds = InvalidityPipeline::analyze_grounds(false, false, false, false);
        assert!(grounds.is_empty());
    }

    #[test]
    fn both_novelty_and_inventiveness_flagged() {
        let grounds = InvalidityPipeline::analyze_grounds(true, true, false, false);
        assert_eq!(grounds.len(), 2);
        assert!(grounds.contains(&InvalidityGround::LackOfNovelty));
        assert!(grounds.contains(&InvalidityGround::LackOfInventiveness));
    }

    #[test]
    fn clarity_issue_added_separately() {
        let grounds = InvalidityPipeline::analyze_grounds(false, false, true, false);
        assert_eq!(grounds.len(), 1);
        assert_eq!(grounds[0], InvalidityGround::LackOfClarity);
    }

    #[test]
    fn petition_with_grounds_has_conclusion() {
        let grounds = vec![InvalidityGround::LackOfNovelty];
        let petition = InvalidityPipeline::generate_petition("CN12345678A", &grounds, vec![]);
        assert!(petition.conclusion.contains("请求宣告专利权全部无效"));
    }

    #[test]
    fn empty_petition_no_grounds() {
        let petition = InvalidityPipeline::generate_petition("CN12345678A", &[], vec![]);
        assert!(petition.conclusion.contains("未发现"));
    }

    #[test]
    fn add_claim_analysis_modifies_petition() {
        let grounds = vec![InvalidityGround::LackOfInventiveness];
        let mut petition = InvalidityPipeline::generate_petition("CN123", &grounds, vec![]);
        InvalidityPipeline::add_claim_analysis(
            &mut petition,
            1,
            vec![InvalidityGround::LackOfInventiveness],
            vec!["ev-1".into()],
            "特征A已被公开".into(),
        );
        assert_eq!(petition.claim_by_claim_analysis.len(), 1);
        assert_eq!(petition.claim_by_claim_analysis[0].claim_number, 1);
    }

    #[test]
    fn legal_basis_format_correct() {
        assert!(
            InvalidityGround::LackOfNovelty
                .legal_basis()
                .contains("22条")
        );
        assert!(
            InvalidityGround::LackOfInventiveness
                .legal_basis()
                .contains("22条")
        );
    }
}
