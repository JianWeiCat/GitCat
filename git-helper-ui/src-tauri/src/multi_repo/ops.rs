//! 批量操作：pull / gc / status

use crate::git_backend::repo::RepoHandle;

/// 批量 pull
pub fn batch_pull(handle: &RepoHandle, rebase: bool) -> Result<String, crate::error::GhError> {
    let output = handle.pull(rebase)?;
    Ok(if output.trim().is_empty() {
        "Already up to date.".into()
    } else {
        output.trim().to_string()
    })
}

/// 批量 gc
pub fn batch_gc(handle: &RepoHandle, aggressive: bool) -> Result<String, crate::error::GhError> {
    handle.gc(aggressive).map(|s| s.trim().to_string())
}

/// 批量状态检查
pub fn batch_status(handle: &RepoHandle) -> Result<String, crate::error::GhError> {
    let status = handle.status()?;
    if status.modified == 0 && status.staged == 0 && status.untracked == 0 {
        Ok(format!("{} - clean", status.branch))
    } else {
        Ok(format!(
            "{} - {} staged, {} modified, {} untracked",
            status.branch, status.staged, status.modified, status.untracked
        ))
    }
}
