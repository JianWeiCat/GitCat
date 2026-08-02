//! 安全重置操作

use crate::git_backend::repo::RepoHandle;

/// 重置模式
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

/// 执行安全重置
pub fn execute_reset(
    repo: &RepoHandle,
    target: &str,
    mode: ResetMode,
    backup: bool,
) -> Result<String, crate::error::GhError> {
    // 备份：创建 backup 分支
    if backup && matches!(mode, ResetMode::Hard) {
        let backup_name = format!("gh-backup-{}", timestamp_simple());
        repo.create_branch(&backup_name)?;
        println!("📦 已备份当前状态到分支: {}", backup_name);
    }

    match mode {
        ResetMode::Soft => {
            repo.reset_soft(target)?;
            Ok(format!("✅ Soft reset 到 {}", target))
        }
        ResetMode::Mixed => {
            repo.reset_mixed(target)?;
            Ok(format!("✅ Mixed reset 到 {}", target))
        }
        ResetMode::Hard => {
            repo.reset_hard(target)?;
            Ok(format!("✅ Hard reset 到 {} (⚠️ 工作区已重置)", target))
        }
    }
}

fn timestamp_simple() -> String {
    std::process::Command::new("date")
        .args(["+%Y%m%d-%H%M%S"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "backup".into())
}
