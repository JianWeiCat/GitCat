//! CLI 命令树定义 (clap v4 Derive 模式)

use clap::{Parser, Subcommand, ValueHint};

/// git-helper (gh) — 纯 Rust 跨平台 Git 仓库增强工具箱
#[derive(Parser, Debug)]
#[command(
    name = "gh",
    about = "Git Helper - 一站式 Git 仓库增强工具箱",
    long_about = "纯 Rust 单二进制、跨平台本地 Git 仓库增强 CLI 工具箱。\n\n\
                  主打一行式快捷命令、批量多仓库操作、代码统计报表、安全预演机制。\n\
                  使用 gh <SUBCOMMAND> --help 查看各子命令详细用法。",
    version,
    disable_help_subcommand = true,
    arg_required_else_help = true,
)]
pub struct Cli {
    /// 子命令
    #[command(subcommand)]
    pub command: Commands,

    /// 仓库路径（默认当前目录）
    #[arg(short = 'r', long, global = true, value_hint = ValueHint::DirPath)]
    pub repo: Option<String>,

    /// 多仓扫描根目录
    #[arg(short = 'R', long, global = true, value_hint = ValueHint::DirPath)]
    pub root: Option<String>,

    /// 扫描深度
    #[arg(short = 'd', long, global = true, default_value = "3")]
    pub depth: usize,

    /// 预演模式（不实际执行修改）
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,

    /// 跳过确认提示
    #[arg(short = 'f', long, global = true)]
    pub force: bool,

    /// 静默输出
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// 调试模式
    #[arg(long, global = true)]
    pub debug: bool,

    /// 禁用彩色输出
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 输出格式
    #[arg(long, global = true, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// 输出到文件
    #[arg(short = 'o', long, global = true, value_hint = ValueHint::FilePath)]
    pub output: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 仓库贡献统计（代码行数、作者排行）
    #[command(visible_alias = "st")]
    Stats(StatsArgs),

    /// 分支批量管理（清理、切换、重命名）
    #[command(visible_alias = "br")]
    Branch(BranchArgs),

    /// 提交日志查看
    Log(LogArgs),

    /// 安全重置操作
    Reset(ResetArgs),

    /// Stash 缓存管理
    Stash(StashArgs),

    /// 多仓库批量运维
    #[command(visible_alias = "mr")]
    Multi(MultiArgs),

    /// 审计日志
    Audit(AuditArgs),

    /// 生成 Shell 补全脚本
    #[command(visible_alias = "comp")]
    Completions(CompletionsArgs),

    /// 配置管理
    Config(ConfigArgs),
}

// ============================================================================
// 模块一：仓库贡献统计
// ============================================================================

/// 仓库贡献统计
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct StatsArgs {
    #[command(subcommand)]
    pub command: Option<StatsSubcommand>,

    /// 按作者过滤
    #[arg(long)]
    pub author: Option<String>,

    /// 起止日期
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,

    /// 指定分支
    #[arg(long)]
    pub branch: Option<String>,

    /// 排序方式
    #[arg(long, value_enum, default_value = "commits")]
    pub sort_by: Option<SortBy>,
}

#[derive(Debug, Subcommand)]
pub enum StatsSubcommand {
    /// 导出统计报表
    Report(ReportArgs),
    /// 多仓库批量统计
    Multi(MultiStatsArgs),
    /// 仓库概览
    Overview(OverviewArgs),
}

#[derive(Debug, clap::Args)]
pub struct ReportArgs {
    /// 输出文件路径
    #[arg(short, long)]
    pub output: Option<String>,
    /// 导出为 Markdown
    #[arg(long)]
    pub md: bool,
    /// 导出为 CSV
    #[arg(long)]
    pub csv: bool,
}

#[derive(Debug, clap::Args)]
pub struct MultiStatsArgs {
    /// 扫描根目录
    #[arg(short = 'R', long)]
    pub root: String,
    /// 扫描深度
    #[arg(short, long, default_value = "3")]
    pub depth: usize,
    /// 按作者过滤
    #[arg(long)]
    pub author: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct OverviewArgs {
    /// 仓库路径
    #[arg(short, long)]
    pub repo: Option<String>,
}

// ============================================================================
// 模块二：分支管理
// ============================================================================

/// 分支批量管理
#[derive(Debug, clap::Args)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub command: BranchSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum BranchSubcommand {
    /// 清理已合并的废弃分支
    Cleanup(CleanupArgs),
    /// 批量切换多仓库至同一分支
    SwitchAll(SwitchAllArgs),
    /// 正则批量重命名本地分支
    Rename(BranchRenameArgs),
    /// 检测长期无提交的休眠分支
    Stale(StaleArgs),
    /// 白名单管理（保护指定分支不被清理）
    Whitelist(WhitelistArgs),
}

