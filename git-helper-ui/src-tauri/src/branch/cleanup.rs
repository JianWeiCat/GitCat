//! 清理已合并的废弃分支

use crate::git_backend::repo::RepoHandle;
use serde::Serialize;

/// 可清理分支信息
pub struct CleanableBranch {
    pub name: String,
    pub last_commit_date: String,
    pub last_commit_message: String,
    pub reason: String,
    pub is_protected: bool,
}

/// 可序列化到前端的响应
#[derive(Debug, Serialize)]
pub struct CleanableBranchResponse {
    pub name: String,
    pub last_commit_date: String,
    pub last_commit_message: String,
    pub reason: String,
    pub is_protected: bool,
}

/// 内置保护分支模式
const BUILTIN_PROTECTED: &[&str] = &["main", "master", "develop", "dev",
    "release/*", "hotfix/*", "production", "staging"];

/// 扫描可清理分支
pub fn scan_cleanable(
    repo: &RepoHandle,
    check_remote: bool,
    whitelist: &[String],
) -> Result<Vec<CleanableBranch>, crate::error::GhError> {
    let current = repo.current_branch()?;
    let branches = repo.local_branches()?;
    let mut cleanable = Vec::new();

    for branch in &branches {
        // 跳过当前分支
        if branch.name == current {
            continue;
        }

        let is_protected = is_protected_branch(&branch.name, whitelist);

        // 检查是否已合并
        match repo.is_merged(&branch.name, &current) {
            Ok(true) => {
                cleanable.push(CleanableBranch {
                    name: branch.name.clone(),
                    last_commit_date: branch.commit_date.clone(),
                    last_commit_message: branch.commit_message.clone(),
                    reason: if is_protected {
                        "已合并但被白名单/内置规则保护".into()
                    } else {
                        format!("已合并到 {}", current)
                    },
                    is_protected,
                });
            }
            Ok(false) => {}
            Err(_) => {} // 跳过无法判断的分支
        }
    }

    // 如果也检查远程
    if check_remote {
        // 远程分支清理需要更多处理，此处简化
    }

    Ok(cleanable)
}

/// 执行分支清理
pub fn execute_cleanup(
    repo: &RepoHandle,
    branches: &[CleanableBranch],
    force: bool,
    dry_run: bool,
) -> Result<Vec<String>, crate::error::GhError> {
    let mut deleted = Vec::new();

    for branch in branches {
        if branch.is_protected {
            continue;
        }

        if dry_run {
            deleted.push(format!("{} (dry-run, 未实际删除)", branch.name));
        } else {
            repo.delete_branch(&branch.name, force)?;
            deleted.push(branch.name.clone());
        }
    }

    Ok(deleted)
}

/// 格式化 dry-run 输出
pub fn format_dry_run(branches: &[CleanableBranch]) -> String {
    let mut output = String::from("=== 分支清理预演 (--dry-run) ===\n\n");

    if branches.is_empty() {
        output.push_str("没有可清理的分支。\n");
        return output;
    }

    for b in branches {
        let icon = if b.is_protected { "🔒" } else { "✅" };
        output.push_str(&format!(
            "{} {}  ({} | {})\n   └─ {}\n",
            icon, b.name, b.last_commit_date, b.last_commit_message, b.reason
        ));
    }

    let cleanable_count = branches.iter().filter(|b| !b.is_protected).count();
    output.push_str(&format!(
        "\n总计: {} 个已合并分支, {} 个可安全删除, {} 个被保护\n",
        branches.len(),
        cleanable_count,
        branches.len() - cleanable_count,
    ));

    if cleanable_count > 0 {
        output.push_str("\n💡 确认无误后使用 --force 执行实际删除。\n");
    }

    output
}

/// 检查分支是否被保护
fn is_protected_branch(name: &str, whitelist: &[String]) -> bool {
    // 内置保护
    if BUILTIN_PROTECTED.iter().any(|p| wildcard_match(p, name)) {
        return true;
    }
    // 自定义白名单
    if whitelist.iter().any(|w| wildcard_match(w, name)) {
        return true;
    }
    false
}

/// 简单的通配符匹配
fn wildcard_match(pattern: &str, name: &str) -> bool {
    if pattern == name {
        return true;
    }
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        return name.starts_with(prefix) && name.len() > prefix.len() + 1;
    }
    false
}
