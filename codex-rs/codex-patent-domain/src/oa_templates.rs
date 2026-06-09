use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RejectionType {
    Novelty,
    Inventiveness,
    Clarity,
    Support,
    SubjectMatter,
    Unity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateSection {
    pub heading: String,
    pub template_text: String,
    pub fill_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OaResponseTemplate {
    pub rejection_type: RejectionType,
    pub sections: Vec<TemplateSection>,
    pub suggestions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Internal placeholders used in raw template strings.
// render_template uses {{field}} for user fill_fields, so we use
// {{__claims__}} / {{__refs__}} as internal markers that get replaced
// in generate_template before the template is returned.
// ---------------------------------------------------------------------------

const CLAIMS_MARKER: &str = "{{__claims__}}";
const REFS_MARKER: &str = "{{__refs__}}";

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Generate an OA response template for the given rejection type.
pub fn generate_template(
    rejection_type: &RejectionType,
    claim_numbers: &[u32],
    prior_art_refs: &[String],
) -> OaResponseTemplate {
    let claims_fmt = format_claims(claim_numbers);
    let refs_fmt = format_refs(prior_art_refs);

    let mut tmpl = match rejection_type {
        RejectionType::Novelty => novelty_template(),
        RejectionType::Inventiveness => inventiveness_template(),
        RejectionType::Clarity => clarity_template(),
        RejectionType::Support => support_template(),
        RejectionType::SubjectMatter => subject_matter_template(),
        RejectionType::Unity => unity_template(),
    };

    // Replace internal markers with actual values.
    for section in &mut tmpl.sections {
        section.template_text = section
            .template_text
            .replace(CLAIMS_MARKER, &claims_fmt)
            .replace(REFS_MARKER, &refs_fmt);
    }

    tmpl
}

/// Render a template into a complete response document, substituting
/// `fill_fields` placeholders with values from `fields`.
///
/// Placeholders in `template_text` use `{{field_name}}` syntax.
/// Unfilled placeholders are left as-is.
pub fn render_template(template: &OaResponseTemplate, fields: &HashMap<String, String>) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(template.sections.len());

    for section in &template.sections {
        let mut text = section.template_text.clone();
        for field in &section.fill_fields {
            if let Some(value) = fields.get(field) {
                let placeholder = format!("{{{{{field}}}}}");
                text = text.replace(&placeholder, value);
            }
        }
        parts.push(format!("{}\n{}", section.heading, text));
    }

    parts.join("\n\n")
}

// ---------------------------------------------------------------------------
// Template builders — use CLAIMS_MARKER / REFS_MARKER for dynamic values,
// and {{field}} for user fill_fields.
// ---------------------------------------------------------------------------

fn novelty_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::Novelty,
        sections: vec![
            TemplateSection {
                heading: "一、事实认定".into(),
                template_text: format!(
                    "审查意见引用{REFS_MARKER}认为权利要求{CLAIMS_MARKER}不具备新颖性。\n\
                     申请人认为，上述对比文件未公开权利要求{CLAIMS_MARKER}的全部技术特征。"
                ),
                fill_fields: vec![],
            },
            TemplateSection {
                heading: "二、区别特征论证".into(),
                template_text: format!(
                    "权利要求{CLAIMS_MARKER}与{REFS_MARKER}相比，至少存在以下区别特征：\n\n\
                     {{{{distinguishing_features}}}}\n\n\
                     上述区别特征在{REFS_MARKER}中既未明确记载，也不能从其公开内容中直接、\
                     唯一地得出。"
                ),
                fill_fields: vec!["distinguishing_features".into()],
            },
            TemplateSection {
                heading: "三、技术效果".into(),
                template_text: format!(
                    "所述区别特征使得本发明能够取得以下技术效果：\n\n\
                     {{{{technical_effects}}}}\n\n\
                     综上，权利要求{{{{claims}}}}相对于{REFS_MARKER}具备新颖性，符合专利法第二十二条第二款的规定。"
                ),
                fill_fields: vec!["technical_effects".into(), "claims".into()],
            },
        ],
        suggestions: vec![
            "逐特征对比，列出权利要求与对比文件的区别".into(),
            "强调区别特征未被对比文件公开或暗示".into(),
            "结合技术效果论证创造性".into(),
        ],
    }
}

