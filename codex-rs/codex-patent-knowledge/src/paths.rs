//! 知识库资源路径解析。
//!
//! 统一管理 `patent_kg.db`、`laws.db`、`card-index.json` 等资源文件的路径。
//!
//! # 路径解析优先级（从高到低）
//!
//! 1. **`BCIP_ASSETS_DIR`** 环境变量 - 完整的 assets 根目录
//! 2. **逐文件环境变量**（`BCIP_PATENT_KG_PATH`、`BCIP_LAW_DB_PATH` 等）- 覆盖单个文件
//! 3. **已知项目相对路径** - 自动检测 CWD 匹配 `codex-patent-assets/` 或 `codex-rs/codex-patent-assets/`
//! 4. **旧默认值 `../codex-patent-assets`** - 保底，可能与 CWD 不匹配

use std::path::PathBuf;
use std::sync::OnceLock;

const CANDIDATE_DIRS: &[&str] = &[
    "codex-patent-assets",          // CWD = codex-rs/
    "../codex-patent-assets",       // 旧默认值（向后兼容）
    "codex-rs/codex-patent-assets", // CWD = 项目根目录
];

static CACHED_BASE_DIR: OnceLock<String> = OnceLock::new();

/// 尝试多策略解析 assets 根目录（结果缓存）。
fn resolve_base_dir() -> String {
    CACHED_BASE_DIR
        .get_or_init(|| {
            // 优先级 1：环境变量
            if let Ok(dir) = std::env::var("BCIP_ASSETS_DIR") {
                let trimmed = dir.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::debug!("使用 BCIP_ASSETS_DIR: {trimmed}");
                    return trimmed;
                }
            }

            // 优先级 2：尝试已知项目相对路径，检测 patent_kg.db 或 laws.db 是否存在
            let markers = ["patent_kg.db", "laws.db"];
            for candidate in CANDIDATE_DIRS {
                let path = PathBuf::from(candidate);
                if markers.iter().any(|m| path.join(m).exists()) || path.is_dir() {
                    tracing::debug!("自动检测到 assets 目录: {candidate}");
                    return candidate.to_string();
                }
            }

            // 优先级 3：尝试从可执行文件路径推断（桌面端 bundle）
            if let Ok(exe) = std::env::current_exe()
                && let Some(exe_dir) = exe.parent()
            {
                let bundle = exe_dir.join("../Resources/codex-patent-assets");
                if bundle.is_dir() {
                    tracing::debug!("从 bundle 检测到 assets 目录: {}", bundle.display());
                    return bundle.to_string_lossy().into_owned();
                }
            }

            // 优先级 4：保底回溯旧默认值
            let fallback = CANDIDATE_DIRS[1];
            tracing::warn!("未找到 assets 目录，使用默认值回退: {fallback} (可能不存在)");
            fallback.to_string()
        })
        .clone()
}

/// 解析知识库资源路径，优先使用环境变量，再尝试多策略自动检测
fn asset_path(filename: &str) -> String {
    let dir = resolve_base_dir();
    PathBuf::from(dir)
        .join(filename)
        .to_string_lossy()
        .to_string()
}

/// patent_kg.db 路径（可被 `BCIP_PATENT_KG_PATH` 环境变量单独覆盖）
pub fn kg_db_path() -> String {
    match std::env::var("BCIP_PATENT_KG_PATH") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => asset_path("patent_kg.db"),
    }
}

/// laws.db 路径（可被 `BCIP_LAW_DB_PATH` 环境变量单独覆盖）
pub fn law_db_path() -> String {
    match std::env::var("BCIP_LAW_DB_PATH") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => asset_path("laws.db"),
    }
}

/// card-index.json 路径（可被 `BCIP_CARD_INDEX_PATH` 环境变量单独覆盖）
pub fn card_index_path() -> String {
    match std::env::var("BCIP_CARD_INDEX_PATH") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => asset_path("card-index.json"),
    }
}

/// .yunpat-semantic-index.sqlite 路径（可被 `BCIP_SEMANTIC_INDEX_PATH` 环境变量单独覆盖）
pub fn semantic_index_path() -> String {
    match std::env::var("BCIP_SEMANTIC_INDEX_PATH") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => asset_path(".yunpat-semantic-index.sqlite"),
    }
}

/// eval_queries.json 路径
pub fn eval_queries_path() -> String {
    asset_path("eval_queries.json")
}

/// 知识库根目录（同 [`resolve_base_dir`]）
pub fn kb_root() -> String {
    resolve_base_dir()
}

/// MLX Embedding 服务 URL
pub fn mlx_url() -> String {
    std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8766".into())
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
        // 无环境变量时回退到旧默认值 "../codex-patent-assets"
        assert_eq!(root, "../codex-patent-assets");
    }

    #[test]
    fn mlx_url_default() {
        unsafe {
            std::env::remove_var("BCIP_MLX_URL");
        }
        let url = mlx_url();
        assert_eq!(url, "http://localhost:8766");
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
