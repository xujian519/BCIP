use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::path::Path;

/// 文本块及其向量嵌入
#[derive(Debug, Clone)]
pub struct EmbeddedChunk {
    pub chunk_id: String,
    pub file_path: String,
    pub title: String,
    pub content: String,
    pub chunk_index: i64,
    pub embedding: Vec<f32>,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub chunk: EmbeddedChunk,
    pub score: f64,
}

/// BGE-M3 语义向量索引（1024 维）
pub struct VectorIndex {
    chunks: Vec<EmbeddedChunk>,
    dim: usize,
}

impl VectorIndex {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("无法打开语义索引: {e}"))?;

        let dim: usize =
            conn.query_row("SELECT embedding_dim FROM chunks LIMIT 1", [], |r| {
                r.get::<_, i64>(0)
            })
            .map_err(|e| format!("无法读取 embedding 维度: {e}"))? as usize;

        let mut stmt = conn
            .prepare(
                "SELECT chunk_id, file_path, title, content, chunk_index, embedding \
                 FROM chunks ORDER BY chunk_id",
            )
            .map_err(|e| format!("prepare error: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                let embed_blob: Vec<u8> = row.get(5)?;
                Ok(EmbeddedChunk {
                    chunk_id: row.get(0)?,
                    file_path: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    chunk_index: row.get(4)?,
                    embedding: decode_embedding(&embed_blob),
                })
            })
            .map_err(|e| format!("query error: {e}"))?;

        let mut chunks = Vec::with_capacity(22000);
        for row in rows.flatten() {
            chunks.push(row);
        }

        Ok(Self { chunks, dim })
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// 对查询向量做余弦相似度搜索，返回 Top-K
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<ScoredChunk> {
        if self.chunks.is_empty() || query_embedding.is_empty() {
            return Vec::new();
        }

        let query_norm = dot_product(query_embedding, query_embedding).sqrt();
        if query_norm == 0.0 {
            return Vec::new();
        }

        let mut scored: Vec<ScoredChunk> = self
            .chunks
            .iter()
            .map(|chunk| {
                let dot = dot_product(query_embedding, &chunk.embedding);
                let norm = dot_product(&chunk.embedding, &chunk.embedding).sqrt();
                let score = if norm == 0.0 {
                    0.0
                } else {
                    (dot / (query_norm * norm)) as f64
                };
                ScoredChunk {
                    chunk: chunk.clone(),
                    score,
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        scored
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// 从 BLOB 解码向量。BGE-M3-MLX-8bit 存储为 f32 的 little-endian 字节
fn decode_embedding(blob: &[u8]) -> Vec<f32> {
    let remainder = blob.len() % 4;
    let valid_len = blob.len() - remainder;
    let mut v = Vec::with_capacity(valid_len / 4);
    for chunk in blob[..valid_len].chunks_exact(4) {
        v.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_embedding() {
        let mut blob = Vec::with_capacity(16);
        for &f in &[1.0f32, 0.0, -1.0, 0.5] {
            blob.extend_from_slice(&f.to_le_bytes());
        }
        let v = decode_embedding(&blob);
        assert_eq!(v.len(), 4);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!((v[3] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let dot = dot_product(&v1, &v2);
        let norm = dot_product(&v1, &v1).sqrt();
        assert!((dot / (norm * norm) - 1.0).abs() < 1e-6);
    }
}
