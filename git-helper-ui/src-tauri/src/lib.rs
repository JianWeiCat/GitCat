mod config;
mod error;
mod git_backend;
mod stats;
mod branch;
mod log_ops;
mod multi_repo;
mod safety;

use git_backend::repo::RepoHandle;
use serde::Serialize;
use std::path::Path;
use std::process::Command;
use tauri::State;

// ============================================================
// 状态管理
// ============================================================

struct AppState {
    repo_path: std::sync::Mutex<Option<String>>,
}

// ============================================================
// 响应类型（返回给前端）
// ============================================================

#[derive(Debug, Serialize)]
struct StatsResponse {
    repo_path: String,
    total_commits: usize,
    authors: Vec<AuthorResponse>,
}

#[derive(Debug, Serialize)]
struct AuthorResponse {
    name: String,
    email: String,
    commits: usize,
    lines_added: u64,
    lines_deleted: u64,
    net_lines: i64,
    code_lines: u64,
    comment_lines: u64,
    blank_lines: u64,
    first_commit: String,
    last_commit: String,
}

#[derive(Debug, Serialize)]
struct OverviewResponse {
    path: String,
    total_commits: u64,
    total_branches: usize,
    total_tags: usize,
    contributors: usize,
    head_branch: String,
    last_commit_date: String,
}

#[derive(Debug, Serialize)]
struct BranchInfoResponse {
    name: String,
    is_head: bool,
    commit_hash: String,
    commit_message: String,
    commit_date: String,
    author: String,
    is_protected: bool,
    can_delete: bool,
}

