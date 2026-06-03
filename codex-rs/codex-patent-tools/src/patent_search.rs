//! 专利检索核心实现。
//!
//! 提供关键词检索、同义词展开、多轮迭代检索（iterative search）等核心功能。
//! 底层封装 Google Patents 数据源，上层提供结构化检索接口。

use crate::google_patents::GooglePatentsInput;
use crate::google_patents::fetch_google_patents;
use codex_patent_knowledge::synonym::SynonymDict;
use futures::stream;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::sync::OnceLock;

/// 专利检索输入参数。
#[derive(Debug, Deserialize)]
pub struct PatentSearchInput {
    /// 检索关键词。
    pub query: String,
    /// 返回结果数量上限。
    #[serde(default = "crate::common::default_limit")]
    pub limit: usize,
    /// 专利号码（精确匹配）。
    pub patent_number: Option<String>,
    /// 是否自动展开同义词。
    pub use_synonyms: Option<bool>,
}

/// 分阶段查询式构建输入参数。
#[derive(Debug, Deserialize)]
pub struct SearchQueryBuilderInput {
    /// 核心概念词。
    pub concept: String,
    /// 限定的检索字段（如 title/abstract）。
    pub field: Option<String>,
}

/// 多轮迭代检索输入参数。
#[derive(Debug, Deserialize)]
pub struct IterativeSearchInput {
    /// 检索关键词。
    pub query: String,
    /// 迭代轮次（默认 3）。
    pub rounds: Option<usize>,
    /// 每轮返回结果数量上限。
    #[serde(default = "crate::common::default_limit")]
    pub limit: usize,
}
    pub query: String,
    #[serde(default = "crate::common::default_limit")]
    pub limit: usize,
    pub patent_number: Option<String>,
    pub use_synonyms: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryBuilderInput {
    pub concept: String,
    pub field: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IterativeSearchInput {
    pub query: String,
    pub rounds: Option<usize>,
    #[serde(default = "crate::common::default_limit")]
    pub limit: usize,
}

/// 单例 SynonymDict，避免每次查询重建字典。
///
/// `SynonymDict::new()` 构造时分配 Vec 与字符串引用，频繁调用会产生明显
/// 内存压力。同义词字典是不可变静态数据，进程内一份就够。
fn shared_synonym_dict() -> &'static SynonymDict {
    static DICT: OnceLock<SynonymDict> = OnceLock::new();
    DICT.get_or_init(SynonymDict::new)
}

pub async fn patent_search(input: PatentSearchInput) -> Result<serde_json::Value, String> {
    let mut query = input.query.clone();
    if input.use_synonyms.unwrap_or(true) {
        let expanded = shared_synonym_dict().expand(&input.query);
        query = expanded.join(" OR ");
    }
    let google_input = GooglePatentsInput {
        query,
        limit: input.limit,
        patent_number: input.patent_number,
    };
    let results = fetch_google_patents(google_input).await?;
    serde_json::to_value(results).map_err(|e| format!("{e}"))
}

pub async fn search_query_builder(
    input: SearchQueryBuilderInput,
) -> Result<serde_json::Value, String> {
    let exact_terms = shared_synonym_dict().expand(&input.concept);
    let mut variants = Vec::new();
    for term in &exact_terms {
        if let Some(ref field) = input.field {
            variants.push(format!("{field} {term}"));
        } else {
            variants.push(term.to_string());
        }
    }
    let dict = shared_synonym_dict();
    let expanded = dict.expand(&input.concept);
    let stage2 = if expanded.len() > 1 {
        expanded.join(" OR ")
    } else {
        format!(
            "{} OR 相关 OR 近似 OR 类似 OR similar OR related",
            input.concept
        )
    };

    Ok(serde_json::json!({
        "stage1_exact": exact_terms,
        "stage2_semantic": stage2,
        "stage3_variants": variants,
    }))
}

/// 并发上限。Google Patents 对单 IP 的非官方接口有节流，过高会被 429。
const ITERATIVE_SEARCH_CONCURRENCY: usize = 3;

pub async fn iterative_search(input: IterativeSearchInput) -> Result<serde_json::Value, String> {
    let rounds = input.rounds.unwrap_or(3).max(1);
    let dict = shared_synonym_dict();

    // synonym 展开是纯函数，不依赖上一轮的网络结果。先在内存里生成
    // 全部 rounds 个 query 字符串，再并发 fetch，避免串行 await 造成的
    // 网络往返延迟线性叠加。
    let mut queries: Vec<String> = Vec::with_capacity(rounds);
    let mut current = input.query.clone();
    for i in 0..rounds {
        queries.push(current.clone());
        if i + 1 < rounds {
            current = dict.expand(&current).join(" OR ");
        }
    }

    let fetches = queries
        .into_iter()
        .map(|q| {
            fetch_google_patents(GooglePatentsInput {
                query: q,
                limit: input.limit,
                patent_number: None,
            })
        })
        .collect::<Vec<_>>();

    let mut all_results = Vec::new();
    let mut stream = stream::iter(fetches).buffer_unordered(ITERATIVE_SEARCH_CONCURRENCY);
    while let Some(round_result) = stream.next().await {
        match round_result {
            Ok(items) if !items.is_empty() => all_results.extend(items),
            Ok(_) => {} // 空结果不再提前 break：并发已发出，干脆等齐
            Err(err) => return Err(err),
        }
    }
    let mut seen = std::collections::HashSet::new();
    all_results.retain(|r| seen.insert(r.patent_number.clone()));
    serde_json::to_value(all_results).map_err(|e| format!("{e}"))
}
