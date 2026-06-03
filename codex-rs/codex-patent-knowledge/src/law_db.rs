//! 法规数据库访问层
//!
//! 提供对专利法律数据库的只读查询接口，支持名称搜索、内容搜索、分级查询等。
//! 使用进程级缓存避免重复打开 SQLite 连接。

use codex_patent_core::LawCategory;
use codex_patent_core::LawDocument;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::params;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

/// 法规数据库访问器
///
/// 包装只读 SQLite 连接，通过进程级缓存复用数据库句柄。
/// 支持按名称、内容、法律层级等维度检索法律条文。
pub struct LawDatabase {
    // 包装在 Arc<Mutex<…>> 内的目的是允许同一进程内多份工具调用复用同一份
    // 只读 SQLite 连接，避免每次都重新 `open` + `PRAGMA`。
    conn: Arc<Mutex<Connection>>,
}

/// 进程级缓存：`path -> 已打开的 LawDatabase`。同一路径只打开一次。
///
/// `LawDatabase::open()` 在 `codex-patent-tools` 的多个工具中都会被
/// `UnifiedSearch::new()` / `with_vector()` 调用。如果不缓存，每次专利分析
/// 都会触发文件打开 + pragma 设置，累积延迟可观。
static LAW_DB_CACHE: std::sync::OnceLock<
    Mutex<std::collections::HashMap<PathBuf, Arc<Mutex<Connection>>>>,
> = std::sync::OnceLock::new();

fn cache_store() -> &'static Mutex<std::collections::HashMap<PathBuf, Arc<Mutex<Connection>>>> {
    LAW_DB_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

impl LawDatabase {
    /// 打开或复用法规数据库连接
    ///
    /// `path` 为 SQLite 数据库文件路径。同一路径在同一进程内只会打开一次，
    /// 后续调用返回缓存连接。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path_buf = path.as_ref().to_path_buf();
        let conn = {
            let mut store = cache_store()
                .lock()
                .map_err(|e| format!("cache lock: {e}"))?;
            if let Some(existing) = store.get(&path_buf) {
                existing.clone()
            } else {
                let new_conn = Connection::open_with_flags(
                    &path_buf,
                    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                )
                .map_err(|e| format!("failed to open law db: {e}"))?;
                new_conn
                    .execute_batch(
                        "PRAGMA cache_size = -4000;
                         PRAGMA locking_mode = NORMAL;",
                    )
                    .map_err(|e| format!("pragma setup: {e}"))?;
                let arc = Arc::new(Mutex::new(new_conn));
                store.insert(path_buf, arc.clone());
                arc
            }
        };
        Ok(Self { conn })
    }

    /// 按法规名称关键词搜索
    pub fn search_by_name(&self, keyword: &str, limit: usize) -> Result<Vec<LawDocument>, String> {
        let pattern = format!("%{keyword}%");
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law WHERE name LIKE ?1 LIMIT ?2";
        self.query_laws(sql, params![pattern, limit])
    }

    /// 按法规正文内容关键词搜索（也匹配名称）
    pub fn search_by_content(
        &self,
        keyword: &str,
        limit: usize,
    ) -> Result<Vec<LawDocument>, String> {
        let pattern = format!("%{keyword}%");
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law WHERE name LIKE ?1 OR content LIKE ?1 LIMIT ?2";
        self.query_laws(sql, params![pattern, limit])
    }

    /// 按法律层级列出法规（法律、行政法规、部门规章等）
    pub fn list_by_level(&self, level: &str, limit: usize) -> Result<Vec<LawDocument>, String> {
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law WHERE level = ?1 ORDER BY publish DESC LIMIT ?2";
        self.query_laws(sql, params![level, limit])
    }

    /// 列出数据库中所有不同的法律层级
    pub fn list_levels(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("conn lock: {e}"))?;
        let sql = "SELECT DISTINCT level FROM law ORDER BY level";
        let mut stmt = conn.prepare(sql).map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("{e}"))?;
        let levels: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(levels)
    }

    /// 列出所有法规分类
    pub fn list_categories(&self) -> Result<Vec<LawCategory>, String> {
        let conn = self.conn.lock().map_err(|e| format!("conn lock: {e}"))?;
        let sql = "SELECT id, name, folder, isSubFolder, \"group\", \"order\" FROM category ORDER BY \"order\"";
        let mut stmt = conn.prepare(sql).map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(LawCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    folder: row.get(2)?,
                    is_sub_folder: row.get::<_, i32>(3)? != 0,
                    group: row.get(4)?,
                    order: row.get(5)?,
                })
            })
            .map_err(|e| format!("{e}"))?;
        let cats: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(cats)
    }

    /// 返回数据库中法规总数
    pub fn count(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("conn lock: {e}"))?;
        conn.query_row("SELECT COUNT(*) FROM law", [], |row| row.get(0))
            .map_err(|e| format!("{e}"))
    }

    /// 分页列出全部法规
    pub fn list_all(&self, limit: usize, offset: usize) -> Result<Vec<LawDocument>, String> {
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law ORDER BY id LIMIT ?1 OFFSET ?2";
        self.query_laws(sql, params![limit, offset])
    }

    fn query_laws<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<LawDocument>, String> {
        let conn = self.conn.lock().map_err(|e| format!("conn lock: {e}"))?;
        let mut stmt = conn.prepare(sql).map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map(params, |row| {
                Ok(LawDocument {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    name: row.get(2)?,
                    filename: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    publish: row.get::<_, Option<String>>(4)?.unwrap_or_default() != "",
                    expired: row.get::<_, i32>(5)? != 0,
                    category_id: row.get::<_, i64>(6)?.to_string(),
                    subtitle: row.get(7)?,
                    content: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                })
            })
            .map_err(|e| format!("{e}"))?;
        let mut laws = Vec::new();
        for result in rows {
            match result {
                Ok(law) => laws.push(law),
                Err(e) => eprintln!("Warning: skipping invalid law entry: {e}"),
            }
        }
        Ok(laws)
    }
}
