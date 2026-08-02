# GitCat

纯 Rust 单二进制、跨平台本地 Git 仓库增强工具箱 —— 一行命令搞定统计、批量、清理、报表。

## 为什么要用 GitCat？

| 日常痛点 | GitCat 方案 |
|---------|---------------|
| 手动 `git log` 拼统计，周报难写 | 一键统计 + Markdown/CSV 导出 |
| 逐个仓库 `cd && git pull` | 扫描目录并发执行 |
| `git branch -d` 怕误删 | 强制 --dry-run 预演 + 白名单保护 |
| `git reset --hard` 没后悔药 | 自动备份分支 + 操作审计日志 |
| Shell 脚本 Windows 跑不了 | 纯 Rust 静态编译，全平台一致 |

## 快速开始

```bash
# 安装
cargo install git-helper

# 查看帮助
gh --help

# 仓库贡献统计
gh stats

# 按作者 + 时间段过滤
gh stats --author "Zhang" --since "2026-01-01"

# 清理已合并分支（先预演，安全！）
gh branch cleanup --dry-run

# 批量检查多仓库状态
gh multi status --root ~/projects
功能模块
命令	功能
gh stats	贡献统计（代码行数、作者排行、报表导出）
gh branch	分支管理（清理、切换、重命名、休眠检测）
gh log	彩色精简日志
gh reset	安全重置（软/混合/硬，自动备份）
gh stash	Stash 批量管理
gh multi	多仓库批量运维（pull / gc / status）
gh audit	操作审计日志
技术栈
语言：Rust 2021 Edition
Git 底层：gix (gitoxide) — 纯 Rust 实现
CLI：clap v4
并发：rayon
平台：Windows / macOS / Linux (x86_64 + aarch64)
开发

git clone https://github.com/JianWeiCat/Git-Helper.git
cd Git-Helper
cargo build
cargo test
License
MIT OR Apache-2.0
