use std::path::PathBuf;

const DEFAULT_ASSETS_DIR: &str = "../codex-patent-assets";

/// 解析知识库资源路径，优先使用环境变量 BCIP_ASSETS_DIR
fn asset_path(filename: &str) -> String {
    let dir = std::env::var("BCIP_ASSETS_DIR").unwrap_or_else(|_| DEFAULT_ASSETS_DIR.into());
    PathBuf::from(dir)
        .join(filename)
        .to_string_lossy()
        .to_string()
}

/// patent_kg.db 路径
pub fn kg_db_path() -> String {
    asset_path("patent_kg.db")
}

/// laws.db 路径
pub fn law_db_path() -> String {
    asset_path("laws.db")
}

/// card-index.json 路径
pub fn card_index_path() -> String {
    asset_path("card-index.json")
}

/// .yunpat-semantic-index.sqlite 路径
pub fn semantic_index_path() -> String {
    asset_path(".yunpat-semantic-index.sqlite")
}

/// eval_queries.json 路径
pub fn eval_queries_path() -> String {
    asset_path("eval_queries.json")
}

/// 知识库根目录
pub fn kb_root() -> String {
    std::env::var("BCIP_ASSETS_DIR").unwrap_or_else(|_| DEFAULT_ASSETS_DIR.into())
}
