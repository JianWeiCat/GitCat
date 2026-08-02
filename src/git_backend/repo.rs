//! 统一仓库句柄 — 封装 Git 操作

use std::path::{Path, PathBuf};
use std::process::Command;

/// Git 操作结果
pub type GitResult<T> = Result<T, crate::error::GhError>;

/// Git 仓库操作句柄（当前使用系统 git 命令，后续可切换 gix）
pub struct RepoHandle {
    pub path: PathBuf,
}

/// 提交信息
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
}

/// 分支信息
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub commit_hash: String,
    pub commit_message: String,
    pub commit_date: String,
    pub author: String,
}

/// Diff 统计
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: u64,
    pub deletions: u64,
}

/// 文件变更
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub added: u64,
    pub deleted: u64,
}

/// 仓库状态
#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub branch: String,
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
}

/// 仓库概览
#[derive(Debug, Clone)]
pub struct RepoOverview {
    pub path: String,
    pub total_commits: u64,
    pub total_branches: usize,
    pub total_tags: usize,
    pub contributors: usize,
    pub head_branch: String,
    pub last_commit_date: String,
}

impl RepoHandle {
    /// 打开 Git 仓库
    pub fn open(path: &Path) -> GitResult<Self> {
        let git_dir = path.join(".git");
        if !git_dir.exists() && !path.join("HEAD").exists() {
            return Err(crate::error::GhError::NotFound(format!(
                "{} 不是有效的 Git 仓库",
                path.display()
            )));
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// 执行 git 命令
    fn git(&self, args: &[&str]) -> GitResult<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|e| crate::error::GhError::Git(format!("无法执行 git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::into_friendly_error_msg(stderr.trim())
                .map(|msg| crate::error::GhError::Git(msg))
                .unwrap_or_else(|| crate::error::GhError::Git(stderr.trim().to_string())));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    // ---- 仓库概览 ----

    /// 获取仓库概览
    pub fn overview(&self) -> GitResult<RepoOverview> {
        let total_commits = self.git(&["rev-list", "--count", "HEAD"])?.trim().parse().unwrap_or(0);

        let branches = self
            .git(&["branch"])?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();

        let tags = self
            .git(&["tag"])?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();

        let contributors = self
            .git(&["shortlog", "-sne", "HEAD"])?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as usize;

        let head_branch = self
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim()
            .to_string();

        let last_commit_date = self
            .git(&["log", "-1", "--format=%ad", "--date=short"])?
            .trim()
            .to_string();

        Ok(RepoOverview {
            path: self.path.display().to_string(),
            total_commits,
            total_branches: branches,
            total_tags: tags,
            contributors,
            head_branch,
            last_commit_date,
        })
    }

    // ---- 提交遍历 ----

    /// 获取提交列表
    pub fn commits(
        &self,
        author: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
        branch: Option<&str>,
        grep: Option<&str>,
        max_count: Option<usize>,
    ) -> GitResult<Vec<CommitInfo>> {
        let max_str = max_count.map(|n| n.to_string());
        let mut args: Vec<&str> = vec!["log", "--format=%H|%an|%ae|%ad|%s", "--date=short"];

        if let Some(a) = author {
            args.push("--author");
            args.push(a);
        }
        if let Some(s) = since {
            args.push("--since");
            args.push(s);
        }
        if let Some(u) = until {
            args.push("--until");
            args.push(u);
        }
        if let Some(g) = grep {
            args.push("--grep");
            args.push(g);
        }
        if let Some(ref n) = max_str {
            args.push("-n");
            args.push(n);
        }
        if let Some(b) = branch {
            args.push(b);
        }

        let output = self.git(&args)?;
        let commits = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, '|').collect();
                if parts.len() < 5 {
                    return None;
                }
                Some(CommitInfo {
                    hash: parts[0].chars().take(7).collect(),
                    author: parts[1].to_string(),
                    email: parts[2].to_string(),
                    date: parts[3].to_string(),
                    message: parts[4].to_string(),
                })
            })
            .collect();

        Ok(commits)
    }

    // ---- Diff 统计 ----

