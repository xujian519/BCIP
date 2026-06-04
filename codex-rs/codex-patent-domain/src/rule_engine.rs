//! 定性规则推理引擎
//!
//! 基于专利审查实践的规则系统,支持新颖性分析、创造性分析、OA 答复策略建议。
//! 规则以纯 Rust 逻辑实现,无需外部 LLM 调用。

use codex_patent_core::AnalysisResult;
use codex_patent_core::AppliedRule;
use codex_patent_core::CaseContext;
use codex_patent_core::CombinationType;
use codex_patent_core::CompareFeature;
use codex_patent_core::InventionType;
use codex_patent_core::PatentError;

use crate::compare::FeatureMatcher;

/// 定性规则推理引擎
pub struct QualitativeRuleEngine {
    novelty_rules: Vec<Rule>,
    inventiveness_rules: Vec<Rule>,
    oa_rules: Vec<Rule>,
}

struct Rule {
    name: &'static str,
    evaluate: fn(&CaseContext) -> RuleOutput,
}

struct RuleOutput {
    applies: bool,
    conclusion: String,
    score: f64,
    confidence: f64,
}

impl QualitativeRuleEngine {
    pub fn new() -> Self {
        Self {
            novelty_rules: build_novelty_rules(),
            inventiveness_rules: build_inventiveness_rules(),
            oa_rules: build_oa_rules(),
        }
    }

    /// 新颖性分析
    pub fn analyze_novelty(&mut self, ctx: &CaseContext) -> Result<AnalysisResult, PatentError> {
        let mut applied = Vec::new();
        let mut total_score = 0.0;
        let mut total_confidence = 0.0;
        let mut count = 0usize;

        for rule in &self.novelty_rules {
            let out = (rule.evaluate)(ctx);
            if out.applies {
                applied.push(AppliedRule {
                    rule_name: rule.name.to_string(),
                    conclusion: out.conclusion.clone(),
                    applies: true,
                    score: out.score,
                });
                total_score += out.score;
                total_confidence += out.confidence;
                count += 1;
            }
        }

        if count == 0 {
            return Ok(AnalysisResult {
                conclusion: "信息不足,无法完成新颖性分析".into(),
                net_score: 0.0,
                confidence: 0.0,
                applied_rules: Vec::new(),
            });
        }

        let avg_score = total_score / count as f64;
        let avg_confidence = total_confidence / count as f64;

        let conclusion = if avg_score > 0.5 {
            "根据现有信息,该发明具备新颖性".into()
        } else {
            "根据现有信息,该发明可能缺乏新颖性".into()
        };

        Ok(AnalysisResult {
            conclusion,
            net_score: avg_score,
            confidence: avg_confidence,
            applied_rules: applied,
        })
    }

    /// 创造性分析
    pub fn analyze_inventiveness(
        &mut self,
        ctx: &CaseContext,
    ) -> Result<AnalysisResult, PatentError> {
        let mut applied = Vec::new();
        let mut total_score = 0.0;
        let mut total_confidence = 0.0;
        let mut count = 0usize;

        for rule in &self.inventiveness_rules {
            let out = (rule.evaluate)(ctx);
            if out.applies {
                applied.push(AppliedRule {
                    rule_name: rule.name.to_string(),
                    conclusion: out.conclusion.clone(),
                    applies: true,
                    score: out.score,
                });
                total_score += out.score;
                total_confidence += out.confidence;
                count += 1;
            }
        }

        if count == 0 {
            return Ok(AnalysisResult {
                conclusion: "信息不足,无法完成创造性分析".into(),
                net_score: 0.0,
                confidence: 0.0,
                applied_rules: Vec::new(),
            });
        }

        let avg_score = total_score / count as f64;
        let avg_confidence = total_confidence / count as f64;

        let conclusion = if avg_score > 0.5 {
            "根据现有信息,该发明具备创造性".into()
        } else {
            "根据现有信息,该发明可能缺乏创造性".into()
        };

        Ok(AnalysisResult {
            conclusion,
            net_score: avg_score,
            confidence: avg_confidence,
            applied_rules: applied,
        })
    }

    /// OA 答复策略建议
    pub fn suggest_oa_strategy(
        &mut self,
        ctx: &CaseContext,
    ) -> Result<AnalysisResult, PatentError> {
        let mut applied = Vec::new();
        let mut total_score = 0.0;
        let mut total_confidence = 0.0;
        let mut count = 0usize;

        for rule in &self.oa_rules {
            let out = (rule.evaluate)(ctx);
            if out.applies {
                applied.push(AppliedRule {
                    rule_name: rule.name.to_string(),
                    conclusion: out.conclusion.clone(),
                    applies: true,
                    score: out.score,
                });
                total_score += out.score;
                total_confidence += out.confidence;
                count += 1;
            }
        }

        if count == 0 {
            return Ok(AnalysisResult {
                conclusion: "无法确定 OA 答复策略,请提供更多信息".into(),
                net_score: 0.0,
                confidence: 0.0,
                applied_rules: Vec::new(),
            });
        }

        let avg_score = total_score / count as f64;
        let avg_confidence = total_confidence / count as f64;

        let conclusion = if avg_score > 0.6 {
            "建议采用修改权利要求的策略".into()
        } else {
            "建议结合意见陈述和权利要求修改".into()
        };

        Ok(AnalysisResult {
            conclusion,
            net_score: avg_score,
            confidence: avg_confidence,
            applied_rules: applied,
        })
    }
}

