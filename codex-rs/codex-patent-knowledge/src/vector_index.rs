//! BGE-M3 语义向量索引
//!
//! 从 SQLite 文件中加载 1024 维向量嵌入，提供余弦相似度搜索。

use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
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
    norms: Vec<f32>,
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

        let norms: Vec<f32> = chunks
            .iter()
            .map(|c| dot_product(&c.embedding, &c.embedding).sqrt())
            .collect();

        Ok(Self { chunks, norms, dim })
    }

    /// 返回索引中的向量总数
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// 返回索引是否为空
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// 返回向量的维度（BGE-M3 为 1024）
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// 对查询向量做余弦相似度搜索，返回 Top-K。
    /// 使用 min-heap 实现 O(n·log k) 选择，避免全排序。
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<ScoredChunk> {
        if self.chunks.is_empty() || query_embedding.is_empty() {
            return Vec::new();
        }

        let query_norm = dot_product(query_embedding, query_embedding).sqrt();
        if query_norm == 0.0 {
            return Vec::new();
        }

        // Min-heap: keeps the worst score at the top for eviction.
        struct HeapEntry {
            score: f64,
            idx: usize,
        }
        impl PartialEq for HeapEntry {
            fn eq(&self, other: &Self) -> bool {
                self.score == other.score
            }
        }
        impl Eq for HeapEntry {}
        impl PartialOrd for HeapEntry {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for HeapEntry {
            fn cmp(&self, other: &Self) -> Ordering {
                // Min-heap: lower score = greater ordering
                other
                    .score
                    .partial_cmp(&self.score)
                    .unwrap_or(Ordering::Equal)
            }
        }

        let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::with_capacity(top_k);

        for (i, chunk) in self.chunks.iter().enumerate() {
            let norm = self.norms[i];
            let score = if norm == 0.0 {
                0.0
            } else {
                let dot = dot_product(query_embedding, &chunk.embedding);
                (dot / (query_norm * norm)) as f64
            };

            if heap.len() < top_k {
                heap.push(HeapEntry { score, idx: i });
            } else if let Some(top) = heap.peek() {
                if score > top.score {
                    heap.pop();
                    heap.push(HeapEntry { score, idx: i });
                }
            }
        }

        let mut results: Vec<ScoredChunk> = heap
            .into_iter()
            .map(|entry| ScoredChunk {
                chunk: self.chunks[entry.idx].clone(),
                score: entry.score,
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
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
