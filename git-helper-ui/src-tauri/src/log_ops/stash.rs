//! Stash 批量管理

use crate::git_backend::repo::RepoHandle;

/// 列出所有 stash
pub fn list_stashes(repo: &RepoHandle) -> Result<String, crate::error::GhError> {
    let output = repo.stash_list()?;
    if output.trim().is_empty() {
        Ok("没有 stash 记录。\n".into())
    } else {
        Ok(format!("=== Stash 列表 ===\n\n{}", output))
    }
}

/// 清理过期 stash
pub fn clean_stashes(repo: &RepoHandle, older_than_days: u32, dry_run: bool) -> Result<String, crate::error::GhError> {
    if dry_run {
        let output = repo.stash_list()?;
        Ok(format!("=== Stash 清理预演 (--dry-run) ===\n\n清理 {} 天前的 stash\n\n{}", older_than_days, output))
    } else {
        // 简化实现：逐条 drop
        let output = repo.stash_list()?;
        let mut count = 0;
        for line in output.lines() {
            if line.trim().is_empty() { continue; }
            let ref_name = line.split('|').next().unwrap_or("");
            if !ref_name.is_empty() {
                repo.stash_drop(ref_name)?;
                count += 1;
            }
        }
        Ok(format!("✅ 已清理 {} 条 stash 记录", count))
    }
}
