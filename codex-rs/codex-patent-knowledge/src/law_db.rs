use codex_patent_core::LawCategory;
use codex_patent_core::LawDocument;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::params;
use std::path::Path;

pub struct LawDatabase {
    conn: Connection,
}

impl LawDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("failed to open law db: {e}"))?;

        conn.execute_batch(
            "PRAGMA cache_size = -4000;
             PRAGMA locking_mode = NORMAL;",
        )
        .map_err(|e| format!("pragma setup: {e}"))?;

        Ok(Self { conn })
    }

    pub fn search_by_name(&self, keyword: &str, limit: usize) -> Result<Vec<LawDocument>, String> {
        let pattern = format!("%{keyword}%");
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law WHERE name LIKE ?1 LIMIT ?2";
        self.query_laws(sql, params![pattern, limit])
    }

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

    pub fn list_by_level(&self, level: &str, limit: usize) -> Result<Vec<LawDocument>, String> {
        let sql = "SELECT id, level, name, filename, publish, expired, category_id, subtitle, content \
                   FROM law WHERE level = ?1 ORDER BY publish DESC LIMIT ?2";
        self.query_laws(sql, params![level, limit])
    }

    pub fn list_levels(&self) -> Result<Vec<String>, String> {
        let sql = "SELECT DISTINCT level FROM law ORDER BY level";
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("{e}"))?;
        let levels: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(levels)
    }

    pub fn list_categories(&self) -> Result<Vec<LawCategory>, String> {
        let sql = "SELECT id, name, folder, isSubFolder, \"group\", \"order\" FROM category ORDER BY \"order\"";
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;
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

    pub fn count(&self) -> Result<usize, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM law", [], |row| row.get(0))
            .map_err(|e| format!("{e}"))
    }

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
        let mut stmt = self.conn.prepare(sql).map_err(|e| format!("{e}"))?;
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
