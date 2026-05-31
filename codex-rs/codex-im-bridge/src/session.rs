use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("数据库错误: {0}")]
    Database(String),
}

#[derive(Debug)]
pub struct SessionStore {
    db: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
pub struct SessionMapping {
    pub platform: String,
    pub chat_id: String,
    pub session_id: String,
}

impl SessionStore {
    pub fn new(path: &PathBuf) -> Result<Self, SessionStoreError> {
        let conn =
            Connection::open(path).map_err(|e| SessionStoreError::Database(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                platform TEXT NOT NULL,
                chat_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (platform, chat_id)
            );",
        )
        .map_err(|e| SessionStoreError::Database(e.to_string()))?;

        info!(?path, "会话存储已打开");

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn save_mapping(
        &self,
        platform: &str,
        chat_id: &str,
        session_id: &str,
    ) -> Result<(), SessionStoreError> {
        let db = self.db.lock().await;
        db.execute(
            "INSERT OR REPLACE INTO sessions (platform, chat_id, session_id, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            rusqlite::params![platform, chat_id, session_id],
        )
        .map_err(|e| SessionStoreError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_session(
        &self,
        platform: &str,
        chat_id: &str,
    ) -> Result<Option<String>, SessionStoreError> {
        let db = self.db.lock().await;
        let result = db.query_row(
            "SELECT session_id FROM sessions WHERE platform = ?1 AND chat_id = ?2",
            rusqlite::params![platform, chat_id],
            |row| row.get(0),
        );

        match result {
            Ok(session_id) => Ok(Some(session_id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SessionStoreError::Database(e.to_string())),
        }
    }

    pub async fn remove_mapping(
        &self,
        platform: &str,
        chat_id: &str,
    ) -> Result<(), SessionStoreError> {
        let db = self.db.lock().await;
        db.execute(
            "DELETE FROM sessions WHERE platform = ?1 AND chat_id = ?2",
            rusqlite::params![platform, chat_id],
        )
        .map_err(|e| SessionStoreError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_sessions(
        &self,
        platform: &str,
    ) -> Result<Vec<SessionMapping>, SessionStoreError> {
        let db = self.db.lock().await;
        let mut stmt = db
            .prepare("SELECT platform, chat_id, session_id FROM sessions WHERE platform = ?1")
            .map_err(|e| SessionStoreError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![platform], |row| {
                Ok(SessionMapping {
                    platform: row.get(0)?,
                    chat_id: row.get(1)?,
                    session_id: row.get(2)?,
                })
            })
            .map_err(|e| SessionStoreError::Database(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| SessionStoreError::Database(e.to_string()))?);
        }
        Ok(result)
    }
}
