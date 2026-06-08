//! Google Patents 数据源交互模块。
//!
//! 封装从 Google Patents 获取专利数据的底层逻辑，包括 HTML 抓取、
//! 重试机制、HTML 解析等。提供 [`fetch_google_patents`] 和 [`download_patent`] 两个核心入口。
//!
//! 内置熔断器保护：连续3次失败后熔断，120秒后半开探测。

use serde::Deserialize;
use serde::Serialize;

use regex::Regex;
use std::sync::OnceLock;

use codex_patent_core::http::{CircuitBreaker, SharedHttpClient};

const MAX_RETRIES: u32 = 2;
const BACKOFF_BASE_SECS: u64 = 2;
const BACKOFF_MAX_SECS: u64 = 60;

static GOOGLE_PATENTS_CB: OnceLock<CircuitBreaker> = OnceLock::new();
static SHARED_CLIENT: OnceLock<SharedHttpClient> = OnceLock::new();

fn get_cb() -> &'static CircuitBreaker {
    GOOGLE_PATENTS_CB.get_or_init(CircuitBreaker::new)
}

fn get_client() -> &'static reqwest::Client {
    SHARED_CLIENT.get_or_init(SharedHttpClient::new).client()
}

fn backoff_delay_secs(attempt: u32) -> u64 {
    let raw = BACKOFF_BASE_SECS * 2u64.pow(attempt);
    raw.min(BACKOFF_MAX_SECS)
}

#[derive(Debug, Deserialize)]
pub struct GooglePatentsInput {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub patent_number: Option<String>,
}

pub fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Clone)]
pub struct PatentResult {
    pub patent_number: String,
    pub title: String,
    pub abstract_text: String,
    pub assignee: Option<String>,
    pub filing_date: Option<String>,
    pub publication_date: Option<String>,
}

pub async fn fetch_google_patents(input: GooglePatentsInput) -> Result<Vec<PatentResult>, String> {
    let cb = get_cb();

    if !cb.allow_request() {
        return Err("Google Patents circuit breaker open (failures >= 3), retry after 120s".into());
    }

    let url = format!(
        "https://patents.google.com/?q={}&num={}",
        urlencoding(&input.query),
        input.limit
    );

    let client = get_client();
    let body = fetch_with_retry(client, &url).await?;

    cb.record_success();
    parse_patent_results(&body, input.limit)
}

async fn fetch_with_retry(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let cb = get_cb();
    let mut attempt = 0;
    loop {
        let resp = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 BCIP-Patent-Search/1.0")
            .send()
            .await
            .map_err(|e| {
                cb.record_failure();
                codex_patent_core::error::retryable_err(format!("HTTP error: {e}"))
            })?;

        if resp.status().is_success() {
            return resp.text().await.map_err(|e| {
                cb.record_failure();
                codex_patent_core::error::retryable_err(format!("read body: {e}"))
            });
        }

        let status = resp.status().as_u16();
        let is_retryable = status == 429 || status >= 500;
        if is_retryable && attempt < MAX_RETRIES {
            cb.record_failure();
            let delay = std::time::Duration::from_secs(backoff_delay_secs(attempt));
            tokio::time::sleep(delay).await;
            attempt += 1;
            continue;
        }

        cb.record_failure();
        let suggestion = match status {
            429 => "请求频率超限，请稍后重试或使用 WebSearch 替代".to_string(),
            403 => "访问被拒绝，请尝试使用 PatentSearch 替代".to_string(),
            404 => "资源不存在，请检查查询参数".to_string(),
            _ => "请检查网络连接或使用其他检索工具".to_string(),
        };
        let msg = resp.text().await.unwrap_or_default();
        let err = serde_json::json!({
            "error": true,
            "tool": "GooglePatentsFetch",
            "status": status,
            "message": format!("HTTP {status}: {}", &msg[..msg.len().min(200)]),
            "suggestion": suggestion,
            "retry_possible": is_retryable,
        });
        return Err(err.to_string());
    }
}

