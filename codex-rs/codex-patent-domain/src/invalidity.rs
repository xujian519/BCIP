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

// ---------------------------------------------------------------------------
// 全面无效宣告分析（增强 API）
// ---------------------------------------------------------------------------

/// 无效宣告理由类型（详细版，含结构化数据）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DetailedInvalidityGround {
    LackOfNovelty {
        prior_art: String,
        affected_claims: Vec<u32>,
    },
    LackOfInventiveness {
        closest_prior_art: String,
        reasoning: String,
        affected_claims: Vec<u32>,
    },
    LackOfUtility {
        reason: String,
    },
    InsufficientDisclosure {
        missing_elements: Vec<String>,
    },
    ClaimsNotSupported {
        unsupported_claims: Vec<u32>,
    },
    ClaimsUnclear {
        unclear_claims: Vec<u32>,
        issues: Vec<String>,
    },
    BeyondOriginalScope {
        additions: Vec<String>,
    },
    NonPatentableSubjectMatter {
        subject_matter: String,
        legal_basis: String,
    },
}

/// 证据需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub evidence_type: String,
    pub description: String,
    pub importance: Importance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Importance {
    Required,
    Important,
    Auxiliary,
}

/// 无效分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidityAnalysis {
    pub grounds: Vec<DetailedInvalidityGround>,
    pub strongest_ground: Option<usize>,
    pub evidence_requirements: Vec<EvidenceRequirement>,
    pub success_probability: f64,
    pub recommended_strategy: String,
}

/// 全面无效分析
pub fn analyze_invalidity(
    patent_claims: &[String],
    patent_spec: &str,
    prior_art: &[String],
) -> InvalidityAnalysis {
    let mut grounds = Vec::new();

    // 1. 新颖性无效
    for (i, pa) in prior_art.iter().enumerate() {
        let affected = find_novelty_conflicts(patent_claims, pa);
        if !affected.is_empty() {
            grounds.push(DetailedInvalidityGround::LackOfNovelty {
                prior_art: format!("对比文件{}", i + 1),
                affected_claims: affected,
            });
        }
    }

    // 2. 充分公开检查
    if let Some(disclosure_issues) = check_sufficiency(patent_spec, patent_claims) {
        grounds.push(DetailedInvalidityGround::InsufficientDisclosure {
            missing_elements: disclosure_issues,
        });
    }

    // 3. 清楚性检查
    let (unclear, issues) = check_claim_clarity(patent_claims);
    if !unclear.is_empty() {
        grounds.push(DetailedInvalidityGround::ClaimsUnclear {
            unclear_claims: unclear,
            issues,
        });
    }

    // 4. 计算成功概率
    let prob = estimate_success(&grounds);
    let strongest = find_strongest_ground(&grounds);
    let strategy = recommend_strategy(&grounds);

    InvalidityAnalysis {
        evidence_requirements: collect_evidence(&grounds),
        strongest_ground: strongest,
        success_probability: prob,
        recommended_strategy: strategy,
        grounds,
    }
}

/// 检查权利要求特征是否在对比文件中出现（简单关键词匹配）
fn find_novelty_conflicts(claims: &[String], prior_art: &str) -> Vec<u32> {
    let pa_lower = prior_art.to_lowercase();
    claims
        .iter()
        .enumerate()
        .filter_map(|(i, claim)| {
            // 提取权利要求中的核心技术特征（简单分词：按中文标点和空格切分）
            let keywords = extract_technical_keywords(claim);
            let matched_count = keywords
                .iter()
                .filter(|kw| pa_lower.contains(&kw.to_lowercase()))
                .count();
            // 超过 60% 的关键词出现在对比文件中，视为冲突
            if !keywords.is_empty() && matched_count as f64 / keywords.len() as f64 > 0.6 {
                Some(i as u32 + 1)
            } else {
                None
            }
        })
        .collect()
}