    /// 获取 commit 的 diff 统计
    pub fn diff_stats(&self, commit_hash: &str) -> GitResult<(DiffStats, Vec<FileChange>)> {
        let output = self.git(&["diff-tree", "--numstat", "--find-renames", commit_hash])?;

        let mut files = Vec::new();
        let mut total_insertions = 0u64;
        let mut total_deletions = 0u64;

        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let added: u64 = parts[0].parse().unwrap_or(0);
                let deleted: u64 = parts[1].parse().unwrap_or(0);
                total_insertions += added;
                total_deletions += deleted;
                files.push(FileChange {
                    path: parts.last().unwrap().to_string(),
                    added,
                    deleted,
                });
            }
        }

        Ok((
            DiffStats {
                files_changed: files.len(),
                insertions: total_insertions,
                deletions: total_deletions,
            },
            files,
        ))
    }

    /// 获取某次提交的完整 diff 内容
    pub fn diff_content(&self, commit_hash: &str) -> GitResult<String> {
        self.git(&["diff-tree", "-p", commit_hash])
    }

    // ---- 分支操作 ----

    /// 获取本地分支列表
    pub fn local_branches(&self) -> GitResult<Vec<BranchInfo>> {
        let output = self.git(&[
            "branch",
            "--format=%(refname:short)|%(HEAD)|%(objectname:short)|%(subject)|%(committerdate:short)|%(authorname)",
        ])?;

        let branches = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(6, '|').collect();
                BranchInfo {
                    name: parts.get(0).unwrap_or(&"").to_string(),
                    is_head: *parts.get(1).unwrap_or(&"") == "*",
                    commit_hash: parts.get(2).unwrap_or(&"").chars().take(7).collect(),
                    commit_message: parts.get(3).unwrap_or(&"").to_string(),
                    commit_date: parts.get(4).unwrap_or(&"").to_string(),
                    author: parts.get(5).unwrap_or(&"").to_string(),
                }
            })
            .collect();

        Ok(branches)
    }

    /// 获取当前分支名
    pub fn current_branch(&self) -> GitResult<String> {
        self.git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .map(|s| s.trim().to_string())
    }

    /// 判断分支是否已合并到目标
    pub fn is_merged(&self, branch: &str, target: &str) -> GitResult<bool> {
        // merge-base == branch tip → 已完整合并
        let merge_base = self.git(&["merge-base", target, branch])?.trim().to_string();
        let branch_tip = self
            .git(&["rev-parse", branch])?
            .trim()
            .to_string();
        Ok(merge_base == branch_tip)
    }

    /// 获取分支最后提交日期
    pub fn branch_last_commit_date(&self, branch: &str) -> GitResult<String> {
        self.git(&["log", "-1", "--format=%ad", "--date=short", branch])
            .map(|s| s.trim().to_string())
    }

    /// 删除本地分支
    pub fn delete_branch(&self, branch: &str, force: bool) -> GitResult<()> {
        let flag = if force { "-D" } else { "-d" };
        self.git(&["branch", flag, branch])?;
        Ok(())
    }

    /// 创建分支
    pub fn create_branch(&self, name: &str) -> GitResult<()> {
        self.git(&["branch", name])?;
        Ok(())
    }

    /// 切换分支
    pub fn checkout(&self, branch: &str) -> GitResult<()> {
        self.git(&["checkout", branch])?;
        Ok(())
    }

    // ---- 仓库状态 ----

    /// 获取仓库状态
    pub fn status(&self) -> GitResult<RepoStatus> {
        let branch = self.current_branch().unwrap_or_else(|_| "HEAD".into());

        let output = self.git(&["status", "--porcelain"])?;
        let mut modified = 0;
        let mut staged = 0;
        let mut untracked = 0;

        for line in output.lines() {
            if line.is_empty() {
                continue;
            }
            let idx = line.chars().nth(0).unwrap_or(' ');
            let wt = line.chars().nth(1).unwrap_or(' ');

            if idx != ' ' && idx != '?' {
                staged += 1;
            }
            if wt != ' ' && wt != '?' {
                modified += 1;
            }
            if idx == '?' && wt == '?' {
                untracked += 1;
            }
        }

        Ok(RepoStatus {
            branch,
            modified,
            staged,
            untracked,
            ahead: 0,
            behind: 0,
        })
    }

    /// 检查是否有未提交更改
    pub fn has_uncommitted_changes(&self) -> GitResult<bool> {
        let output = self.git(&["status", "--porcelain"])?;
        Ok(output.trim().lines().any(|l| !l.is_empty()))
    }

    // ---- 操作 ----

    /// git pull
    pub fn pull(&self, rebase: bool) -> GitResult<String> {
        let mut args = vec!["pull"];
        if rebase {
            args.push("--rebase");
        }
        self.git(&args)
    }

    /// git gc
    pub fn gc(&self, aggressive: bool) -> GitResult<String> {
        let mut args = vec!["gc"];
        if aggressive {
            args.push("--aggressive");
        }
        self.git(&args)
    }

    /// git stash list
    pub fn stash_list(&self) -> GitResult<String> {
        self.git(&["stash", "list", "--format=%gd|%ar|%s"])
    }

    /// git stash drop
    pub fn stash_drop(&self, stash_ref: &str) -> GitResult<()> {
        self.git(&["stash", "drop", stash_ref])?;
        Ok(())
    }

    /// git stash clear
    pub fn stash_clear(&self) -> GitResult<()> {
        self.git(&["stash", "clear"])?;
        Ok(())
    }

    /// git reset --soft
    pub fn reset_soft(&self, target: &str) -> GitResult<()> {
        self.git(&["reset", "--soft", target])?;
        Ok(())
    }

    /// git reset --mixed
    pub fn reset_mixed(&self, target: &str) -> GitResult<()> {
        self.git(&["reset", "--mixed", target])?;
        Ok(())
    }

    /// git reset --hard
    pub fn reset_hard(&self, target: &str) -> GitResult<()> {
        self.git(&["reset", "--hard", target])?;
        Ok(())
    }
}