fn inventiveness_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::Inventiveness,
        sections: vec![
            TemplateSection {
                heading: "一、最接近现有技术".into(),
                template_text: format!(
                    "确定{REFS_MARKER}作为最接近现有技术。\n\n\
                     {{{{closest_prior_art_analysis}}}}"
                ),
                fill_fields: vec!["closest_prior_art_analysis".into()],
            },
            TemplateSection {
                heading: "二、区别特征与实际解决的技术问题".into(),
                template_text: format!(
                    "权利要求{CLAIMS_MARKER}与{REFS_MARKER}的区别特征为：\n\n\
                     {{{{distinguishing_features}}}}\n\n\
                     基于上述区别特征，本发明实际解决的技术问题是：\n\n\
                     {{{{technical_problem}}}}"
                ),
                fill_fields: vec!["distinguishing_features".into(), "technical_problem".into()],
            },
            TemplateSection {
                heading: "三、技术启示判断".into(),
                template_text: "审查意见认为存在技术启示，但申请人认为：\n\n\
                     {{no_motivation_argument}}\n\n\
                     首先，其他对比文件未给出将上述区别特征应用于最接近现有技术的启示。\n\
                     其次，上述区别特征的结合并非本领域技术人员容易想到的。\n\
                     最后，本发明取得了预料不到的技术效果。"
                    .into(),
                fill_fields: vec!["no_motivation_argument".into()],
            },
            TemplateSection {
                heading: "四、结论".into(),
                template_text: format!(
                    "综上，权利要求{CLAIMS_MARKER}相对于{REFS_MARKER}及本领域的公知常识具备突出的\
                     实质性特点和显著的进步，符合专利法第二十二条第三款关于创造性的规定。"
                ),
                fill_fields: vec![],
            },
        ],
        suggestions: vec![
            "准确认定最接近现有技术，避免扩大其公开内容".into(),
            "确定的实际解决技术问题应基于区别特征客观确定".into(),
            "从多个角度论证不存在技术启示".into(),
            "若有预料不到的效果，重点论述".into(),
        ],
    }
}

fn clarity_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::Clarity,
        sections: vec![
            TemplateSection {
                heading: "一、术语澄清".into(),
                template_text: format!(
                    "关于审查意见指出的权利要求{CLAIMS_MARKER}中术语不清晰的问题，申请人说明如下：\n\n\
                     {{{{term_clarification}}}}\n\n\
                     该术语在说明书第{{{{specification_paragraph}}}}段中有明确定义，\
                     本领域技术人员能够清楚理解其含义。"
                ),
                fill_fields: vec![
                    "term_clarification".into(),
                    "specification_paragraph".into(),
                ],
            },
            TemplateSection {
                heading: "二、修改说明（如适用）".into(),
                template_text: "为消除审查意见中的疑虑，申请人对权利要求进行了如下修改：\n\n\
                     {{modification_description}}\n\n\
                     上述修改来源于说明书第{{modification_source}}段记载的内容，\
                     未超出原说明书和权利要求书记载的范围。"
                    .into(),
                fill_fields: vec![
                    "modification_description".into(),
                    "modification_source".into(),
                ],
            },
        ],
        suggestions: vec![
            "引用说明书原文说明术语含义".into(),
            "如需修改，确保修改内容有说明书支持".into(),
            "避免引入新的不清楚限定".into(),
        ],
    }
}

fn support_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::Support,
        sections: vec![
            TemplateSection {
                heading: "一、补充说明".into(),
                template_text: format!(
                    "关于审查意见指出的权利要求{CLAIMS_MARKER}未得到说明书支持的问题，\
                     申请人补充说明如下：\n\n\
                     {{{{support_explanation}}}}"
                ),
                fill_fields: vec!["support_explanation".into()],
            },
            TemplateSection {
                heading: "二、实施例引用".into(),
                template_text: "说明书提供了充分的实施例来支持权利要求的保护范围：\n\n\
                     {{embodiment_references}}\n\n\
                     具体而言，参见说明书实施例{{embodiment_numbers}}，\
                     其公开了实现权利要求{{claims_ref}}技术方案的完整技术内容。"
                    .into(),
                fill_fields: vec![
                    "embodiment_references".into(),
                    "embodiment_numbers".into(),
                    "claims_ref".into(),
                ],
            },
        ],
        suggestions: vec![
            "引用具体实施例逐条回应".into(),
            "论证权利要求的概括合理，不超出说明书范围".into(),
            "必要时适当缩小保护范围".into(),
        ],
    }
}

