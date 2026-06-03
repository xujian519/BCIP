use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FormalCheckInput {
    pub claims: Vec<String>,
    pub specification_sections: Option<Vec<String>>,
    pub invention_title: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct QualityAssessInput {
    pub claims: Vec<String>,
    pub specification_word_count: usize,
}

pub struct ReviewTools;

impl ReviewTools {
    pub fn formal_check(input: FormalCheckInput) -> Result<serde_json::Value, String> {
        let mut issues = Vec::new();
        // Claim numbering check
        for (i, claim) in input.claims.iter().enumerate() {
            if !claim.contains("根据权利要求") && !claim.contains(&format!("权利要求{}", i + 1))
            {
                if i == 0 {
                    continue;
                }
                issues.push(format!("权利要求{} 可能缺少正确编号", i + 1));
            }
        }
        // Reference validity check
        use regex::Regex;
        let re = Regex::new(r"根据权利要求(\d+)").unwrap();
        for claim in &input.claims {
            for cap in re.captures_iter(claim) {
                let ref_num: usize = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                if ref_num == 0 || ref_num > input.claims.len() {
                    issues.push(format!("无效引用: 引用不存在的权利要求{}", ref_num));
                }
            }
        }
        // Section completeness (5 mandatory sections per 实施细则第17条)
        if let Some(ref sections) = input.specification_sections {
            for req in &[
                "技术领域",
                "背景技术",
                "发明内容",
                "附图说明",
                "具体实施方式",
            ] {
                if !sections.iter().any(|s| s.contains(req)) {
                    issues.push(format!("缺少必要章节: {}", req));
                }
            }
        }
        // Invention title length check (实施细则第18条: ≤25字)
        if let Some(ref title) = input.invention_title {
            let char_count = title.chars().count();
            if char_count > 25 {
                issues.push(format!("发明名称过长: {}字（不超过25字）", char_count));
            }
            let promo_words = ["最佳", "最优", "最好", "革命性", "最先进", "新型", "新"];
            for word in &promo_words {
                if title.contains(word) {
                    issues.push(format!("发明名称含禁止用词: \"{}\"（细则第18条）", word));
                }
            }
        }
        // Commercial promotion words in claims
        let promo_in_claims = ["最佳", "最优", "最好", "最先进", "革命性"];
        for (i, claim) in input.claims.iter().enumerate() {
            for word in &promo_in_claims {
                if claim.contains(word) {
                    issues.push(format!("权利要求{}含禁止用词: \"{}\"", i + 1, word));
                }
            }
        }
        Ok(serde_json::json!({"passed": issues.is_empty(), "issues": issues}))
    }

    pub fn quality_assess(input: QualityAssessInput) -> Result<serde_json::Value, String> {
        let claims = &input.claims;
        let word_count = input.specification_word_count;

        if claims.is_empty() {
            return Ok(serde_json::json!({
                "overall_score": 0.0,
                "claim_count": 0,
                "word_count": word_count,
                "dimensions": {},
                "suggestions": vec!["缺少权利要求"],
            }));
        }

        // 维度1: 权利要求数量合理性 (0-100)
        let claim_count_score = match claims.len() {
            1..=3 => 70.0,
            4..=10 => 90.0,
            11..=20 => 80.0,
            _ => 60.0,
        };

        // 维度2: 独立/从属比例 (0-100)
        let independent_count = claims
            .iter()
            .filter(|c| !c.contains("根据权利要求"))
            .count();
        let dependent_count = claims.len() - independent_count;
        let dependency_ratio_score = if independent_count == 0 {
            30.0
        } else if independent_count > 3 {
            50.0
        } else {
            let ratio = dependent_count as f64 / independent_count as f64;
            if ratio >= 1.0 && ratio <= 5.0 {
                95.0
            } else if ratio > 5.0 && ratio <= 10.0 {
                75.0
            } else if ratio < 1.0 {
                60.0
            } else {
                50.0
            }
        };

        // 维度3: 说明书充分性 (0-100)
        let word_count_score = if word_count >= 3000 {
            95.0
        } else if word_count >= 1500 {
            80.0
        } else if word_count >= 500 {
            60.0
        } else {
            30.0
        };

        // 维度4: 引用完整性 (0-100)
        let reference_score = {
            let mut score: f64 = 100.0;
            use regex::Regex;
            let re = Regex::new(r"根据权利要求(\d+)").unwrap();
            for (i, claim) in claims.iter().enumerate() {
                if claim.contains("根据权利要求") {
                    for cap in re.captures_iter(claim) {
                        let ref_num: usize = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                        if ref_num == 0 || ref_num > claims.len() || ref_num >= i + 1 {
                            score -= 20.0;
                        }
                    }
                }
            }
            score.max(0.0)
        };

        // 维度5: 权利要求长度合理性 (0-100)
        let length_score = {
            let avg_len: f64 = claims.iter().map(|c| c.chars().count()).sum::<usize>() as f64
                / claims.len() as f64;
            if avg_len >= 30.0 && avg_len <= 300.0 {
                90.0
            } else if avg_len < 30.0 {
                50.0
            } else {
                60.0
            }
        };

        // 加权总分
        let overall: f64 = (claim_count_score * 0.15
            + dependency_ratio_score * 0.25
            + word_count_score * 0.25
            + reference_score * 0.25
            + length_score * 0.10)
            / 100.0;

        // 生成建议
        let mut suggestions = Vec::new();
        if dependency_ratio_score < 70.0 {
            suggestions.push("建议调整独立/从属权利要求比例至1:3~1:5");
        }
        if word_count_score < 70.0 {
            suggestions.push("说明书内容偏短，建议补充至3000字以上");
        }
        if reference_score < 80.0 {
            suggestions.push("部分从属权利要求引用编号有误，请检查引用链");
        }
        if length_score < 70.0 {
            suggestions.push("权利要求长度不均匀，建议优化表述");
        }

        Ok(serde_json::json!({
            "overall_score": (overall * 100.0).round() / 100.0,
            "claim_count": claims.len(),
            "word_count": word_count,
            "independent_claims": independent_count,
            "dependent_claims": dependent_count,
            "dimensions": {
                "claim_count_score": claim_count_score,
                "dependency_ratio_score": dependency_ratio_score,
                "word_count_score": word_count_score,
                "reference_score": reference_score,
                "length_score": length_score,
            },
            "suggestions": suggestions,
        }))
    }
}

pub fn register_review_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("FormalCheck".into(), |input| {
        Box::pin(async move {
            let parsed: FormalCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::formal_check(parsed)
        })
    });
    t.insert("QualityAssess".into(), |input| {
        Box::pin(async move {
            let parsed: QualityAssessInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::quality_assess(parsed)
        })
    });
    t
}
