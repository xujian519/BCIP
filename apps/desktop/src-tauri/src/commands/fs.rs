use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified_at: u64,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub is_directory: bool,
    pub modified_at: u64,
    pub created_at: u64,
    pub readonly: bool,
}

#[command]
pub async fn read_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }
    
    let mut entries = Vec::new();
    
    match tokio::fs::read_dir(&path).await {
        Ok(mut dir) => {
            while let Ok(Some(entry)) = dir.next_entry().await {
                let file_type = match entry.file_type().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();
                let is_directory = file_type.is_dir();

                let (size, modified_at) = if is_directory {
                    (0, 0)
                } else {
                    match entry.metadata().await {
                        Ok(metadata) => (
                            metadata.len(),
                            metadata
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0),
                        ),
                        Err(_) => (0, 0),
                    }
                };

                entries.push(FileEntry {
                    name,
                    path,
                    is_directory,
                    size,
                    modified_at,
                    created_at: modified_at,
                });
            }
            
            entries.sort_by(|a, b| {
                match (a.is_directory, b.is_directory) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
            
            Ok(entries)
        }
        Err(e) => Err(format!("读取目录失败: {}", e)),
    }
}

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("读取文件失败: {}", e)),
    }
}

#[command]
pub async fn read_file_binary(path: String) -> Result<Vec<u8>, String> {
    match tokio::fs::read(&path).await {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("读取文件失败: {}", e)),
    }
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    match tokio::fs::write(&path, content).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("写入文件失败: {}", e)),
    }
}

#[command]
pub async fn write_file_binary(path: String, content: Vec<u8>) -> Result<(), String> {
    match tokio::fs::write(&path, content).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("写入文件失败: {}", e)),
    }
}

#[command]
pub async fn create_dir(path: String) -> Result<(), String> {
    match tokio::fs::create_dir_all(&path).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("创建目录失败: {}", e)),
    }
}

#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    
    match tokio::fs::metadata(&path).await {
        Ok(metadata) => {
            if metadata.is_dir() {
                match tokio::fs::remove_dir_all(&path).await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("删除目录失败: {}", e)),
                }
            } else {
                match tokio::fs::remove_file(&path).await {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("删除文件失败: {}", e)),
                }
            }
        }
        Err(e) => Err(format!("获取文件信息失败: {}", e)),
    }
}

#[command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    let path = PathBuf::from(&path);
    
    match tokio::fs::metadata(&path).await {
        Ok(metadata) => {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            
            let extension = path.extension()
                .map(|e| e.to_string_lossy().to_string());
            
            Ok(FileInfo {
                path: path.to_string_lossy().to_string(),
                name,
                extension,
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                modified_at: metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                created_at: metadata.created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                readonly: metadata.permissions().readonly(),
            })
        }
        Err(e) => Err(format!("获取文件信息失败: {}", e)),
    }
}
