//! 定性规则推理引擎
//!
//! 基于专利审查实践的规则系统,支持新颖性分析、创造性分析、OA 答复策略建议。
//! 规则以纯 Rust 逻辑实现,无需外部 LLM 调用。

use codex_patent_core::AnalysisResult;
use codex_patent_core::AppliedRule;
use codex_patent_core::CaseContext;
use codex_patent_core::PatentError;

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
        Rule {
            name: "IR-01: 技术效果显著性",
            evaluate: |ctx| {
                if let Some(ref effect) = ctx.technical_effect {
                    if effect.len() > 20 {
                        RuleOutput {
                            applies: true,
                            conclusion: "发明具有明确的技术效果描述".into(),
                            score: 0.7,
                            confidence: 0.7,
                        }
                    } else {
                        RuleOutput {
                            applies: true,
                            conclusion: "技术效果描述不够充分".into(),
                            score: 0.4,
                            confidence: 0.6,
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
            name: "IR-02: 性能提升幅度",
            evaluate: |ctx| {
                if let Some(improvement) = ctx.performance_improvement {
                    if improvement > 0.5 {
                        RuleOutput {
                            applies: true,
                            conclusion: format!(
                                "性能提升{:.0}%,具有显著的进步",
                                improvement * 100.0
                            ),
                            score: 0.85,
                            confidence: 0.8,
                        }
                    } else if improvement > 0.1 {
                        RuleOutput {
                            applies: true,
                            conclusion: format!("性能提升{:.0}%,进步较为明显", improvement * 100.0),
                            score: 0.6,
                            confidence: 0.7,
                        }
                    } else {
                        RuleOutput {
                            applies: true,
                            conclusion: format!("性能提升{:.0}%,进步不够显著", improvement * 100.0),
                            score: 0.3,
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
            name: "IR-03: 显而易见性判断",
            evaluate: |ctx| {
                if let Some(obvious) = ctx.obviousness {
                    if obvious {
                        RuleOutput {
                            applies: true,
                            conclusion: "发明对本领域技术人员而言是显而易见的".into(),
                            score: 0.15,
                            confidence: 0.7,
                        }
                    } else {
                        RuleOutput {
                            applies: true,
                            conclusion: "发明对本领域技术人员而言非显而易见".into(),
                            score: 0.8,
                            confidence: 0.75,
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
        let result = engine.analyze_novelty(&ctx).unwrap();
        assert!(!result.applied_rules.is_empty());
        assert!(result.net_score > 0.5);
    }

    #[test]
    fn test_inventiveness_analysis() {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some("一种新算法".into()),
            technical_effect: Some("提高了处理速度".into()),
            performance_improvement: Some(0.6),
            obviousness: Some(false),
            ..Default::default()
        };
        let result = engine.analyze_inventiveness(&ctx).unwrap();
        assert!(!result.applied_rules.is_empty());
        assert!(result.net_score > 0.5);
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
        let result = engine.suggest_oa_strategy(&ctx).unwrap();
        assert!(!result.applied_rules.is_empty());
    }
}
