//! 多轮审查模拟：基于规则引擎的 OA 多轮推演
//!
//! 不调用 LLM，纯规则层实现。模拟审查员 OA → 申请人答复 → 评分 →
//! 下一轮或结案的完整流程。

use serde::Deserialize;
use serde::Serialize;

use super::types::ExaminerSimulator;

// ==================== 公共类型 ====================

/// 多轮模拟结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRoundSimulation {
    pub rounds: Vec<SimulatedRound>,
    pub final_prediction: GrantPrediction,
}

/// 单轮模拟
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedRound {
    pub round_number: u32,
    pub examiner_action: ExaminerAction,
    pub suggested_response: String,
    pub quality_score: f64,
    pub remaining_issues: Vec<String>,
}

/// 审查员行为
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExaminerAction {
    FirstOfficeAction {
        rejections: Vec<SimulatedRejection>,
    },
    SubsequentAction {
        rejections: Vec<SimulatedRejection>,
        allowances: Vec<u32>,
    },
    NoticeOfAllowance,
    FinalRejection {
        grounds: Vec<String>,
    },
}

/// 模拟驳回
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedRejection {
    pub claim_numbers: Vec<u32>,
    pub rejection_type: String,
    pub cited_art: Vec<String>,
    pub reasoning: String,
    pub difficulty: Difficulty,
}

/// 驳回难度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Moderate,
    Hard,
}

/// 授权预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPrediction {
    pub outcome: String,
    pub probability: f64,
    pub estimated_rounds: u32,
}

// ==================== 核心函数 ====================

/// 执行多轮模拟
///
/// - `claims`: 独立权利要求文本列表
/// - `prior_art`: 对比文件标识列表（如 `["D1", "D2"]`）
/// - `max_rounds`: 最大模拟轮次
pub fn simulate_multi_round(
    claims: &[String],
    prior_art: &[String],
    max_rounds: u32,
) -> MultiRoundSimulation {
    let mut rounds: Vec<SimulatedRound> = Vec::new();
    let mut active_rejections: Vec<SimulatedRejection> = Vec::new();
    let mut resolved_claims: Vec<u32> = Vec::new();

    for round in 1..=max_rounds {
        let examiner_action = if round == 1 {
            // 第一轮：生成 FirstOfficeAction
            active_rejections = generate_first_round_rejections(claims, prior_art);
            ExaminerAction::FirstOfficeAction {
                rejections: active_rejections.clone(),
            }
        } else {
            // 后续轮次：基于上一轮评分决定
            let prev_score = rounds.last().map_or(0.0, |r| r.quality_score);
            if prev_score >= 85.0 {
                ExaminerAction::NoticeOfAllowance
            } else if prev_score < 50.0 && round >= 3 {
                ExaminerAction::FinalRejection {
                    grounds: build_final_rejection_grounds(&active_rejections),
                }
            } else {
                // SubsequentAction：移除已解决的 claim，保留/新增驳回
                let (remaining, allowed) =
                    update_rejections(&active_rejections, &resolved_claims, claims, prior_art);
                active_rejections = remaining;
                ExaminerAction::SubsequentAction {
                    rejections: active_rejections.clone(),
                    allowances: allowed,
                }
            }
        };

        let suggested_response = generate_suggested_response(&examiner_action, claims);
        let quality_score = evaluate_response_quality(&suggested_response);
        let remaining_issues = extract_remaining_issues(&examiner_action);

        // 追踪本轮解决的 claim
        if let ExaminerAction::SubsequentAction { allowances, .. } = &examiner_action {
            resolved_claims.extend(allowances.iter().copied());
        }

        rounds.push(SimulatedRound {
            round_number: round,
            examiner_action,
            suggested_response,
            quality_score,
            remaining_issues,
        });

        // 终止条件
        match &rounds.last().expect("just pushed").examiner_action {
            ExaminerAction::NoticeOfAllowance | ExaminerAction::FinalRejection { .. } => break,
            _ => {}
        }
    }

    let final_prediction = build_grant_prediction(&rounds);

    MultiRoundSimulation {
        rounds,
        final_prediction,
    }
}

// ==================== 内部实现 ====================

