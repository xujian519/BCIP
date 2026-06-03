use serde::Deserialize;
use serde::Serialize;

const MAX_RETRIES: u32 = 2;

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
    let url = format!(
        "https://patents.google.com/?q={}&num={}",
        urlencoding(&input.query),
        input.limit
    );

    let client = reqwest::Client::new();
    let body = fetch_with_retry(&client, &url).await?;
    parse_patent_results(&body, input.limit)
}

async fn fetch_with_retry(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let mut attempt = 0;
    loop {
        let resp = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 BCIP-Patent-Search/1.0")
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?;

        if resp.status().is_success() {
            return resp.text().await.map_err(|e| format!("read body: {e}"));
        }

        let status = resp.status().as_u16();
        if status == 429 && attempt < MAX_RETRIES {
            let delay = std::time::Duration::from_secs(2u64.pow(attempt));
            tokio::time::sleep(delay).await;
            attempt += 1;
            continue;
        }

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
            "retry_possible": status == 429,
        });
        return Err(err.to_string());
    }
}

fn parse_patent_results(html: &str, limit: usize) -> Result<Vec<PatentResult>, String> {
    let mut results = Vec::new();
    let pn_re = regex::Regex::new(r#"(CN|US|EP|WO|JP|KR|DE|GB|FR)\d{6,12}[A-Z]?\d?"#).unwrap();
    let mut seen = std::collections::HashSet::new();

    // Google Patents search results wrap each hit in a <search-result> or <article>.
    // Split the HTML into segments around patent numbers and extract surrounding context.
    let segments = split_by_patent_numbers(html, &pn_re);

    for seg in &segments {
        if results.len() >= limit {
            break;
        }
        let pn = match pn_re.find(seg) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        if !seen.insert(pn.clone()) {
            continue;
        }

        let title = extract_title(seg);
        let abstract_text = extract_abstract(seg);
        let assignee = extract_assignee(seg);
        let publication_date = extract_date(seg);

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

/// Split HTML into segments, each starting from a patent number occurrence.
fn split_by_patent_numbers<'a>(html: &'a str, re: &regex::Regex) -> Vec<&'a str> {
    let positions: Vec<usize> = re.find_iter(html).map(|m| m.start()).collect();
    if positions.is_empty() {
        return Vec::new();
    }
    let mut segments = Vec::with_capacity(positions.len());
    for (i, &start) in positions.iter().enumerate() {
        let end = positions
            .get(i + 1)
            .copied()
            .unwrap_or(html.len())
            .min(start + 4000); // cap segment size to avoid spanning entire page
        segments.push(&html[start..end]);
    }
    segments
}

fn extract_title(segment: &str) -> String {
    // <h3> ... <a ...>Title</a> ... </h3>
    let re = regex::Regex::new(r"(?is)<h3[^>]*>.*?<a[^>]*>(.*?)</a>.*?</h3>").unwrap();
    if let Some(cap) = re.captures(segment) {
        let raw = cap.get(1).unwrap().as_str();
        return clean_html_text(raw);
    }
    // Fallback: class containing "title"
    let re2 = regex::Regex::new(r#"(?is)class="[^"]*title[^"]*"[^>]*>(.*?)<"#).unwrap();
    if let Some(cap) = re2.captures(segment) {
        return clean_html_text(cap.get(1).unwrap().as_str());
    }
    String::new()
}

fn extract_abstract(segment: &str) -> String {
    // class containing "abstract"
    let re = regex::Regex::new(r#"(?is)class="[^"]*abstract[^"]*"[^>]*>(.*?)<"#).unwrap();
    if let Some(cap) = re.captures(segment) {
        return clean_html_text(cap.get(1).unwrap().as_str());
    }
    // Fallback: class containing "snippet"
    let re2 = regex::Regex::new(r#"(?is)class="[^"]*snippet[^"]*"[^>]*>(.*?)<"#).unwrap();
    if let Some(cap) = re2.captures(segment) {
        return clean_html_text(cap.get(1).unwrap().as_str());
    }
    String::new()
}

fn extract_assignee(segment: &str) -> Option<String> {
    let re = regex::Regex::new(r#"(?is)class="[^"]*assignee[^"]*"[^>]*>(.*?)<"#).unwrap();
    re.captures(segment)
        .map(|cap| clean_html_text(cap.get(1).unwrap().as_str()))
        .filter(|s| !s.is_empty())
}

fn extract_date(segment: &str) -> Option<String> {
    let re = regex::Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b").unwrap();
    re.captures(segment).map(|cap| {
        format!(
            "{}-{}-{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
            cap.get(3).unwrap().as_str()
        )
    })
}

fn clean_html_text(raw: &str) -> String {
    // Strip tags, decode entities, trim whitespace
    let stripped = regex::Regex::new(r"<[^>]*>")
        .unwrap()
        .replace_all(raw, "")
        .to_string();
    let decoded = stripped
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    decoded.trim().to_string()
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+").replace('&', "%26").replace('=', "%3D")
}

#[derive(Debug, Deserialize)]
pub struct PatentDownloadInput {
    pub patent_number: String,
}

pub async fn download_patent(input: PatentDownloadInput) -> Result<String, String> {
    let url = format!(
        "https://patentimages.storage.googleapis.com/pdfs/{}.pdf",
        input.patent_number
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("HTTP: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("PDF not found for {}", input.patent_number));
    }
    let bytes = resp.bytes().await.map_err(|e| format!("read: {e}"))?;
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
        let results = parse_patent_results("", 10).unwrap();
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

        let results = parse_patent_results(html, 10).unwrap();
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
        let results = parse_patent_results(html, 10).unwrap();
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
        let results = parse_patent_results(html, 10).unwrap();
        assert_eq!(results.len(), 2);
    }
}
