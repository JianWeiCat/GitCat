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

/// 供只读提交图使用的拓扑节点。
#[derive(Debug, Clone)]
pub struct CommitGraphNode {
    pub hash: String,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Current working-tree language breakdown, calculated from tracked files.
#[derive(Debug, Clone)]
pub struct LanguageStat {
    pub language: String,
    pub bytes: u64,
    pub files: usize,
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

/// A single path in the index or working tree.
#[derive(Debug, Clone)]
pub struct WorkingTreeFile {
    pub path: String,
    pub index_status: char,
    pub worktree_status: char,
    pub staged: bool,
    pub modified: bool,
    pub untracked: bool,
    pub conflicted: bool,
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
        let has_head = self.git(&["rev-parse", "--verify", "HEAD"]).is_ok();
        let total_commits = if has_head {
            self.git(&["rev-list", "--count", "HEAD"])?
                .trim()
                .parse()
                .unwrap_or(0)
        } else { 0 };

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

        let contributors = if has_head {
            self.git(&["shortlog", "-sne", "HEAD"])?
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count() as usize
        } else { 0 };

        let head_branch = self.current_branch().unwrap_or_else(|_| "HEAD".into());

        let last_commit_date = if has_head {
            self.git(&["log", "-1", "--format=%ad", "--date=short"])?
                .trim()
                .to_string()
        } else { String::new() };

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

    /// 获取所有引用可达提交的拓扑顺序，供提交图渲染。
    pub fn commit_graph(&self, max_count: usize) -> GitResult<Vec<CommitGraphNode>> {
        if self.git(&["rev-parse", "--verify", "HEAD"]).is_err() {
            return Ok(Vec::new());
        }
        let limit = max_count.to_string();
        let output = self.git(&[
            "log", "--all", "--topo-order", "--date-order", "--decorate=short",
            "--format=%H%x1f%P%x1f%D%x1f%an%x1f%ad%x1f%s%x1e", "--date=short", "-n", &limit,
        ])?;

        Ok(output.split('\x1e').filter_map(|record| {
            let fields: Vec<&str> = record.trim().split('\x1f').collect();
            if fields.len() < 6 || fields[0].is_empty() { return None; }
            Some(CommitGraphNode {
                hash: fields[0].chars().take(10).collect(),
                parents: fields[1].split_whitespace().map(|parent| parent.chars().take(10).collect()).collect(),
                refs: fields[2].split(',').map(str::trim).filter(|reference| !reference.is_empty()).map(str::to_string).collect(),
                author: fields[3].to_string(),
                date: fields[4].to_string(),
                message: fields[5].to_string(),
            })
        }).collect())
    }

    /// Group tracked, non-binary files by language. This intentionally describes the
    /// checked-out repository rather than historic churn, which is easier to explain
    /// to someone opening a repository for the first time.
    pub fn language_stats(&self) -> GitResult<Vec<LanguageStat>> {
        use std::collections::BTreeMap;

        let output = self.git(&["ls-files", "-z"])?;
        let mut totals: BTreeMap<String, (u64, usize)> = BTreeMap::new();

        for relative in output.split('\0').filter(|path| !path.is_empty()) {
            let file = self.path.join(relative);
            let Ok(metadata) = std::fs::metadata(&file) else { continue; };
            if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 20 * 1024 * 1024 {
                continue;
            }
            let language = language_for_path(relative);
            let entry = totals.entry(language).or_insert((0, 0));
            entry.0 += metadata.len();
            entry.1 += 1;
        }

        let mut result: Vec<_> = totals
            .into_iter()
            .map(|(language, (bytes, files))| LanguageStat { language, bytes, files })
            .collect();
        result.sort_by(|left, right| right.bytes.cmp(&left.bytes));
        Ok(result)
    }

    // ---- Diff 统计 ----

    /// 获取 commit 的 diff 统计
    pub fn diff_stats(&self, commit_hash: &str) -> GitResult<(DiffStats, Vec<FileChange>)> {
        let output = self.git(&["diff-tree", "--root", "-r", "-m", "--first-parent", "--numstat", "--find-renames", commit_hash])?;

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
        self.git(&["diff-tree", "--root", "-r", "-m", "--first-parent", "-p", commit_hash])
    }