#[derive(Debug, clap::Args)]
pub struct CleanupArgs {
    /// 同时清理远程已合并分支
    #[arg(long)]
    pub remote: bool,
    /// 白名单配置文件路径
    #[arg(long)]
    pub whitelist: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct SwitchAllArgs {
    /// 扫描根目录
    #[arg(short = 'R', long)]
    pub root: String,
    /// 目标分支名
    #[arg(short, long)]
    pub branch: String,
}

#[derive(Debug, clap::Args)]
pub struct BranchRenameArgs {
    /// 正则匹配模式
    #[arg(short, long)]
    pub pattern: String,
    /// 替换字符串
    #[arg(short, long)]
    pub replace: String,
}

#[derive(Debug, clap::Args)]
pub struct StaleArgs {
    /// 休眠月数阈值（默认 3）
    #[arg(short, long, default_value = "3")]
    pub months: u32,
}

#[derive(Debug, clap::Args)]
pub struct WhitelistArgs {
    #[command(subcommand)]
    pub command: WhitelistSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum WhitelistSubcommand {
    /// 显示当前白名单
    Show,
    /// 添加保护分支
    Add(WhitelistModifyArgs),
    /// 移除保护分支
    Remove(WhitelistModifyArgs),
}

#[derive(Debug, clap::Args)]
pub struct WhitelistModifyArgs {
    /// 分支名称
    pub branch: String,
}

// ============================================================================
// 模块三：日志与操作
// ============================================================================

/// 提交日志查看
#[derive(Debug, clap::Args)]
pub struct LogArgs {
    /// 按作者过滤
    #[arg(long)]
    pub author: Option<String>,
    /// 起止日期
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,
    /// 关键词搜索
    #[arg(long)]
    pub grep: Option<String>,
    /// 最大显示条数
    #[arg(short = 'm', long)]
    pub max_count: Option<usize>,
}

/// 安全重置操作
#[derive(Debug, clap::Args)]
pub struct ResetArgs {
    #[command(subcommand)]
    pub command: ResetSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ResetSubcommand {
    /// 软重置（仅移动 HEAD）
    Soft(ResetTargetArgs),
    /// 混合重置（默认模式：移动 HEAD + 重置暂存区）
    Mixed(ResetTargetArgs),
    /// 硬重置（⚠️ 危险：移动 HEAD + 重置暂存区 + 重置工作区）
    Hard(ResetTargetArgs),
}

#[derive(Debug, clap::Args)]
pub struct ResetTargetArgs {
    /// 目标提交（hash / branch / tag / HEAD~N）
    pub target: String,
    /// 操作前自动创建备份分支
    #[arg(long)]
    pub backup: bool,
}

/// Stash 缓存管理
#[derive(Debug, clap::Args)]
pub struct StashArgs {
    #[command(subcommand)]
    pub command: StashSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum StashSubcommand {
    /// 列出所有 stash
    List,
    /// 批量清理过期 stash
    Clean(StashCleanArgs),
}

#[derive(Debug, clap::Args)]
pub struct StashCleanArgs {
    /// 清理 N 天前的 stash
    #[arg(long, default_value = "30")]
    pub older_than: u32,
}

// ============================================================================
// 模块四：多仓库批量运维
// ============================================================================

/// 多仓库批量运维
#[derive(Debug, clap::Args)]
pub struct MultiArgs {
    #[command(subcommand)]
    pub command: MultiSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MultiSubcommand {
    /// 扫描目录下所有 Git 仓库
    Scan(ScanArgs),
    /// 批量 git pull
    Pull(PullArgs),
    /// 批量 git gc（垃圾回收）
    Gc(GcArgs),
    /// 批量状态检查
    Status(StatusArgs),
}

#[derive(Debug, clap::Args)]
pub struct ScanArgs {
    /// 扫描根目录
    #[arg(short = 'R', long, default_value = ".")]
    pub root: String,
    /// 扫描深度
    #[arg(short, long, default_value = "3")]
    pub depth: usize,
}

#[derive(Debug, clap::Args)]
pub struct PullArgs {
    /// 扫描根目录
    #[arg(short = 'R', long, default_value = ".")]
    pub root: String,
    /// 使用 rebase 模式
    #[arg(long)]
    pub rebase: bool,
}

#[derive(Debug, clap::Args)]
pub struct GcArgs {
    /// 扫描根目录
    #[arg(short = 'R', long, default_value = ".")]
    pub root: String,
    /// 激进模式（更彻底的垃圾回收）
    #[arg(long)]
    pub aggressive: bool,
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    /// 扫描根目录
    #[arg(short = 'R', long, default_value = ".")]
    pub root: String,
    /// 简洁输出
    #[arg(short, long)]
    pub short: bool,
}

// ============================================================================
// 模块五：安全与审计
// ============================================================================

/// 审计日志
#[derive(Debug, clap::Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub command: AuditSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AuditSubcommand {
    /// 查看操作日志
    Log,
    /// 尝试撤销某次操作
    Revert(AuditRevertArgs),
}

#[derive(Debug, clap::Args)]
pub struct AuditRevertArgs {
    /// 操作 ID
    pub id: String,
}

/// 生成 Shell 补全脚本
#[derive(Debug, clap::Args)]
pub struct CompletionsArgs {
    /// Shell 类型
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

/// 配置管理
#[derive(Debug, clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// 显示当前配置
    Show,
    /// 设置配置项
    Set(ConfigSetArgs),
}

#[derive(Debug, clap::Args)]
pub struct ConfigSetArgs {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: String,
}

// ============================================================================
// 共享枚举
// ============================================================================

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// 终端表格
    Table,
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// Markdown 格式
    Md,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SortBy {
    /// 按提交数排序
    Commits,
    /// 按新增行数排序
    Added,
    /// 按净增行数排序
    Net,
    /// 按活跃天数排序
    Active,
}

// ============================================================================
// 入口函数
// ============================================================================

/// 命令执行的上下文参数
pub struct CmdCtx {
    pub repo_path: Option<String>,
    #[allow(dead_code)]
    pub root: Option<String>,
    pub depth: usize,
    pub dry_run: bool,
    pub force: bool,
    pub quiet: bool,
    #[allow(dead_code)]
    pub format: OutputFormat,
    #[allow(dead_code)]
    pub output: Option<String>,
}

pub fn run() -> miette::Result<()> {
    let cli = Cli::parse();

    // 提取全局参数，避免 match 中的部分移动问题
    let repo_path = cli.repo.clone();
    let root = cli.root.clone();
    let depth = cli.depth;
    let dry_run = cli.dry_run;
    let force = cli.force;
    let quiet = cli.quiet;
    let format = cli.format.clone();
    let output = cli.output.clone();

    let ctx = CmdCtx {
        repo_path,
        root,
        depth,
        dry_run,
        force,
        quiet,
        format,
        output,
    };

    let result = match cli.command {
        Commands::Stats(args) => cmd_stats(args, &ctx),
        Commands::Branch(args) => cmd_branch(args, &ctx),
        Commands::Log(args) => cmd_log(args, &ctx),
        Commands::Reset(args) => cmd_reset(args, &ctx),
        Commands::Stash(args) => cmd_stash(args, &ctx),
        Commands::Multi(args) => cmd_multi(args, &ctx),
        Commands::Audit(args) => cmd_audit(args, &ctx),
        Commands::Completions(args) => cmd_completions(args),
        Commands::Config(args) => cmd_config(args, &ctx),
    };

    match result {
        Ok(msg) => {
            if !quiet && !msg.is_empty() {
                println!("{}", msg);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}

fn get_repo(ctx: &CmdCtx) -> Result<crate::git_backend::repo::RepoHandle, crate::error::GhError> {
    let path = ctx.repo_path.as_deref().unwrap_or(".");
    crate::git_backend::repo::RepoHandle::open(std::path::Path::new(path))
}

// ---- stats ----

fn cmd_stats(args: StatsArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    if let Some(sub) = args.command {
        return match sub {
            StatsSubcommand::Report(r) => {
                let repo = get_repo(ctx)?;
                let result = crate::stats::engine::analyze(
                    &repo, args.author.as_deref(), args.since.as_deref(),
                    args.until.as_deref(), args.branch.as_deref(),
                )?;
                let format = if r.md { crate::stats::exporter::ExportFormat::Markdown }
                    else if r.csv { crate::stats::exporter::ExportFormat::Csv }
                    else { crate::stats::exporter::ExportFormat::Json };

                let output_path = r.output.as_deref().unwrap_or("stats-report.md");
                crate::stats::exporter::export(&result, output_path, format)?;
                Ok(format!("报表已导出: {}", output_path))
            }
            StatsSubcommand::Multi(m) => {
                let root = std::path::Path::new(&m.root);
                let repos = crate::multi_repo::scanner::scan_repos(root, m.depth)?;
                let handles: Vec<_> = repos.iter().filter_map(|p| {
                    crate::git_backend::repo::RepoHandle::open(p).ok().map(|h| (p.display().to_string(), h))
                }).collect();

                let mut all_authors: Vec<crate::stats::engine::AuthorStats> = Vec::new();
                for (_, handle) in &handles {
                    if let Ok(result) = crate::stats::engine::analyze(
                        handle, m.author.as_deref(), None, None, None,
                    ) {
                        all_authors.extend(result.authors);
                    }
                }
                // 合并同名作者
                let mut merged: std::collections::HashMap<String, crate::stats::engine::AuthorStats> =
                    std::collections::HashMap::new();
                for a in all_authors {
                    let key = a.email.to_lowercase();
                    let entry = merged.entry(key).or_insert_with(|| a.clone());
                    if a.email != entry.email { continue; }
                    entry.commits += a.commits;
                    entry.lines_added += a.lines_added;
                    entry.lines_deleted += a.lines_deleted;
                    entry.code_lines += a.code_lines;
                }
                let mut authors: Vec<_> = merged.into_values().collect();
                authors.sort_by(|a, b| b.commits.cmp(&a.commits));

                Ok(format_multi_stats(&authors, handles.len()))
            }
            StatsSubcommand::Overview(_) => {
                let repo = get_repo(ctx)?;
                let overview = repo.overview()?;
                Ok(format_overview(&overview))
            }
        };
    }

    // 默认：当前仓库统计
    let repo = get_repo(ctx)?;
    let result = crate::stats::engine::analyze(
        &repo, args.author.as_deref(), args.since.as_deref(),
        args.until.as_deref(), args.branch.as_deref(),
    )?;

    let sort_by = match args.sort_by.as_ref().unwrap_or(&SortBy::Commits) {
        SortBy::Commits => crate::stats::report::SortBy::Commits,
        SortBy::Added => crate::stats::report::SortBy::Added,
        SortBy::Net => crate::stats::report::SortBy::Net,
        SortBy::Active => crate::stats::report::SortBy::Commits,
    };

    match ctx.format {
        OutputFormat::Table => Ok(crate::stats::report::format_table(&result, &sort_by)),
        OutputFormat::Json => Ok(crate::stats::report::format_json(&result)),
        OutputFormat::Md => Ok(crate::stats::report::format_markdown(&result)),
        OutputFormat::Csv => Ok(crate::stats::report::format_markdown(&result)),
    }
}

fn format_multi_stats(authors: &[crate::stats::engine::AuthorStats], repo_count: usize) -> String {
    let mut s = format!("=== 多仓库统计汇总 ({} 个仓库) ===\n\n", repo_count);
    s.push_str(&format!("{:<4} {:<20} {:<8} {:<10}\n", "排名", "作者", "提交数", "新增行"));
    s.push_str(&"-".repeat(60));
    s.push('\n');
    for (i, a) in authors.iter().enumerate() {
        s.push_str(&format!("{:<4} {:<20} {:<8} {:<10}\n", i + 1, a.name, a.commits, a.lines_added));
    }
    s
}

fn format_overview(overview: &crate::git_backend::repo::RepoOverview) -> String {
    format!(
        "=== 仓库概览 ===\n\n\
         路径: {}\n总提交数: {}\n分支数: {}\n标签数: {}\n贡献者: {}\n当前分支: {}\n最后提交: {}",
        overview.path, overview.total_commits, overview.total_branches,
        overview.total_tags, overview.contributors, overview.head_branch,
        overview.last_commit_date,
    )
}

// ---- branch ----

fn cmd_branch(args: BranchArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    match args.command {
        BranchSubcommand::Cleanup(c) => {
            let guard = crate::safety::guard::check_safety("branch", Some("cleanup"), ctx.dry_run, ctx.force)?;
            let repo = get_repo(ctx)?;
            let whitelist = crate::branch::whitelist::WhitelistConfig::load(c.whitelist.as_deref())?;

            let branches = crate::branch::cleanup::scan_cleanable(&repo, c.remote, &whitelist.protected)?;

            if matches!(guard, crate::safety::guard::SafetyDecision::DryRun) {
                Ok(crate::branch::cleanup::format_dry_run(&branches))
            } else {
                let deleted = crate::branch::cleanup::execute_cleanup(&repo, &branches, ctx.force, false)?;
                Ok(format!("✅ 已删除 {} 个分支:\n{}", deleted.len(), deleted.join("\n")))
            }
        }
        BranchSubcommand::SwitchAll(s) => {
            let root = std::path::Path::new(&s.root);
            let repos = crate::multi_repo::scanner::scan_repos(root, ctx.depth)?;
            let handles: Vec<_> = repos.iter().filter_map(|p| {
                crate::git_backend::repo::RepoHandle::open(p).ok().map(|h| (p.display().to_string(), h))
            }).collect();
            let results = crate::branch::batch_ops::batch_checkout(&handles, &s.branch);
            let mut msg = String::new();
            for (path, r) in &results {
                match r {
                    Ok(()) => msg.push_str(&format!("✅ {} → {}\n", path, s.branch)),
                    Err(e) => msg.push_str(&format!("❌ {} : {}\n", path, e)),
                }
            }
            Ok(msg)
        }
        BranchSubcommand::Rename(r) => {
            Ok(format!("分支重命名功能: pattern={}, replace={} (开发中)", r.pattern, r.replace))
        }
        BranchSubcommand::Stale(s) => {
            let repo = get_repo(ctx)?;
            let stale = crate::branch::stale::find_stale(&repo, s.months)?;
            if stale.is_empty() {
                Ok(format!("没有 {} 个月以上未活动的分支。", s.months))
            } else {
                let mut msg = format!("=== 休眠分支 ({} 个月以上未活动) ===\n\n", s.months);
                for b in &stale {
                    msg.push_str(&format!("💤 {} ({} | {})\n", b.name, b.last_commit_date, b.last_commit_author));
                }
                msg.push_str(&format!("\n共 {} 个休眠分支", stale.len()));
                Ok(msg)
            }
        }
        BranchSubcommand::Whitelist(w) => match w.command {
            WhitelistSubcommand::Show => {
                let wl = crate::branch::whitelist::WhitelistConfig::load(None)?;
                Ok(format!("=== 白名单 ===\n{}", wl.protected.join("\n")))
            }
            WhitelistSubcommand::Add(a) => {
                let mut wl = crate::branch::whitelist::WhitelistConfig::load(None)?;
                wl.add(a.branch.clone());
                wl.save()?;
                Ok(format!("✅ 已添加 '{}' 到白名单", a.branch))
            }
            WhitelistSubcommand::Remove(r) => {
                let mut wl = crate::branch::whitelist::WhitelistConfig::load(None)?;
                wl.remove(&r.branch);
                wl.save()?;
                Ok(format!("✅ 已从白名单移除 '{}'", r.branch))
            }
        },
    }
}

// ---- log ----

fn cmd_log(args: LogArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    let repo = get_repo(ctx)?;
    let commits = repo.commits(
        args.author.as_deref(), args.since.as_deref(),
        args.until.as_deref(), None, args.grep.as_deref(),
        args.max_count,
    )?;
    Ok(crate::log_ops::viewer::format_log(&commits))
}

// ---- reset ----

fn cmd_reset(args: ResetArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    let (target_args, mode) = match args.command {
        ResetSubcommand::Soft(a) => (a, crate::log_ops::reset::ResetMode::Soft),
        ResetSubcommand::Mixed(a) => (a, crate::log_ops::reset::ResetMode::Mixed),
        ResetSubcommand::Hard(a) => (a, crate::log_ops::reset::ResetMode::Hard),
    };

    crate::safety::guard::check_safety("reset", Some("hard"), ctx.dry_run, ctx.force)?;
    let repo = get_repo(ctx)?;
    crate::log_ops::reset::execute_reset(&repo, &target_args.target, mode, target_args.backup)
}

// ---- stash ----

fn cmd_stash(args: StashArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    match args.command {
        StashSubcommand::List => {
            let repo = get_repo(ctx)?;
            crate::log_ops::stash::list_stashes(&repo)
        }
        StashSubcommand::Clean(c) => {
            crate::safety::guard::check_safety("stash", Some("clean"), ctx.dry_run, ctx.force)?;
            let repo = get_repo(ctx)?;
            crate::log_ops::stash::clean_stashes(&repo, c.older_than, ctx.dry_run)
        }
    }
}

// ---- multi ----

fn cmd_multi(args: MultiArgs, ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    match args.command {
        MultiSubcommand::Scan(s) => {
            let root = std::path::Path::new(&s.root);
            let repos = crate::multi_repo::scanner::scan_repos(root, s.depth)?;
            if repos.is_empty() {
                Ok(format!("在 '{}' 下没有找到 Git 仓库。", s.root))
            } else {
                let mut msg = format!("=== 找到 {} 个 Git 仓库 ===\n\n", repos.len());
                for r in &repos {
                    msg.push_str(&format!("📁 {}\n", r.display()));
                }
                Ok(msg)
            }
        }
        MultiSubcommand::Pull(p) => {
            let root = std::path::Path::new(&p.root);
            let repos = crate::multi_repo::scanner::scan_repos(root, ctx.depth)?;
            let results = crate::multi_repo::runner::run_parallel(&repos, |handle| {
                crate::multi_repo::ops::batch_pull(handle, p.rebase)
            });
            Ok(crate::multi_repo::summary::format_summary(&results, "git pull"))
        }
        MultiSubcommand::Gc(g) => {
            crate::safety::guard::check_safety("multi", Some("gc"), ctx.dry_run, ctx.force)?;
            let root = std::path::Path::new(&g.root);
            let repos = crate::multi_repo::scanner::scan_repos(root, ctx.depth)?;
            let results = crate::multi_repo::runner::run_parallel(&repos, |handle| {
                crate::multi_repo::ops::batch_gc(handle, g.aggressive)
            });
            Ok(crate::multi_repo::summary::format_summary(&results, "git gc"))
        }
        MultiSubcommand::Status(s) => {
            let root = std::path::Path::new(&s.root);
            let repos = crate::multi_repo::scanner::scan_repos(root, ctx.depth)?;
            let results = crate::multi_repo::runner::run_parallel(&repos, |handle| {
                crate::multi_repo::ops::batch_status(handle)
            });
            Ok(crate::multi_repo::summary::format_summary(&results, "status"))
        }
    }
}

// ---- audit ----

fn cmd_audit(args: AuditArgs, _ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    let log = crate::safety::audit::AuditLog::open()?;
    match args.command {
        AuditSubcommand::Log => {
            let entries = log.read_all()?;
            Ok(crate::safety::recovery::format_audit_log(&entries))
        }
        AuditSubcommand::Revert(r) => {
            match crate::safety::recovery::find_entry(&log, &r.id)? {
                Some(entry) => Ok(format!("找到操作记录:\n{} | {} | {}\n\n⏳ 回滚功能开发中，仅支持 branch cleanup 和 reset 的回滚。", entry.operation, entry.timestamp, entry.result)),
                None => Ok(format!("未找到操作 ID: {}", r.id)),
            }
        }
    }
}

// ---- completions ----

fn cmd_completions(args: CompletionsArgs) -> Result<String, crate::error::GhError> {
    use clap::CommandFactory;
    use clap_complete::generate;
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, &name, &mut std::io::stdout());
    Ok(String::new())
}

// ---- config ----

fn cmd_config(args: ConfigArgs, _ctx: &CmdCtx) -> Result<String, crate::error::GhError> {
    match args.command {
        ConfigSubcommand::Show => {
            let config = crate::config::AppConfig::load()?;
            Ok(toml::to_string_pretty(&config)
                .map_err(|e| crate::error::GhError::Config(format!("序列化失败: {}", e)))?)
        }
        ConfigSubcommand::Set(s) => {
            let mut config = crate::config::AppConfig::load()?;
            match s.key.as_str() {
                "scan_depth" => config.general.scan_depth = s.value.parse().unwrap_or(3),
                "default_author" => config.general.default_author = s.value.clone(),
                "table_style" => config.display.table_style = s.value.clone(),
                _ => return Err(crate::error::GhError::Args(format!("未知配置项: {}", s.key))),
            }
            config.save()?;
            let val = &s.value;
            Ok(format!("✅ {} = {}", s.key, val))
        }
    }
}
