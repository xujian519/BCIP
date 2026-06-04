//! 知识库资源路径解析。
//!
//! 统一管理 `patent_kg.db`、`laws.db`、`card-index.json` 等资源文件的路径，
//! 优先使用 `BCIP_ASSETS_DIR` 环境变量，默认回退到 `../codex-patent-assets`。

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

/// MLX Embedding 服务 URL
pub fn mlx_url() -> String {
    std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8009".into())
}

/// MLX Embedding 服务 API Key
pub fn mlx_api_key() -> Option<String> {
    std::env::var("BCIP_MLX_API_KEY").ok()
}

/// MLX Embedding 模型名
pub fn mlx_model() -> String {
    std::env::var("BCIP_MLX_MODEL").unwrap_or_else(|_| "bge-m3-mlx-8bit".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kg_db_path_default() {
        unsafe {
            std::env::remove_var("BCIP_ASSETS_DIR");
        }
        let path = kg_db_path();
        assert!(path.contains("patent_kg.db"));
    }

    #[test]
    fn law_db_path_default() {
        unsafe {
            std::env::remove_var("BCIP_ASSETS_DIR");
        }
        let path = law_db_path();
        assert!(path.contains("laws.db"));
    }

    #[test]
    fn card_index_path_default() {
        unsafe {
            std::env::remove_var("BCIP_ASSETS_DIR");
        }
        let path = card_index_path();
        assert!(path.contains("card-index.json"));
    }

    #[test]
    fn kb_root_default() {
        unsafe {
            std::env::remove_var("BCIP_ASSETS_DIR");
        }
        let root = kb_root();
        assert_eq!(root, DEFAULT_ASSETS_DIR);
    }

    #[test]
    fn mlx_url_default() {
        unsafe {
            std::env::remove_var("BCIP_MLX_URL");
        }
        let url = mlx_url();
        assert_eq!(url, "http://localhost:8009");
    }

    #[test]
    fn mlx_model_default() {
        unsafe {
            std::env::remove_var("BCIP_MLX_MODEL");
        }
        let model = mlx_model();
        assert_eq!(model, "bge-m3-mlx-8bit");
    }
}