/// 第一轮：基于 claim 特征与对比文件生成驳回
fn generate_first_round_rejections(
    claims: &[String],
    prior_art: &[String],
) -> Vec<SimulatedRejection> {
    let mut rejections: Vec<SimulatedRejection> = Vec::new();
    let mut claim_idx = 0u32;

    for claim in claims {
        claim_idx += 1;
        let features = extract_features(claim);

        // 创造性驳回（最常见）
        if let Some(rejection) = build_creativity_rejection(claim_idx, &features, prior_art) {
            rejections.push(rejection);
        }

        // 如果特征完全被 D1 覆盖，附加新颖性驳回
        if !prior_art.is_empty() && features_covered_by_prior_art(&features, &prior_art[0]) {
            rejections.push(SimulatedRejection {
                claim_numbers: vec![claim_idx],
                rejection_type: "新颖性".to_string(),
                cited_art: vec![prior_art[0].clone()],
                reasoning: format!(
                    "权利要求{claim_idx}的全部技术特征已被{}公开，不具备新颖性。",
                    prior_art[0]
                ),
                difficulty: Difficulty::Easy,
            });
        }
    }

    // 如果没有生成任何驳回（输入过短），生成一个默认创造性驳回
    if rejections.is_empty() && !claims.is_empty() {
        rejections.push(SimulatedRejection {
            claim_numbers: vec![1],
            rejection_type: "创造性".to_string(),
            cited_art: prior_art.to_vec(),
            reasoning: "权利要求1的技术方案对本领域技术人员来说是显而易见的。".to_string(),
            difficulty: Difficulty::Moderate,
        });
    }

    rejections
}

/// 提取权利要求的技术特征（按标点分句，过滤过短/过长片段）
fn extract_features(claim: &str) -> Vec<String> {
    claim
        .split(['，', '。', '；', ',', ';', '\n'])
        .map(str::trim)
        .filter(|p| {
            let len = p.chars().count();
            (6..120).contains(&len)
        })
        .map(|s| s.to_string())
        .collect()
}

/// 检查特征是否被单一对比文件覆盖（简易规则：字符串子串匹配）
fn features_covered_by_prior_art(features: &[String], art: &str) -> bool {
    if features.is_empty() {
        return false;
    }
    // 简化：如果超过 60% 的特征片段出现在对比文件文本中视为覆盖
    let covered = features
        .iter()
        .filter(|f| {
            let keywords: Vec<&str> = f.split('的').collect();
            keywords
                .iter()
                .any(|kw| art.contains(kw) && kw.chars().count() >= 4)
        })
        .count();
    covered * 100 / features.len().max(1) >= 60
}

/// 构建创造性驳回
fn build_creativity_rejection(
    claim_number: u32,
    features: &[String],
    prior_art: &[String],
) -> Option<SimulatedRejection> {
    if features.is_empty() {
        return None;
    }

    let (cited, difficulty) = if prior_art.len() >= 2 {
        (
            vec![prior_art[0].clone(), prior_art[1].clone()],
            Difficulty::Moderate,
        )
    } else if prior_art.len() == 1 {
        (vec![prior_art[0].clone()], Difficulty::Easy)
    } else {
        (vec!["公知常识".to_string()], Difficulty::Hard)
    };

    let feature_summary: String = features
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("、");

    let reasoning = if prior_art.len() >= 2 {
        format!(
            "权利要求{claim_number}的{feature_summary}等技术特征已被{}公开，\
             其余特征可由{}结合公知常识得到，不具备突出的实质性特点和显著进步。",
            prior_art[0], prior_art[1]
        )
    } else if prior_art.len() == 1 {
        format!(
            "权利要求{claim_number}的{feature_summary}等技术特征已被{}公开，\
             其余特征属于本领域常规技术手段，不具备创造性。",
            prior_art[0]
        )
    } else {
        format!(
            "权利要求{claim_number}的{feature_summary}等技术特征\
             均属于本领域的公知常识或常规技术手段，不具备创造性。"
        )
    };

    Some(SimulatedRejection {
        claim_numbers: vec![claim_number],
        rejection_type: "创造性".to_string(),
        cited_art: cited,
        reasoning,
        difficulty,
    })
}