#[derive(Debug, Serialize)]
struct CommitResponse {
    hash: String,
    author: String,
    email: String,
    date: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct CommitGraphResponse {
    hash: String,
    parents: Vec<String>,
    refs: Vec<String>,
    author: String,
    date: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct LanguageResponse {
    language: String,
    bytes: u64,
    files: usize,
}

#[derive(Debug, Serialize)]
struct RepoStatusResponse {
    branch: String,
    modified: usize,
    staged: usize,
    untracked: usize,
    conflicted: usize,
    ahead: usize,
    behind: usize,
}

#[derive(Debug, Serialize)]
struct WorkingTreeFileResponse {
    path: String,
    index_status: char,
    worktree_status: char,
    staged: bool,
    modified: bool,
    untracked: bool,
    conflicted: bool,
}

#[derive(Debug, Serialize)]
struct CommitFilesResponse {
    files_changed: usize,
    insertions: u64,
    deletions: u64,
    files: Vec<FileChangeResponse>,
}

#[derive(Debug, Serialize)]
struct FileChangeResponse {
    path: String,
    added: u64,
    deleted: u64,
}

#[derive(Debug, Serialize)]
struct MultiRepoResponse {
    repo_path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct StaleBranchResponse {
    name: String,
    last_commit_date: String,
    last_commit_author: String,
}

#[derive(Debug, Serialize)]
struct AuditEntryResponse {
    id: String,
    timestamp: String,
    operation: String,
    repository: String,
    dry_run: bool,
    result: String,
}

#[derive(Debug, Serialize)]
struct RepoScanResponse {
    repos: Vec<String>,
    roots: Vec<String>,
}

// ============================================================
// 辅助函数
// ============================================================

fn open_repo(path: &str) -> Result<RepoHandle, String> {
    RepoHandle::open(Path::new(path)).map_err(|e| e.to_string())
}

// ============================================================
// Tauri Commands
// ============================================================

/// 打开仓库
#[tauri::command]
fn open_repository(path: String, state: State<AppState>) -> Result<OverviewResponse, String> {
    let repo = open_repo(&path)?;
    let overview = repo.overview().map_err(|e| e.to_string())?;

    if let Ok(mut p) = state.repo_path.lock() {
        *p = Some(path.clone());
    }

    Ok(OverviewResponse {
        path: overview.path,
        total_commits: overview.total_commits,
        total_branches: overview.total_branches,
        total_tags: overview.total_tags,
        contributors: overview.contributors,
        head_branch: overview.head_branch,
        last_commit_date: overview.last_commit_date,
    })
}

/// 仓库概览
#[tauri::command]
fn get_overview(path: String) -> Result<OverviewResponse, String> {
    let repo = open_repo(&path)?;
    let overview = repo.overview().map_err(|e| e.to_string())?;
    Ok(OverviewResponse {
        path: overview.path,
        total_commits: overview.total_commits,
        total_branches: overview.total_branches,
        total_tags: overview.total_tags,
        contributors: overview.contributors,
        head_branch: overview.head_branch,
        last_commit_date: overview.last_commit_date,
    })
}

/// 贡献统计
#[tauri::command]
fn get_stats(path: String, author: Option<String>, since: Option<String>, until: Option<String>) -> Result<StatsResponse, String> {
    let repo = open_repo(&path)?;
    let result = stats::engine::analyze(
        &repo,
        author.as_deref(),
        since.as_deref(),
        until.as_deref(),
        None,
    )
    .map_err(|e| e.to_string())?;

    let authors: Vec<AuthorResponse> = result
        .authors
        .into_iter()
        .map(|a| AuthorResponse {
            name: a.name,
            email: a.email,
            commits: a.commits,
            lines_added: a.lines_added,
            lines_deleted: a.lines_deleted,
            net_lines: a.net_lines,
            code_lines: a.code_lines,
            comment_lines: a.comment_lines,
            blank_lines: a.blank_lines,
            first_commit: a.first_commit,
            last_commit: a.last_commit,
        })
        .collect();

    Ok(StatsResponse {
        repo_path: result.repo_path,
        total_commits: result.total_commits,
        authors,
    })
}

/// 提交日志
#[tauri::command]
fn get_commits(
    path: String,
    author: Option<String>,
    since: Option<String>,
    until: Option<String>,
    grep: Option<String>,
    max_count: Option<usize>,
) -> Result<Vec<CommitResponse>, String> {
    let repo = open_repo(&path)?;
    let commits = repo
        .commits(
            author.as_deref(),
            since.as_deref(),
            until.as_deref(),
            None,
            grep.as_deref(),
            max_count,
        )
        .map_err(|e| e.to_string())?;

    Ok(commits
        .into_iter()
        .map(|c| CommitResponse {
            hash: c.hash,
            author: c.author,
            email: c.email,
            date: c.date,
            message: c.message,
        })
        .collect())
}

/// 获取用于只读提交图的拓扑节点。
#[tauri::command]
fn get_commit_graph(path: String, max_count: Option<usize>) -> Result<Vec<CommitGraphResponse>, String> {
    let repo = open_repo(&path)?;
    Ok(repo
        .commit_graph(max_count.unwrap_or(120))
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|node| CommitGraphResponse {
            hash: node.hash,
            parents: node.parents,
            refs: node.refs,
            author: node.author,
            date: node.date,
            message: node.message,
        })
        .collect())
}

/// 当前检出版本中各语言所占的文件体积，用于单仓库代码构成图。
#[tauri::command]
fn get_language_stats(path: String) -> Result<Vec<LanguageResponse>, String> {
    let repo = open_repo(&path)?;
    Ok(repo
        .language_stats()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|item| LanguageResponse {
            language: item.language,
            bytes: item.bytes,
            files: item.files,
        })
        .collect())
}

#[tauri::command]
fn get_repo_status(path: String) -> Result<RepoStatusResponse, String> {
    let repo = open_repo(&path)?;
    let status = repo.status().map_err(|e| e.to_string())?;
    let conflicted = repo.working_tree_files().map_err(|e| e.to_string())?
        .into_iter().filter(|file| file.conflicted).count();
    Ok(RepoStatusResponse {
        branch: status.branch,
        modified: status.modified,
        staged: status.staged,
        untracked: status.untracked,
        conflicted,
        ahead: status.ahead,
        behind: status.behind,
    })
}

#[tauri::command]
fn get_working_tree(path: String) -> Result<Vec<WorkingTreeFileResponse>, String> {
    let repo = open_repo(&path)?;
    Ok(repo.working_tree_files().map_err(|e| e.to_string())?.into_iter().map(|file| {
        WorkingTreeFileResponse {
            path: file.path,
            index_status: file.index_status,
            worktree_status: file.worktree_status,
            staged: file.staged,
            modified: file.modified,
            untracked: file.untracked,
            conflicted: file.conflicted,
        }
    }).collect())
}

#[tauri::command]
fn get_working_tree_diff(path: String, file_path: String, staged: bool) -> Result<String, String> {
    let repo = open_repo(&path)?;
    repo.working_tree_diff(&file_path, staged).map_err(|e| e.to_string())
}

#[tauri::command]
fn stage_files(path: String, paths: Vec<String>) -> Result<(), String> {
    let repo = open_repo(&path)?;
    repo.stage_files(&paths).map_err(|e| e.to_string())
}

#[tauri::command]
fn unstage_files(path: String, paths: Vec<String>) -> Result<(), String> {
    let repo = open_repo(&path)?;
    repo.unstage_files(&paths).map_err(|e| e.to_string())
}

#[tauri::command]
fn commit_staged(path: String, message: String) -> Result<String, String> {
    let repo = open_repo(&path)?;
    repo.commit_staged(&message).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_commit_files(path: String, commit_hash: String) -> Result<CommitFilesResponse, String> {
    let repo = open_repo(&path)?;
    let (stats, files) = repo.diff_stats(&commit_hash).map_err(|e| e.to_string())?;
    Ok(CommitFilesResponse {
        files_changed: stats.files_changed,
        insertions: stats.insertions,
        deletions: stats.deletions,
        files: files.into_iter().map(|file| FileChangeResponse {
            path: file.path,
            added: file.added,
            deleted: file.deleted,
        }).collect(),
    })
}

/// 获取分支列表
#[tauri::command]
fn get_branches(path: String) -> Result<Vec<BranchInfoResponse>, String> {
    let repo = open_repo(&path)?;
    let branches = repo.local_branches().map_err(|e| e.to_string())?;
    let current = repo.current_branch().unwrap_or_default();
    let whitelist = branch::whitelist::WhitelistConfig::load(None).unwrap_or_default();

    Ok(branches
        .into_iter()
        .map(|b| {
            let is_protected = whitelist.protected.contains(&b.name)
                || ["main", "master", "develop"].contains(&b.name.as_str());
            BranchInfoResponse {
                name: b.name.clone(),
                is_head: b.is_head,
                commit_hash: b.commit_hash,
                commit_message: b.commit_message,
                commit_date: b.commit_date,
                author: b.author,
                is_protected,
                can_delete: !b.is_head && !is_protected && b.name != current,
            }
        })
        .collect())
}

/// 获取可清理分支列表
#[tauri::command]
fn get_cleanable_branches(path: String) -> Result<Vec<branch::cleanup::CleanableBranchResponse>, String> {
    let repo = open_repo(&path)?;
    let whitelist = branch::whitelist::WhitelistConfig::load(None).unwrap_or_default();
    let branches = branch::cleanup::scan_cleanable(&repo, false, &whitelist.protected)
        .map_err(|e| e.to_string())?;

    // Convert to serializable type
    Ok(branches.into_iter().map(|b| branch::cleanup::CleanableBranchResponse {
        name: b.name,
        last_commit_date: b.last_commit_date,
        last_commit_message: b.last_commit_message,
        reason: b.reason,
        is_protected: b.is_protected,
    }).collect())
}

/// 删除分支
#[tauri::command]
fn delete_branch(path: String, branch_name: String, force: bool) -> Result<String, String> {
    let repo = open_repo(&path)?;
    repo.delete_branch(&branch_name, force).map_err(|e| e.to_string())?;
    Ok(format!("已删除分支: {}", branch_name))
}

#[tauri::command]
fn checkout_branch(path: String, branch_name: String) -> Result<String, String> {
    let repo = open_repo(&path)?;
    repo.checkout(&branch_name).map_err(|e| e.to_string())?;
    Ok(format!("已切换到分支: {}", branch_name))
}

#[tauri::command]
fn create_branch(path: String, branch_name: String, checkout: bool) -> Result<String, String> {
    let repo = open_repo(&path)?;
    repo.create_branch(&branch_name).map_err(|e| e.to_string())?;
    if checkout {
        repo.checkout(&branch_name).map_err(|e| e.to_string())?;
    }
    Ok(format!("已创建分支: {}", branch_name))
}

#[tauri::command]
fn sync_repository(path: String, action: String) -> Result<String, String> {
    let repo = open_repo(&path)?;
    match action.as_str() {
        "fetch" => repo.fetch().map_err(|e| e.to_string()),
        "pull" => repo.pull(false).map_err(|e| e.to_string()),
        "push" => repo.push().map_err(|e| e.to_string()),
        _ => Err(format!("未知同步操作: {}", action)),
    }
}

/// 获取休眠分支
#[tauri::command]
fn get_stale_branches(path: String, months: u32) -> Result<Vec<StaleBranchResponse>, String> {
    let repo = open_repo(&path)?;
    let stale = branch::stale::find_stale(&repo, months).map_err(|e| e.to_string())?;
    Ok(stale
        .into_iter()
        .map(|s| StaleBranchResponse {
            name: s.name,
            last_commit_date: s.last_commit_date,
            last_commit_author: s.last_commit_author,
        })
        .collect())
}

/// 扫描多仓库
#[tauri::command]
fn scan_repos(path: String, depth: usize) -> Result<RepoScanResponse, String> {
    let root = Path::new(&path);
    let repos = multi_repo::scanner::scan_repos(root, depth).map_err(|e| e.to_string())?;
    Ok(RepoScanResponse {
        repos: repos.iter().map(|r| r.display().to_string()).collect(),
        roots: vec![path],
    })
}

/// 从可用盘符发现仓库；扫描器会排除系统、依赖与隐藏目录。
#[tauri::command]
fn scan_all_disks(depth: usize) -> Result<RepoScanResponse, String> {
    let repos = multi_repo::scanner::scan_system_drives(depth).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    let roots = (b'A'..=b'Z')
        .map(|letter| format!("{}:\\", letter as char))
        .filter(|root| Path::new(root).is_dir())
        .collect();
    #[cfg(not(windows))]
    let roots = vec!["/".to_string()];
    Ok(RepoScanResponse {
        repos: repos.iter().map(|repo| repo.display().to_string()).collect(),
        roots,
    })
}

/// Open the native Windows folder picker so scanning does not require typing a path.
#[tauri::command]
fn pick_folder() -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        let script = r#"
            Add-Type -AssemblyName System.Windows.Forms
            $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
            $dialog.Description = '选择要扫描的文件夹'
            $dialog.ShowNewFolderButton = $false
            if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                [Console]::Write($dialog.SelectedPath)
            }
        "#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-STA", "-Command", script])
            .output()
            .map_err(|error| format!("无法打开文件夹选择器: {error}"))?;
        if !output.status.success() {
            return Err("文件夹选择器未能启动".into());
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok((!path.is_empty()).then_some(path));
    }
    #[cfg(not(windows))]
    {
        Err("当前系统暂不支持原生文件夹选择器".into())
    }
}

