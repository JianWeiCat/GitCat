//! 批量分支操作：多仓库切换等

use crate::git_backend::repo::RepoHandle;

/// 批量切换多仓库到同一分支
pub fn batch_checkout(
    repos: &[(String, RepoHandle)],
    branch: &str,
) -> Vec<(String, Result<(), String>)> {
    repos
        .iter()
        .map(|(path, repo)| {
            let result = repo.checkout(branch).map_err(|e| e.to_string());
            (path.clone(), result)
        })
        .collect()
}
