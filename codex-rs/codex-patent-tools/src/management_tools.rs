//! 专利案件管理工具集。
//!
//! 提供专利生命周期管理、模板库、商标分析、年费计算、期限追踪等管理功能。

use serde::Deserialize;

/// 专利管理输入参数。
#[derive(Debug, Deserialize)]
pub struct PatentManageInput {
    /// 操作类型（如 "list", "create", "update", "delete"）。
    #[serde(default = "default_pm_action")]
    pub action: String,
    /// 专利 ID。
    pub patent_id: Option<String>,
    /// 关联数据。
    pub data: Option<serde_json::Value>,
}
fn default_pm_action() -> String {
    "list".into()
}

/// 流程图生成输入参数。
#[derive(Debug, Deserialize)]
pub struct ProcessChartInput {
    /// 流程类型（如 "application", "invalidation"）。
    #[serde(default = "default_process_type")]
    pub process_type: String,
}
fn default_process_type() -> String {
    "application".into()
}

/// 商标分析输入参数。
#[derive(Debug, Deserialize)]
pub struct TrademarkAnalysisInput {
    /// 商标名称。
    pub mark: String,
}

/// 年费计算器输入参数。
#[derive(Debug, Deserialize)]
pub struct FeeCalculatorInput {
    /// 专利类型：invention / utility_model / design。
    pub patent_type: String,
    /// 年度（从第 1 年开始）。
    pub year: u32,
    /// 申请人类型：individual / enterprise。
    pub applicant_type: Option<String>,
}

/// 期限追踪输入参数。
#[derive(Debug, Deserialize)]
pub struct DeadlineTrackerInput {
    /// 事件类型：oa_response / annual_fee / reexamination / invalidation。
    pub event_type: String,
    /// 参考日期（ISO 格式 "YYYY-MM-DD"）。
    pub reference_date: String,
    /// OA 轮次（仅 oa_response 使用）。
    pub round: Option<u32>,
}

/// 专利案件管理工具集。
pub struct ManagementTools;

impl ManagementTools {
    /// 专利生命周期管理：列出可用状态和当前操作。
    pub fn patent_manager(input: PatentManageInput) -> Result<serde_json::Value, String> {
        let states = [
            "draft",
            "filed",
            "published",
            "examined",
            "granted",
            "maintained",
            "expired",
        ];
        Ok(
            serde_json::json!({"action": input.action, "valid_states": states, "message": "Patent lifecycle: draft→filed→published→examined→granted→maintained→expired"}),
        )
    }

    /// 获取模板库中的指定模板结构。
    pub fn template_library(template_type: &str) -> Result<serde_json::Value, String> {
        let templates: [(&str, &str, &str); 5] = [
            (
                "oa_response",
                "审查意见答复模板",
                "一、修改说明\n二、意见陈述\n三、结论",
            ),
            (
                "patent_application",
                "专利申请模板",
                "一、技术领域\n二、背景技术\n三、发明内容\n四、附图说明\n五、具体实施方式",
            ),
            (
                "invalidation",
                "无效宣告请求模板",
                "一、请求人信息\n二、专利信息\n三、无效理由\n四、证据清单\n五、详细陈述",
            ),
            (
                "reexamination",
                "复审请求模板",
                "一、复审请求人\n二、驳回决定\n三、复审理由\n四、证据",
            ),
            (
                "examination_opinion",
                "审查意见模板",
                "一、引用对比文件\n二、新颖性分析\n三、创造性分析\n四、其他问题",
            ),
        ];
        let found = templates
            .iter()
            .find(|(id, _, _)| id.contains(template_type))
            .map(|(_, n, c)| (*n, *c))
            .unwrap_or(("通用模板", ""));
        Ok(serde_json::json!({"template_name": found.0, "structure": found.1}))
    }