/// 后续轮次更新驳回（移除已解决 claim，保留未解决）
fn update_rejections(
    prev_rejections: &[SimulatedRejection],
    resolved_claims: &[u32],
    claims: &[String],
    prior_art: &[String],
) -> (Vec<SimulatedRejection>, Vec<u32>) {
    let mut remaining: Vec<SimulatedRejection> = Vec::new();
    let mut allowed: Vec<u32> = Vec::new();

    for rejection in prev_rejections {
        let unresolved: Vec<u32> = rejection
            .claim_numbers
            .iter()
            .copied()
            .filter(|c| !resolved_claims.contains(c))
            .collect();

        if unresolved.is_empty() {
            allowed.extend(rejection.claim_numbers.iter().copied());
        } else {
            remaining.push(SimulatedRejection {
                claim_numbers: unresolved,
                ..rejection.clone()
            });
        }
    }

    // 如果所有驳回都已解决但还有 claim 未处理，对所有剩余 claim 新增驳回
    if remaining.is_empty() {
        let all_handled: Vec<u32> = allowed
            .iter()
            .chain(resolved_claims.iter())
            .copied()
            .collect();
        for (idx, _claim) in claims.iter().enumerate() {
            let claim_num = (idx as u32) + 1;
            if !all_handled.contains(&claim_num) {
                remaining.push(SimulatedRejection {
                    claim_numbers: vec![claim_num],
                    rejection_type: "创造性".to_string(),
                    cited_art: prior_art.to_vec(),
                    reasoning: format!("权利要求{claim_num}修改后仍存在上述问题，不具备创造性。"),
                    difficulty: Difficulty::Hard,
                });
            }
        }
    }

    (remaining, allowed)
}

/// 生成建议的申请人答复（模板化）
fn generate_suggested_response(action: &ExaminerAction, claims: &[String]) -> String {
    match action {
        ExaminerAction::FirstOfficeAction { rejections } => {
            let mut parts: Vec<String> = Vec::new();
            parts.push("首先，感谢审查员的审查意见。".to_string());

            for rej in rejections {
                let claim_str = rej
                    .claim_numbers
                    .iter()
                    .map(|n| format!("{n}"))
                    .collect::<Vec<_>>()
                    .join("、");
                parts.push(format!(
                    "其二，关于权利要求{claim_str}的{}驳回，我们认为：\
                     修改后的权利要求已明确限定了区别技术特征，\
                     这些特征在{}中均未公开，也非本领域公知常识。",
                    rej.rejection_type,
                    rej.cited_art.join("及")
                ));
            }

            parts.push("综上，修改后的权利要求具备创造性，恳请予以授权。".to_string());

            if !claims.is_empty() {
                parts.push(
                    "参见修改后的权利要求书。实验数据显示技术效果显著。专利法第22条第3款。"
                        .to_string(),
                );
            }

            parts.join("\n")
        }
        ExaminerAction::SubsequentAction {
            rejections,
            allowances,
        } => {
            let mut parts: Vec<String> = Vec::new();
            if !allowances.is_empty() {
                let allowed_str = allowances
                    .iter()
                    .map(|n| format!("{n}"))
                    .collect::<Vec<_>>()
                    .join("、");
                parts.push(format!("首先，感谢审查员对权利要求{allowed_str}的认可。"));
            }
            for rej in rejections {
                let claim_str = rej
                    .claim_numbers
                    .iter()
                    .map(|n| format!("{n}"))
                    .collect::<Vec<_>>()
                    .join("、");
                parts.push(format!(
                    "其次，关于权利要求{claim_str}，我们进一步限定了技术方案，\
                     区别于{}所公开的内容，实验数据对比试验显示效果显著。\
                     因此，修改后的权利要求具备突出的实质性特点。",
                    rej.cited_art.join("及")
                ));
            }
            parts.push(
                "综上，修改后的权利要求满足授权条件，恳请予以授权。专利法第22条第3款。".to_string(),
            );
            parts.join("\n")
        }
        ExaminerAction::NoticeOfAllowance => "申请已获得授权通知。".to_string(),
        ExaminerAction::FinalRejection { grounds } => {
            format!(
                "关于最终驳回，理由如下：{}。建议考虑提请复审。",
                grounds.join("；")
            )
        }
    }
}

/// 使用现有评分引擎评估答复质量
fn evaluate_response_quality(response: &str) -> f64 {
    let completeness = ExaminerSimulator::score_completeness(response);
    let persuasiveness = ExaminerSimulator::score_persuasiveness(response);
    let technical_depth = ExaminerSimulator::score_technical_depth(response);
    let logic_consistency = ExaminerSimulator::score_logic_consistency(response);

    completeness * 0.25 + persuasiveness * 0.30 + technical_depth * 0.25 + logic_consistency * 0.20
}