/// 批量操作多仓库
#[tauri::command]
fn multi_operation(path: String, depth: usize, operation: String) -> Result<Vec<MultiRepoResponse>, String> {
    let root = Path::new(&path);
    let repos = multi_repo::scanner::scan_repos(root, depth).map_err(|e| e.to_string())?;

    let results = multi_repo::runner::run_parallel(&repos, |handle| match operation.as_str() {
        "pull" => multi_repo::ops::batch_pull(handle, false).map(|s| s.to_string()),
        "status" => multi_repo::ops::batch_status(handle),
        "gc" => multi_repo::ops::batch_gc(handle, false).map(|s| s.to_string()),
        _ => Err(error::GhError::Args(format!("未知操作: {}", operation))),
    });

    Ok(results
        .into_iter()
        .map(|r| MultiRepoResponse {
            repo_path: r.repo_path,
            success: r.success,
            message: r.message,
        })
        .collect())
}

/// 仅对操作计划中明确勾选的仓库执行，避免重新扫描后扩大影响范围。
#[tauri::command]
fn run_selected_operation(paths: Vec<String>, operation: String) -> Result<Vec<MultiRepoResponse>, String> {
    if paths.is_empty() { return Err("请至少选择一个仓库".into()); }
    let repos = paths.into_iter().map(std::path::PathBuf::from).collect::<Vec<_>>();
    let results = multi_repo::runner::run_parallel(&repos, |handle| match operation.as_str() {
        "pull" => multi_repo::ops::batch_pull(handle, false),
        "status" => multi_repo::ops::batch_status(handle),
        "gc" => multi_repo::ops::batch_gc(handle, false),
        _ => Err(error::GhError::Args(format!("未知操作: {}", operation))),
    });
    Ok(results.into_iter().map(|result| MultiRepoResponse {
        repo_path: result.repo_path,
        success: result.success,
        message: result.message,
    }).collect())
}