    /// 对商标名称进行可注册性分析（含显著性分级、禁用标志检查）。
    pub fn trademark_analysis(mark: &str) -> Result<serde_json::Value, String> {
        if mark.trim().is_empty() {
            return Err("商标名称不能为空".to_string());
        }

        let mut score: f64 = 100.0;
        let mut issues: Vec<String> = Vec::new();
        let mut strengths: Vec<String> = Vec::new();

        // 1. 长度检查
        let char_count = mark.chars().count();
        if char_count < 2 {
            score -= 30.0;
            issues.push("商标过短，缺乏显著性".to_string());
        } else if char_count > 20 {
            score -= 10.0;
            issues.push("商标过长，可能影响识别".to_string());
        }

        // 2. 显著性分级判断
        let (distinctiveness, dist_score) = analyze_distinctiveness(mark);
        score *= dist_score / 100.0;
        if dist_score >= 80.0 {
            strengths.push(format!("显著性较强（{}）", distinctiveness));
        } else if dist_score >= 50.0 {
            strengths.push(format!("显著性一般（{}）", distinctiveness));
        } else {
            issues.push(format!("显著性较弱（{}）", distinctiveness));
        }

        // 3. 禁用标志检查（商标法第10条）
        let forbidden_marks = [
            (
                "国家名称",
                &["中国", "中华", "全国", "国家", "国际"] as &[&str],
            ),
            ("国家机关", &["政府", "法院", "检察院", "公安", "军队"]),
            (
                "国际组织",
                &["联合国", "WTO", "WHO", "世界卫生", "世界贸易"],
            ),
            ("红十字", &["红十字", "红新月", "RedCross", "RedCross"]),
            ("官方标志", &["国旗", "国徽", "国歌", "军旗", "军徽"]),
        ];
        for (category, marks) in &forbidden_marks {
            for &m in *marks {
                if mark.contains(m) {
                    score -= 40.0;
                    issues.push(format!("含禁用标志（商标法第10条）: {} - {}", category, m));
                }
            }
        }

        // 4. 缺乏显著性标志检查（商标法第11条）
        let generic_terms = [
            "优质", "最佳", "第一", "超级", "最好", "高级", "精品", "顶级", "通用", "标准", "普通",
            "常规", "基本",
        ];
        for term in &generic_terms {
            if mark.contains(term) {
                score -= 15.0;
                issues.push(format!("含通用描述词汇（商标法第11条）: {}", term));
            }
        }

        // 5. 欺骗性检查（商标法第10条第7项）
        let deceptive_patterns = [
            ("夸大宣传", r"(?:最高|最好|最佳|最强|第一|冠军|王|皇|帝)"),
            ("虚假产地", r"(?:原装|进口|国产|正牌)\w*(?:酒|茶|烟)"),
        ];
        for (name, pattern) in &deceptive_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(mark) {
                    score -= 20.0;
                    issues.push(format!("可能具有欺骗性: {}", name));
                }
            }
        }

        let final_score = score.max(0.0).min(100.0);
        let recommendation = if final_score >= 80.0 {
            "建议申请注册"
        } else if final_score >= 60.0 {
            "可以尝试申请，但存在一定风险"
        } else if final_score >= 40.0 {
            "注册难度较大，建议修改商标"
        } else {
            "不建议申请注册"
        };

        Ok(serde_json::json!({
            "mark": mark,
            "registrability_score": final_score,
            "distinctiveness": distinctiveness,
            "recommendation": recommendation,
            "issues": issues,
            "strengths": strengths,
            "char_count": char_count,
        }))
    }

    /// 生成指定专利流程的 Mermaid 流程图。
    pub fn process_chart(process_type: &str) -> Result<serde_json::Value, String> {
        let chart = match process_type {
            "application" => {
                "graph LR\n  A[发明构思] --> B[撰写申请]\n  B --> C[提交申请]\n  C --> D[初步审查]\n  D --> E[公布]\n  E --> F[实质审查]\n  F --> G[授权]"
            }
            "invalidation" => {
                "graph LR\n  A[发现无效理由] --> B[收集证据]\n  B --> C[撰写请求]\n  C --> D[提交请求]\n  D --> E[口审]\n  E --> F[决定]"
            }
            _ => "graph LR\n  A[开始] --> B[处理]\n  B --> C[完成]",
        };
        Ok(serde_json::json!({"process_type": process_type, "mermaid": chart}))
    }

    /// 计算指定专利类型和年度的年费金额（含费用减缓计算）。
    pub fn fee_calculator(input: FeeCalculatorInput) -> Result<serde_json::Value, String> {
        if input.year == 0 {
            return Err("年度必须大于0".to_string());
        }

        let is_individual = input.applicant_type.as_deref() == Some("individual");

        let (annual_fee, description) = match input.patent_type.as_str() {
            "invention" => {
                let fee = match input.year {
                    1..=3 => 900,
                    4..=6 => 1200,
                    7..=9 => 2000,
                    10..=12 => 4000,
                    13..=15 => 6000,
                    16..=20 => 8000,
                    _ => {
                        return Err(format!(
                            "发明专利年费年度超出范围（1-20），当前: {}",
                            input.year
                        ))
                    }
                };
                (fee, "发明专利年费")
            }
            "utility_model" => {
                let fee = match input.year {
                    1..=3 => 600,
                    4..=5 => 900,
                    6..=8 => 1200,
                    9..=10 => 2000,
                    _ => {
                        return Err(format!(
                            "实用新型年费年度超出范围（1-10），当前: {}",
                            input.year
                        ))
                    }
                };
                (fee, "实用新型专利年费")
            }
            "design" => {
                let fee = match input.year {
                    1..=3 => 600,
                    4..=5 => 900,
                    6..=8 => 1200,
                    9..=10 => 2000,
                    11..=15 => 3000,
                    _ => {
                        return Err(format!(
                            "外观设计年费年度超出范围（1-15），当前: {}",
                            input.year
                        ))
                    }
                };
                (fee, "外观设计年费")
            }
            _ => {
                return Err(format!(
                    "未知专利类型: {}，支持 invention/utility_model/design",
                    input.patent_type
                ))
            }
        };

        // 个人申请可享受费用减缓（85%减免）
        let (final_fee, reduction) = if is_individual {
            let reduced = (annual_fee as f64 * 0.15) as u32;
            (reduced, format!("个人申请享受85%减缓，原费{}元", annual_fee))
        } else {
            (annual_fee, "标准费率".to_string())
        };

        Ok(serde_json::json!({
            "patent_type": input.patent_type,
            "year": input.year,
            "annual_fee": final_fee,
            "original_fee": annual_fee,
            "applicant_type": input.applicant_type.unwrap_or_else(|| "enterprise".into()),
            "description": description,
            "reduction": reduction,
            "currency": "CNY",
        }))
    }

    /// 计算专利事务期限（OA 答复、年费、复审、无效请求等）。
    pub fn deadline_tracker(input: DeadlineTrackerInput) -> Result<serde_json::Value, String> {
        let ref_date = chrono::NaiveDate::parse_from_str(&input.reference_date, "%Y-%m-%d")
            .map_err(|e| format!("日期格式错误（需 YYYY-MM-DD）: {e}"))?;
        let today = chrono::Local::now().date_naive();

        let (deadline, description) = match input.event_type.as_str() {
            "oa_response" => {
                // 第一次审查意见: 4个月; 第二次及以后: 2个月
                let months = if input.round.unwrap_or(1) <= 1 { 4 } else { 2 };
                let dl = ref_date + chrono::Months::new(months);
                (
                    dl,
                    format!(
                        "第{}次审查意见答复期限（{}个月）",
                        input.round.unwrap_or(1),
                        months
                    ),
                )
            }
            "annual_fee" => {
                // 年费缴纳期限: 申请日前1个月，宽限期6个月
                let dl = ref_date - chrono::Months::new(1);
                (dl, "年费缴纳期限（申请日前1个月）".to_string())
            }
            "reexamination" => {
                // 复审请求期限: 收到驳回决定之日起3个月
                let dl = ref_date + chrono::Months::new(3);
                (dl, "复审请求期限（3个月）".to_string())
            }
            "invalidation" => {
                // 无效宣告请求: 授权公告日起任何时间
                let dl = ref_date + chrono::Months::new(6);
                (dl, "无效宣告准备期限（建议6个月内）".to_string())
            }
            _ => {
                return Err(format!(
                    "未知事件类型: {}，支持 oa_response/annual_fee/reexamination/invalidation",
                    input.event_type
                ))
            }
        };

        let days_remaining = (deadline - today).num_days();
        let is_overdue = days_remaining < 0;
        let urgency = if is_overdue {
            "已过期"
        } else if days_remaining <= 7 {
            "紧急"
        } else if days_remaining <= 30 {
            "注意"
        } else {
            "正常"
        };

        Ok(serde_json::json!({
            "event_type": input.event_type,
            "reference_date": input.reference_date,
            "deadline": deadline.to_string(),
            "days_remaining": days_remaining,
            "is_overdue": is_overdue,
            "urgency": urgency,
            "description": description,
        }))
    }
}

