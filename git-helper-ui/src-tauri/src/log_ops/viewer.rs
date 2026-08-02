//! 彩色精简日志查看器

use crate::git_backend::repo::CommitInfo;

/// 格式化提交日志为彩色表格
pub fn format_log(commits: &[CommitInfo]) -> String {
    if commits.is_empty() {
        return "没有找到匹配的提交记录。\n".into();
    }

    let mut output = String::new();

    for commit in commits {
        output.push_str(&format!(
            "◉ {}  {}  {}\n",
            commit.hash,
            truncate(&commit.author, 15),
            commit.date,
        ));
        output.push_str(&format!("│  {}\n", commit.message));
        output.push('\n');
    }

    output.push_str(&format!("共 {} 条提交记录\n", commits.len()));
    output
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}