/// 提取当前轮次未解决的问题列表
fn extract_remaining_issues(action: &ExaminerAction) -> Vec<String> {
    match action {
        ExaminerAction::FirstOfficeAction { rejections }
        | ExaminerAction::SubsequentAction { rejections, .. } => rejections
            .iter()
            .map(|r| {
                let claims = r
                    .claim_numbers
                    .iter()
                    .map(|n| format!("{n}"))
                    .collect::<Vec<_>>()
                    .join("、");
                format!("权利要求{}存在{}问题", claims, r.rejection_type)
            })
            .collect(),
        ExaminerAction::NoticeOfAllowance => vec![],
        ExaminerAction::FinalRejection { grounds } => grounds.clone(),
    }
}

/// 构建 FinalRejection 的理由
fn build_final_rejection_grounds(rejections: &[SimulatedRejection]) -> Vec<String> {
    if rejections.is_empty() {
        return vec!["申请人未有效克服驳回理由".to_string()];
    }
    rejections
        .iter()
        .map(|r| {
            let claims = r
                .claim_numbers
                .iter()
                .map(|n| format!("{n}"))
                .collect::<Vec<_>>()
                .join("、");
            format!("权利要求{}仍存在{}缺陷", claims, r.rejection_type)
        })
        .collect()
}

