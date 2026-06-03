use std::path::PathBuf;

const DEFAULT_ASSETS_DIR: &str = "../codex-patent-assets";

/// 解析知识库资产根目录。
///
/// 优先级：
/// 1. `BCIP_ASSETS_DIR` 环境变量（显式覆盖）
/// 2. 相对于可执行文件的打包路径（生产环境）
/// 3. 默认 `../codex-patent-assets`（开发模式）
fn resolve_assets_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("BCIP_ASSETS_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // CLI 包布局: exe 在 bin/ → <pkg>/codex-resources/codex-patent-assets/
            if exe_dir.file_name() == Some(std::ffi::OsStr::new("bin")) {
                if let Some(pkg_root) = exe_dir.parent() {
                    let pkg_assets = pkg_root
                        .join("codex-resources")
                        .join("codex-patent-assets");
                    if pkg_assets.is_dir() {
                        return pkg_assets;
                    }
                }
            }
            // macOS .app 包: exe 在 Contents/MacOS/ → Contents/Resources/codex-patent-assets/
            if exe_dir.file_name() == Some(std::ffi::OsStr::new("MacOS")) {
                if let Some(contents) = exe_dir.parent() {
                    let bundle_assets = contents.join("Resources").join("codex-patent-assets");
                    if bundle_assets.is_dir() {
                        return bundle_assets;
                    }
                }
            }
        }
    }

    PathBuf::from(DEFAULT_ASSETS_DIR)
}

fn asset_path(filename: &str) -> String {
    resolve_assets_dir()
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
    resolve_assets_dir().to_string_lossy().to_string()
}

/// 语义索引是否可用
pub fn semantic_index_available() -> bool {
    PathBuf::from(semantic_index_path()).is_file()
}