/// 从权利要求中提取技术关键词
fn extract_technical_keywords(claim: &str) -> Vec<String> {
    let stop_words = [
        "的", "所述", "其", "包括", "包含", "具有", "一种", "一个", "及", "与", "或", "在", "于",
        "由", "对", "将", "该", "本", "中", "上", "下", "内", "外", "前", "后", "用于", "通过",
        "根据", "其中", "和", "以及", "至", "为", "设有", "设有",
    ];
    // First pass: split on punctuation
    let segments = claim.split(|c: char| {
        c == '，' || c == '。' || c == '、' || c == '；' || c == ' ' || c == ',' || c == ';'
    });
    // Second pass: for each segment, further split on structural words (stop words)
    let mut keywords = Vec::new();
    for seg in segments {
        let mut remaining = seg.trim();
        // Strip leading stop words
        while let Some(sw) = stop_words.iter().find(|sw| remaining.starts_with(*sw)) {
            remaining = remaining[sw.len()..].trim();
        }
        while !remaining.is_empty() {
            // Find earliest stop-word boundary
            let earliest = stop_words
                .iter()
                .filter_map(|sw| remaining.find(sw).map(|pos| (pos, sw.len())))
                .filter(|(pos, _)| *pos > 0) // skip stop-word at position 0 (handled by trimming)
                .min_by_key(|(pos, _)| *pos);
            match earliest {
                Some((pos, len)) => {
                    let head = remaining[..pos].trim();
                    if head.len() >= 2 && !stop_words.contains(&head) {
                        keywords.push(head.to_string());
                    }
                    remaining = remaining[pos + len..].trim();
                }
                None => {
                    let head = remaining.trim();
                    if head.len() >= 2 && !stop_words.contains(&head) {
                        keywords.push(head.to_string());
                    }
                    break;
                }
            }
        }
    }
    keywords
}