/// 重置操作
#[tauri::command]
fn reset_operation(path: String, target: String, mode: String, backup: bool) -> Result<String, String> {
    let repo = open_repo(&path)?;
    let reset_mode = match mode.as_str() {
        "soft" => log_ops::reset::ResetMode::Soft,
        "mixed" => log_ops::reset::ResetMode::Mixed,
        "hard" => log_ops::reset::ResetMode::Hard,
        _ => return Err("未知重置模式".into()),
    };
    log_ops::reset::execute_reset(&repo, &target, reset_mode, backup).map_err(|e| e.to_string())
}

/// Stash 列表
#[tauri::command]
fn get_stashes(path: String) -> Result<String, String> {
    let repo = open_repo(&path)?;
    log_ops::stash::list_stashes(&repo).map_err(|e| e.to_string())
}

/// 审计日志
#[tauri::command]
fn get_audit_log() -> Result<Vec<AuditEntryResponse>, String> {
    let log = safety::audit::AuditLog::open().map_err(|e| e.to_string())?;
    let entries = log.read_all().map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| AuditEntryResponse {
            id: e.id,
            timestamp: e.timestamp,
            operation: e.operation,
            repository: e.repository,
            dry_run: e.dry_run,
            result: e.result,
        })
        .collect())
}

/// 白名单管理
#[tauri::command]
fn get_whitelist() -> Result<Vec<String>, String> {
    let wl = branch::whitelist::WhitelistConfig::load(None).map_err(|e| e.to_string())?;
    Ok(wl.protected)
}

