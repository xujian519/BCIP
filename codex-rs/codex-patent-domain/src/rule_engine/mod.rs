//! 定性规则推理引擎
//!
//! 基于专利审查实践的规则系统,支持新颖性分析、创造性分析、OA 答复策略建议。
//! 规则以纯 Rust 逻辑实现,无需外部 LLM 调用。

use codex_patent_core::AnalysisResult;
use codex_patent_core::AppliedRule;
use codex_patent_core::CaseContext;
use codex_patent_core::PatentError;

mod inventiveness;
mod novelty;
mod oa;

/// 定性规则推理引擎
pub struct QualitativeRuleEngine {
    novelty_rules: Vec<Rule>,
    inventiveness_rules: Vec<Rule>,
    oa_rules: Vec<Rule>,
}

pub(super) struct Rule {
    name: &'static str,
    evaluate: fn(&CaseContext) -> RuleOutput,
}

pub(super) struct RuleOutput {
    applies: bool,
    conclusion: String,
    score: f64,
    confidence: f64,
}

impl QualitativeRuleEngine {
    pub fn new() -> Self {
        Self {
            novelty_rules: novelty::build_novelty_rules(),
            inventiveness_rules: inventiveness::build_inventiveness_rules(),
            oa_rules: oa::build_oa_rules(),
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
            is_combination: Some(codex_patent_core::CombinationType::Synergistic),
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
            invention_type: Some(codex_patent_core::InventionType::Selection),
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
            is_combination: Some(codex_patent_core::CombinationType::SimpleStack),
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
            invention_type: Some(codex_patent_core::InventionType::Selection),
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
