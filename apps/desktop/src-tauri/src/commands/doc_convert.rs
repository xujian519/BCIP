use crate::config;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocConvertResult {
    pub output_path: String,
    pub from_cache: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibreOfficeStatus {
    pub available: bool,
    pub path: Option<String>,
}

fn find_soffice_executable() -> Option<PathBuf> {
    if let Ok(output) = Command::new("which").arg("soffice").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mac_path = PathBuf::from("/Applications/LibreOffice.app/Contents/MacOS/soffice");
        if mac_path.is_file() {
            return Some(mac_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        for candidate in [
            r"C:\Program Files\LibreOffice\program\soffice.exe",
            r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
        ] {
            let path = PathBuf::from(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    if let Ok(output) = Command::new("which").arg("libreoffice").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}

fn file_modified_secs(path: &Path) -> Result<u64, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("无法读取文件信息: {e}"))?;
    let modified = metadata
        .modified()
        .map_err(|e| format!("无法读取修改时间: {e}"))?;
    Ok(modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("无效修改时间: {e}"))?
        .as_secs())
}

fn cache_key(path: &Path, modified_secs: u64, size: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    modified_secs.hash(&mut hasher);
    size.hash(&mut hasher);
    hasher.finish()
}

fn preview_cache_dir(input_path: &Path, modified_secs: u64, size: u64) -> Result<PathBuf, String> {
    let home = config::find_bcip_home()?;
    let key = cache_key(input_path, modified_secs, size);
    Ok(home.join("cache").join("doc-preview").join(format!("{key:016x}")))
}

fn expected_output_path(input_path: &Path, cache_dir: &Path) -> PathBuf {
    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "converted".to_string());
    cache_dir.join(format!("{stem}.docx"))
}

fn run_soffice_convert(soffice: &Path, input_path: &Path, outdir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(outdir).map_err(|e| format!("无法创建缓存目录: {e}"))?;

    let status = Command::new(soffice)
        .arg("--headless")
        .arg("--norestore")
        .arg("--nologo")
        .arg("--convert-to")
        .arg("docx")
        .arg("--outdir")
        .arg(outdir)
        .arg(input_path)
        .status()
        .map_err(|e| format!("无法启动 LibreOffice: {e}"))?;

    if !status.success() {
        return Err(format!(
            "LibreOffice 转换失败（退出码 {:?}）",
            status.code()
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn libreoffice_status() -> LibreOfficeStatus {
    match find_soffice_executable() {
        Some(path) => LibreOfficeStatus {
            available: true,
            path: Some(path.to_string_lossy().to_string()),
        },
        None => LibreOfficeStatus {
            available: false,
            path: None,
        },
    }
}

/// 将旧版 `.doc` 转为 `.docx`（带 ~/.bcip/cache/doc-preview 缓存）。
#[tauri::command]
pub fn convert_doc_to_docx(input_path: String) -> Result<DocConvertResult, String> {
    let input = PathBuf::from(&input_path);
    if !input.is_file() {
        return Err(format!("文件不存在: {input_path}"));
    }

    let extension = input
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if extension != "doc" {
        return Err("仅支持 .doc 格式转换".to_string());
    }

    let metadata = std::fs::metadata(&input).map_err(|e| format!("无法读取文件信息: {e}"))?;
    let modified_secs = file_modified_secs(&input)?;
    let size = metadata.len();
    let cache_dir = preview_cache_dir(&input, modified_secs, size)?;
    let output_path = expected_output_path(&input, &cache_dir);

    if output_path.is_file() {
        if let Ok(output_modified) = file_modified_secs(&output_path) {
            if output_modified >= modified_secs {
                return Ok(DocConvertResult {
                    output_path: output_path.to_string_lossy().to_string(),
                    from_cache: true,
                });
            }
        }
    }

    let soffice = find_soffice_executable().ok_or_else(|| {
        "未找到 LibreOffice（soffice）。请安装 LibreOffice 后重试。".to_string()
    })?;

    run_soffice_convert(&soffice, &input, &cache_dir)?;

    if !output_path.is_file() {
        return Err("转换完成但未找到输出 .docx 文件".to_string());
    }

    Ok(DocConvertResult {
        output_path: output_path.to_string_lossy().to_string(),
        from_cache: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_output_uses_stem() {
        let input = PathBuf::from("/tmp/report.doc");
        let cache = PathBuf::from("/cache/abc");
        assert_eq!(
            expected_output_path(&input, &cache),
            PathBuf::from("/cache/abc/report.docx")
        );
    }
}
