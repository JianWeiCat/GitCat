//! 多仓库并发执行器

use std::path::PathBuf;
use crate::git_backend::repo::RepoHandle;

/// 操作执行结果
pub struct OpResult {
    pub repo_path: String,
    pub success: bool,
    pub message: String,
}

/// 并发执行操作
pub fn run_parallel<F>(
    repos: &[PathBuf],
    operation: F,
) -> Vec<OpResult>
where
    F: Fn(&RepoHandle) -> Result<String, crate::error::GhError> + Sync,
{
    use rayon::prelude::*;

    repos
        .par_iter()
        .map(|path| {
            match RepoHandle::open(path) {
                Ok(handle) => match operation(&handle) {
                    Ok(msg) => OpResult {
                        repo_path: path.display().to_string(),
                        success: true,
                        message: msg,
                    },
                    Err(e) => OpResult {
                        repo_path: path.display().to_string(),
                        success: false,
                        message: format!("操作失败: {}", e),
                    },
                },
                Err(e) => OpResult {
                    repo_path: path.display().to_string(),
                    success: false,
                    message: format!("无法打开仓库: {}", e),
                },
            }
        })
        .collect()
}