impl Default for QualitativeRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 新颖性规则 ====================

fn build_novelty_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "NR-01: 单独对比原则",
            evaluate: |ctx| {
                if let Some(contains_all) = ctx.prior_art_contains_all {
                    if contains_all {
                        RuleOutput {
                            applies: true,
                            conclusion: "对比文件包含了发明的全部技术特征,新颖性受到质疑".into(),
                            score: 0.1,
                            confidence: 0.8,
                        }
                    } else {
                        RuleOutput {
                            applies: true,
                            conclusion: "对比文件未包含全部技术特征,存在新颖性空间".into(),
                            score: 0.8,
                            confidence: 0.7,
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
            name: "NR-02: 区别技术特征",
            evaluate: |ctx| {
                if let Some(ref diffs) = ctx.differences {
                    if !diffs.is_empty() {
                        RuleOutput {
                            applies: true,
                            conclusion: format!(
                                "存在{}个区别技术特征:{}",
                                diffs.len(),
                                diffs.join("、")
                            ),
                            score: 0.7 + 0.1 * (diffs.len().min(3) as f64),
                            confidence: 0.75,
                        }
                    } else {
                        RuleOutput {
                            applies: true,
                            conclusion: "未发现区别技术特征".into(),
                            score: 0.05,
                            confidence: 0.9,
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
            name: "NR-03: 实质相同判断",
            evaluate: |ctx| {
                if let (Some(diffs), Some(invention)) = (&ctx.differences, &ctx.invention) {
                    if diffs.is_empty() && !invention.is_empty() {
                        RuleOutput {
                            applies: true,
                            conclusion: "发明与对比文件实质相同,缺乏新颖性".into(),
                            score: 0.05,
                            confidence: 0.85,
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
    ]
}

// ==================== 创造性规则 ====================

fn build_inventiveness_rules() -> Vec<Rule> {
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

// ==================== OA 答复规则 ====================

fn build_oa_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "OA-01: 新颖性驳回应对",
            evaluate: |ctx| {
                if let Some(ref rt) = ctx.rejection_type {
                    if rt.contains("新颖性") || rt.contains("new") || rt == "X" {
                        RuleOutput {
                            applies: true,
                            conclusion: "新颖性驳回:建议强调区别技术特征,或修改权利要求增加限定"
                                .into(),
                            score: 0.7,
                            confidence: 0.8,
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
            name: "OA-02: 创造性驳回应对",
            evaluate: |ctx| {
                if let Some(ref rt) = ctx.rejection_type {
                    if rt.contains("创造性") || rt.contains("inventive") || rt == "Y" {
                        let has_effects = ctx
                            .technical_effects
                            .as_ref()
                            .is_some_and(|e| !e.is_empty());
                        let has_diffs = ctx.differences.as_ref().is_some_and(|d| !d.is_empty());
                        if has_effects && has_diffs {
                            RuleOutput {
                                applies: true,
                                conclusion:
                                    "创造性驳回:有区别特征和技术效果支撑,建议详细论述非显而易见性"
                                        .into(),
                                score: 0.75,
                                confidence: 0.8,
                            }
                        } else {
                            RuleOutput {
                                applies: true,
                                conclusion:
                                    "创造性驳回:建议补充技术效果论证,或修改权利要求引入区别特征"
                                        .into(),
                                score: 0.5,
                                confidence: 0.7,
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
            name: "OA-03: 跨领域组合应对",
            evaluate: |ctx| {
                if let Some(true) = ctx.prior_art_different_field {
                    RuleOutput {
                        applies: true,
                        conclusion: "对比文件来自不同技术领域:可论证不存在技术启示".into(),
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novelty_analysis_with_differences() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some("一种数据处理方法".into()),
            prior_art_contains_all: Some(false),
            differences: Some(vec!["特征A".into(), "特征B".into()]),
            ..Default::default()
        };
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        assert!(!result.applied_rules.is_empty());
        assert!(result.net_score > 0.5);
    }

    #[test]
    fn test_inventiveness_analysis() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some("一种新算法".into()),
            distinguishing_features: Some(vec!["自适应学习率调整".into(), "多模态融合".into()]),
            actual_problem_solved: Some("提高模型收敛速度".into()),
            has_unexpected_effect: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        assert!(!result.applied_rules.is_empty());
        assert!(result.net_score > 0.5);
    }

    #[test]
    fn test_ir01_distinguishing_features() {
        let mut engine = QualitativeRuleEngine::new();
        use codex_patent_core::FeatureType;
        use codex_patent_core::ParsedFeature;
        let ctx = CaseContext {
            claim_features: Some(vec![
                ParsedFeature {
                    id: "C1".into(),
                    description: "温度传感器".into(),
                    feature_type: FeatureType::Element,
                    component: None,
                    parameters: vec![],
                },
                ParsedFeature {
                    id: "C2".into(),
                    description: "AI自适应控制模块".into(),
                    feature_type: FeatureType::Element,
                    component: None,
                    parameters: vec![],
                },
            ]),
            prior_art_features: Some(vec![ParsedFeature {
                id: "P1".into(),
                description: "温度传感器".into(),
                feature_type: FeatureType::Element,
                component: None,
                parameters: vec![],
            }]),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir01 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-01"));
        assert!(ir01.is_some(), "IR-01 应该触发");
        assert!(ir01.expect("test analysis result should be present").score > 0.3);
    }

    #[test]
    fn test_ir02_problem_reformulation() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            distinguishing_features: Some(vec!["加密芯片".into()]),
            actual_problem_solved: Some("提高数据安全性".into()),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir02 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-02"));
        assert!(ir02.is_some());
        assert!(ir02.expect("test analysis result should be present").score > 0.5);
    }

    #[test]
    fn test_ir04_teaching_away() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            has_teaching_away: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir04 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-04"));
        assert!(ir04.is_some());
        assert!(ir04.expect("test analysis result should be present").score > 0.7);
    }

    #[test]
    fn test_ir05_combination_synergistic() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            is_combination: Some(CombinationType::Synergistic),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir05 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-05"));
        assert!(ir05.is_some());
        assert!(ir05.expect("test analysis result should be present").score > 0.7);
    }

    #[test]
    fn test_ir06_selection_with_effect() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention_type: Some(InventionType::Selection),
            has_unexpected_effect: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir06 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-06"));
        assert!(ir06.is_some());
        assert!(ir06.expect("test analysis result should be present").score > 0.7);
    }

    #[test]
    fn test_secondary_factors() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            has_unexpected_effect: Some(true),
            has_technical_prejudice: Some(true),
            has_long_felt_need: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        assert!(result.applied_rules.len() >= 3, "应触发 IR-07/08/09");
        assert!(result.net_score > 0.7);
    }

    #[test]
    fn test_oa_strategy() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            rejection_type: Some("创造性".into()),
            differences: Some(vec!["区别特征1".into()]),
            technical_effects: Some(vec!["提高了效率".into()]),
            ..Default::default()
        };
        let result = engine
            .suggest_oa_strategy(&ctx)
            .expect("test tool call should succeed");
        assert!(!result.applied_rules.is_empty());
    }

    #[test]
    fn test_novelty_empty_context_returns_insufficient_info() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext::default();
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        assert_eq!(result.net_score, 0.0);
        assert!(result.applied_rules.is_empty());
        assert!(result.conclusion.contains("信息不足"));
    }

    #[test]
    fn test_nr01_prior_art_contains_all() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            prior_art_contains_all: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        let nr01 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("NR-01"));
        assert!(nr01.is_some());
        assert!(nr01.expect("test analysis result should be present").score < 0.2);
        assert!(
            nr01.expect("test analysis result should be present")
                .conclusion
                .contains("全部技术特征")
        );
    }

    #[test]
    fn test_nr01_prior_art_not_contains_all() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            prior_art_contains_all: Some(false),
            ..Default::default()
        };
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        let nr01 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("NR-01"));
        assert!(nr01.is_some());
        assert!(nr01.expect("test analysis result should be present").score > 0.7);
    }

    #[test]
    fn test_nr02_empty_differences() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            differences: Some(vec![]),
            ..Default::default()
        };
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        let nr02 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("NR-02"));
        assert!(nr02.is_some());
        assert!(nr02.expect("test analysis result should be present").score < 0.1);
    }

    #[test]
    fn test_nr03_substantially_same() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some("一种数据处理方法".into()),
            differences: Some(vec![]),
            ..Default::default()
        };
        let result = engine
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        let nr03 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("NR-03"));
        assert!(nr03.is_some());
        assert!(
            nr03.expect("test analysis result should be present")
                .conclusion
                .contains("实质相同")
        );
        assert!(nr03.expect("test analysis result should be present").score < 0.1);
    }

    #[test]
    fn test_inventiveness_empty_context_returns_insufficient_info() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext::default();
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        assert_eq!(result.net_score, 0.0);
        assert!(result.applied_rules.is_empty());
        assert!(result.conclusion.contains("信息不足"));
    }

    #[test]
    fn test_ir03_common_knowledge_short_feature() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            distinguishing_features: Some(vec!["弹簧".into()]),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir03 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-03"));
        assert!(ir03.is_some());
        assert!(
            ir03.expect("test analysis result should be present")
                .conclusion
                .contains("公知常识")
        );
        assert!(ir03.expect("test analysis result should be present").score < 0.3);
    }

    #[test]
    fn test_ir03_non_common_knowledge_long_feature() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            distinguishing_features: Some(vec![
                "基于深度学习的多模态特征融合与自适应权重分配机制".into(),
            ]),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir03 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-03"));
        assert!(ir03.is_some());
        assert!(ir03.expect("test analysis result should be present").score > 0.5);
    }

    #[test]
    fn test_ir05_combination_simple_stack() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            is_combination: Some(CombinationType::SimpleStack),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir05 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-05"));
        assert!(ir05.is_some());
        assert!(ir05.expect("test analysis result should be present").score < 0.3);
        assert!(
            ir05.expect("test analysis result should be present")
                .conclusion
                .contains("简单叠加")
        );
    }

    #[test]
    fn test_ir06_selection_without_effect() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention_type: Some(InventionType::Selection),
            has_unexpected_effect: Some(false),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir06 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-06"));
        assert!(ir06.is_some());
        assert!(ir06.expect("test analysis result should be present").score < 0.4);
    }

    #[test]
    fn test_ir10_technical_effect() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            technical_effect: Some("显著提高了数据传输效率".into()),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir10 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-10"));
        assert!(ir10.is_some());
        assert!(ir10.expect("test analysis result should be present").score > 0.6);
    }

    #[test]
    fn test_ir11_performance_improvement_high() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            performance_improvement: Some(0.8),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir11 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-11"));
        assert!(ir11.is_some());
        assert!(ir11.expect("test analysis result should be present").score > 0.8);
        assert!(
            ir11.expect("test analysis result should be present")
                .conclusion
                .contains("80%")
        );
    }

    #[test]
    fn test_ir11_performance_improvement_low() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            performance_improvement: Some(0.05),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir11 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-11"));
        assert!(ir11.is_some());
        assert!(ir11.expect("test analysis result should be present").score < 0.35);
    }

    #[test]
    fn test_ir12_obviousness_obvious() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            obviousness: Some(true),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir12 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-12"));
        assert!(ir12.is_some());
        assert!(ir12.expect("test analysis result should be present").score < 0.2);
        assert!(
            ir12.expect("test analysis result should be present")
                .conclusion
                .contains("显而易见")
        );
    }

    #[test]
    fn test_ir12_obviousness_non_obvious() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            obviousness: Some(false),
            ..Default::default()
        };
        let result = engine
            .analyze_inventiveness(&ctx)
            .expect("test tool call should succeed");
        let ir12 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("IR-12"));
        assert!(ir12.is_some());
        assert!(ir12.expect("test analysis result should be present").score > 0.7);
    }

    #[test]
    fn test_oa_strategy_novelty_rejection() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            rejection_type: Some("新颖性".into()),
            ..Default::default()
        };
        let result = engine
            .suggest_oa_strategy(&ctx)
            .expect("test tool call should succeed");
        let oa01 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("OA-01"));
        assert!(oa01.is_some());
        assert!(
            oa01.expect("test analysis result should be present")
                .conclusion
                .contains("新颖性驳回")
        );
    }

    #[test]
    fn test_oa_strategy_cross_field() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            prior_art_different_field: Some(true),
            ..Default::default()
        };
        let result = engine
            .suggest_oa_strategy(&ctx)
            .expect("test tool call should succeed");
        let oa03 = result
            .applied_rules
            .iter()
            .find(|r| r.rule_name.contains("OA-03"));
        assert!(oa03.is_some());
        assert!(
            oa03.expect("test analysis result should be present")
                .conclusion
                .contains("不同技术领域")
        );
    }

    #[test]
    fn test_oa_strategy_no_matching_rules() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            rejection_type: Some("不清楚".into()),
            ..Default::default()
        };
        let result = engine
            .suggest_oa_strategy(&ctx)
            .expect("test tool call should succeed");
        assert!(result.applied_rules.is_empty());
        assert!(result.conclusion.contains("无法确定"));
    }

    #[test]
    fn test_default_impl() {
        let mut engine1 = QualitativeRuleEngine::new();
        let mut engine2 = QualitativeRuleEngine::default();
        let ctx = CaseContext {
            prior_art_contains_all: Some(false),
            ..Default::default()
        };
        let r1 = engine1
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        let r2 = engine2
            .analyze_novelty(&ctx)
            .expect("test tool call should succeed");
        assert_eq!(r1.net_score, r2.net_score);
    }
}
