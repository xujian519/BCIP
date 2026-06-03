//! 具体检查函数 — 每个函数返回 `true` 表示检查通过。

use super::regex_cache::get_or_compile_regex;
use super::schema::PatentDocument;
use super::schema::Target;

/// 从文档中提取指定 target + field 对应的文本值。
pub(crate) fn resolve_field<'a>(
    doc: &'a PatentDocument,
    target: Target,
    field: &str,
) -> (Option<&'a str>, String) {
    match target {
        Target::Specification => match field {
            "title" => (doc.title.as_deref(), "specification.title".into()),
            "text" | "body" => (doc.specification.as_deref(), "specification.text".into()),
            _ => (None, format!("specification.{field}")),
        },
        Target::Claims => match field {
            "text" => (None, "claims".into()),
            "claims" => (None, "claims".into()),
            _ => (None, format!("claims.{field}")),
        },
        Target::Abstract => match field {
            "text" => (doc.abstract_text.as_deref(), "abstract.text".into()),
            "title" => (doc.title.as_deref(), "abstract.title".into()),
            _ => (None, format!("abstract.{field}")),
        },
    }
}

/// `required` 检查:字段非空。
pub fn check_required(doc: &PatentDocument, target: Target, field: &str) -> bool {
    let (value, _) = resolve_field(doc, target, field);
    value.is_some_and(|s| !s.trim().is_empty())
}

/// `pattern` 检查:字段匹配正则。
pub fn check_pattern(doc: &PatentDocument, target: Target, field: &str, pattern: &str) -> bool {
    let Some(re) = get_or_compile_regex(pattern) else {
        return false;
    };

    if target == Target::Claims && field == "text" {
        if doc.claims.is_empty() {
            return false;
        }
        return doc.claims.iter().all(|c| re.is_match(c));
    }

    let (value, _) = resolve_field(doc, target, field);
    value.is_some_and(|s| re.is_match(s))
}

/// `min_length` 检查。
pub fn check_min_length(doc: &PatentDocument, target: Target, field: &str, min: usize) -> bool {
    if target == Target::Claims && field == "claims" {
        return doc.claims.len() >= min;
    }

    let (value, _) = resolve_field(doc, target, field);
    value.is_some_and(|s| s.chars().count() >= min)
}

/// `max_length` 检查。
pub fn check_max_length(doc: &PatentDocument, target: Target, field: &str, max: usize) -> bool {
    if target == Target::Claims && field == "claims" {
        return doc.claims.len() <= max;
    }

    let (value, _) = resolve_field(doc, target, field);
    value.is_some_and(|s| s.chars().count() <= max)
}

/// `enum` 检查:字段值在枚举列表中。
pub fn check_enum(doc: &PatentDocument, target: Target, field: &str, values: &[&str]) -> bool {
    let (value, _) = resolve_field(doc, target, field);
    value.is_some_and(|s| values.contains(&s))
}

/// 生成位置描述。
pub fn location_for(target: Target, field: &str) -> String {
    let target_str = match target {
        Target::Specification => "specification",
        Target::Claims => "claims",
        Target::Abstract => "abstract",
    };
    format!("{target_str}.{field}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> PatentDocument {
        PatentDocument {
            title: Some("一种数据处理方法".into()),
            abstract_text: Some("本发明公开了一种数据处理方法。".into()),
            claims: vec![
                "1. 一种数据处理方法,其特征在于包括步骤A。".into(),
                "2. 根据权利要求1所述的方法,其特征在于还包括步骤B。".into(),
            ],
            specification: Some("具体实施方式如下:...".into()),
            drawings: vec!["图1".into()],
        }
    }

    #[test]
    fn check_required_passes_for_present_field() {
        let doc = sample_doc();
        assert!(check_required(&doc, Target::Specification, "title"));
        assert!(check_required(&doc, Target::Specification, "text"));
        assert!(check_required(&doc, Target::Abstract, "text"));
    }

    #[test]
    fn check_required_fails_for_missing_field() {
        let doc = PatentDocument::default();
        assert!(!check_required(&doc, Target::Specification, "title"));
        assert!(!check_required(&doc, Target::Abstract, "text"));
    }

    #[test]
    fn check_pattern_matches_claims() {
        let doc = sample_doc();
        assert!(check_pattern(&doc, Target::Claims, "text", "其特征在于"));
    }

    #[test]
    fn check_pattern_fails_for_non_matching() {
        let doc = sample_doc();
        assert!(!check_pattern(
            &doc,
            Target::Specification,
            "title",
            "^\\d+$"
        ));
    }

    #[test]
    fn check_min_length_passes() {
        let doc = sample_doc();
        assert!(check_min_length(&doc, Target::Specification, "title", 3));
        assert!(check_min_length(&doc, Target::Claims, "claims", 2));
    }

    #[test]
    fn check_min_length_fails() {
        let doc = sample_doc();
        assert!(!check_min_length(&doc, Target::Claims, "claims", 10));
    }

    #[test]
    fn check_max_length_passes() {
        let doc = sample_doc();
        assert!(check_max_length(&doc, Target::Claims, "claims", 5));
    }

    #[test]
    fn check_enum_passes() {
        let doc = PatentDocument {
            title: Some("method".into()),
            ..PatentDocument::default()
        };
        assert!(check_enum(
            &doc,
            Target::Specification,
            "title",
            &["method", "apparatus"]
        ));
    }

    #[test]
    fn check_enum_fails() {
        let doc = PatentDocument {
            title: Some("device".into()),
            ..PatentDocument::default()
        };
        assert!(!check_enum(
            &doc,
            Target::Specification,
            "title",
            &["method", "apparatus"]
        ));
    }

    #[test]
    fn location_for_formats_correctly() {
        assert_eq!(
            location_for(Target::Specification, "title"),
            "specification.title"
        );
        assert_eq!(location_for(Target::Claims, "text"), "claims.text");
        assert_eq!(location_for(Target::Abstract, "text"), "abstract.text");
    }
}