fn analyze_distinctiveness(mark: &str) -> (String, f64) {
    // 臆造词：无字典含义的新造词（纯大写英文缩写或含数字的组合）
    let is_coinage = mark
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && mark.len() >= 3;

    // 任意词：与商品无关的常见词（如"苹果"用于电脑）
    let arbitrary_words = [
        "苹果",
        "Apple",
        "小米",
        "Xiaomi",
        "天鹅",
        "天鹅湖",
        "Shell",
        "shell",
    ];
    let is_arbitrary = arbitrary_words.iter().any(|w| mark == *w);

    // 暗示性词：暗示商品特点但需要想象
    let suggestive_patterns = [r"(\w+)科技", r"(\w+)智能", r"Smart", r"Quick", r"Easy"];
    let is_suggestive = suggestive_patterns.iter().any(|p| {
        regex::Regex::new(p)
            .map(|re| re.is_match(mark))
            .unwrap_or(false)
    });

    // 描述性词：直接描述商品特征
    let descriptive_patterns = [
        r"(?:快速|高效|安全|稳定|可靠|节能|环保|耐用|便携|智能)",
        r"(?:Fast|Safe|Quick|Green|Smart|Clean|Easy|Pro|Mini|Super)",
    ];
    let is_descriptive = descriptive_patterns.iter().any(|p| {
        regex::Regex::new(p)
            .map(|re| re.is_match(mark))
            .unwrap_or(false)
    });

    // 通用词：商品通用名称
    let generic_words = ["电脑", "手机", "计算机", "Computer", "Phone", "Software"];
    let is_generic = generic_words.iter().any(|w| mark.contains(w));

    if is_generic {
        ("通用词".into(), 10.0)
    } else if is_descriptive {
        ("描述性词".into(), 30.0)
    } else if is_coinage {
        ("臆造词".into(), 95.0)
    } else if is_arbitrary {
        ("任意词".into(), 90.0)
    } else if is_suggestive {
        ("暗示性词".into(), 65.0)
    } else {
        ("需综合判断".into(), 60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn patent_manager_default_action() {
        let input = PatentManageInput {
            action: "list".into(),
            patent_id: None,
            data: None,
        };
        let result = ManagementTools::patent_manager(input).unwrap();
        assert_eq!(result["action"], "list");
        let states = result["valid_states"].as_array().unwrap();
        assert!(states.len() >= 5);
    }

    #[test]
    fn template_library_oa_response() {
        let result = ManagementTools::template_library("oa_response").unwrap();
        assert_eq!(result["template_name"], "审查意见答复模板");
        assert!(result["structure"].as_str().unwrap().contains("修改说明"));
    }

    #[test]
    fn template_library_unknown() {
        let result = ManagementTools::template_library("nonexistent").unwrap();
        assert_eq!(result["template_name"], "通用模板");
    }

    #[test]
    fn trademark_analysis_empty_mark_error() {
        let result = ManagementTools::trademark_analysis("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能为空"));
    }

    #[test]
    fn trademark_analysis_coinage_high_score() {
        let result = ManagementTools::trademark_analysis("IBM").unwrap();
        let score = result["registrability_score"].as_f64().unwrap();
        assert!(score >= 80.0, "coinage mark should score >= 80, got {score}");
        assert!(result["distinctiveness"].as_str().unwrap().contains("臆造"));
    }

    #[test]
    fn trademark_analysis_forbidden_words() {
        let result = ManagementTools::trademark_analysis("中国XX品牌").unwrap();
        let issues = result["issues"].as_array().unwrap();
        assert!(issues.iter().any(|i| i.as_str().unwrap().contains("禁用标志")));
    }

    #[test]
    fn trademark_analysis_generic_low_score() {
        let result = ManagementTools::trademark_analysis("电脑").unwrap();
        let score = result["registrability_score"].as_f64().unwrap();
        assert!(score < 20.0, "generic word should score very low, got {score}");
    }

    #[test]
    fn fee_calculator_invention_year1() {
        let input = FeeCalculatorInput {
            patent_type: "invention".into(),
            year: 1,
            applicant_type: None,
        };
        let result = ManagementTools::fee_calculator(input).unwrap();
        assert_eq!(result["annual_fee"], 900);
        assert_eq!(result["original_fee"], 900);
    }

    #[test]
    fn fee_calculator_utility_model_year4() {
        let input = FeeCalculatorInput {
            patent_type: "utility_model".into(),
            year: 4,
            applicant_type: None,
        };
        let result = ManagementTools::fee_calculator(input).unwrap();
        assert_eq!(result["annual_fee"], 900);
    }

    #[test]
    fn fee_calculator_invention_individual_85_percent_reduction() {
        let input = FeeCalculatorInput {
            patent_type: "invention".into(),
            year: 1,
            applicant_type: Some("individual".into()),
        };
        let result = ManagementTools::fee_calculator(input).unwrap();
        assert_eq!(result["annual_fee"], 135);
        assert_eq!(result["original_fee"], 900);
    }

    #[test]
    fn fee_calculator_year0_error() {
        let input = FeeCalculatorInput {
            patent_type: "invention".into(),
            year: 0,
            applicant_type: None,
        };
        assert!(ManagementTools::fee_calculator(input).is_err());
    }

    #[test]
    fn fee_calculator_invalid_type_error() {
        let input = FeeCalculatorInput {
            patent_type: "unknown".into(),
            year: 1,
            applicant_type: None,
        };
        assert!(ManagementTools::fee_calculator(input).is_err());
    }

    #[test]
    fn fee_calculator_invention_out_of_range() {
        let input = FeeCalculatorInput {
            patent_type: "invention".into(),
            year: 25,
            applicant_type: None,
        };
        assert!(ManagementTools::fee_calculator(input).is_err());
    }

    #[test]
    fn deadline_tracker_oa_response_first_round() {
        let input = DeadlineTrackerInput {
            event_type: "oa_response".into(),
            reference_date: "2025-01-01".into(),
            round: Some(1),
        };
        let result = ManagementTools::deadline_tracker(input).unwrap();
        assert!(result["description"].as_str().unwrap().contains("4个月"));
        assert_eq!(result["deadline"], "2025-05-01");
    }

    #[test]
    fn deadline_tracker_oa_response_second_round() {
        let input = DeadlineTrackerInput {
            event_type: "oa_response".into(),
            reference_date: "2025-01-01".into(),
            round: Some(2),
        };
        let result = ManagementTools::deadline_tracker(input).unwrap();
        assert!(result["description"].as_str().unwrap().contains("2个月"));
        assert_eq!(result["deadline"], "2025-03-01");
    }

    #[test]
    fn deadline_tracker_reexamination() {
        let input = DeadlineTrackerInput {
            event_type: "reexamination".into(),
            reference_date: "2025-01-01".into(),
            round: None,
        };
        let result = ManagementTools::deadline_tracker(input).unwrap();
        assert_eq!(result["deadline"], "2025-04-01");
    }

    #[test]
    fn deadline_tracker_invalid_date_error() {
        let input = DeadlineTrackerInput {
            event_type: "oa_response".into(),
            reference_date: "not-a-date".into(),
            round: None,
        };
        assert!(ManagementTools::deadline_tracker(input).is_err());
    }

    #[test]
    fn process_chart_application() {
        let result = ManagementTools::process_chart("application").unwrap();
        assert!(result["mermaid"].as_str().unwrap().contains("graph LR"));
        assert!(result["mermaid"].as_str().unwrap().contains("授权"));
    }
}

pub fn register_management_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("PatentManager".into(), |input| {
        Box::pin(async move {
            let parsed: PatentManageInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ManagementTools::patent_manager(parsed)
        })
    });
    t.insert("ProcessChart".into(), |input| {
        Box::pin(async move {
            let parsed: ProcessChartInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ManagementTools::process_chart(&parsed.process_type)
        })
    });
    t.insert("TrademarkAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: TrademarkAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ManagementTools::trademark_analysis(&parsed.mark)
        })
    });
    t.insert("FeeCalculator".into(), |input| {
        Box::pin(async move {
            let parsed: FeeCalculatorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ManagementTools::fee_calculator(parsed)
        })
    });
    t.insert("DeadlineTracker".into(), |input| {
        Box::pin(async move {
            let parsed: DeadlineTrackerInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ManagementTools::deadline_tracker(parsed)
        })
    });
    t
}