/// 检查说明书是否充分公开
fn check_sufficiency(spec: &str, claims: &[String]) -> Option<Vec<String>> {
    let mut missing = Vec::new();
    let spec_lower = spec.to_lowercase();

    // 检查是否有"具体实施方式"或"实施例"部分
    if !spec_lower.contains("具体实施方式") && !spec_lower.contains("实施例") {
        missing.push("缺少具体实施方式或实施例".into());
    }

    // 检查是否有具体参数（数字 + 单位）
    let has_parameters = claims
        .iter()
        .any(|c| c.chars().any(|ch| ch.is_ascii_digit()));
    if has_parameters {
        let spec_has_numbers = spec
            .lines()
            .any(|line| line.chars().any(|ch| ch.is_ascii_digit()));
        if !spec_has_numbers {
            missing.push("权利要求中含参数但说明书中无具体数值支持".into());
        }
    }

    // 检查权利要求中的关键术语是否有说明书解释
    for (i, claim) in claims.iter().enumerate() {
        let kw = extract_technical_keywords(claim);
        let unexplained: Vec<&str> = kw
            .iter()
            .filter(|k| k.len() >= 3 && !spec_lower.contains(&k.to_lowercase()))
            .map(|s| s.as_str())
            .take(3) // 只报告前3个未解释的关键词
            .collect();
        if !unexplained.is_empty() {
            missing.push(format!(
                "权利要求{}中术语在说明书中未充分解释：{}",
                i + 1,
                unexplained.join("、")
            ));
        }
    }

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// 检查权利要求清楚性
fn check_claim_clarity(claims: &[String]) -> (Vec<u32>, Vec<String>) {
    let vague_terms = [
        "大约",
        "约",
        "左右",
        "近似",
        "相当",
        "适当",
        "基本上",
        "大致",
        "优选",
        "较佳",
        "最好是",
        "大概",
        "可能",
        "或许",
        "等等",
        "例如",
    ];

    let mut unclear_claims = Vec::new();
    let mut issues = Vec::new();

    for (i, claim) in claims.iter().enumerate() {
        let found: Vec<&str> = vague_terms
            .iter()
            .filter(|vt| claim.contains(**vt))
            .copied()
            .collect();
        if !found.is_empty() {
            unclear_claims.push(i as u32 + 1);
            issues.push(format!(
                "权利要求{}包含模糊用语：{}",
                i + 1,
                found.join("、")
            ));
        }
    }

    (unclear_claims, issues)
}

/// 基于理由数量和类型估算成功概率
fn estimate_success(grounds: &[DetailedInvalidityGround]) -> f64 {
    if grounds.is_empty() {
        return 0.0;
    }

    let mut score = 0.0_f64;
    for g in grounds {
        score += match g {
            // 新颖性理由权重最高
            DetailedInvalidityGround::LackOfNovelty {
                affected_claims, ..
            } => 0.35 + 0.05 * affected_claims.len().min(5) as f64,
            // 创造性理由权重次之
            DetailedInvalidityGround::LackOfInventiveness {
                affected_claims, ..
            } => 0.30 + 0.05 * affected_claims.len().min(5) as f64,
            DetailedInvalidityGround::LackOfUtility { .. } => 0.25,
            // 公开不充分是强力理由
            DetailedInvalidityGround::InsufficientDisclosure { missing_elements } => {
                0.20 + 0.05 * missing_elements.len().min(4) as f64
            }
            DetailedInvalidityGround::ClaimsNotSupported { .. } => 0.20,
            DetailedInvalidityGround::ClaimsUnclear { unclear_claims, .. } => {
                0.15 + 0.03 * unclear_claims.len().min(5) as f64
            }
            DetailedInvalidityGround::BeyondOriginalScope { .. } => 0.30,
            DetailedInvalidityGround::NonPatentableSubjectMatter { .. } => 0.35,
        };
    }

    // 多理由叠加但有上限
    score.min(0.95)
}

/// 找出最强的无效理由索引
fn find_strongest_ground(grounds: &[DetailedInvalidityGround]) -> Option<usize> {
    if grounds.is_empty() {
        return None;
    }
    // 新颖性 > 创造性 > 不属于保护客体 > 公开不充分 > 不清楚
    let priority = |g: &DetailedInvalidityGround| -> u8 {
        match g {
            DetailedInvalidityGround::LackOfNovelty { .. } => 7,
            DetailedInvalidityGround::NonPatentableSubjectMatter { .. } => 6,
            DetailedInvalidityGround::LackOfInventiveness { .. } => 5,
            DetailedInvalidityGround::BeyondOriginalScope { .. } => 4,
            DetailedInvalidityGround::LackOfUtility { .. } => 3,
            DetailedInvalidityGround::InsufficientDisclosure { .. } => 2,
            DetailedInvalidityGround::ClaimsNotSupported { .. } => 1,
            DetailedInvalidityGround::ClaimsUnclear { .. } => 0,
        }
    };
    grounds
        .iter()
        .enumerate()
        .max_by_key(|(_, g)| priority(g))
        .map(|(i, _)| i)
}

/// 根据理由推荐策略
fn recommend_strategy(grounds: &[DetailedInvalidityGround]) -> String {
    if grounds.is_empty() {
        return "未发现明显无效理由，建议补充检索更多对比文件".into();
    }

    let mut parts = Vec::new();

    let has_novelty = grounds
        .iter()
        .any(|g| matches!(g, DetailedInvalidityGround::LackOfNovelty { .. }));
    let has_inventiveness = grounds
        .iter()
        .any(|g| matches!(g, DetailedInvalidityGround::LackOfInventiveness { .. }));
    let has_disclosure = grounds
        .iter()
        .any(|g| matches!(g, DetailedInvalidityGround::InsufficientDisclosure { .. }));
    let has_unclear = grounds
        .iter()
        .any(|g| matches!(g, DetailedInvalidityGround::ClaimsUnclear { .. }));

    if has_novelty {
        parts.push("以新颖性作为首要攻击点，优先针对独立权利要求");
    }
    if has_inventiveness {
        parts.push("结合多篇对比文件论证创造性不足");
    }
    if has_disclosure {
        parts.push("同步主张公开不充分，削弱专利权稳定性");
    }
    if has_unclear {
        parts.push("主张权利要求不清楚以限制保护范围解释");
    }

    if parts.is_empty() {
        parts.push("综合运用各项无效理由");
    }

    parts.join("；") + "。"
}

/// 根据理由收集证据需求
fn collect_evidence(grounds: &[DetailedInvalidityGround]) -> Vec<EvidenceRequirement> {
    let mut reqs = Vec::new();

    for g in grounds {
        match g {
            DetailedInvalidityGround::LackOfNovelty {
                prior_art,
                affected_claims,
            } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "对比文件".into(),
                    description: format!(
                        "{prior_art}用于否定权利要求{}的新颖性",
                        format_claim_numbers(affected_claims)
                    ),
                    importance: Importance::Required,
                });
            }
            DetailedInvalidityGround::LackOfInventiveness {
                closest_prior_art,
                reasoning,
                affected_claims,
            } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "对比文件".into(),
                    description: format!("{closest_prior_art}作为最接近现有技术：{reasoning}"),
                    importance: Importance::Required,
                });
                reqs.push(EvidenceRequirement {
                    evidence_type: "技术常识证据".into(),
                    description: format!(
                        "证明权利要求{}的惯用手段",
                        format_claim_numbers(affected_claims)
                    ),
                    importance: Importance::Important,
                });
            }
            DetailedInvalidityGround::LackOfUtility { reason } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "技术证据".into(),
                    description: format!("证明缺乏实用性：{reason}"),
                    importance: Importance::Required,
                });
            }
            DetailedInvalidityGround::InsufficientDisclosure { missing_elements } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "说明书分析".into(),
                    description: format!("证明公开不充分：{}", missing_elements.join("；")),
                    importance: Importance::Important,
                });
            }
            DetailedInvalidityGround::ClaimsNotSupported { unsupported_claims } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "说明书对比".into(),
                    description: format!(
                        "证明权利要求{}得不到说明书支持",
                        format_claim_numbers(unsupported_claims)
                    ),
                    importance: Importance::Important,
                });
            }
            DetailedInvalidityGround::ClaimsUnclear {
                unclear_claims,
                issues,
            } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "权利要求分析".into(),
                    description: format!(
                        "权利要求{}不清楚：{}",
                        format_claim_numbers(unclear_claims),
                        issues.join("；")
                    ),
                    importance: Importance::Auxiliary,
                });
            }
            DetailedInvalidityGround::BeyondOriginalScope { additions } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "原始申请文件".into(),
                    description: format!("证明修改超出原始范围：{}", additions.join("；")),
                    importance: Importance::Required,
                });
            }
            DetailedInvalidityGround::NonPatentableSubjectMatter {
                subject_matter,
                legal_basis,
            } => {
                reqs.push(EvidenceRequirement {
                    evidence_type: "法律依据".into(),
                    description: format!("{subject_matter}不属于专利保护客体（{legal_basis}）"),
                    importance: Importance::Required,
                });
            }
        }
    }

    reqs
}

