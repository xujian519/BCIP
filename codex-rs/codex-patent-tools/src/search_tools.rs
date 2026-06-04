//! 搜索工具注册模块。
//!
//! 统一注册专利检索、批量检索、专利族追踪等工具到全局工具注册表。
//! 工具列表：
//! - `PatentSearch` — 关键词/号码检索
//! - `GooglePatentsFetch` — Google Patents 原始检索
//! - `SearchQueryBuilder` — 分阶段查询式构建
//! - `IterativeSearch` — 多轮迭代检索（同义词展开）
//! - `PatentDownload` — 专利 PDF 下载
//! - `PatentFamilyTracker` — 专利族追踪

use crate::google_patents::*;
use crate::patent_search::*;
use std::collections::HashMap;

/// 注册全部搜索工具到全局注册表。
pub fn register_search_tools() -> HashMap<String, crate::ToolHandler> {
    let mut tools: HashMap<String, crate::ToolHandler> = HashMap::new();

    tools.insert("PatentSearch".to_string(), |input| {
        Box::pin(async {
            let parsed: PatentSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            patent_search(parsed).await
        })
    });

    tools.insert("GooglePatentsFetch".to_string(), |input| {
        Box::pin(async {
            let parsed: GooglePatentsInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let results = fetch_google_patents(parsed).await?;
            serde_json::to_value(results).map_err(|e| format!("{e}"))
        })
    });

    tools.insert("SearchQueryBuilder".to_string(), |input| {
        Box::pin(async {
            let parsed: SearchQueryBuilderInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            search_query_builder(parsed).await
        })
    });

    tools.insert("IterativeSearch".to_string(), |input| {
        Box::pin(async {
            let parsed: IterativeSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            iterative_search(parsed).await
        })
    });

    tools.insert("PatentDownload".to_string(), |input| {
        Box::pin(async {
            let parsed: PatentDownloadInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let filename = download_patent(parsed).await?;
            Ok(serde_json::json!({"downloaded_file": filename}))
        })
    });

    tools.insert("PatentFamilyTracker".to_string(), |input| {
        Box::pin(async move {
            let patent_number = input
                .get("patent_number")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少必填字段: patent_number".to_string())?;
            let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

            let query = format!("priority:{}", patent_number);
            let google_input = GooglePatentsInput {
                query,
                limit,
                patent_number: None,
            };
            let family = fetch_google_patents(google_input).await?;

            let results: Vec<serde_json::Value> = family
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "patent_number": p.patent_number,
                        "title": p.title,
                        "assignee": p.assignee,
                        "publication_date": p.publication_date,
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "patent_number": patent_number,
                "family_members": results,
                "total": results.len(),
            }))
        })
    });

    tools
}