    /// List changed paths without losing spaces or non-ASCII file names.
    pub fn working_tree_files(&self) -> GitResult<Vec<WorkingTreeFile>> {
        let output = self.git(&["status", "--porcelain=v1", "-z", "--untracked-files=all"])?;
        let mut records = output.split('\0').filter(|record| !record.is_empty());
        let mut files = Vec::new();

        while let Some(record) = records.next() {
            if record.len() < 3 { continue; }
            let bytes = record.as_bytes();
            let index_status = bytes[0] as char;
            let worktree_status = bytes[1] as char;
            let path = record[3..].to_string();
            let renamed_or_copied = matches!(index_status, 'R' | 'C') || matches!(worktree_status, 'R' | 'C');
            if renamed_or_copied {
                // In porcelain -z mode the destination is in this record and the source
                // follows as a second NUL-terminated field.
                let _ = records.next();
            }
            let untracked = index_status == '?' && worktree_status == '?';
            let conflicted = index_status == 'U'
                || worktree_status == 'U'
                || matches!((index_status, worktree_status), ('A', 'A') | ('D', 'D'));
            files.push(WorkingTreeFile {
                path,
                index_status,
                worktree_status,
                staged: !untracked && index_status != ' ',
                modified: !untracked && worktree_status != ' ',
                untracked,
                conflicted,
            });
        }

        files.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(files)
    }

    pub fn working_tree_diff(&self, path: &str, staged: bool) -> GitResult<String> {
        if staged {
            self.git(&["diff", "--cached", "--", path])
        } else {
            self.git(&["diff", "--", path])
        }
    }

    pub fn stage_files(&self, paths: &[String]) -> GitResult<()> {
        if paths.is_empty() { return Ok(()); }
        let mut args = vec!["add", "--"];
        args.extend(paths.iter().map(String::as_str));
        self.git(&args)?;
        Ok(())
    }

    pub fn unstage_files(&self, paths: &[String]) -> GitResult<()> {
        if paths.is_empty() { return Ok(()); }
        let has_head = self.git(&["rev-parse", "--verify", "HEAD"]).is_ok();
        let mut args = if has_head {
            vec!["reset", "--"]
        } else {
            vec!["rm", "--cached", "-r", "--ignore-unmatch", "--"]
        };
        args.extend(paths.iter().map(String::as_str));
        self.git(&args)?;
        Ok(())
    }