/// 格式化权利要求编号列表
fn format_claim_numbers(claims: &[u32]) -> String {
    if claims.is_empty() {
        return String::new();
    }
    let nums: Vec<String> = claims.iter().map(|n| n.to_string()).collect();
    nums.join("、")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_grounds_when_all_checks_pass() {
        let grounds = analyze_grounds(false, false, false, false);
        assert!(grounds.is_empty());
    }

    #[test]
    fn both_novelty_and_inventiveness_flagged() {
        let grounds = analyze_grounds(true, true, false, false);
        assert_eq!(grounds.len(), 2);
        assert!(grounds.contains(&InvalidityGround::LackOfNovelty));
        assert!(grounds.contains(&InvalidityGround::LackOfInventiveness));
    }

    #[test]
    fn clarity_issue_added_separately() {
        let grounds = analyze_grounds(false, false, true, false);
        assert_eq!(grounds.len(), 1);
        assert_eq!(grounds[0], InvalidityGround::LackOfClarity);
    }

    #[test]
    fn petition_with_grounds_has_conclusion() {
        let grounds = vec![InvalidityGround::LackOfNovelty];
        let petition = generate_petition("CN12345678A", &grounds, vec![]);
        assert!(petition.conclusion.contains("请求宣告专利权全部无效"));
    }

    #[test]
    fn empty_petition_no_grounds() {
        let petition = generate_petition("CN12345678A", &[], vec![]);
        assert!(petition.conclusion.contains("未发现"));
    }

    #[test]
    fn add_claim_analysis_modifies_petition() {
        let grounds = vec![InvalidityGround::LackOfInventiveness];
        let mut petition = generate_petition("CN123", &grounds, vec![]);
        add_claim_analysis(
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

    // ---- Enhanced invalidity analysis tests ----

    #[test]
    fn novelty_conflict_detected_when_keywords_overlap() {
        let claims = vec!["一种温度传感器，包括热敏元件和信号处理电路".into()];
        let prior_art = "热敏元件用于温度检测，信号处理电路将模拟信号转换为数字信号";
        let affected = find_novelty_conflicts(&claims, prior_art);
        assert!(!affected.is_empty());
        assert_eq!(affected[0], 1);
    }

    #[test]
    fn no_novelty_conflict_when_keywords_differ() {
        let claims = vec!["一种量子计算装置，包括量子比特阵列和纠错模块".into()];
        let prior_art = "经典计算机通过CPU执行指令";
        let affected = find_novelty_conflicts(&claims, prior_art);
        assert!(affected.is_empty());
    }

    #[test]
    fn sufficiency_check_fails_without_embodiments() {
        let spec = "技术领域：本发明涉及传感器。";
        let claims = vec!["一种温度传感器".into()];
        let result = check_sufficiency(spec, &claims);
        assert!(result.is_some());
        let missing = result.unwrap();
        assert!(missing.iter().any(|m| m.contains("实施例")));
    }

    #[test]
    fn sufficiency_check_passes_with_full_spec() {
        let spec_good =
            "具体实施方式：实施例1中，温度传感器包括热敏元件，信号处理电路连接至热敏元件。";
        let claims_good = vec!["一种温度传感器，包括热敏元件".into()];
        let result = check_sufficiency(spec_good, &claims_good);
        assert!(result.is_none());
    }

    #[test]
    fn clarity_check_detects_vague_terms() {
        let claims = vec![
            "一种装置，其长度大约为10cm".into(),
            "一种方法，包括加热至适当的温度".into(),
        ];
        let (unclear, issues) = check_claim_clarity(&claims);
        assert_eq!(unclear.len(), 2);
        assert_eq!(unclear, vec![1, 2]);
        assert!(issues[0].contains("大约"));
        assert!(issues[1].contains("适当"));
    }

    #[test]
    fn clarity_check_passes_with_precise_claims() {
        let claims = vec!["一种传感器，包括热敏电阻，其阻值为10kΩ±1%".into()];
        let (unclear, issues) = check_claim_clarity(&claims);
        assert!(unclear.is_empty());
        assert!(issues.is_empty());
    }

    #[test]
    fn estimate_success_returns_zero_for_no_grounds() {
        let prob = estimate_success(&[]);
        assert!((prob - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn estimate_success_increases_with_more_grounds() {
        let g1 = DetailedInvalidityGround::LackOfNovelty {
            prior_art: "PA1".into(),
            affected_claims: vec![1],
        };
        let g2 = DetailedInvalidityGround::ClaimsUnclear {
            unclear_claims: vec![2],
            issues: vec!["模糊".into()],
        };
        let prob_one = estimate_success(&[g1.clone()]);
        let prob_two = estimate_success(&[g1, g2]);
        assert!(prob_two > prob_one);
        assert!(prob_two <= 0.95);
    }

    #[test]
    fn find_strongest_prefers_novelty_over_clarity() {
        let grounds = vec![
            DetailedInvalidityGround::ClaimsUnclear {
                unclear_claims: vec![1],
                issues: vec!["模糊".into()],
            },
            DetailedInvalidityGround::LackOfNovelty {
                prior_art: "PA1".into(),
                affected_claims: vec![1],
            },
        ];
        let idx = find_strongest_ground(&grounds).unwrap();
        assert_eq!(idx, 1); // novelty at index 1 is stronger
    }

    #[test]
    fn find_strongest_returns_none_for_empty() {
        assert_eq!(find_strongest_ground(&[]), None);
    }

    #[test]
    fn recommend_strategy_empty_grounds() {
        let s = recommend_strategy(&[]);
        assert!(s.contains("补充检索"));
    }

    #[test]
    fn recommend_strategy_with_novelty() {
        let grounds = vec![DetailedInvalidityGround::LackOfNovelty {
            prior_art: "PA1".into(),
            affected_claims: vec![1],
        }];
        let s = recommend_strategy(&grounds);
        assert!(s.contains("新颖性"));
    }

    #[test]
    fn collect_evidence_returns_required_for_novelty() {
        let grounds = vec![DetailedInvalidityGround::LackOfNovelty {
            prior_art: "PA1".into(),
            affected_claims: vec![1, 2],
        }];
        let ev = collect_evidence(&grounds);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].importance, Importance::Required);
        assert!(ev[0].description.contains("1、2"));
    }

    #[test]
    fn full_analyze_invalidity_integration() {
        let claims =
            vec!["一种温度传感器，包括热敏元件和信号处理电路，温度范围大约为-40℃至125℃".into()];
        let spec = "技术领域：传感器。";
        let prior_art = vec!["热敏元件和信号处理电路用于温度检测装置".into()];

        let result = analyze_invalidity(&claims, spec, &prior_art);

        // Should have novelty ground (prior art matches keywords)
        assert!(!result.grounds.is_empty());
        // Should have clarity issue ("大约")
        let has_clarity = result
            .grounds
            .iter()
            .any(|g| matches!(g, DetailedInvalidityGround::ClaimsUnclear { .. }));
        assert!(has_clarity);
        // Should have sufficiency issue (no 实施例 in spec)
        let has_sufficiency = result
            .grounds
            .iter()
            .any(|g| matches!(g, DetailedInvalidityGround::InsufficientDisclosure { .. }));
        assert!(has_sufficiency);
        // Success probability should be non-zero
        assert!(result.success_probability > 0.0);
        // Should have evidence requirements
        assert!(!result.evidence_requirements.is_empty());
    }

    #[test]
    fn format_claim_numbers_works() {
        assert_eq!(format_claim_numbers(&[1, 3, 5]), "1、3、5");
        assert_eq!(format_claim_numbers(&[]), "");
    }
}
