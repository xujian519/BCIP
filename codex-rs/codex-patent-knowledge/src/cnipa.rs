//! CNIPA 公布公告数据源
//!
//! 构造 CNIPA 搜索 URL 并解析返回的搜索结果 HTML。
//! 不直接执行浏览器请求，由上层 MCP 或工具调用。

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::sync::LazyLock;

static RE_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*>([^<]*)</a>"#).expect("invalid regex: RE_LINK")
});
static RE_ABSTRACT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"摘要.*?[：:]\s*(.*?)(?:主权项|申请日)").expect("invalid regex: RE_ABSTRACT")
});
static RE_FIELD_PUB_NUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"公开号\s*[：:]\s*([^<\n]+)").expect("invalid regex: RE_FIELD_PUB_NUM")
});
static RE_FIELD_APP_NUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"申请号\s*[：:]\s*([^<\n]+)").expect("invalid regex: RE_FIELD_APP_NUM")
});
static RE_FIELD_APPLICANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"申请人\s*[：:]\s*([^<\n]+)").expect("invalid regex: RE_FIELD_APPLICANT")
});
static RE_FIELD_IPC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"IPC\s*[：:]\s*([^<\n]+)").expect("invalid regex: RE_FIELD_IPC"));

/// CNIPA 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnipaSearchHit {
    pub title: String,
    pub pub_number: String,
    pub application_number: String,
    pub applicant: String,
    pub ipc: String,
    pub abstract_text: String,
    pub link: String,
}

/// CNIPA 搜索 URL 构建器
pub struct CnipaSearchBuilder;

impl CnipaSearchBuilder {
    pub const BASE_URL: &str = "https://epub.cnipa.gov.cn/patentoutline.action";

    /// 构建关键词搜索 URL
    pub fn search_by_keyword(keyword: &str) -> String {
        format!(
            "{}?searchStr={}&showType=1&pageSize=10",
            Self::BASE_URL,
            urlencoding(keyword),
        )
    }

    /// 构建申请号搜索 URL
    pub fn search_by_application_number(app_number: &str) -> String {
        format!(
            "{}?applicationNumber={}&showType=1",
            Self::BASE_URL,
            urlencoding(app_number),
        )
    }

    /// 构建公开号搜索 URL
    pub fn search_by_pub_number(pub_number: &str) -> String {
        format!(
            "{}?pubNumber={}&showType=1",
            Self::BASE_URL,
            urlencoding(pub_number),
        )
    }
}

/// 简单的 URL 编码
fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// CNIPA 搜索结果解析器
pub struct CnipaParser;

fn extract_field_cached(html: &str, re: &LazyLock<Regex>) -> String {
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

impl CnipaParser {
    /// 从 HTML 中提取搜索结果
    pub fn parse_search_results(html: &str) -> Vec<CnipaSearchHit> {
        let mut results = Vec::new();

        for cap in RE_LINK.captures_iter(html) {
            let link = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = cap
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() || link.is_empty() {
                continue;
            }

            results.push(CnipaSearchHit {
                title,
                pub_number: extract_field_cached(html, &RE_FIELD_PUB_NUM),
                application_number: extract_field_cached(html, &RE_FIELD_APP_NUM),
                applicant: extract_field_cached(html, &RE_FIELD_APPLICANT),
                ipc: extract_field_cached(html, &RE_FIELD_IPC),
                abstract_text: Self::extract_abstract(html),
                link,
            });
        }

        results.dedup_by(|a, b| a.pub_number == b.pub_number);
        results
    }

    fn extract_abstract(html: &str) -> String {
        RE_ABSTRACT
            .captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_url_contains_keyword() {
        let url = CnipaSearchBuilder::search_by_keyword("人工智能");
        assert!(url.starts_with(CnipaSearchBuilder::BASE_URL));
        assert!(url.contains("searchStr="));
    }

    #[test]
    fn pub_number_url_includes_parameter() {
        let url = CnipaSearchBuilder::search_by_pub_number("CN12345678A");
        assert!(url.contains("CN12345678A"));
    }

    #[test]
    fn url_encoding_handles_chinese() {
        let encoded = urlencoding("AI芯片");
        assert!(!encoded.contains("芯"));
        assert!(encoded.contains("AI"));
    }

    #[test]
    fn empty_html_returns_empty_results() {
        let results = CnipaParser::parse_search_results("");
        assert!(results.is_empty());
    }

    #[test]
    fn parse_minimal_html_extracts_no_hits() {
        let html = "<div><p>无搜索结果</p></div>";
        let results = CnipaParser::parse_search_results(html);
        assert!(results.is_empty());
    }
}