fn subject_matter_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::SubjectMatter,
        sections: vec![
            TemplateSection {
                heading: "一、技术三要素论证".into(),
                template_text: format!(
                    "权利要求{CLAIMS_MARKER}的技术方案满足技术三要素的要求：\n\n\
                     （1）技术问题：本发明要解决的技术问题是{{{{technical_problem}}}}。\n\n\
                     （2）技术手段：为解决上述技术问题，本发明采用了{{{{technical_means}}}}\
                     的技术手段。\n\n\
                     （3）技术效果：通过上述技术手段，本发明取得了{{{{technical_effect}}}}\
                     的技术效果。"
                ),
                fill_fields: vec![
                    "technical_problem".into(),
                    "technical_means".into(),
                    "technical_effect".into(),
                ],
            },
            TemplateSection {
                heading: "二、结论".into(),
                template_text: format!(
                    "综上，权利要求{CLAIMS_MARKER}的技术方案利用了自然规律，采用了技术手段，\
                     解决了技术问题并获得了技术效果，属于专利法第二条第二款规定的技术方案，\
                     符合专利法第二十五条的规定。"
                ),
                fill_fields: vec![],
            },
        ],
        suggestions: vec![
            "分别论述技术问题、技术手段和技术效果".into(),
            "避免使用商业方法或智力活动的表述".into(),
            "强调技术手段与自然规律的结合".into(),
        ],
    }
}

