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
    #[arg(short = 'n', long)]
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

pub fn run() -> miette::Result<()> {
    let _cli = Cli::parse();

    // TODO: 实现各子命令分发
    println!("git-helper (gh) v{}", env!("CARGO_PKG_VERSION"));
    println!("命令框架已就绪，各模块开发中...");

    Ok(())
}
