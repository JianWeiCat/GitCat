//! 仓库扫描器 — 递归扫描目录下所有 Git 仓库

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SKIPPED_DIRECTORIES: &[&str] = &[
    "$recycle.bin", "system volume information", "windows", "program files",
    "program files (x86)", "programdata", "appdata", "windowsapps", "node_modules", "target", ".cargo",
    ".rustup",
];

fn should_visit(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 { return true; }
    let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
    name != ".git" && !name.starts_with('.') && !SKIPPED_DIRECTORIES.contains(&name.as_str())
}

fn is_git_marker(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == ".git") && (path.is_dir() || path.is_file())
}

/// 扫描目录下所有 Git 仓库
pub fn scan_repos(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>, crate::error::GhError> {
    let mut repos = Vec::new();

    // 先检查根目录本身是否是 Git 仓库
    if is_git_marker(&root.join(".git")) {
        repos.push(root.to_path_buf());
    }

    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(should_visit)
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && is_git_marker(&entry.path().join(".git")) {
            repos.push(entry.path().to_path_buf());
        }
    }

    repos.sort();
    repos.dedup();
    Ok(repos)
}

/// 扫描当前机器的本地卷。深度应保持保守，避免遍历系统和依赖目录。
pub fn scan_system_drives(max_depth: usize) -> Result<Vec<PathBuf>, crate::error::GhError> {
    #[cfg(windows)]
    let roots: Vec<PathBuf> = (b'A'..=b'Z')
        .map(|letter| PathBuf::from(format!("{}:\\", letter as char)))
        .filter(|root| root.is_dir())
        .collect();
    #[cfg(not(windows))]
    let roots = vec![PathBuf::from("/")];

    let mut repos = Vec::new();
    for root in roots { repos.extend(scan_repos(&root, max_depth)?); }
    repos.sort();
    repos.dedup();
    Ok(repos)
}
