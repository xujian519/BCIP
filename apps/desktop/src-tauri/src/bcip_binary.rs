//! 解析 bcip 可执行文件：PATH 与外置 sidecar，按 `--version` 择优。

use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::{AppHandle, Manager};

#[derive(Clone, Debug)]
pub struct ResolvedBcip {
    pub path: PathBuf,
    /// `path` = 系统 PATH；`sidecar` = 应用 bundle 内嵌
    pub source: String,
    pub version: Option<String>,
}

pub fn resolve_bcip_binary(app: Option<&AppHandle>) -> Option<ResolvedBcip> {
    let path_candidate = find_on_path().map(|path| ResolvedBcip {
        source: "path".to_string(),
        path,
        version: None,
    });
    let workspace_candidate = find_workspace_dev_binary().map(|path| ResolvedBcip {
        source: "workspace".to_string(),
        path,
        version: None,
    });
    let sidecar_candidate = find_sidecar(app).map(|path| ResolvedBcip {
        source: "sidecar".to_string(),
        path,
        version: None,
    });

    let path_candidate = path_candidate.map(mutate_with_version);
    let workspace_candidate = workspace_candidate.map(mutate_with_version);
    let sidecar_candidate = sidecar_candidate.map(mutate_with_version);

    let mut candidates: Vec<ResolvedBcip> = Vec::new();
    if let Some(r) = path_candidate {
        candidates.push(r);
    }
    if let Some(r) = workspace_candidate {
        candidates.push(r);
    }
    if let Some(r) = sidecar_candidate {
        candidates.push(r);
    }
    pick_best_candidate(candidates)
}

fn pick_best_candidate(candidates: Vec<ResolvedBcip>) -> Option<ResolvedBcip> {
    fn source_rank(source: &str) -> u8 {
        match source {
            "path" => 3,
            "workspace" => 2,
            "sidecar" => 1,
            _ => 0,
        }
    }

    candidates.into_iter().max_by(|a, b| {
        let va = version_key(a.version.as_deref().unwrap_or(""));
        let vb = version_key(b.version.as_deref().unwrap_or(""));
        va.cmp(&vb)
            .then(source_rank(&a.source).cmp(&source_rank(&b.source)))
    })
}

fn mutate_with_version(mut r: ResolvedBcip) -> ResolvedBcip {
    r.version = read_version(&r.path);
    r
}

pub fn check_bcip_installed(app: Option<&AppHandle>) -> BcipCheckResult {
    match resolve_bcip_binary(app) {
        Some(r) => BcipCheckResult {
            installed: true,
            version: r.version,
            path: Some(r.path.display().to_string()),
            source: Some(r.source),
        },
        None => BcipCheckResult {
            installed: false,
            version: None,
            path: None,
            source: None,
        },
    }
}

#[derive(serde::Serialize)]
pub struct BcipCheckResult {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub source: Option<String>,
}

/// 开发态：仓库内 `codex-rs/target/{debug,release}/codex`，无需全局 PATH。
fn find_workspace_dev_binary() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("BCIP_DEV_CODEX") {
        let path = PathBuf::from(&explicit);
        if path.is_file() {
            return Some(path);
        }
    }

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for rel in [
        "codex-rs/target/debug/bcip",
        "codex-rs/target/debug/codex",
        "codex-rs/target/release/bcip",
        "codex-rs/target/release/codex",
    ] {
        let path = repo_root.join(rel);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn find_on_path() -> Option<PathBuf> {
    let output = if cfg!(windows) {
        Command::new("where").arg("bcip").output()
    } else {
        Command::new("which").arg("bcip").output()
    }
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if line.is_empty() {
        return None;
    }
    let path = PathBuf::from(line);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn find_sidecar(app: Option<&AppHandle>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(handle) = app {
        if let Ok(res) = handle.path().resource_dir() {
            candidates.push(res.join("bin").join("bcip"));
            candidates.push(res.join("bcip"));
            push_bcip_glob(&res.join("bin"), &mut candidates);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(mac_os) = exe.parent() {
            candidates.push(mac_os.join("bcip"));
            if let Some(contents) = mac_os.parent() {
                candidates.push(contents.join("Resources").join("bin").join("bcip"));
                candidates.push(contents.join("Resources").join("bcip"));
            }
        }
    }

    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    push_bcip_glob(&bin_dir, &mut candidates);

    candidates.into_iter().find(|p| p.is_file())
}

fn push_bcip_glob(dir: &Path, candidates: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("bcip") && path.is_file() {
            candidates.push(path);
        }
    }
}

fn read_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::from_utf8_lossy(&output.stderr).trim().to_string().into()
    } else {
        Some(trimmed.to_string())
    }
}


fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse().ok())
        .collect()
}
