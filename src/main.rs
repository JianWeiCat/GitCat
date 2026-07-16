//! git-helper (gh) — 纯 Rust 跨平台 Git 仓库增强工具箱
//!
//! 一行命令搞定统计、批量、清理、报表。

mod cli;
mod config;
mod error;
mod git_backend;
mod stats;
mod branch;
mod log_ops;
mod multi_repo;
mod safety;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> miette::Result<()> {
    // 初始化日志系统
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // 解析 CLI 并执行
    cli::run()
}
