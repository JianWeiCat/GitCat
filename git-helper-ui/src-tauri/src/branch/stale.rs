//! 休眠分支检测

use crate::git_backend::repo::RepoHandle;

pub struct StaleBranch {
    pub name: String,
    pub last_commit_date: String,
    pub last_commit_author: String,
}

/// 检测长期未提交的休眠分支
pub fn find_stale(repo: &RepoHandle, months: u32) -> Result<Vec<StaleBranch>, crate::error::GhError> {
    let branches = repo.local_branches()?;
    let current = repo.current_branch()?;
    let mut stale = Vec::new();

    for branch in &branches {
        if branch.name == current {
            continue;
        }

        let date_str = &branch.commit_date;
        if is_older_than_months(date_str, months) {
            stale.push(StaleBranch {
                name: branch.name.clone(),
                last_commit_date: branch.commit_date.clone(),
                last_commit_author: branch.author.clone(),
            });
        }
    }

    Ok(stale)
}

/// 简单的日期比较：格式为 YYYY-MM-DD
fn is_older_than_months(date_str: &str, months: u32) -> bool {
    if date_str.is_empty() || date_str.len() < 10 {
        return false;
    }

    // 获取今天的日期
    let today = today_ymd();
    let (year, month, _day) = match parse_date(date_str) {
        Some(d) => d,
        None => return false,
    };

    // 计算月份差
    let month_diff = (today.0 - year) * 12 + (today.1 as i32 - month as i32);
    month_diff >= months as i32
}

fn today_ymd() -> (i32, u32, u32) {
    // 用系统 git 命令获取当前日期
    if let Ok(output) = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
    {
        let s = String::from_utf8_lossy(&output.stdout);
        if let Some(d) = parse_date(s.trim()) {
            return d;
        }
    }
    // fallback
    (2026, 7, 19)
}

fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s[..10].split('-').collect();
    if parts.len() == 3 {
        let year = parts[0].parse().ok()?;
        let month = parts[1].parse().ok()?;
        let day = parts[2].parse().ok()?;
        Some((year, month, day))
    } else {
        None
    }
}
