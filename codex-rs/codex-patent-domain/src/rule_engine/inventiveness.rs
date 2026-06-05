use codex_patent_core::CombinationType;
use codex_patent_core::CompareFeature;
use codex_patent_core::InventionType;

use crate::compare::FeatureMatcher;

use super::Rule;
use super::RuleOutput;

// ==================== 创造性规则 ====================

pub(super) fn build_inventiveness_rules() -> Vec<Rule> {
    vec![
        // ── 步骤2: 区别特征与技术问题 ──
        Rule {
            name: "IR-01: 区别特征识别",
            evaluate: |ctx| match (&ctx.claim_features, &ctx.prior_art_features) {
                (Some(claims), Some(prior)) if !claims.is_empty() && !prior.is_empty() => {
                    let target: Vec<CompareFeature> = claims
                        .iter()
                        .map(|f| CompareFeature {
                            id: f.id.clone(),
                            description: f.description.clone(),
                        })
                        .collect();
                    let prior_feats: Vec<CompareFeature> = prior
                        .iter()
                        .map(|f| CompareFeature {
                            id: f.id.clone(),
                            description: f.description.clone(),
                        })
                        .collect();
                    let result = FeatureMatcher::compare(&target, &prior_feats);
                    let dist_count =
                        result.different_features.len() + result.missing_features.len();
                    let coverage = result.coverage_ratio;
                    RuleOutput {
                        applies: true,
                        conclusion: format!(
                            "识别到{dist_count}个区别特征，特征覆盖率{:.0}%",
                            coverage * 100.0
                        ),
                        score: if dist_count == 0 {
                            0.1
                        } else {
                            (0.3 + 0.14 * (dist_count.min(5) as f64)).min(1.0)
                        },
                        confidence: 0.8,
                    }
                }
                _ => RuleOutput {
                    applies: false,
                    conclusion: String::new(),
                    score: 0.0,
                    confidence: 0.0,
                },
            },
        },
        Rule {
            name: "IR-02: 技术问题重定",
            evaluate: |ctx| {
                if let Some(ref dists) = ctx.distinguishing_features {
                    if dists.is_empty() {
                        return RuleOutput {
                            applies: true,
                            conclusion: "无区别特征，无法重新确定技术问题".into(),
                            score: 0.05,
                            confidence: 0.9,
                        };
                    }
                    match &ctx.actual_problem_solved {
                        Some(problem) if !problem.is_empty() => RuleOutput {
                            applies: true,
                            conclusion: format!(
                                "基于{}个区别特征重新确定技术问题：{problem}",
                                dists.len()
                            ),
                            score: 0.7,
                            confidence: 0.75,
                        },
                        _ => RuleOutput {
                            applies: true,
                            conclusion: format!(
                                "存在{}个区别特征但未明确实际解决的技术问题",
                                dists.len()
                            ),
                            score: 0.4,
                            confidence: 0.6,
                        },
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        // ── 步骤3: 技术启示判断 ──
        Rule {
            name: "IR-03: 公知常识判定",
            evaluate: |ctx| {
                if let Some(ref dists) = ctx.distinguishing_features {
                    if dists.is_empty() {
                        return RuleOutput {
                            applies: false,
                            conclusion: String::new(),
                            score: 0.0,
                            confidence: 0.0,
                        };
                    }
                    // 启发式：区别特征越短越可能是公知常识
                    let avg_len = dists.iter().map(|d| d.len()).sum::<usize>() as f64
                        / dists.len().max(1) as f64;
                    let is_common = avg_len < 10.0;
                    RuleOutput {
                        applies: true,
                        conclusion: if is_common {
                            "区别特征描述简短，可能属于公知常识".into()
                        } else {
                            "区别特征具有特定技术含义，非显然公知常识".into()
                        },
                        score: if is_common { 0.2 } else { 0.65 },
                        confidence: 0.5,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        Rule {
            name: "IR-04: 相反教导检测",
            evaluate: |ctx| {
                if let Some(true) = ctx.has_teaching_away {
                    RuleOutput {
                        applies: true,
                        conclusion: "存在相反教导(teaching away)，削弱技术启示".into(),
                        score: 0.8,
                        confidence: 0.75,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        // ── 发明类型判断 ──
        Rule {
            name: "IR-05: 组合发明判断",
            evaluate: |ctx| match ctx.is_combination {
                Some(CombinationType::Synergistic) => RuleOutput {
                    applies: true,
                    conclusion: "组合发明各要素功能上彼此支持产生协同效果，具备创造性".into(),
                    score: 0.8,
                    confidence: 0.75,
                },
                Some(CombinationType::SimpleStack) => RuleOutput {
                    applies: true,
                    conclusion: "组合发明为简单叠加，各要素各自发挥常规功能".into(),
                    score: 0.25,
                    confidence: 0.7,
                },
                None => RuleOutput {
                    applies: false,
                    conclusion: String::new(),
                    score: 0.0,
                    confidence: 0.0,
                },
            },
        },
        Rule {
            name: "IR-06: 选择发明判断",
            evaluate: |ctx| {
                if ctx.invention_type == Some(InventionType::Selection) {
                    let has_effect = ctx.has_unexpected_effect.unwrap_or(false);
                    RuleOutput {
                        applies: true,
                        conclusion: if has_effect {
                            "选择发明的特定范围产生预料不到的技术效果".into()
                        } else {
                            "选择发明未证明产生预料不到效果".into()
                        },
                        score: if has_effect { 0.75 } else { 0.35 },
                        confidence: 0.7,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        // ── 辅助因素 ──
        Rule {
            name: "IR-07: 预料不到效果",
            evaluate: |ctx| {
                if let Some(true) = ctx.has_unexpected_effect {
                    RuleOutput {
                        applies: true,
                        conclusion: "发明产生了预料不到的技术效果（辅助因素正向）".into(),
                        score: 0.8,
                        confidence: 0.75,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        Rule {
            name: "IR-08: 技术偏见克服",
            evaluate: |ctx| {
                if let Some(true) = ctx.has_technical_prejudice {
                    RuleOutput {
                        applies: true,
                        conclusion: "发明克服了技术偏见（辅助因素正向）".into(),
                        score: 0.8,
                        confidence: 0.7,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        Rule {
            name: "IR-09: 长期难题",
            evaluate: |ctx| {
                if let Some(true) = ctx.has_long_felt_need {
                    RuleOutput {
                        applies: true,
                        conclusion: "发明解决了本领域长期存在的需求（辅助因素正向）".into(),
                        score: 0.75,
                        confidence: 0.7,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        // ── 旧字段兼容规则 ──
        Rule {
            name: "IR-10: 技术效果评估",
            evaluate: |ctx| {
                if let Some(ref effect) = ctx.technical_effect {
                    if !effect.is_empty() {
                        RuleOutput {
                            applies: true,
                            conclusion: "发明具有明确的技术效果".into(),
                            score: 0.65,
                            confidence: 0.6,
                        }
                    } else {
                        RuleOutput {
                            applies: false,
                            conclusion: String::new(),
                            score: 0.0,
                            confidence: 0.0,
                        }
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        Rule {
            name: "IR-11: 性能提升幅度",
            evaluate: |ctx| {
                if let Some(improvement) = ctx.performance_improvement {
                    let score = if improvement > 0.5 {
                        0.85
                    } else if improvement > 0.1 {
                        0.6
                    } else {
                        0.3
                    };
                    RuleOutput {
                        applies: true,
                        conclusion: format!("性能提升{:.0}%", improvement * 100.0),
                        score,
                        confidence: 0.7,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
        Rule {
            name: "IR-12: 非显而易见性",
            evaluate: |ctx| {
                if let Some(obvious) = ctx.obviousness {
                    RuleOutput {
                        applies: true,
                        conclusion: if obvious {
                            "对本领域技术人员而言显而易见".into()
                        } else {
                            "对本领域技术人员而言非显而易见".into()
                        },
                        score: if obvious { 0.15 } else { 0.75 },
                        confidence: 0.65,
                    }
                } else {
                    RuleOutput {
                        applies: false,
                        conclusion: String::new(),
                        score: 0.0,
                        confidence: 0.0,
                    }
                }
            },
        },
    ]
}