fn unity_template() -> OaResponseTemplate {
    OaResponseTemplate {
        rejection_type: RejectionType::Unity,
        sections: vec![
            TemplateSection {
                heading: "一、单一性论述".into(),
                template_text: format!(
                    "权利要求{CLAIMS_MARKER}属于一个总的发明构思，具有技术关联性。具体论述如下：\n\n\
                     {{{{unity_argument}}}}"
                ),
                fill_fields: vec!["unity_argument".into()],
            },
            TemplateSection {
                heading: "二、相同或相应的技术特征".into(),
                template_text: "各项权利要求之间包含相同或相应的技术特征：\n\n\
                     {{common_technical_features}}\n\n\
                     上述技术特征使得各权利要求之间在技术上相互关联，\
                     形成一个总的发明构思。"
                    .into(),
                fill_fields: vec!["common_technical_features".into()],
            },
        ],
        suggestions: vec![
            "明确各项权利要求之间的共同技术特征".into(),
            "论证共同特征对发明整体的技术贡献".into(),
            "必要时删除不符合单一性的权利要求".into(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_claims(claims: &[u32]) -> String {
    match claims.len() {
        0 => "N".into(),
        1 => claims[0].to_string(),
        _ => {
            let last = claims.len() - 1;
            let mut s = claims[..last]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("、");
            s.push_str(&format!("和{}", claims[last]));
            s
        }
    }
}

fn format_refs(refs: &[String]) -> String {
    if refs.is_empty() {
        return "对比文件".into();
    }
    refs.join("、")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn generate_novelty_template_has_three_sections() {
        let tmpl = generate_template(
            &RejectionType::Novelty,
            &[1, 2],
            &["D1".into(), "D2".into()],
        );
        assert_eq!(tmpl.rejection_type, RejectionType::Novelty);
        assert_eq!(tmpl.sections.len(), 3);
        assert!(!tmpl.suggestions.is_empty());
        assert!(tmpl.sections[1].template_text.contains("D1、D2"));
        assert!(tmpl.sections[1].template_text.contains("1和2"));
    }

    #[test]
    fn generate_inventiveness_template_has_four_sections() {
        let tmpl = generate_template(&RejectionType::Inventiveness, &[1], &["D1".into()]);
        assert_eq!(tmpl.sections.len(), 4);
        assert!(tmpl.sections[0].heading.contains("最接近现有技术"));
        assert!(tmpl.sections[1].heading.contains("区别特征"));
        assert!(tmpl.sections[2].heading.contains("技术启示"));
        assert!(tmpl.sections[3].heading.contains("结论"));
    }

    #[test]
    fn generate_clarity_template() {
        let tmpl = generate_template(&RejectionType::Clarity, &[3], &[]);
        assert_eq!(tmpl.sections.len(), 2);
        assert_eq!(tmpl.rejection_type, RejectionType::Clarity);
    }

    #[test]
    fn generate_support_template() {
        let tmpl = generate_template(&RejectionType::Support, &[1, 2, 3], &[]);
        assert_eq!(tmpl.sections.len(), 2);
        assert!(tmpl.sections[1].heading.contains("实施例"));
    }

    #[test]
    fn generate_subject_matter_template() {
        let tmpl = generate_template(&RejectionType::SubjectMatter, &[1], &[]);
        assert_eq!(tmpl.sections.len(), 2);
        let body = &tmpl.sections[0].template_text;
        assert!(body.contains("技术问题"));
        assert!(body.contains("技术手段"));
        assert!(body.contains("技术效果"));
    }

    #[test]
    fn generate_unity_template() {
        let tmpl = generate_template(&RejectionType::Unity, &[1, 5], &[]);
        assert_eq!(tmpl.sections.len(), 2);
        assert!(tmpl.sections[1].heading.contains("相同或相应的技术特征"));
    }

    #[test]
    fn render_template_substitutes_fields() {
        let tmpl = generate_template(&RejectionType::Novelty, &[1], &["D1".into()]);
        let mut fields = HashMap::new();
        fields.insert("distinguishing_features".into(), "特征A：XXX".into());
        fields.insert("technical_effects".into(), "提高了效率".into());
        fields.insert("claims".into(), "1".into());

        let rendered = render_template(&tmpl, &fields);
        assert!(rendered.contains("特征A：XXX"));
        assert!(rendered.contains("提高了效率"));
        assert!(!rendered.contains("{{distinguishing_features}}"));
    }

    #[test]
    fn render_template_leaves_unfilled_placeholders() {
        let tmpl = generate_template(&RejectionType::Novelty, &[1], &["D1".into()]);
        let fields = HashMap::new();
        let rendered = render_template(&tmpl, &fields);
        assert!(rendered.contains("{{distinguishing_features}}"));
    }

    #[test]
    fn format_claims_single() {
        assert_eq!(format_claims(&[5]), "5");
    }

    #[test]
    fn format_claims_multiple() {
        assert_eq!(format_claims(&[1, 2, 3]), "1、2和3");
    }

    #[test]
    fn format_claims_empty() {
        assert_eq!(format_claims(&[]), "N");
    }

    #[test]
    fn format_refs_empty() {
        assert_eq!(format_refs(&[]), "对比文件");
    }

    #[test]
    fn format_refs_multiple() {
        assert_eq!(format_refs(&["D1".into(), "D2".into()]), "D1、D2");
    }

    #[test]
    fn render_inventiveness_template() {
        let tmpl = generate_template(&RejectionType::Inventiveness, &[1], &["D1".into()]);
        let mut fields = HashMap::new();
        fields.insert("closest_prior_art_analysis".into(), "D1公开了A".into());
        fields.insert("distinguishing_features".into(), "特征B".into());
        fields.insert("technical_problem".into(), "如何提高精度".into());
        fields.insert("no_motivation_argument".into(), "D2未给出结合启示".into());

        let rendered = render_template(&tmpl, &fields);
        assert!(rendered.contains("D1公开了A"));
        assert!(rendered.contains("特征B"));
        assert!(rendered.contains("如何提高精度"));
        assert!(rendered.contains("D2未给出结合启示"));
    }

    #[test]
    fn all_rejection_types_have_sections_and_suggestions() {
        let types = [
            RejectionType::Novelty,
            RejectionType::Inventiveness,
            RejectionType::Clarity,
            RejectionType::Support,
            RejectionType::SubjectMatter,
            RejectionType::Unity,
        ];
        for rt in &types {
            let tmpl = generate_template(rt, &[1], &["D1".into()]);
            assert!(tmpl.sections.len() >= 2, "{rt:?} should have >= 2 sections");
            assert!(
                !tmpl.suggestions.is_empty(),
                "{rt:?} should have suggestions"
            );
            for (i, sec) in tmpl.sections.iter().enumerate() {
                assert!(!sec.heading.is_empty(), "section {i} heading empty");
                assert!(
                    !sec.template_text.is_empty(),
                    "section {i} template_text empty"
                );
            }
        }
    }

    #[test]
    fn rejection_type_serde_roundtrip() {
        let rt = RejectionType::Inventiveness;
        let json = serde_json::to_string(&rt).unwrap();
        let rt2: RejectionType = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, rt2);
    }

    #[test]
    fn template_section_serde_roundtrip() {
        let section = TemplateSection {
            heading: "Test".into(),
            template_text: "Body {{field}}".into(),
            fill_fields: vec!["field".into()],
        };
        let json = serde_json::to_string(&section).unwrap();
        let section2: TemplateSection = serde_json::from_str(&json).unwrap();
        assert_eq!(section, section2);
    }

    #[test]
    fn oa_response_template_serde_roundtrip() {
        let tmpl = generate_template(&RejectionType::Novelty, &[1, 2], &["D1".into()]);
        let json = serde_json::to_string(&tmpl).unwrap();
        let tmpl2: OaResponseTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(tmpl.rejection_type, tmpl2.rejection_type);
        assert_eq!(tmpl.sections.len(), tmpl2.sections.len());
    }
}
