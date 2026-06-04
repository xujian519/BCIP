//! CNIPA 公布公告数据源
//!
//! 构造 CNIPA 搜索 URL 并解析返回的搜索结果 HTML。
//! 不直接执行浏览器请求，由上层 MCP 或工具调用。

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

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

impl CnipaParser {
    /// 从 HTML 中提取搜索结果
    pub fn parse_search_results(html: &str) -> Vec<CnipaSearchHit> {
        let mut results = Vec::new();

        if let Ok(re) = Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*>([^<]*)</a>"#) {
            for cap in re.captures_iter(html) {
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
                    pub_number: Self::extract_field(html, "公开号"),
                    application_number: Self::extract_field(html, "申请号"),
                    applicant: Self::extract_field(html, "申请人"),
                    ipc: Self::extract_field(html, "IPC"),
                    abstract_text: Self::extract_abstract(html),
                    link,
                });
            }
        }

        results.dedup_by(|a, b| a.pub_number == b.pub_number);
        results
    }

    fn extract_field(html: &str, field_name: &str) -> String {
        let pattern = format!(r#"{}\s*[：:]\s*([^<\n]+)"#, regex::escape(field_name));
        if let Ok(re) = Regex::new(&pattern) {
            re.captures(html)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }

    fn extract_abstract(html: &str) -> String {
        if let Ok(re) = Regex::new(r"摘要.*?[：:]\s*(.*?)(?:主权项|申请日)") {
            re.captures(html)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
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
