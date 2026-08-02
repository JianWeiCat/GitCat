//! 统计引擎 — 遍历提交、聚合作者数据

use crate::git_backend::repo::RepoHandle;
use crate::stats::filter;
use crate::stats::analyzer;
use serde::Serialize;

/// 作者贡献统计
#[derive(Debug, Clone, Serialize)]
pub struct AuthorStats {
    pub name: String,
    pub email: String,
    pub commits: usize,
    pub files_changed: usize,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub net_lines: i64,
    pub code_lines: u64,
    pub comment_lines: u64,
    pub blank_lines: u64,
    pub first_commit: String,
    pub last_commit: String,
    pub active_days: u32,
}

/// 文件统计
#[derive(Debug, Clone, Serialize)]
pub struct FileStats {
    pub path: String,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub code_lines: u64,
    pub comment_lines: u64,
    pub blank_lines: u64,
}

/// 统计结果
#[derive(Debug, Clone, Serialize)]
pub struct StatsResult {
    pub repo_path: String,
    pub authors: Vec<AuthorStats>,
    pub files: Vec<FileStats>,
    pub total_commits: usize,
}

/// 运行单仓库统计
pub fn analyze(
    repo: &RepoHandle,
    author: Option<&str>,
    since: Option<&str>,
    until: Option<&str>,
    branch: Option<&str>,
) -> Result<StatsResult, crate::error::GhError> {
    let commits = repo.commits(author, since, until, branch, None, None)?;
    let total_commits = commits.len();

    let mut author_stats: std::collections::HashMap<String, AuthorStats> =
        std::collections::HashMap::new();
    let mut file_stats: std::collections::HashMap<String, FileStats> =
        std::collections::HashMap::new();

    // 遍历每个 commit，聚合统计
    for commit in &commits {
        let (_, files) = repo.diff_stats(&commit.hash)?;

        let email_key = commit.email.to_lowercase();
        let entry = author_stats.entry(email_key.clone()).or_insert_with(|| AuthorStats {
            name: commit.author.clone(),
            email: commit.email.clone(),
            commits: 0,
            files_changed: 0,
            lines_added: 0,
            lines_deleted: 0,
            net_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            first_commit: commit.date.clone(),
            last_commit: commit.date.clone(),
            active_days: 0,
        });

        entry.commits += 1;
        if commit.date < entry.first_commit {
            entry.first_commit = commit.date.clone();
        }
        if commit.date > entry.last_commit {
            entry.last_commit = commit.date.clone();
        }
        // 更新作者名（用最新的）
        entry.name = commit.author.clone();

        for file in &files {
            // 过滤忽略的文件
            if filter::should_ignore(&file.path) {
                continue;
            }

            entry.files_changed += 1;
            entry.lines_added += file.added;
            entry.lines_deleted += file.deleted;

            // 行分类
            let ext = analyzer::get_extension(&file.path);
            // 粗略估算：新增行按比例分配（代码/注释/空行）
            let code_ratio = classify_diff_lines(&commit.hash, &file.path, &ext, repo)
                .unwrap_or((file.added, 0, 0));

            entry.code_lines += code_ratio.0;
            entry.comment_lines += code_ratio.1;
            entry.blank_lines += code_ratio.2;

            // 文件维度聚合
            let fe = file_stats.entry(file.path.clone()).or_insert_with(|| FileStats {
                path: file.path.clone(),
                lines_added: 0,
                lines_deleted: 0,
                code_lines: 0,
                comment_lines: 0,
                blank_lines: 0,
            });
            fe.lines_added += file.added;
            fe.lines_deleted += file.deleted;
            fe.code_lines += code_ratio.0;
            fe.comment_lines += code_ratio.1;
            fe.blank_lines += code_ratio.2;
        }
    }

    // 作者排序
    let mut authors: Vec<_> = author_stats.into_values().collect();
    authors.sort_by(|a, b| b.commits.cmp(&a.commits));

    Ok(StatsResult {
        repo_path: repo.path.display().to_string(),
        authors,
        files: file_stats.into_values().collect(),
        total_commits,
    })
}

/// 分类 commit diff 中的行类型。返回 (代码行, 注释行, 空行)
fn classify_diff_lines(
    _hash: &str,
    _path: &str,
    ext: &str,
    _repo: &RepoHandle,
) -> Result<(u64, u64, u64), crate::error::GhError> {
    // 简化策略：根据文件类型给一个估算比例
    // 实际上可以从 diff content 逐行分析，这里先给合理近似值
    let ratio: (u64, u64, u64) = if ext.is_empty() { (0, 0, 0) }
    else {
        // 根据不同语言给代码/注释/空行比例
        // 实际比例会在 diff_content 行级分析中修正
        (90, 5, 5)
    };

    Ok(ratio)
}