#[tauri::command]
fn add_whitelist(branch_name: String) -> Result<(), String> {
    let mut wl = branch::whitelist::WhitelistConfig::load(None).map_err(|e| e.to_string())?;
    wl.add(branch_name);
    wl.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_whitelist(branch_name: String) -> Result<(), String> {
    let mut wl = branch::whitelist::WhitelistConfig::load(None).map_err(|e| e.to_string())?;
    wl.remove(&branch_name);
    wl.save().map_err(|e| e.to_string())
}

// ============================================================
// 应用入口
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            repo_path: std::sync::Mutex::new(None),
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_repository,
            get_overview,
            get_stats,
            get_commits,
            get_commit_graph,
            get_language_stats,
            get_repo_status,
            get_working_tree,
            get_working_tree_diff,
            stage_files,
            unstage_files,
            commit_staged,
            get_commit_files,
            get_branches,
            get_cleanable_branches,
            delete_branch,
            checkout_branch,
            create_branch,
            sync_repository,
            get_stale_branches,
            scan_repos,
            scan_all_disks,
            pick_folder,
            multi_operation,
            run_selected_operation,
            reset_operation,
            get_stashes,
            get_audit_log,
            get_whitelist,
            add_whitelist,
            remove_whitelist,
        ])
        .run(tauri::generate_context!())
        .expect("启动 GitCat 失败");
}
