use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GooglePatentsInput {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub patent_number: Option<String>,
}

pub fn default_limit() -> usize { 10 }

#[derive(Debug, Serialize)]
pub struct PatentResult {
    pub patent_number: String,
    pub title: String,
    pub abstract_text: String,
    pub assignee: Option<String>,
    pub filing_date: Option<String>,
    pub publication_date: Option<String>,
}

pub async fn fetch_google_patents(input: GooglePatentsInput) -> Result<Vec<PatentResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://patents.google.com/?q={}&num={}", 
        urlencoding(&input.query), input.limit);
    
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 BCIP-Patent-Search/1.0")
        .send().await.map_err(|e| format!("HTTP error: {e}"))?;
    let body = resp.text().await.map_err(|e| format!("read body: {e}"))?;
    
    parse_patent_results(&body, input.limit)
}

fn parse_patent_results(html: &str, limit: usize) -> Result<Vec<PatentResult>, String> {
    let mut results = Vec::new();
    let re = regex::Regex::new(r#"(CN|US|EP|WO|JP|KR|DE|GB|FR)\d{6,12}[A-Z]?\d?"#).unwrap();
    let mut seen = std::collections::HashSet::new();
    
    for cap in re.find_iter(html) {
        if results.len() >= limit { break; }
        let pn = cap.as_str().to_string();
        if seen.insert(pn.clone()) {
            results.push(PatentResult {
                patent_number: pn,
                title: String::new(),
                abstract_text: String::new(),
                assignee: None,
                filing_date: None,
                publication_date: None,
            });
        }
    }
    Ok(results)
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "+").replace('&', "%26").replace('=', "%3D")
}

#[derive(Debug, Deserialize)]
pub struct PatentDownloadInput {
    pub patent_number: String,
}

pub async fn download_patent(input: PatentDownloadInput) -> Result<String, String> {
    let url = format!("https://patentimages.storage.googleapis.com/pdfs/{}.pdf", input.patent_number);
    let client = reqwest::Client::new();
    let resp = client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send().await.map_err(|e| format!("HTTP: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("PDF not found for {}", input.patent_number));
    }
    let bytes = resp.bytes().await.map_err(|e| format!("read: {e}"))?;
    let filename = format!("{}.pdf", input.patent_number);
    std::fs::write(&filename, &bytes).map_err(|e| format!("write: {e}"))?;
    Ok(filename)
}