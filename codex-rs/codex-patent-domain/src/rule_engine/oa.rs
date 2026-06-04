use super::{Rule, RuleOutput};

// ==================== OA 答复规则 ====================

pub(super) fn build_oa_rules() -> Vec<Rule> {
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
