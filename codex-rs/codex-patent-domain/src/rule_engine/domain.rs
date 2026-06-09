//! 技术领域差异化分析策略

use codex_patent_core::{AnalysisResult, AppliedRule, CaseContext};
use serde::{Deserialize, Serialize};

/// 技术领域分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechDomain {
    Mechanical,
    Electrical,
    Chemical,
    Biotechnology,
    Software,
    Communication,
    General,
}

/// 化学领域分析参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChemicalAnalysisParams {
    pub parameter_ranges: Vec<(String, String, String)>,
    pub embodiment_count: usize,
    pub has_comparative_data: bool,
    pub has_unexpected_effect: bool,
}

/// 软件领域分析参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareAnalysisParams {
    pub algorithm_steps: usize,
    pub has_technical_effect: bool,
    pub solves_technical_problem: bool,
    pub hardware_involved: bool,
}

/// 领域感知的分析上下文
#[derive(Debug, Clone)]
pub struct DomainAwareContext {
    pub base: CaseContext,
    pub tech_domain: TechDomain,
    pub chemical_params: Option<ChemicalAnalysisParams>,
    pub software_params: Option<SoftwareAnalysisParams>,
}

/// 根据发明描述自动识别技术领域
pub fn detect_tech_domain(description: &str) -> TechDomain {
    let chem_kws = ["化合物", "合成", "催化剂", "聚合物", "反应", "配比", "浓度"];
    let sw_kws = ["算法", "模型", "训练", "数据集", "神经网络", "编码", "解码"];
    let mech_kws = ["轴", "齿轮", "轴承", "弹簧", "凸轮", "传动", "壳体"];
    let elec_kws = ["电路", "电压", "电流", "芯片", "晶体管", "信号"];
    let bio_kws = ["基因", "蛋白", "细胞", "抗体", "序列", "表达"];
    let comm_kws = ["协议", "信道", "调制", "编码", "传输", "基站"];

    let scores = [
        (TechDomain::Chemical, count_matches(description, &chem_kws)),
        (TechDomain::Software, count_matches(description, &sw_kws)),
        (
            TechDomain::Mechanical,
            count_matches(description, &mech_kws),
        ),
        (
            TechDomain::Electrical,
            count_matches(description, &elec_kws),
        ),
        (
            TechDomain::Biotechnology,
            count_matches(description, &bio_kws),
        ),
        (
            TechDomain::Communication,
            count_matches(description, &comm_kws),
        ),
    ];

    scores
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .filter(|(_, c)| *c > 0)
        .map(|(d, _)| d)
        .unwrap_or(TechDomain::General)
}

fn count_matches(text: &str, keywords: &[&str]) -> usize {
    keywords.iter().filter(|kw| text.contains(*kw)).count()
}

/// 领域感知的创造性分析
pub fn analyze_inventiveness_domain_aware(
    base_result: &AnalysisResult,
    ctx: &DomainAwareContext,
) -> AnalysisResult {
    let mut applied = base_result.applied_rules.clone();
    let mut bonus = 0.0_f64;
    let mut bonus_count = 0usize;

    match ctx.tech_domain {
        TechDomain::Chemical => {
            if let Some(params) = &ctx.chemical_params {
                if params.embodiment_count >= 3 {
                    applied.push(AppliedRule {
                        rule_name: "chemical_embodiment_sufficiency".into(),
                        conclusion: "化学领域实施例充分".into(),
                        applies: true,
                        score: 0.8,
                    });
                    bonus += 0.8;
                    bonus_count += 1;
                }
                if params.has_unexpected_effect {
                    applied.push(AppliedRule {
                        rule_name: "chemical_unexpected_effect".into(),
                        conclusion: "存在预料不到的技术效果".into(),
                        applies: true,
                        score: 0.95,
                    });
                    bonus += 0.95;
                    bonus_count += 1;
                }
                if params.has_comparative_data {
                    applied.push(AppliedRule {
                        rule_name: "chemical_comparative_data".into(),
                        conclusion: "提供了对比实验数据".into(),
                        applies: true,
                        score: 0.85,
                    });
                    bonus += 0.85;
                    bonus_count += 1;
                }
            }
        }
        TechDomain::Software => {
            if let Some(params) = &ctx.software_params {
                if params.has_technical_effect && params.solves_technical_problem {
                    applied.push(AppliedRule {
                        rule_name: "software_technical_nature".into(),
                        conclusion: "解决了技术问题并产生了技术效果".into(),
                        applies: true,
                        score: 0.8,
                    });
                    bonus += 0.8;
                    bonus_count += 1;
                }
                if params.hardware_involved {
                    applied.push(AppliedRule {
                        rule_name: "software_hardware_coupling".into(),
                        conclusion: "涉及硬件协同，增强技术性".into(),
                        applies: true,
                        score: 0.75,
                    });
                    bonus += 0.75;
                    bonus_count += 1;
                }
            }
        }
        TechDomain::Mechanical
        | TechDomain::Electrical
        | TechDomain::Biotechnology
        | TechDomain::Communication
        | TechDomain::General => {}
    }

    let all_scores: Vec<f64> = applied.iter().map(|r| r.score).collect();
    let total = all_scores.iter().sum::<f64>() + bonus;
    let total_count = all_scores.len() + bonus_count;
    let avg = if total_count > 0 {
        total / total_count as f64
    } else {
        base_result.net_score
    };

    AnalysisResult {
        conclusion: if avg > 0.5 {
            "具备创造性".into()
        } else {
            "可能缺乏创造性".into()
        },
        net_score: avg,
        confidence: base_result.confidence,
        applied_rules: applied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_detect_chemical_domain() {
        let desc = "本发明涉及一种新型聚合物催化剂，通过特定配比合成化合物，反应浓度可控";
        assert_eq!(detect_tech_domain(desc), TechDomain::Chemical);
    }

    #[test]
    fn test_detect_software_domain() {
        let desc = "本发明提出一种基于神经网络的编码算法，通过训练数据集优化解码模型";
        assert_eq!(detect_tech_domain(desc), TechDomain::Software);
    }

    #[test]
    fn test_detect_general_domain() {
        let desc = "本发明涉及一种日常用品的设计改进";
        assert_eq!(detect_tech_domain(desc), TechDomain::General);
    }

    #[test]
    fn test_domain_aware_inventiveness_chemical() {
        let base = AnalysisResult {
            conclusion: "初步分析".into(),
            net_score: 0.5,
            confidence: 0.7,
            applied_rules: vec![],
        };

        let ctx = DomainAwareContext {
            base: CaseContext::default(),
            tech_domain: TechDomain::Chemical,
            chemical_params: Some(ChemicalAnalysisParams {
                parameter_ranges: vec![("温度".into(), "50℃".into(), "100℃".into())],
                embodiment_count: 5,
                has_comparative_data: true,
                has_unexpected_effect: true,
            }),
            software_params: None,
        };

        let result = analyze_inventiveness_domain_aware(&base, &ctx);
        assert_eq!(result.conclusion, "具备创造性");
        assert!(result.net_score > 0.5);
        assert_eq!(result.applied_rules.len(), 3);
    }
}