fn parse_patent_results(html: &str, limit: usize) -> Result<Vec<PatentResult>, String> {
    static PN_RE: OnceLock<Regex> = OnceLock::new();
    static BLOCK_RE: OnceLock<Regex> = OnceLock::new();
    let pn_re = PN_RE.get_or_init(|| {
        Regex::new(r#"(CN|US|EP|WO|JP|KR|DE|GB|FR)\d{6,12}[A-Z]?\d?"#)
            .expect("regex: PN_RE 静态字符串")
    });
    let block_re = BLOCK_RE.get_or_init(|| {
        Regex::new(r"(?is)<search-result[^>]*>(.*?)</search-result>")
            .expect("regex: BLOCK_RE 静态字符串")
    });

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in pn_re.captures_iter(html) {
        if results.len() >= limit {
            break;
        }
        let m = cap.get(0).expect("capture 组 0 应始终存在（完整匹配）");
        let pn = m.as_str().to_string();
        if !seen.insert(pn.clone()) {
            continue;
        }

        // Find the search-result block containing this patent number
        let block = block_re
            .captures_iter(html)
            .find(|b| {
                let full = b.get(0).expect("capture 组 0 应始终存在（完整匹配）");
                full.start() <= m.start() && m.end() <= full.end()
            })
            .map(|b| b.get(1).expect("块内容捕获组应存在").as_str())
            .unwrap_or(html);

        let title = extract_title(block);
        let abstract_text = extract_abstract(block);
        let assignee = extract_assignee(block);
        let publication_date = extract_date(block);

        results.push(PatentResult {
            patent_number: pn,
            title,
            abstract_text,
            assignee,
            filing_date: None,
            publication_date,
        });
    }
    Ok(results)
}

fn extract_title(block: &str) -> String {
    static H3_RE: OnceLock<Regex> = OnceLock::new();
    static TITLE_RE: OnceLock<Regex> = OnceLock::new();
    let h3_re = H3_RE.get_or_init(|| {
        Regex::new(r"(?i)<h3[^>]*>.*?<a[^>]*>(.*?)</a>").expect("regex: H3_RE 静态字符串")
    });
    if let Some(cap) = h3_re.captures(block) {
        return clean_html_text(cap.get(1).expect("h3 标题捕获组应存在").as_str());
    }
    let re = TITLE_RE.get_or_init(|| {
        Regex::new(r#"(?is)class="[^"]*title[^"]*"[^>]*>(.*?)<"#)
            .expect("regex: TITLE_RE 静态字符串")
    });
    if let Some(cap) = re.captures(block) {
        return clean_html_text(cap.get(1).expect("title 类内容捕获组应存在").as_str());
    }
    String::new()
}

fn extract_abstract(segment: &str) -> String {
    static ABSTRACT_RE: OnceLock<Regex> = OnceLock::new();
    static SNIPPET_RE: OnceLock<Regex> = OnceLock::new();
    let re = ABSTRACT_RE.get_or_init(|| {
        Regex::new(r#"(?is)class="[^"]*abstract[^"]*"[^>]*>(.*?)<"#)
            .expect("regex: ABSTRACT_RE 静态字符串")
    });
    if let Some(cap) = re.captures(segment) {
        return clean_html_text(cap.get(1).expect("abstract 类内容捕获组应存在").as_str());
    }
    let re2 = SNIPPET_RE.get_or_init(|| {
        Regex::new(r#"(?is)class="[^"]*snippet[^"]*"[^>]*>(.*?)<"#)
            .expect("regex: SNIPPET_RE 静态字符串")
    });
    if let Some(cap) = re2.captures(segment) {
        return clean_html_text(cap.get(1).expect("snippet 类内容捕获组应存在").as_str());
    }
    String::new()
}

fn extract_assignee(segment: &str) -> Option<String> {
    static ASSIGNEE_RE: OnceLock<Regex> = OnceLock::new();
    let re = ASSIGNEE_RE.get_or_init(|| {
        Regex::new(r#"(?is)class="[^"]*assignee[^"]*"[^>]*>(.*?)<"#)
            .expect("regex: ASSIGNEE_RE 静态字符串")
    });
    re.captures(segment)
        .map(|cap| clean_html_text(cap.get(1).expect("assignee 类内容捕获组应存在").as_str()))
        .filter(|s| !s.is_empty())
}

fn extract_date(segment: &str) -> Option<String> {
    static DATE_RE: OnceLock<Regex> = OnceLock::new();
    let re = DATE_RE.get_or_init(|| {
        Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b").expect("regex: DATE_RE 静态字符串")
    });
    re.captures(segment).map(|cap| {
        format!(
            "{}-{}-{}",
            cap.get(1).expect("日期-年捕获组应存在").as_str(),
            cap.get(2).expect("日期-月捕获组应存在").as_str(),
            cap.get(3).expect("日期-日捕获组应存在").as_str()
        )
    })
}

fn clean_html_text(raw: &str) -> String {
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    let stripped = TAG_RE
        .get_or_init(|| Regex::new(r"<[^>]*>").expect("regex: TAG_RE 静态字符串"))
        .replace_all(raw, "");
    let s = stripped
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    s.trim().to_string()
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+").replace('&', "%26").replace('=', "%3D")
}

/// 专利 PDF 下载输入参数。
#[derive(Debug, Deserialize)]
pub struct PatentDownloadInput {
    /// 要下载的专利号码。
    pub patent_number: String,
}

/// 从 Google Patents 下载专利 PDF 到本地。
///
/// 下载目录由环境变量 `BCIP_DOWNLOAD_DIR` 指定，默认使用系统临时目录。
/// 文件命名为 `{patent_number}.pdf`。
pub async fn download_patent(input: PatentDownloadInput) -> Result<String, String> {
    let url = format!(
        "https://patentimages.storage.googleapis.com/pdfs/{}.pdf",
        input.patent_number
    );
    let client = get_client();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| codex_patent_core::error::retryable_err(format!("HTTP: {e}")))?;
    if !resp.status().is_success() {
        return Err(codex_patent_core::error::retryable_err(format!(
            "PDF not found for {}",
            input.patent_number
        )));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| codex_patent_core::error::retryable_err(format!("read: {e}")))?;
    let dir = std::env::var("BCIP_DOWNLOAD_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());
    let _ = std::fs::create_dir_all(&dir);
    let filename = format!("{}/{}.pdf", dir, input.patent_number);
    tokio::fs::write(&filename, &bytes)
        .await
        .map_err(|e| format!("write: {e}"))?;
    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let results = parse_patent_results("", 10).expect("empty parsing should succeed");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_with_sample_html() {
        let html = r#"
        <search-result>
          <h3><a href="/patent/CN109876543A">一种基于深度学习的图像识别方法</a></h3>
          <div class="abstract">本发明公开了一种基于深度学习的图像识别方法，包括以下步骤...</div>
          <span class="assignee">华为技术有限公司</span>
          <time>2021-06-15</time>
        </search-result>
        <search-result>
          <h3><a href="/patent/US2021009876B2">Deep learning based image recognition</a></h3>
          <div class="abstract snippet">A method for image recognition based on deep learning...</div>
          2021-03-20
        </search-result>
        "#;

        let results = parse_patent_results(html, 10).expect("test parsing should succeed");
        assert_eq!(results.len(), 2);

        let cn = &results[0];
        assert_eq!(cn.patent_number, "CN109876543A");
        assert_eq!(cn.title, "一种基于深度学习的图像识别方法");
        assert!(cn.abstract_text.contains("深度学习"));
        assert_eq!(cn.assignee.as_deref(), Some("华为技术有限公司"));
        assert_eq!(cn.publication_date.as_deref(), Some("2021-06-15"));

        let us = &results[1];
        assert_eq!(us.patent_number, "US2021009876B2");
        assert_eq!(us.title, "Deep learning based image recognition");
        assert!(us.abstract_text.contains("image recognition"));
    }

    #[test]
    fn test_parse_fallback_to_patent_number_only() {
        let html = r#"some text CN123456789A more text without tags"#;
        let results = parse_patent_results(html, 10).expect("test parsing should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].patent_number, "CN123456789A");
        assert!(results[0].title.is_empty());
        assert!(results[0].abstract_text.is_empty());
    }

    #[test]
    fn test_clean_html_text() {
        assert_eq!(clean_html_text("<b>hello</b> &amp; world"), "hello & world");
        assert_eq!(clean_html_text("  trim  "), "trim");
    }

    #[test]
    fn test_dedup_within_parse() {
        let html = "CN111111111A CN111111111A CN222222222A";
        let results = parse_patent_results(html, 10).expect("test parsing should succeed");
        assert_eq!(results.len(), 2);
    }
}
