use super::{Rule, RuleOutput};

// ==================== 新颖性规则 ====================

pub(super) fn build_novelty_rules() -> Vec<Rule> {
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
