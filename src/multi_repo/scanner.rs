//! 仓库扫描器 — 递归扫描目录下所有 Git 仓库

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 扫描目录下所有 Git 仓库
pub fn scan_repos(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>, crate::error::GhError> {
    let mut repos = Vec::new();

    // 先检查根目录本身是否是 Git 仓库
    if root.join(".git").is_dir() {
        repos.push(root.to_path_buf());
    }

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            // 跳过根目录本身（已检查）
            if e.depth() == 0 {
                return true;
            }
            !name.starts_with('.') || name == ".git"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
        }
    }

    repos.sort();
    repos.dedup();
    Ok(repos)
}
