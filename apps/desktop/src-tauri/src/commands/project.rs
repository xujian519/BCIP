use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub project_path: Option<String>,
    pub bcip_dir_exists: bool,
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("proj-{ts}")
}

/// 创建新项目：在选定目录下创建 .bcip/ 结构
#[command]
pub async fn project_create(path: String) -> Result<ProjectInfo, String> {
    let root = PathBuf::from(&path);

    if root.exists() && root.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
        return Err(format!("目录不为空: {}", root.display()));
    }

    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|e| format!("创建项目目录失败: {e}"))?;

    let bcip_dir = root.join(".bcip");
    tokio::fs::create_dir_all(&bcip_dir)
        .await
        .map_err(|e| format!("创建 .bcip 目录失败: {e}"))?;

    let project_json = bcip_dir.join("project.json");
    let project_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "未命名项目".to_string());

    let id = generate_id();
    let info = ProjectInfo {
        id: id.clone(),
        name: project_name.clone(),
        path: path.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let json = serde_json::to_string_pretty(&info).map_err(|e| format!("序列化失败: {e}"))?;
    tokio::fs::write(&project_json, json)
        .await
        .map_err(|e| format!("写入 project.json 失败: {e}"))?;

    let readme = root.join("README.md");
    let readme_content = format!("# {project_name}\n\n## 项目概述\n\n");
    tokio::fs::write(&readme, readme_content)
        .await
        .map_err(|e| format!("写入 README.md 失败: {e}"))?;

    Ok(info)
}

/// 扫描已知项目（通过 .bcip/ 目录检测）
#[command]
pub async fn project_list(search_paths: Option<Vec<String>>) -> Result<Vec<ProjectInfo>, String> {
    let paths = search_paths.unwrap_or_else(|| {
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        vec![
            format!("{home}/Projects"),
            format!("{home}/BCIP"),
        ]
    });

    let mut projects = Vec::new();

    for search_path in &paths {
        let root = PathBuf::from(search_path);
        if !root.exists() {
            continue;
        }

        scan_for_projects(&root, &mut projects).await;
    }

    Ok(projects)
}

async fn scan_for_projects(root: &PathBuf, projects: &mut Vec<ProjectInfo>) {
    let mut dir = match tokio::fs::read_dir(root).await {
        Ok(d) => d,
        Err(_) => return,
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let bcip_dir = path.join(".bcip");
        let project_json = bcip_dir.join("project.json");

        if project_json.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&project_json).await {
                if let Ok(info) = serde_json::from_str::<ProjectInfo>(&content) {
                    projects.push(info);
                    continue;
                }
            }
        }

        // 限制递归深度，仅扫描一层子目录
        let bcip_exists = bcip_dir.exists();
        if bcip_exists {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let path_str = path.to_string_lossy().to_string();
            let created_at = tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            projects.push(ProjectInfo {
                id: generate_id(),
                name,
                path: path_str,
                created_at,
            });
        }
    }
}

#[command]
pub async fn project_get_workspace(path: Option<String>) -> Result<WorkspaceInfo, String> {
    let project_path = if let Some(p) = path {
        let pb = PathBuf::from(&p);
        if pb.exists() {
            Some(p)
        } else {
            return Err(format!("路径不存在: {p}"));
        }
    } else {
        None
    };

    let bcip_dir_exists = project_path
        .as_ref()
        .map(|p| PathBuf::from(p).join(".bcip").exists())
        .unwrap_or(false);

    Ok(WorkspaceInfo {
        project_path,
        bcip_dir_exists,
    })
}