    pub fn commit_staged(&self, message: &str) -> GitResult<String> {
        let message = message.trim();
        if message.is_empty() {
            return Err(crate::error::GhError::Args("提交说明不能为空".into()));
        }
        self.git(&["commit", "-m", message])?;
        self.git(&["rev-parse", "--short", "HEAD"]).map(|value| value.trim().to_string())
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
        self.git(&["symbolic-ref", "--quiet", "--short", "HEAD"])
            .or_else(|_| self.git(&["rev-parse", "--abbrev-ref", "HEAD"]))
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

        let (behind, ahead) = self.git(&["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
            .ok()
            .and_then(|counts| {
                let mut parts = counts.split_whitespace();
                Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
            })
            .unwrap_or((0, 0));

        Ok(RepoStatus {
            branch,
            modified,
            staged,
            untracked,
            ahead,
            behind,
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

    pub fn fetch(&self) -> GitResult<String> {
        self.git(&["fetch", "--prune"])
    }

    pub fn push(&self) -> GitResult<String> {
        self.git(&["push"])
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

fn language_for_path(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let language = match extension.as_str() {
        "rs" => "Rust", "ts" | "tsx" => "TypeScript", "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "py" => "Python", "go" => "Go", "java" => "Java", "kt" | "kts" => "Kotlin",
        "c" | "h" => "C", "cc" | "cpp" | "cxx" | "hpp" => "C++", "cs" => "C#",
        "rb" => "Ruby", "php" => "PHP", "swift" => "Swift", "dart" => "Dart",
        "vue" => "Vue", "svelte" => "Svelte", "html" | "htm" => "HTML", "css" | "scss" | "sass" | "less" => "CSS",
        "json" => "JSON", "yml" | "yaml" => "YAML", "toml" => "TOML", "xml" => "XML",
        "md" | "mdx" => "Markdown", "sql" => "SQL", "sh" | "bash" | "zsh" | "ps1" => "Shell",
        _ if name == "dockerfile" || name == "makefile" => "Build scripts",
        _ => "Other",
    };
    language.to_string()
}

#[cfg(test)]
mod tests {
    use super::RepoHandle;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("gitcat-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            git(&path, &["init"]);
            git(&path, &["config", "user.name", "GitCat Test"]);
            git(&path, &["config", "user.email", "gitcat@example.test"]);
            Self(path)
        }

        fn handle(&self) -> RepoHandle { RepoHandle::open(&self.0).unwrap() }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    fn git(path: &Path, args: &[&str]) {
        let output = Command::new("git").args(args).current_dir(path).output().unwrap();
        assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    }

    fn commit_file(repo: &TempRepo, name: &str, content: &str, message: &str) {
        std::fs::write(repo.0.join(name), content).unwrap();
        git(&repo.0, &["add", "--", name]);
        git(&repo.0, &["commit", "-m", message]);
    }

    #[test]
    fn working_tree_keeps_spaces_and_distinguishes_groups() {
        let repo = TempRepo::new();
        commit_file(&repo, "tracked.txt", "one\n", "initial");
        std::fs::write(repo.0.join("tracked.txt"), "one\ntwo\n").unwrap();
        std::fs::write(repo.0.join("未跟踪 file.txt"), "new\n").unwrap();
        std::fs::write(repo.0.join("staged.txt"), "staged\n").unwrap();
        git(&repo.0, &["add", "--", "staged.txt"]);

        let files = repo.handle().working_tree_files().unwrap();
        assert!(files.iter().any(|file| file.path == "tracked.txt" && file.modified));
        assert!(files.iter().any(|file| file.path == "未跟踪 file.txt" && file.untracked));
        assert!(files.iter().any(|file| file.path == "staged.txt" && file.staged));
    }

    #[test]
    fn stage_unstage_and_commit_round_trip() {
        let repo = TempRepo::new();
        commit_file(&repo, "base.txt", "base\n", "initial");
        std::fs::write(repo.0.join("next.txt"), "next\n").unwrap();
        let handle = repo.handle();
        handle.stage_files(&["next.txt".into()]).unwrap();
        assert!(handle.working_tree_files().unwrap().iter().any(|file| file.path == "next.txt" && file.staged));
        handle.unstage_files(&["next.txt".into()]).unwrap();
        assert!(handle.working_tree_files().unwrap().iter().any(|file| file.path == "next.txt" && file.untracked));
        handle.stage_files(&["next.txt".into()]).unwrap();
        let hash = handle.commit_staged("add next").unwrap();
        assert!(!hash.is_empty());
        assert!(handle.working_tree_files().unwrap().is_empty());
    }

    #[test]
    fn merge_commit_exposes_multiple_parents_and_changed_files() {
        let repo = TempRepo::new();
        commit_file(&repo, "base.txt", "base\n", "initial");
        let main = repo.handle().current_branch().unwrap();
        git(&repo.0, &["checkout", "-b", "feature"]);
        commit_file(&repo, "feature.txt", "feature\n", "feature work");
        git(&repo.0, &["checkout", &main]);
        commit_file(&repo, "main.txt", "main\n", "main work");
        git(&repo.0, &["merge", "--no-ff", "feature", "-m", "merge feature"]);

        let handle = repo.handle();
        let graph = handle.commit_graph(20).unwrap();
        let merge = graph.iter().find(|node| node.message == "merge feature").unwrap();
        assert_eq!(merge.parents.len(), 2);
        let (stats, files) = handle.diff_stats(&merge.hash).unwrap();
        assert!(stats.files_changed > 0);
        assert!(!files.is_empty());
    }
}
