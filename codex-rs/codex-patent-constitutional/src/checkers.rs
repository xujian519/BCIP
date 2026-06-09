use crate::model::*;
use crate::types::RuleCheckResult;

pub fn evaluate_rule(
    rule: &ConstitutionalRule,
    _tool_name: &str,
    input_text: &str,
    output_text: Option<&str>,
) -> RuleCheckResult {
    let severity = RuleSeverity::parse(&rule.severity);
    let action = RuleAction::parse(&rule.action);

    let text = output_text.unwrap_or(input_text);

    let (passed, details, confidence) = match &rule.check {
        RuleCheck::KeywordBlocklist {
            keywords,
            context_ban,
            absolute_ban,
            ..
        } => {
            let all_ban: Vec<&String> = keywords
                .iter()
                .chain(context_ban.iter())
                .chain(absolute_ban.iter())
                .collect();
            let mut found = Vec::new();
            for pattern in &all_ban {
                if input_text.contains(pattern.trim_matches('"')) {
                    found.push((*pattern).clone());
                }
            }
            if found.is_empty() {
                (true, vec!["未命中禁用词".into()], 0.95)
            } else {
                (
                    false,
                    found.iter().map(|f| format!("命中禁用词: {}", f)).collect(),
                    0.9,
                )
            }
        }
        RuleCheck::PatternAnalysis {
            pure_software_markers,
            hardware_integration_markers,
            guidance: _,
        } => {
            let pure_hits: Vec<&String> = pure_software_markers
                .iter()
                .filter(|p| input_text.contains(p.trim_matches('"')))
                .collect();
            let hw_hits: Vec<&String> = hardware_integration_markers
                .iter()
                .filter(|p| input_text.contains(p.trim_matches('"')))
                .collect();
            if !pure_hits.is_empty() && hw_hits.is_empty() {
                (false, vec!["纯软件方案，需结合硬件分析".into()], 0.7)
            } else {
                (true, vec!["通过模式分析".into()], 0.85)
            }
        }
        RuleCheck::CategoryDetection {
            categories,
            assessment: _,
        } => {
            let mut matches = Vec::new();
            for (cat_name, cat_def) in categories {
                let cat_hits: Vec<&String> = cat_def
                    .patterns
                    .iter()
                    .filter(|p| input_text.contains(p.trim_matches('"')))
                    .collect();
                if !cat_hits.is_empty() {
                    matches.push(format!("[{}] 命中 {} 个模式", cat_name, cat_hits.len()));
                }
            }
            if matches.is_empty() {
                (true, vec!["未命中排除客体类别".into()], 0.9)
            } else {
                (false, matches, 0.8)
            }
        }
        RuleCheck::StructuralAnalysis {
            requires_all,
            min_confidence,
        } => {
            let mut missing = Vec::new();
            for elem in requires_all {
                let has_elem = elem
                    .patterns
                    .iter()
                    .any(|p| input_text.contains(p.trim_matches('"')));
                if !has_elem {
                    missing.push(elem.element.clone());
                }
            }
            if missing.is_empty() {
                (true, vec!["三要素完整".into()], *min_confidence + 0.2)
            } else {
                (
                    false,
                    missing.iter().map(|m| format!("缺少要素: {}", m)).collect(),
                    *min_confidence,
                )
            }
        }
        RuleCheck::SpecificationAnalysis {
            dimensions,
            assessment: _,
        } => {
            let mut dim_results = Vec::new();
            for dim in dimensions {
                let all_checks_pass = dim
                    .checks
                    .iter()
                    .all(|c| input_text.contains(c.trim_matches('"')));
                if !all_checks_pass {
                    dim_results.push(format!("维度 '{}' 未全部满足", dim.dimension));
                }
            }
            if dim_results.is_empty() {
                (true, vec!["说明书分析维度全部通过".into()], 0.85)
            } else {
                (false, dim_results, 0.7)
            }
        }
        RuleCheck::SectionStructure {
            required_sections,
            forbidden_content: _,
        } => {
            let mut missing_sections = Vec::new();
            for section in required_sections {
                let found = section
                    .patterns
                    .iter()
                    .any(|p| input_text.contains(p.trim_matches('"')));
                if !found {
                    missing_sections.push(section.name.clone());
                }
            }
            if missing_sections.is_empty() {
                (true, vec!["章节结构完整".into()], 0.9)
            } else {
                (
                    false,
                    missing_sections
                        .iter()
                        .map(|s| format!("缺少章节: {}", s))
                        .collect(),
                    0.75,
                )
            }
        }

        // --- Previously fallback variants ---
        RuleCheck::ClaimClarityAnalysis {
            unclear_terms,
            over_broad,
            mixed_categories,
            chained_references,
            assessment: _,
        } => {
            let mut issues = Vec::new();

            // Check for unclear terms in output
            for term in unclear_terms {
                if text.contains(term) {
                    issues.push(format!("发现不清晰术语: '{}'", term));
                }
            }

            // Check for over-broad expressions
            for phrase in over_broad {
                if text.contains(phrase) {
                    issues.push(format!("发现过度宽泛表述: '{}'", phrase));
                }
            }

            // Check mixed categories patterns
            for pat in &mixed_categories.patterns {
                if text.contains(pat) {
                    issues.push(format!("发现混合类别表述: '{}'", pat));
                }
            }

            // Check chained reference rule (warn if text mentions it)
            if !chained_references.rule.is_empty() && !text.contains(&chained_references.rule) {
                issues.push(format!("未遵循引用链规则: '{}'", chained_references.rule));
            }

            if issues.is_empty() {
                (true, vec!["权利要求清晰性检查通过".into()], 0.85)
            } else {
                (false, issues, 0.75)
            }
        }

        RuleCheck::SupportAnalysis {
            methods,
            severity_if_unsupported: _,
        } => {
            let mut missing = Vec::new();
            for method in methods {
                let any_rule_mentioned = method
                    .rules
                    .iter()
                    .any(|r| text.contains(r.trim_matches('"')));
                if !any_rule_mentioned && !text.contains(&method.method) {
                    missing.push(method.method.clone());
                }
            }
            if missing.is_empty() {
                (true, vec!["支撑分析方法覆盖完整".into()], 0.8)
            } else {
                (
                    false,
                    missing
                        .iter()
                        .map(|m| format!("缺少支撑分析方法: {}", m))
                        .collect(),
                    0.7,
                )
            }
        }

        RuleCheck::EssentialFeatureAnalysis {
            principles,
            indicators,
        } => {
            let mut issues = Vec::new();

            // Check that principles are referenced
            for principle in principles {
                if !text.contains(principle) {
                    issues.push(format!("未引用必要特征原则: '{}'", principle));
                }
            }

            // Check "too many" indicators
            let too_many_hits: Vec<&String> = indicators
                .too_many
                .patterns
                .iter()
                .filter(|p| text.contains(p.trim_matches('"')))
                .collect();
            if !too_many_hits.is_empty() {
                issues.push(format!(
                    "可能包含过多特征: 命中 {} 个指标",
                    too_many_hits.len()
                ));
            }

            // Check "too few" indicators
            let too_few_hits: Vec<&String> = indicators
                .too_few
                .patterns
                .iter()
                .filter(|p| text.contains(p.trim_matches('"')))
                .collect();
            if !too_few_hits.is_empty() {
                issues.push(format!("可能特征不足: 命中 {} 个指标", too_few_hits.len()));
            }

            if issues.is_empty() {
                (true, vec!["必要特征分析通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::DependencyValidation { rules } => {
            let mut violations = Vec::new();
            for dep in rules {
                let has_error = if let Some(fmt) = &dep.format {
                    text.contains(&dep.error_pattern) || text.contains(fmt)
                } else {
                    text.contains(&dep.error_pattern)
                };
                if has_error {
                    violations.push(format!("依赖规则违反: {} — {}", dep.rule, dep.description));
                }
            }
            if violations.is_empty() {
                (true, vec!["从属权利要求验证通过".into()], 0.85)
            } else {
                (false, violations, 0.75)
            }
        }

        RuleCheck::NoveltyAnalysis {
            prior_art_scope,
            comparison_principles,
        } => {
            let mut issues = Vec::new();

            // Check that prior art scope areas are discussed
            let scope_covered = prior_art_scope
                .iter()
                .filter(|s| text.contains(s.trim_matches('"')))
                .count();
            if !prior_art_scope.is_empty() && scope_covered == 0 {
                issues.push("未涉及现有技术范围分析".into());
            }

            // Check that comparison principles are applied
            let principles_applied: Vec<&ComparisonPrinciple> = comparison_principles
                .iter()
                .filter(|p| text.contains(&p.principle))
                .collect();
            if principles_applied.is_empty() && !comparison_principles.is_empty() {
                issues.push("未应用对比原则".into());
            }

            if issues.is_empty() {
                (
                    true,
                    vec![format!(
                        "新颖性分析通过 (覆盖 {}/{} 对比原则)",
                        principles_applied.len(),
                        comparison_principles.len()
                    )],
                    0.8,
                )
            } else {
                (false, issues, 0.65)
            }
        }

        RuleCheck::GracePeriodAnalysis { conditions } => {
            let mut issues = Vec::new();
            for cond in conditions {
                let requirements_met = cond
                    .requirements
                    .iter()
                    .filter(|r| text.contains(r.trim_matches('"')))
                    .count();
                if requirements_met < cond.requirements.len() {
                    issues.push(format!(
                        "宽限期条件 '{}' 未完全满足 ({}/{})",
                        cond.condition_type,
                        requirements_met,
                        cond.requirements.len()
                    ));
                }
            }
            if issues.is_empty() {
                (true, vec!["宽限期条件分析通过".into()], 0.8)
            } else {
                (false, issues, 0.65)
            }
        }

        RuleCheck::InventivenessAnalysis {
            method: _,
            steps,
            secondary_indicators,
            standard_lower: _,
        } => {
            let mut issues = Vec::new();

            // Check each inventiveness step is referenced
            for step in steps {
                let step_covered = step
                    .criteria
                    .iter()
                    .any(|c| text.contains(c.trim_matches('"')));
                if !step_covered {
                    issues.push(format!("创造性步骤 {} '{}' 未体现", step.step, step.name));
                }
            }

            // Check for positive secondary indicators
            let positive_hits: Vec<&String> = secondary_indicators
                .positive
                .iter()
                .filter(|p| text.contains(p.trim_matches('"')))
                .collect();
            if !positive_hits.is_empty() {
                issues.push(format!(
                    "发现积极辅助指标 ({}/{}): {}",
                    positive_hits.len(),
                    secondary_indicators.positive.len(),
                    positive_hits
                        .iter()
                        .map(|p| p.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            if issues.is_empty() {
                (true, vec!["创造性分析步骤完整".into()], 0.8)
            } else {
                // Having positive indicators is actually good, so only fail on missing steps
                let step_issues: Vec<_> = issues
                    .iter()
                    .filter(|i| i.contains("未体现"))
                    .cloned()
                    .collect();
                if step_issues.is_empty() {
                    (true, issues, 0.75)
                } else {
                    (false, issues, 0.7)
                }
            }
        }

        RuleCheck::UtilityAnalysis {
            grounds_for_rejection,
        } => {
            let mut warnings = Vec::new();
            for ground in grounds_for_rejection {
                let examples_found: Vec<&String> = ground
                    .examples
                    .iter()
                    .filter(|e| text.contains(e.trim_matches('"')))
                    .collect();
                if !examples_found.is_empty() {
                    warnings.push(format!(
                        "实用性驳回理由 '{}' 可能适用: {}",
                        ground.ground,
                        examples_found
                            .iter()
                            .map(|e| e.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            if warnings.is_empty() {
                (true, vec!["实用性分析通过".into()], 0.85)
            } else {
                (false, warnings, 0.7)
            }
        }

        RuleCheck::UnityAnalysis {
            same_inventive_concept,
            allowed_combinations,
            guidance: _,
        } => {
            let criteria_covered: Vec<&String> = same_inventive_concept
                .criteria
                .iter()
                .filter(|c| text.contains(c.trim_matches('"')))
                .collect();
            let combos_covered: Vec<&String> = allowed_combinations
                .iter()
                .filter(|c| text.contains(c.trim_matches('"')))
                .collect();

            if criteria_covered.is_empty() && combos_covered.is_empty() {
                (false, vec!["未涉及单一性判断准则或允许组合".into()], 0.6)
            } else {
                (
                    true,
                    vec![format!(
                        "单一性分析通过 (准则 {}/{}, 组合 {}/{})",
                        criteria_covered.len(),
                        same_inventive_concept.criteria.len(),
                        combos_covered.len(),
                        allowed_combinations.len()
                    )],
                    0.8,
                )
            }
        }

        RuleCheck::DivisionalRules {
            timing,
            constraints,
        } => {
            let mut issues = Vec::new();
            for t in timing {
                if !text.contains(t) {
                    issues.push(format!("未涉及分案时机要求: '{}'", t));
                }
            }
            for c in constraints {
                if text.contains(c) {
                    issues.push(format!("违反分案约束: '{}'", c));
                }
            }
            if issues.is_empty() {
                (true, vec!["分案申请规则检查通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::AmendmentAnalysis {
            principles,
            permissible,
        } => {
            let mut issues = Vec::new();
            for principle in principles {
                if let Some(forbidden) = &principle.forbidden {
                    for f in forbidden {
                        if text.contains(f) {
                            issues.push(format!(
                                "修改违反原则 '{}': 包含 '{}'",
                                principle.principle, f
                            ));
                        }
                    }
                }
            }
            let perm_covered: Vec<&String> = permissible
                .iter()
                .filter(|p| text.contains(p.trim_matches('"')))
                .collect();
            if perm_covered.is_empty() && !permissible.is_empty() {
                issues.push("未涉及任何允许的修改方式".into());
            }
            if issues.is_empty() {
                (true, vec!["修改分析通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::ScopeComparison { direction } => {
            // Check that the direction of scope comparison is discussed
            if text.contains(direction) {
                (
                    true,
                    vec![format!("范围比较方向 '{}' 已体现", direction)],
                    0.8,
                )
            } else {
                (
                    false,
                    vec![format!("未体现范围比较方向: '{}'", direction)],
                    0.65,
                )
            }
        }

        RuleCheck::TimingAnalysis {
            invention,
            utility,
            design,
        } => {
            let all_timing = invention.iter().chain(utility.iter()).chain(design.iter());
            let covered: Vec<&String> = all_timing
                .filter(|t| text.contains(t.trim_matches('"')))
                .collect();
            let total = invention.len() + utility.len() + design.len();
            if total == 0 {
                (true, vec!["无时限要求".into()], 0.9)
            } else if covered.is_empty() {
                (false, vec!["未涉及任何时限要求".into()], 0.65)
            } else {
                (
                    true,
                    vec![format!("时限分析通过 ({}/{})", covered.len(), total)],
                    0.8,
                )
            }
        }

        RuleCheck::PriorityAnalysis {
            priority_type,
            time_limit,
            requirements,
            constraints,
            special_notes,
        } => {
            let mut issues = Vec::new();

            // Check that priority type is mentioned
            if !text.contains(priority_type) {
                issues.push(format!("未提及优先权类型: '{}'", priority_type));
            }

            // Check time limit constraints
            for val in time_limit.values() {
                if !text.contains(val) {
                    issues.push(format!("未涉及时限: '{}'", val));
                }
            }

            // Check requirements are covered
            let req_covered: Vec<&String> = requirements
                .iter()
                .filter(|r| text.contains(r.trim_matches('"')))
                .collect();
            if !requirements.is_empty() && req_covered.is_empty() {
                issues.push("未涉及优先权要求".into());
            }

            // Check constraints are not violated
            for note in constraints {
                if text.contains(note) {
                    issues.push(format!("触发优先权约束: '{}'", note));
                }
            }

            // Note special notes as informational
            let notes_hit: Vec<&String> = special_notes
                .iter()
                .filter(|n| text.contains(n.trim_matches('"')))
                .collect();

            if issues.is_empty() {
                let mut ok = vec!["优先权分析通过".into()];
                if !notes_hit.is_empty() {
                    ok.push(format!("涉及特别注意事项: {}", notes_hit.len()));
                }
                (true, ok, 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::SameSubjectAnalysis {
            criteria,
            assessment: _,
        } => {
            let criteria_covered: Vec<&String> = criteria
                .iter()
                .filter(|c| text.contains(c.trim_matches('"')))
                .collect();
            if criteria_covered.is_empty() && !criteria.is_empty() {
                (false, vec!["未涉及相同主题判断准则".into()], 0.6)
            } else {
                (
                    true,
                    vec![format!(
                        "相同主题分析通过 (覆盖 {}/{})",
                        criteria_covered.len(),
                        criteria.len()
                    )],
                    0.8,
                )
            }
        }

        RuleCheck::DeadlineAnalysis {
            deadlines,
            consequences,
        } => {
            let mut issues = Vec::new();
            for dl in deadlines {
                if !text.contains(&dl.period) && !text.contains(&dl.scenario) {
                    issues.push(format!("未涉及期限场景: '{}'", dl.scenario));
                }
            }
            // Check consequences are acknowledged
            let cons_acknowledged: Vec<&String> = consequences
                .iter()
                .filter(|c| text.contains(c.trim_matches('"')))
                .collect();
            if cons_acknowledged.is_empty() && !consequences.is_empty() {
                issues.push("未涉及逾期后果说明".into());
            }
            if issues.is_empty() {
                (true, vec!["期限分析通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::OaResponseStrategy {
            oa_type,
            valid_strategies,
            invalid_strategies,
        } => {
            let mut issues = Vec::new();

            // Check OA type is recognized
            if !text.contains(oa_type) {
                issues.push(format!("未识别审查意见类型: '{}'", oa_type));
            }

            // Check no invalid strategies are used
            for inv in invalid_strategies {
                if text.contains(inv) {
                    issues.push(format!("使用了无效策略: '{}'", inv));
                }
            }

            // Check at least one valid strategy is referenced
            let valid_covered: Vec<&StrategyDef> = valid_strategies
                .iter()
                .filter(|s| text.contains(&s.strategy))
                .collect();
            if valid_covered.is_empty() && !valid_strategies.is_empty() {
                issues.push("未引用任何有效答复策略".into());
            }

            if issues.is_empty() {
                (true, vec!["审查意见答复策略检查通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::ReexaminationRules {
            requirements,
            scope,
        } => {
            let mut issues = Vec::new();
            let req_covered: Vec<&String> = requirements
                .iter()
                .filter(|r| text.contains(r.trim_matches('"')))
                .collect();
            if req_covered.is_empty() && !requirements.is_empty() {
                issues.push("未涉及复审要求".into());
            }
            let scope_covered: Vec<&String> = scope
                .iter()
                .filter(|s| text.contains(s.trim_matches('"')))
                .collect();
            if scope_covered.is_empty() && !scope.is_empty() {
                issues.push("未涉及复审范围".into());
            }
            if issues.is_empty() {
                (true, vec!["复审规则检查通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::InvalidationAnalysis {
            grounds,
            restrictions,
        } => {
            let mut issues = Vec::new();
            for ground in grounds {
                if !text.contains(&ground.ground) && !text.contains(&ground.description) {
                    issues.push(format!("未涉及无效理由: '{}'", ground.ground));
                }
            }
            for r in restrictions {
                if text.contains(r) {
                    issues.push(format!("触发无效限制: '{}'", r));
                }
            }
            if issues.is_empty() {
                (true, vec!["无效宣告分析通过".into()], 0.8)
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::InvalidationAmendmentRules { allowed, forbidden } => {
            let mut issues = Vec::new();
            for f in forbidden {
                if text.contains(f) {
                    issues.push(format!("包含无效修改禁止项: '{}'", f));
                }
            }
            if issues.is_empty() {
                let allowed_covered: Vec<&AmendmentMethod> = allowed
                    .iter()
                    .filter(|a| text.contains(&a.method))
                    .collect();
                (
                    true,
                    vec![format!(
                        "无效修改规则通过 (允许方式 {}/{})",
                        allowed_covered.len(),
                        allowed.len()
                    )],
                    0.8,
                )
            } else {
                (false, issues, 0.7)
            }
        }

        RuleCheck::InfringementAnalysis {
            principles,
            defenses,
        } => {
            let principles_covered: Vec<&InfringementPrinciple> = principles
                .iter()
                .filter(|p| text.contains(&p.principle))
                .collect();
            let defenses_covered: Vec<&DefenseDef> = defenses
                .iter()
                .filter(|d| text.contains(&d.defense))
                .collect();

            if principles_covered.is_empty() && !principles.is_empty() {
                (false, vec!["未涉及任何侵权判定原则".into()], 0.65)
            } else {
                let mut details_vec = vec![format!(
                    "侵权分析通过 (原则 {}/{}, 抗辩 {}/{})",
                    principles_covered.len(),
                    principles.len(),
                    defenses_covered.len(),
                    defenses.len()
                )];
                // Informational: note which defenses are available
                if !defenses.is_empty() && defenses_covered.is_empty() {
                    details_vec.push("未涉及抗辩理由 (信息提示)".into());
                }
                (true, details_vec, 0.8)
            }
        }

        RuleCheck::DamagesAnalysis {
            calculation_order,
            punitive,
        } => {
            let methods_covered: Vec<&DamageMethod> = calculation_order
                .iter()
                .filter(|m| text.contains(&m.method))
                .collect();

            // Check if punitive conditions are discussed
            let punitive_discussed = text.contains(&punitive.condition);

            if methods_covered.is_empty() && !calculation_order.is_empty() {
                (false, vec!["未涉及任何损害赔偿计算方式".into()], 0.65)
            } else {
                let mut details_vec = vec![format!(
                    "损害赔偿分析通过 (计算方式 {}/{})",
                    methods_covered.len(),
                    calculation_order.len()
                )];
                if punitive_discussed {
                    details_vec.push(format!(
                        "惩罚性赔偿条件已讨论 (倍数: {})",
                        punitive.multiplier
                    ));
                }
                (true, details_vec, 0.8)
            }
        }
    };

    RuleCheckResult {
        rule_id: rule.id.clone(),
        rule_name: rule.name.clone(),
        severity,
        action,
        legal_basis: rule.legal_basis.clone(),
        passed,
        details,
        confidence,
    }
}
