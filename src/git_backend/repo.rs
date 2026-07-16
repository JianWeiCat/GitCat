//! 统一仓库句柄

use std::path::Path;

/// Git 仓库操作句柄
pub struct RepoHandle {
    pub path: Box<Path>,
}

impl RepoHandle {
    /// 打开 Git 仓库
    pub fn open(path: &Path) -> Result<Self, crate::error::GhError> {
        let git_dir = path.join(".git");
        if !git_dir.exists() && !path.join("HEAD").exists() {
            return Err(crate::error::GhError::NotFound(
                format!("{} 不是有效的 Git 仓库", path.display())
            ));
        }
        Ok(Self { path: path.into() })
    }
}