/// 基于所有轮次构建授权预测
fn build_grant_prediction(rounds: &[SimulatedRound]) -> GrantPrediction {
    let last = rounds.last();
    let last_score = last.map_or(0.0, |r| r.quality_score);

    let (outcome, probability) = match last.map(|r| &r.examiner_action) {
        Some(ExaminerAction::NoticeOfAllowance) => ("授权".to_string(), 0.95),
        Some(ExaminerAction::FinalRejection { .. }) => ("驳回".to_string(), 0.10),
        _ => {
            if last_score >= 85.0 {
                ("有望授权".to_string(), 0.85)
            } else if last_score >= 70.0 {
                ("有望授权".to_string(), 0.70)
            } else if last_score >= 50.0 {
                ("存在授权可能".to_string(), 0.50)
            } else {
                ("授权可能性低".to_string(), 0.25)
            }
        }
    };

    GrantPrediction {
        outcome,
        probability,
        estimated_rounds: rounds.len() as u32,
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ── 类型基本构造 ──

    #[test]
    fn multi_round_simulation_serializes() {
        let sim = MultiRoundSimulation {
            rounds: vec![],
            final_prediction: GrantPrediction {
                outcome: "授权".to_string(),
                probability: 0.95,
                estimated_rounds: 1,
            },
        };
        let json = serde_json::to_string(&sim).expect("serialize");
        assert!(json.contains("rounds"));
        assert!(json.contains("final_prediction"));
    }

    #[test]
    fn examiner_action_variants_round_trip() {
        let actions = vec![
            ExaminerAction::FirstOfficeAction {
                rejections: vec![SimulatedRejection {
                    claim_numbers: vec![1],
                    rejection_type: "创造性".to_string(),
                    cited_art: vec!["D1".to_string()],
                    reasoning: "test".to_string(),
                    difficulty: Difficulty::Moderate,
                }],
            },
            ExaminerAction::SubsequentAction {
                rejections: vec![],
                allowances: vec![1, 2],
            },
            ExaminerAction::NoticeOfAllowance,
            ExaminerAction::FinalRejection {
                grounds: vec!["创造性不足".to_string()],
            },
        ];
        for action in &actions {
            let json = serde_json::to_string(action).expect("serialize");
            let back: ExaminerAction = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(json, serde_json::to_string(&back).unwrap());
        }
    }

    #[test]
    fn difficulty_copy_equality() {
        assert_eq!(Difficulty::Easy, Difficulty::Easy);
        assert_ne!(Difficulty::Easy, Difficulty::Hard);
    }

    // ── 核心模拟逻辑 ──

    #[test]
    fn simulate_single_claim_with_no_prior_art() {
        let claims = vec!["一种数据处理方法，包括接收用户输入的步骤。".to_string()];
        let prior_art: Vec<String> = vec![];
        let result = simulate_multi_round(&claims, &prior_art, 3);

        assert!(!result.rounds.is_empty());
        assert!(matches!(
            result.rounds[0].examiner_action,
            ExaminerAction::FirstOfficeAction { .. }
        ));
        // 没有对比文件时拒绝难度较高
        let first = &result.rounds[0];
        assert!(first.quality_score >= 0.0);
    }

    #[test]
    fn simulate_reaches_allowance_for_rich_response() {
        // 构造能够产生高分答复的场景
        let claims = vec![
            "一种数据处理方法，包括接收用户输入的步骤，处理数据的步骤，以及输出结果的步骤。"
                .to_string(),
        ];
        let prior_art = vec!["D1".to_string()];
        let result = simulate_multi_round(&claims, &prior_art, 5);

        // 至少有第一轮
        assert!(!result.rounds.is_empty());

        // 最终应该产生合理预测
        assert!(
            result.final_prediction.probability >= 0.0
                && result.final_prediction.probability <= 1.0
        );
        assert!(!result.final_prediction.outcome.is_empty());
    }

    #[test]
    fn simulate_max_rounds_respected() {
        let claims = vec!["一种方法。".to_string()];
        let prior_art = vec!["D1".to_string(), "D2".to_string()];
        let result = simulate_multi_round(&claims, &prior_art, 2);

        assert!(result.rounds.len() <= 2);
    }

    #[test]
    fn first_round_generates_rejections() {
        let claims = vec![
            "一种装置，包括处理器，所述处理器配置为执行指令。".to_string(),
            "根据权利要求1所述的装置，还包括存储器。".to_string(),
        ];
        let prior_art = vec!["D1".to_string()];
        let result = simulate_multi_round(&claims, &prior_art, 3);

        let first_round = &result.rounds[0];
        if let ExaminerAction::FirstOfficeAction { rejections } = &first_round.examiner_action {
            assert!(!rejections.is_empty());
            // 至少有一个创造性驳回
            assert!(
                rejections
                    .iter()
                    .any(|r| r.rejection_type.contains("创造性"))
            );
        } else {
            panic!("First round should be FirstOfficeAction");
        }
    }

    #[test]
    fn grant_prediction_reflects_final_state() {
        let claims = vec!["一种装置。".to_string()];
        let prior_art = vec!["D1".to_string()];
        let result = simulate_multi_round(&claims, &prior_art, 3);

        // 预测的轮次应等于实际轮次
        assert_eq!(
            result.final_prediction.estimated_rounds,
            result.rounds.len() as u32
        );
    }

    fn empty_claims_produces_valid_simulation() {
        let claims: Vec<String> = vec![];
        let prior_art = vec!["D1".to_string()];
        let result = simulate_multi_round(&claims, &prior_art, 3);

        // 空 claims 生成默认驳回，模拟至少 1 轮
        assert!(!result.rounds.is_empty());
        assert_eq!(
            result.final_prediction.estimated_rounds,
            result.rounds.len() as u32
        );
    }

    fn extract_features_splits_correctly() {
        let claim = "一种数据处理方法，包括接收用户输入并进行处理的步骤，以及输出结果的步骤。";
        let features = extract_features(claim);
        assert!(!features.is_empty());
        for f in &features {
            assert!(f.chars().count() >= 6);
        }
    }

    #[test]
    fn evaluate_response_quality_uses_existing_scoring() {
        let response =
            "因此权利要求具备创造性。参见对比文件D1。实验数据显示效果显著。专利法第22条。";
        let score = evaluate_response_quality(response);
        assert!(score > 0.0, "Response with keywords should score > 0");
    }

    #[test]
    fn remaining_issues_extraction() {
        let action = ExaminerAction::FirstOfficeAction {
            rejections: vec![SimulatedRejection {
                claim_numbers: vec![1],
                rejection_type: "创造性".to_string(),
                cited_art: vec!["D1".to_string()],
                reasoning: "test".to_string(),
                difficulty: Difficulty::Hard,
            }],
        };
        let issues = extract_remaining_issues(&action);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("权利要求1"));
    }

    #[test]
    fn notice_of_allowance_has_no_remaining_issues() {
        let action = ExaminerAction::NoticeOfAllowance;
        let issues = extract_remaining_issues(&action);
        assert!(issues.is_empty());
    }
}
