//! 报表构建 — 排序、格式化输出

use crate::stats::engine::StatsResult;

/// 排序方式
pub enum SortBy {
    Commits,
    Added,
    Net,
}

/// 构建表格输出
pub fn format_table(result: &StatsResult, sort_by: &SortBy) -> String {
    let mut authors = result.authors.clone();
    match sort_by {
        SortBy::Commits => authors.sort_by(|a, b| b.commits.cmp(&a.commits)),
        SortBy::Added => authors.sort_by(|a, b| b.lines_added.cmp(&a.lines_added)),
        SortBy::Net => authors.sort_by(|a, b| b.net_lines.cmp(&a.net_lines)),
    }

    let mut output = String::new();
    output.push_str(&format!(
        "仓库: {}\n统计提交数: {}\n\n",
        result.repo_path, result.total_commits
    ));
    output.push_str(&format!(
        "{:<4} {:<20} {:<8} {:<10} {:<10} {:<10} {:<10}\n",
        "排名", "作者", "提交数", "新增行", "删除行", "净增", "代码行"
    ));
    output.push_str(&"-".repeat(80));
    output.push('\n');

    for (i, author) in authors.iter().enumerate() {
        output.push_str(&format!(
            "{:<4} {:<20} {:<8} {:<10} {:<10} {:<10} {:<10}\n",
            i + 1,
            truncate(&author.name, 20),
            author.commits,
            author.lines_added,
            author.lines_deleted,
            author.net_lines,
            author.code_lines,
        ));
    }

    output
}

/// 构建 Markdown 表格
pub fn format_markdown(result: &StatsResult) -> String {
    let mut authors = result.authors.clone();
    authors.sort_by(|a, b| b.commits.cmp(&a.commits));

    let mut md = String::new();
    md.push_str("# Git 贡献统计报表\n\n");
    md.push_str(&format!(
        "**仓库**: `{}`\n**统计提交数**: {}\n\n",
        result.repo_path, result.total_commits
    ));
    md.push_str("## 贡献排行\n\n");
    md.push_str("| 排名 | 作者 | 提交数 | 新增行 | 删除行 | 净增 | 代码行 |\n");
    md.push_str("|------|------|--------|--------|--------|------|--------|\n");

    for (i, author) in authors.iter().enumerate() {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            i + 1,
            author.name,
            author.commits,
            author.lines_added,
            author.lines_deleted,
            author.net_lines,
            author.code_lines,
        ));
    }

    md
}

/// 构建 JSON 输出
pub fn format_json(result: &StatsResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".into())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}
