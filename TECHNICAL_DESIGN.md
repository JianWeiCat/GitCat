# git-helper 技术选型与实现方案

> **项目代号**：`git-helper` | **CLI 简写**：`gh`
> **技术栈**：Rust 2024 Edition | **最低 Rust 版本**：1.85+
> **文档版本**：v1.0 | **日期**：2026-07-16

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术架构总览](#2-技术架构总览)
3. [技术选型详析](#3-技术选型详析)
4. [项目工程结构](#4-项目工程结构)
5. [模块一：仓库贡献统计](#5-模块一仓库贡献统计)
6. [模块二：分支批量管理](#6-模块二分支批量管理)
7. [模块三：提交日志与操作封装](#7-模块三提交日志与操作封装)
8. [模块四：多仓库批量运维](#8-模块四多仓库批量运维)
9. [模块五：安全兜底机制](#9-模块五安全兜底机制)
10. [CLI 命令矩阵设计](#10-cli-命令矩阵设计)
11. [CI/CD 与分发方案](#11-cicd-与分发方案)
12. [测试策略](#12-测试策略)
13. [性能基准与优化](#13-性能基准与优化)
14. [风险与缓解](#14-风险与缓解)

---

## 1. 项目概述

### 1.1 一句话定位

**纯 Rust 单二进制、跨平台本地 Git 仓库增强工具箱——一行命令搞定统计、批量、清理、报表。**

### 1.2 核心价值主张

| 维度 | 现状痛点 | git-helper 方案 |
|------|----------|----------------|
| 贡献统计 | 手动 `git log` + shell 拼接，无法跨仓汇总 | 一键多仓并行统计，彩色表格 + Markdown/CSV 导出 |
| 分支清理 | 逐个仓库手动 `git branch -d`，易误删 | 批量预演 + 白名单保护 + 一键清理 |
| 多仓运维 | 逐个 `cd && git pull`，效率极低 | 扫描目录并发 `pull/gc/status` |
| 安全兜底 | `git reset --hard` 无后悔药 | 强制 `--dry-run` 拦截 + 操作日志备份 |
| 跨平台 | Shell 脚本 Windows 不可用 | 纯 Rust 静态编译，全平台一致体验 |

### 1.3 与竞品的差异化

```
                  git-extras   git-branchless   gitui/lazygit   git-helper
                  (Shell)      (Rust)           (Rust/Go TUI)   (本方案)
──────────────────────────────────────────────────────────────────────────
一行命令批量多仓    ✗            ✗                ✗               ✓
代码贡献统计报表    ✗            ✗                ✗               ✓ (核心亮点)
纯 Rust 无依赖      ✗            ✓                ✓               ✓
安全预演机制        ✗            ✗                ✗               ✓ (核心特色)
分支批量管理        ✓ (部分)     ✓ (强)           ✗               ✓
跨平台一致体验      ✗            ✓                ✓               ✓
操作日志备份        ✗            ✗                ✗               ✓
```

---

## 2. 技术架构总览

### 2.1 分层架构图

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Layer (clap v5)                   │
│   ┌──────────┬──────────┬──────────┬──────────┬───────┐ │
│   │  stats   │  branch  │   log    │  multi   │ safe  │ │
│   │  统计模块  │  分支模块  │  日志模块  │  多仓模块  │ 安全  │ │
│   └──────────┴──────────┴──────────┴──────────┴───────┘ │
├─────────────────────────────────────────────────────────┤
│                  Business Logic Layer                    │
│   ┌──────────┬──────────┬──────────┬──────────────────┐ │
│   │ StatsEngine│BranchMgr│ LogParser│ MultiRepoRunner │ │
│   │ 统计引擎   │ 分支管理  │ 日志解析  │   多仓并发执行器  │ │
│   └──────────┴──────────┴──────────┴──────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                   Core Abstraction                      │
│   ┌──────────────────┬───────────────┬────────────────┐ │
│   │   RepoHandle      │  DiffAnalyzer │  ReportExporter│ │
│   │   统一仓库句柄     │  差异分析器    │   报表导出器    │ │
│   └──────────────────┴───────────────┴────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                  Git Backend Layer                       │
│   ┌────────────────────────────────────────────────────┐│
│   │              gix (gitoxide) 纯 Rust 实现            ││
│   │  ┌─────────┬──────────┬──────────┬──────────────┐  ││
│   │  │ Repository│ Object  │ Reference│  Revision    │  ││
│   │  │  仓库打开  │ 对象解析  │  引用操作  │  版本区间解析 │  ││
│   │  └─────────┴──────────┴──────────┴──────────────┘  ││
│   └────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────┤
│                  Infrastructure                         │
│   ┌────────┬────────┬────────┬────────┬───────────────┐ │
│   │ rayon  │walkdir │indicatif│tracing│ serde+csv+toml│ │
│   │ 并行计算 │目录扫描 │ 进度条  │ 日志系统 │  序列化导出    │ │
│   └────────┴────────┴────────┴────────┴───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 2.2 核心数据流

```
用户输入命令 (CLI)
      │
      ▼
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│ clap 解析    │───▶│ 安全守卫      │───▶│ 业务逻辑分发      │
│ 子命令+参数  │    │ dry-run检查  │    │ (模块路由)       │
└─────────────┘    └──────────────┘    └───────┬─────────┘
                                               │
                    ┌───────────────────────────┤
                    ▼                           ▼
            ┌──────────────┐           ┌──────────────┐
            │ 单仓操作路径   │           │ 多仓操作路径   │
            │ RepoHandle::  │           │ walkdir 扫描  │
            │ open(path)    │           │ → rayon 并行  │
            └──────┬───────┘           └──────┬───────┘
                   │                          │
                   ▼                          ▼
            ┌──────────────┐           ┌──────────────┐
            │ gix 解析仓库  │           │ 每个仓库独立   │
            │ commit/tree/  │           │ RepoHandle    │
            │ blob/diff     │           │ 并行执行       │
            └──────┬───────┘           └──────┬───────┘
                   │                          │
                   └──────────┬───────────────┘
                              ▼
                    ┌──────────────────┐
                    │ 结果聚合/格式化    │
                    │ 表格/CSV/Markdown │
                    └──────────┬───────┘
                               ▼
                    ┌──────────────────┐
                    │ 终端输出/文件写入  │
                    │ colored + 进度条  │
                    └──────────────────┘
```

---

## 3. 技术选型详析

### 3.1 核心技术栈

| 类别 | 库 | 版本 | 选型理由 |
|------|---|------|---------|
| CLI 框架 | **clap** | 5.x | 派生宏 API，子命令结构清晰，内置自动补全生成、参数校验、彩色帮助 |
| Git 底层 | **gix** (gitoxide) | 0.70+ | 纯 Rust 实现，无 FFI 依赖，跨平台一致，性能优于 libgit2 绑定，API 符合 Rust 习惯 |
| 文件遍历 | **walkdir** | 2.x | 成熟稳定，递归遍历性能好，支持过滤和深度控制 |
| 并行计算 | **rayon** | 1.x | 零成本抽象并行迭代器，多仓库统计天然适合数据并行 |
| 进度条 | **indicatif** | 0.17+ | 多进度条支持，与 rayon 配合良好，模板可定制 |
| 终端彩色 | **colored** / **console** | 2.x / 0.15+ | 跨平台终端样式，Windows Terminal / cmd 均支持 |
| 序列化 | **serde** + **serde_json** + **csv** + **toml** | 1.x | serde 生态标准，配置文件用 TOML，报表导出用 CSV |
| 日志 | **tracing** + **tracing-subscriber** | 0.1+ / 0.3+ | 结构化日志，支持 JSON/文本输出，性能好 |
| 错误处理 | **thiserror** + **miette** / **color-eyre** | 2.x / 0.7+ | 结构化错误类型 + 友好错误报告，带源码定位和解决建议 |
| 日期时间 | **time** / **chrono** | 0.3+ / 0.4+ | 提交时间解析与格式化 |
| 正则 | **regex** | 1.x | 分支名匹配、日志过滤 |
| 表格输出 | **tabled** | 0.17+ | 终端表格渲染，支持多种样式，可导出 Markdown |
| 配置文件 | **toml** + **dirs** | 0.8+ / 5.x | TOML 配置文件读写，dirs 获取系统配置目录 |
| 异步（可选） | **tokio** | 1.x | 仅当需要与远程交互时引入（如检查远程分支状态） |

### 3.2 关键选型决策详析

#### 3.2.1 gix (gitoxide) vs libgit2 vs git2

```
                    git2 (libgit2 绑定)        gix (gitoxide)
─────────────────────────────────────────────────────────────────
实现方式            C 库 FFI 绑定              纯 Rust
编译               需要 cmake + libgit2 C 库   纯 cargo build
交叉编译           困难（C 工具链）            简单（rustup target）
Windows 兼容       msys2/cmake 依赖          零额外依赖
API 风格           C 风格，不完全 Rust 习惯     Rust 原生，Iterator 友好
成熟度             非常成熟（十年级）          活跃开发，核心功能稳定
性能               优                          优（部分场景更快）
包体积             小（共享 .so/.dll）          略大（静态链接）
SHA1/HASH 加速    可选 OpenSSL                纯 Rust 实现
```

**最终选择：gix**。理由：
1. **跨平台编译零成本**——GitHub Actions 一次配置全平台产出；
2. **单二进制分发**——无需担心系统 Git 版本或 libgit2 ABI 兼容性；
3. **API 更符合 Rust 习惯**——开发效率更高；
4. **gitoxide 已足够成熟**——核心 read-only 操作（log/blame/diff/rev-parse）稳定可用；
5. **备选方案**：若遇 gix 尚未支持的边角写入操作（如 `git reset --hard`），可降级调用系统 `git` 命令行（`std::process::Command`）作为 fallback。

#### 3.2.2 clap v5 Derive vs Builder

**选择：Derive 模式为主 + Builder 补充**

```rust
// Derive 模式：结构化定义，可读性强，适合主命令树
#[derive(Parser)]
#[command(name = "gh", about = "Git Helper - 一站式 Git 仓库增强工具箱")]
enum Cli {
    /// 仓库贡献统计
    Stats(StatsArgs),
    /// 分支批量管理
    Branch(BranchArgs),
    /// 提交日志操作
    Log(LogArgs),
    /// 多仓库批量运维
    Multi(MultiArgs),
}
```

**理由**：命令树层级多（5 大模块 → 15+ 子命令），Derive 模式让每个模块的命令结构独立定义在各自模块文件中，编译时类型检查完备。

#### 3.2.3 rayon 并行策略

```rust
// 多仓库统计：数据并行（每个仓库独立解析）
use rayon::prelude::*;
use walkdir::WalkDir;

fn multi_repo_stats(root: &Path) -> Vec<RepoStats> {
    let repos: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join(".git").is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();

    repos
        .par_iter()                              // rayon 并行迭代
        .progress_count(repos.len() as u64)      // indicatif 进度
        .filter_map(|path| analyze_repo(path).ok())
        .collect()
}
```

#### 3.2.4 不使用嵌入式数据库 sled 的理由

- 操作日志量极小（每次危险操作一条记录），**JSON Lines 文件** 完全满足；
- `sled` 增加约 2MB 包体积，对单文件分发不友好；
- JSON Lines 人眼可读，方便用户直接查看操作历史。

---

## 4. 项目工程结构

### 4.1 目录树

```
git-helper/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE                          # MIT OR Apache-2.0
├── CHANGELOG.md
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                   # 全平台编译 + 测试
│   │   ├── release.yml              # 自动发布 + 打包
│   │   └── audit.yml                # cargo-deny 许可证/安全审计
│   └── dependabot.yml
├── src/
│   ├── main.rs                      # 入口，tracing 初始化
│   ├── cli.rs                       # clap 命令树定义
│   ├── error.rs                     # 统一错误类型 (thiserror)
│   ├── config.rs                    # 配置文件加载 (TOML)
│   │
│   ├── git_backend/                 # Git 底层抽象层
│   │   ├── mod.rs
│   │   ├── repo.rs                  # RepoHandle: 统一仓库句柄
│   │   ├── commit.rs               # 提交遍历、解析
│   │   ├── diff.rs                  # Diff 分析 (增删行/文件)
│   │   ├── branch.rs                # 分支操作封装
│   │   ├── reference.rs             # 引用操作
│   │   └── fallback.rs             # gix 能力不足时调用系统 git
│   │
│   ├── stats/                       # 模块一：贡献统计
│   │   ├── mod.rs
│   │   ├── engine.rs                # 统计引擎核心
│   │   ├── analyzer.rs              # Diff 行级分析 (代码/注释/空行)
│   │   ├── filter.rs                # 文件过滤 (编译产物/配置/锁文件)
│   │   ├── report.rs                # 报表构建 (排序/聚合)
│   │   └── exporter.rs             # Markdown/CSV 导出
│   │
│   ├── branch/                      # 模块二：分支管理
│   │   ├── mod.rs
│   │   ├── cleanup.rs               # 废弃分支清理
│   │   ├── batch_ops.rs            # 批量切换/重命名
│   │   ├── stale.rs                # 休眠分支检测
│   │   └── whitelist.rs            # 白名单管理
│   │
│   ├── log_ops/                     # 模块三：日志与操作
│   │   ├── mod.rs
│   │   ├── viewer.rs               # 彩色精简日志
│   │   ├── reset.rs                # 软/硬重置封装
│   │   └── stash.rs                # Stash 批量管理
│   │
│   ├── multi_repo/                  # 模块四：多仓库运维
│   │   ├── mod.rs
│   │   ├── scanner.rs              # walkdir 仓库扫描
│   │   ├── runner.rs               # 并发执行器 (rayon)
│   │   ├── ops.rs                  # 批量 pull/gc/status
│   │   └── summary.rs              # 多仓结果汇总
│   │
│   └── safety/                      # 模块五：安全兜底
│       ├── mod.rs
│       ├── guard.rs                 # 操作拦截、dry-run 强制检查
│       ├── audit.rs                # 操作日志记录 (JSON Lines)
│       └── recovery.rs             # 撤销入口 (基于 audit log)
│
├── tests/                           # 集成测试
│   ├── common.rs                    # 测试工具 (创建临时 git 仓库)
│   ├── stats_test.rs
│   ├── branch_test.rs
│   ├── log_test.rs
│   ├── multi_repo_test.rs
│   └── safety_test.rs
│
├── fixtures/                        # 测试用静态仓库数据
│   └── sample_repo/
│       └── ...
│
├── completions/                     # 预生成的补全脚本
│   ├── _gh                          # zsh
│   ├── gh.bash                      # bash
│   ├── gh.fish                      # fish
│   └── _gh.ps1                      # powershell
│
└── docs/
    ├── architecture.md
    ├── commands.md                  # 全命令参考
    └── screenshots/                 # 功能截图
```

### 4.2 Cargo.toml 核心依赖

```toml
[package]
name = "git-helper"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "纯 Rust 跨平台 Git 仓库增强工具箱"
license = "MIT OR Apache-2.0"
repository = "https://github.com/xxx/git-helper"

[[bin]]
name = "gh"
path = "src/main.rs"

[dependencies]
# CLI
clap = { version = "5", features = ["derive", "env", "wrap_help"] }
clap_complete = "5"

# Git backend (纯 Rust)
gix = { version = "0.70", features = ["blob-diff", "revision", "worktree-mutation"] }
gix-filter = "0.16"

# File system
walkdir = "2"

# Concurrency
rayon = "1"

# Terminal UI
indicatif = "0.17"
console = "0.15"
colored = "2"
tabled = "0.17"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"
toml = "0.8"

# Error handling
thiserror = "2"
miette = { version = "7", features = ["fancy"] }

# Logging
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Config & data dirs
dirs = "5"

# DateTime
time = { version = "0.3", features = ["formatting", "macros"] }

# Regex
regex = "1"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"       # CLI 集成测试
predicates = "3"       # 断言组合器
criterion = "0.5"      # 性能基准测试

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

### 4.3 Feature Flags 设计

```toml
[features]
default = ["tui-progress", "color"]

# 进度条显示 (默认开启，纯脚本环境可关闭)
tui-progress = ["indicatif", "console"]

# 终端彩色输出 (默认开启)
color = ["colored", "tabled/color"]

# 异步远程检查 (可选)
remote = ["tokio", "gix/blocking-network-client"]

# 操作日志备份
audit-log = []
```

---

## 5. 模块一：仓库贡献统计

### 5.1 功能清单

| 子命令 | 功能 | 关键参数 |
|--------|------|---------|
| `gh stats` | 单仓贡献统计 | `--author`, `--since`, `--until`, `--branch`, `--format` |
| `gh stats report` | 报表导出 | `--output <FILE>`, `--md` / `--csv` |
| `gh stats multi` | 多仓批量统计 | `--root <DIR>`, `--depth <N>` |
| `gh stats overview` | 仓库概览 | `--repo <PATH>` |

### 5.2 核心数据结构

```rust
/// 文件变更统计
#[derive(Debug, Clone, Serialize)]
struct FileDiffStats {
    path: String,
    lines_added: u64,
    lines_deleted: u64,
    code_lines: u64,        // 有效代码行
    comment_lines: u64,      // 注释行
    blank_lines: u64,        // 空行
}

/// 单次提交统计
#[derive(Debug, Clone, Serialize)]
struct CommitStats {
    commit_id: String,       // 短 hash (7 位)
    author: String,
    email: String,
    timestamp: OffsetDateTime,
    message: String,
    files_changed: usize,
    diff: FileDiffStats,
}

/// 贡献者汇总统计
#[derive(Debug, Clone, Serialize)]
struct AuthorStats {
    name: String,
    email: String,
    commits: usize,
    files_changed: usize,
    lines_added: u64,
    lines_deleted: u64,
    net_lines: i64,          // added - deleted
    code_lines: u64,
    comment_lines: u64,
    blank_lines: u64,
    first_commit: OffsetDateTime,
    last_commit: OffsetDateTime,
    active_days: u32,
}

/// 仓库概览
#[derive(Debug, Clone, Serialize)]
struct RepoOverview {
    path: String,
    total_commits: u64,
    total_branches: usize,
    total_tags: usize,
    contributors: usize,
    repo_size_bytes: u64,
    first_commit: OffsetDateTime,
    last_commit: OffsetDateTime,
    is_bare: bool,
}
```

### 5.3 统计引擎核心流程

```
RepoHandle::open(path)
       │
       ▼
┌──────────────────────┐
│  1. 解析 rev-range   │  branch + since/until → gix::revision::Spec
│     (版本区间)        │
└──────┬───────────────┘
       ▼
┌──────────────────────┐
│  2. 遍历 commits     │  gix::revision::Walk (支持 --author 过滤)
│     (提交遍历)        │  indicatif::ProgressBar 显示进度
└──────┬───────────────┘
       ▼
┌──────────────────────┐
│  3. 逐 commit 分析    │  遍历 parent..commit 的 tree diff
│     (差异分析)        │  对每个变更文件执行文件过滤
└──────┬───────────────┘
       ▼
┌──────────────────────┐
│  4. 行级分类          │  逐行判断：空行 / 注释 / 代码
│     (代码/注释/空行)   │  支持主流语言后缀 (.rs/.py/.js/.ts/.java/.go ...)
└──────┬───────────────┘
       ▼
┌──────────────────────┐
│  5. 按作者聚合        │  HashMap<AuthorEmail, AuthorStats>
│     (聚合 + 排序)     │  按 commits / lines_added / net_lines 可排序
└──────┬───────────────┘
       ▼
┌──────────────────────┐
│  6. 格式化输出        │  终端表格 (tabled) 或 Markdown/CSV 文件
└──────────────────────┘
```

### 5.4 关键实现细节

#### 5.4.1 文件过滤规则

```rust
/// 自动跳过的文件和目录模式
const IGNORE_PATTERNS: &[&str] = &[
    // 编译产物
    "*.o", "*.so", "*.dylib", "*.dll", "*.exe", "*.class", "*.pyc",
    "*.wasm", "*.obj",
    // 包管理器锁文件
    "Cargo.lock", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
    "Gemfile.lock", "poetry.lock", "Pipfile.lock",
    // 自动生成
    "*.generated.*", "*.pb.go", "*.pb.rs",
    // 资源/二进制
    "*.png", "*.jpg", "*.jpeg", "*.gif", "*.ico", "*.svg",
    "*.woff", "*.woff2", "*.ttf", "*.eot",
    "*.mp3", "*.mp4", "*.webm",
    // 构建目录
    "target/", "node_modules/", "dist/", "build/", ".next/",
    "__pycache__/", "*.egg-info/", "vendor/",
    // IDE / 工具
    ".idea/", ".vscode/", "*.swp", "*.swo", "*~",
];

fn should_ignore(path: &str) -> bool {
    IGNORE_PATTERNS.iter().any(|pattern| {
        if pattern.ends_with('/') {
            path.contains(pattern)
        } else if pattern.starts_with('*') {
            path.ends_with(&pattern[1..])
        } else {
            path == pattern || path.starts_with(pattern)
        }
    })
}
```

#### 5.4.2 行类型分类算法

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineType {
    Code,
    Comment,
    Blank,
}

/// 基于文件扩展名的行分类
fn classify_line(line: &str, ext: &str) -> LineType {
    if line.trim().is_empty() {
        return LineType::Blank;
    }

    let trimmed = line.trim_start();

    match ext {
        // Rust / C / C++ / Java / JS / TS / Go / Swift / Kotlin
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "java" |
        "js" | "ts" | "jsx" | "tsx" | "go" | "swift" | "kt" | "kts" |
        "scala" | "dart" | "cs" => {
            if trimmed.starts_with("//") || trimmed.starts_with("/*") ||
               trimmed.starts_with('*') || trimmed == "*/" {
                LineType::Comment
            } else {
                LineType::Code
            }
        }
        // Python / Ruby / Shell / Perl / YAML / TOML
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "pl" |
        "yaml" | "yml" | "toml" | "r" => {
            if trimmed.starts_with('#') {
                LineType::Comment
            } else {
                LineType::Code
            }
        }
        // Hashbang / 未知扩展：视为代码
        _ => LineType::Code,
    }
}
```

#### 5.4.3 多仓并行统计

```rust
use rayon::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn multi_repo_stats(
    root: &Path,
    depth: usize,
    author: Option<&str>,
) -> Result<Vec<(String, AuthorStats)>> {
    // 1. 扫描仓库
    let repos = scan_repos(root, depth)?;

    // 2. 多进度条
    let mp = MultiProgress::new();
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
    ).unwrap();

    // 3. 并行统计（每个仓库独立运行）
    let results: Vec<_> = repos
        .par_iter()
        .map(|repo_path| {
            let pb = mp.add(ProgressBar::new(0)); // commit 数未知时用 spinner
            pb.set_style(style.clone());
            pb.set_message(repo_path.display().to_string());

            let stats = analyze_repo(repo_path, author, Some(pb.clone()));
            pb.finish_and_clear();
            (repo_path.display().to_string(), stats)
        })
        .collect();

    Ok(results)
}
```

#### 5.4.4 报表导出格式

**Markdown 输出示例：**

```markdown
# Git 贡献统计报表

**仓库**: `/home/user/projects/my-app`
**统计区间**: 2026-01-01 ~ 2026-07-16
**生成时间**: 2026-07-16 15:30:00

## 贡献排行

| 排名 | 作者 | 提交数 | 文件变更 | +行数 | -行数 | 净增 | 有效代码 |
|------|------|--------|---------|-------|-------|------|---------|
| 1 | Zhang San | 142 | 380 | 12,430 | 3,210 | +9,220 | 9,800 |
| 2 | Li Si | 87 | 210 | 5,600 | 2,100 | +3,500 | 4,200 |

## 语言分布

| 语言 | 文件数 | 代码行 | 注释行 |
|------|--------|--------|--------|
| Rust | 45 | 12,300 | 1,200 |
| TypeScript | 32 | 8,900 | 450 |
```

---

## 6. 模块二：分支批量管理

### 6.1 功能清单

| 子命令 | 功能 | 关键参数 |
|--------|------|---------|
| `gh branch cleanup` | 清理已合并分支 | `--dry-run`, `--remote`, `--whitelist`, `--force` |
| `gh branch switch-all` | 批量切换多仓分支 | `--root <DIR>`, `--branch <NAME>` |
| `gh branch rename` | 正则批量重命名 | `--pattern <REGEX>`, `--replace <STR>`, `--dry-run` |
| `gh branch stale` | 休眠分支检测 | `--months <N>` (默认 3) |

### 6.2 废弃分支清理流程

```
gh branch cleanup --dry-run
       │
       ▼
┌──────────────────────────┐
│ 1. 解析当前分支           │  获取 HEAD 指向，防止删除当前分支
└──────────┬───────────────┘
       ▼
┌──────────────────────────┐
│ 2. 加载白名单             │  ~/.config/gh/whitelist.toml
│    + 内置保护分支          │  内置: main, master, develop, release/*
└──────────┬───────────────┘
       ▼
┌──────────────────────────┐
│ 3. 枚举本地分支           │  gix::reference::iter()
│    过滤已合并到 HEAD 的分支│  检查 merge-base == branch.tip
└──────────┬───────────────┘
       ▼
┌──────────────────────────┐
│ 4. 打印预演结果           │  ✅ safe-delete / ⚠️ whitelist-protected
│    (彩色标注保护状态)      │  ❌ current-branch / 🔒 force-protected
└──────────┬───────────────┘
       ▼
┌──────────────────────────┐
│ 5. 无 --dry-run 参数？    │  → 直接拦截，提示加 --dry-run 先预览
│    安全检查               │  → --force 跳过二次确认
└──────────┬───────────────┘
       ▼
┌──────────────────────────┐
│ 6. 执行删除               │  gix 删除分支引用，记录 audit log
└──────────────────────────┘
```

### 6.3 白名单配置

```toml
# ~/.config/git-helper/whitelist.toml

[global]
# 全局保护分支 (支持 glob 模式)
protected = [
    "main",
    "master",
    "develop",
    "release/*",
    "hotfix/*",
    "production",
    "staging",
]

[repo."my-critical-project"]
# 仓库级覆盖
protected = [
    "never-delete-this",
]
```

### 6.4 休眠分支检测

```rust
struct StaleBranch {
    name: String,
    last_commit_date: OffsetDateTime,
    last_commit_author: String,
    months_inactive: u32,
    is_merged: bool,
}

fn find_stale_branches(repo: &RepoHandle, months: u32) -> Vec<StaleBranch> {
    let cutoff = OffsetDateTime::now_utc() - time::Duration::days(months as i64 * 30);
    let head_commit = repo.head_commit().ok();

    repo.local_branches()
        .iter()
        .filter_map(|branch| {
            let tip = branch.tip_commit()?;
            if tip.time() < cutoff {
                let is_merged = head_commit.as_ref()
                    .map(|h| repo.is_ancestor(&tip, h).unwrap_or(false))
                    .unwrap_or(false);
                Some(StaleBranch {
                    name: branch.name().to_string(),
                    last_commit_date: tip.time(),
                    last_commit_author: tip.author().name.to_string(),
                    months_inactive: (OffsetDateTime::now_utc() - tip.time()).whole_days() as u32 / 30,
                    is_merged,
                })
            } else {
                None
            }
        })
        .collect()
}
```

---

## 7. 模块三：提交日志与操作封装

### 7.1 功能清单

| 子命令 | 功能 | 关键参数 |
|--------|------|---------|
| `gh log` | 彩色精简日志 | `--author`, `--since`, `--until`, `--grep`, `--max-count` |
| `gh reset` | 安全重置封装 | `--soft`, `--mixed`, `--hard`, `--backup` |
| `gh stash list` | 列出所有 stash | `--format` |
| `gh stash clean` | 批量删除过期 stash | `--older-than <DAYS>` |

### 7.2 精简日志格式化

```
┌──────────────────────────────────────────────────────────────┐
│  gh log --author "Zhang" --since "1 week ago" --max-count 5  │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ◉ a1b2c3d  Zhang San  2026-07-15 14:30   (HEAD → main)     │
│  │  feat: add stats export to markdown                       │
│  │  3 files changed, +120 -8                                │
│  │                                                           │
│  ◉ e4f5g6h  Li Si      2026-07-15 10:15                     │
│  │  fix: resolve branch cleanup edge case                    │
│  │  1 file changed, +15 -3                                  │
│  │                                                           │
│  ◉ i7j8k9l  Zhang San  2026-07-14 18:00                     │
│     refactor: extract diff analyzer                          │
│     5 files changed, +230 -180                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 7.3 安全重置封装

```rust
/// 重置操作的保险封装
enum ResetMode {
    Soft,    // HEAD 移动，暂存区不变，工作区不变
    Mixed,   // HEAD 移动，暂存区重置，工作区不变
    Hard,    // HEAD 移动，暂存区重置，工作区重置 (危险!)
}

fn safe_reset(repo: &RepoHandle, target: &str, mode: ResetMode, backup: bool) -> Result<()> {
    // 1. 安全检查
    if matches!(mode, ResetMode::Hard) {
        // 检查是否有未提交的更改
        if repo.has_uncommitted_changes()? {
            if backup {
                // 自动创建 backup branch
                let backup_name = format!("gh-backup-{}", OffsetDateTime::now_utc()
                    .format(&Rfc3339)?);
                repo.create_branch(&backup_name)?;
                println!("📦 已备份当前状态到分支: {}", backup_name);
            } else {
                // 交互确认
                let confirmed = dialoguer::Confirm::new()
                    .with_prompt("⚠️  检测到未提交更改，继续 hard reset 将永久丢失更改。确认?")
                    .default(false)
                    .interact()?;
                if !confirmed {
                    anyhow::bail!("操作已取消");
                }
            }
        }
    }

    // 2. 记录操作日志
    audit_log(AuditEntry {
        operation: "reset".into(),
        target: target.into(),
        mode: format!("{:?}", mode),
        backup_branch: if backup { Some(...) } else { None },
        timestamp: OffsetDateTime::now_utc(),
    });

    // 3. 执行重置
    match mode {
        ResetMode::Soft  => repo.reset_soft(target),
        ResetMode::Mixed => repo.reset_mixed(target),
        ResetMode::Hard  => repo.reset_hard(target),
    }
}
```

---

## 8. 模块四：多仓库批量运维

### 8.1 功能清单

| 子命令 | 功能 | 关键参数 |
|--------|------|---------|
| `gh multi pull` | 批量 git pull | `--root`, `--depth`, `--rebase` |
| `gh multi gc` | 批量垃圾回收 | `--root`, `--aggressive` |
| `gh multi status` | 批量状态检查 | `--root`, `--short` |
| `gh multi scan` | 列出所有仓库 | `--root`, `--depth` |

### 8.2 仓库扫描器

```rust
/// 递归扫描目录下的所有 Git 仓库
fn scan_repos(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            // 跳过隐藏目录和常见非代码目录
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') ||
            name == ".git"  // 保留 .git 用于检测
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir() && e.path().file_name() == Some(OsStr::new(".git")))
        .map(|e| e.path().parent().unwrap().to_path_buf())
        .collect::<Vec<_>>()
        .into_iter()
        .sorted()
        .dedup()
        .collect::<Vec<_>>();
    Ok(repos)
}
```

### 8.3 并发执行器

```rust
/// 多仓库并发操作执行器
struct MultiRepoRunner {
    repos: Vec<PathBuf>,
    parallel_jobs: usize,  // 默认 = CPU 核心数
}

impl MultiRepoRunner {
    fn run<F, T>(&self, operation: F) -> Vec<(PathBuf, Result<T>)>
    where
        F: Fn(&Path) -> Result<T> + Send + Sync,
        T: Send,
    {
        let mp = MultiProgress::new();

        self.repos
            .par_iter()
            .map(|repo_path| {
                let pb = mp.add(ProgressBar::new_spinner());
                pb.set_message(repo_path.display().to_string());

                let result = operation(repo_path);

                match &result {
                    Ok(_) => pb.finish_with_message(format!("✅ {}", repo_path.display())),
                    Err(e) => pb.finish_with_message(format!("❌ {}: {}", repo_path.display(), e)),
                }

                (repo_path.clone(), result)
            })
            .collect()
    }
}
```

### 8.4 批量状态检查输出

```
┌─────────────────────────────────────────────────────────────┐
│  gh multi status --root ~/projects                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ✅ my-app              main      clean                    │
│  ⚠️  api-server          develop   3 modified, 1 untracked  │
│  ✅ docs-site           main      clean                    │
│  ❌ legacy-module       master    REPO NOT FOUND (skipped)  │
│  ⚠️  shared-lib          feat/v2   5 staged, 2 modified     │
│                                                             │
│  Summary: 5 repos scanned, 2 clean, 2 dirty, 1 error       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 9. 模块五：安全兜底机制

### 9.1 设计原则

```
         用户操作
            │
            ▼
  ┌──────────────────┐
  │ 是否危险操作?      │──── No ───▶ 正常执行
  │ (delete/reset/   │
  │  force-push...)  │
  └────────┬─────────┘
           │ Yes
           ▼
  ┌──────────────────┐
  │ 是否带 --dry-run? │──── Yes ──▶ 执行预演，显示将要发生的更改，不实际执行
  └────────┬─────────┘
           │ No
           ▼
  ┌──────────────────┐
  │ ❌ 拦截操作       │
  │ 提示: "危险操作!  │
  │ 请先使用 --dry-run│
  │ 预览将要发生的     │
  │ 更改。确认无误后   │
  │ 加 --force 执行"  │
  └──────────────────┘
```

### 9.2 操作日志记录

```rust
/// 操作审计条目
#[derive(Debug, Serialize, Deserialize)]
struct AuditEntry {
    id: String,                    // UUID v4
    timestamp: String,             // RFC 3339
    operation: String,             // "branch-cleanup", "reset-hard", "stash-drop"
    repository: String,            // 仓库路径
    details: serde_json::Value,    // 操作详情 (JSON)
    dry_run: bool,                 // 是否为预演
    reverted: bool,                // 是否已被撤销
}

// 存储位置: ~/.local/share/git-helper/audit.jsonl
// 格式: 每行一条 JSON，人眼可读，支持 tail/grep
```

### 9.3 危险操作注册表

```rust
/// 需要强制预演的危险操作列表
const DANGEROUS_OPERATIONS: &[(&str, &str)] = &[
    ("branch", "cleanup"),
    ("branch", "delete"),
    ("branch", "force-delete"),
    ("reset", "hard"),
    ("stash", "clear"),
    ("stash", "drop"),
    ("multi", "gc"),           // 批量 gc 可能改变仓库
    ("remote", "prune"),
];

fn requires_dry_run(subcommand: &[&str]) -> bool {
    DANGEROUS_OPERATIONS.iter().any(|(module, cmd)| {
        subcommand.len() >= 2 && subcommand[0] == *module && subcommand[1] == *cmd
    })
}
```

### 9.4 错误友好提示

```rust
/// 友好错误转换
fn into_friendly_error(err: gix::Error, context: &str) -> miette::Report {
    let friendly = match err.to_string() {
        s if s.contains("not a git repository") =>
            "当前目录不是一个 Git 仓库。请进入 Git 仓库目录后重试，或使用 --root 指定仓库路径。".into(),
        s if s.contains("detached HEAD") =>
            "当前处于 detached HEAD 状态。建议先切换到具体分支 (git switch <branch>)。".into(),
        s if s.contains("unborn branch") =>
            "当前仓库尚未创建任何提交，请先执行首次提交后再操作。".into(),
        s if s.contains("permission denied") =>
            "权限不足。请检查文件/目录权限，或确认仓库未被其他进程占用。".into(),
        s if s.contains("reference already exists") =>
            "目标引用已存在。如需强制覆盖，请使用 --force 参数。".into(),
        s if s.contains("uncommitted changes") =>
            "检测到未提交更改，操作被阻止。请先 git commit 或 git stash 保存更改。".into(),
        _ => format!("操作失败: {}\n\n💡 提示: 使用 gh --help 查看命令用法，或加 --debug 查看详细日志", err),
    };

    miette::Report::new(friendly).wrap_err(context.to_string())
}
```

---

## 10. CLI 命令矩阵设计

### 10.1 完整命令树

```
gh
├── stats                          # 仓库贡献统计
│   ├── [默认]                      # 当前仓库统计（表格输出）
│   ├── report                     # 导出统计报表
│   ├── multi                      # 多仓库批量统计
│   └── overview                   # 仓库概览
│
├── branch                         # 分支批量管理
│   ├── cleanup                    # 清理已合并分支
│   ├── switch-all                 # 批量切换多仓分支
│   ├── rename                     # 正则批量重命名
│   ├── stale                      # 休眠分支检测
│   └── whitelist                  # 白名单管理
│       ├── show                   # 显示当前白名单
│       ├── add <BRANCH>          # 添加保护分支
│       └── remove <BRANCH>       # 移除保护分支
│
├── log                            # 提交日志
│   └── [默认]                      # 彩色精简日志
│
├── reset                          # 安全重置
│   ├── soft <TARGET>              # 软重置
│   ├── mixed <TARGET>            # 混合重置
│   └── hard <TARGET>             # 硬重置（需确认）
│
├── stash                          # Stash 管理
│   ├── list                       # 列出所有 stash
│   └── clean                      # 批量清理过期 stash
│
├── multi                          # 多仓库批量运维
│   ├── scan                       # 扫描目录列出仓库
│   ├── pull                       # 批量 git pull
│   ├── gc                         # 批量 git gc
│   └── status                     # 批量状态检查
│
├── audit                          # 审计日志
│   ├── log                        # 查看操作日志
│   └── revert <ID>               # 尝试撤销（有限支持）
│
├── completions <SHELL>            # 生成补全脚本
└── config                         # 配置管理
    ├── show                       # 显示当前配置
    └── set <KEY> <VALUE>         # 设置配置项
```

### 10.2 全局参数

| 参数 | 短写 | 说明 |
|------|------|------|
| `--repo <PATH>` | `-r` | 指定仓库路径，默认当前目录 |
| `--root <DIR>` | `-R` | 多仓扫描根目录 |
| `--depth <N>` | `-d` | 扫描深度 |
| `--dry-run` | `-n` | 预演模式，不实际修改 |
| `--force` | `-f` | 跳过确认 |
| `--quiet` | `-q` | 静默模式 |
| `--debug` | | 输出调试日志 |
| `--no-color` | | 禁用彩色输出 |
| `--format` | | 输出格式 (table/json/csv/md) |
| `--output <FILE>` | `-o` | 输出到文件 |

---

## 11. CI/CD 与分发方案

### 11.1 GitHub Actions CI 矩阵

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-24.04, macos-15, windows-2025]
        toolchain: [stable, beta]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: ${{ matrix.toolchain }}
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-features
      - run: cargo test --no-default-features
      - run: cargo build --release

  lint:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - run: cargo deny check
```

### 11.2 Release 自动打包

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*.*.*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-24.04
            ext: ""
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-24.04
            ext: ""
          - target: x86_64-apple-darwin
            os: macos-15
            ext: ""
          - target: aarch64-apple-darwin
            os: macos-15
            ext: ""
          - target: x86_64-pc-windows-msvc
            os: windows-2025
            ext: ".exe"
          - target: aarch64-pc-windows-msvc
            os: windows-2025
            ext: ".exe"

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar -czf gh-${{ matrix.target }}.tar.gz gh${{ matrix.ext }}
          shasum -a 256 gh-${{ matrix.target }}.tar.gz > gh-${{ matrix.target }}.sha256
      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/gh-${{ matrix.target }}.tar.gz*
          generate_release_notes: true
```

### 11.3 安装方式

```bash
# 方式一：直接下载二进制
curl -L https://github.com/xxx/git-helper/releases/latest/download/gh-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv gh /usr/local/bin/

# 方式二：cargo install
cargo install git-helper

# 方式三：包管理器 (计划)
brew install git-helper          # macOS Homebrew
scoop install git-helper         # Windows Scoop
paru -S git-helper               # Arch Linux AUR
```

### 11.4 补全脚本生成

```bash
# 安装时自动生成到对应目录
gh completions bash  | sudo tee /usr/share/bash-completion/completions/gh
gh completions zsh   | sudo tee /usr/share/zsh/site-functions/_gh
gh completions fish  | sudo tee /usr/share/fish/vendor_completions.d/gh.fish
gh completions powershell | Out-File -Encoding UTF8 $PROFILE.CurrentUserCurrentHost
```

---

## 12. 测试策略

### 12.1 测试层次

```
           ┌──────────────────┐
           │   E2E 测试        │ ← assert_cmd 运行编译后的二进制
           │   (主要流程验证)   │
           ├──────────────────┤
           │   集成测试         │ ← 在临时 Git 仓库上测试模块组合
           │   (模块边界)       │
           ├──────────────────┤
           │   单元测试         │ ← 纯函数、解析器、过滤器
           │   (核心逻辑)       │
           ├──────────────────┤
           │   基准测试         │ ← criterion 性能回归
           │   (热路径)         │
           └──────────────────┘
```

### 12.2 测试辅助工具

```rust
// tests/common.rs
use std::process::Command;
use tempfile::TempDir;

/// 创建临时 Git 仓库，包含预定义的提交历史
pub fn create_test_repo() -> TestRepo {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path();

    // 初始化 Git 仓库
    run_git(repo_path, &["init"]);
    run_git(repo_path, &["config", "user.name", "Test User"]);
    run_git(repo_path, &["config", "user.email", "test@example.com"]);

    // 创建提交历史
    create_file(repo_path, "README.md", "# Test Repo\n");
    run_git(repo_path, &["add", "."]);
    run_git(repo_path, &["commit", "-m", "initial commit"]);

    create_file(repo_path, "src/main.rs", "fn main() {}\n");
    run_git(repo_path, &["add", "."]);
    run_git(repo_path, &["commit", "-m", "feat: add main.rs"]);

    // 创建分支
    run_git(repo_path, &["branch", "feature/test"]);

    TestRepo { dir, path: repo_path.to_path_buf() }
}

pub fn run_git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success(),
        "git {} failed: {}", args.join(" "),
        String::from_utf8_lossy(&output.stderr));
}
```

### 12.3 关键测试用例

```rust
// 单元测试: 行分类
#[test]
fn test_line_classification() {
    assert_eq!(classify_line("fn main() {", "rs"), LineType::Code);
    assert_eq!(classify_line("// comment", "rs"), LineType::Comment);
    assert_eq!(classify_line("", "rs"), LineType::Blank);
    assert_eq!(classify_line("   ", "py"), LineType::Blank);
    assert_eq!(classify_line("# TODO", "py"), LineType::Comment);
}

// 单元测试: 文件过滤
#[test]
fn test_ignore_patterns() {
    assert!(should_ignore("target/debug/foo.o"));
    assert!(should_ignore("node_modules/react/index.js"));
    assert!(!should_ignore("src/main.rs"));
    assert!(!should_ignore("README.md"));
}

// 集成测试: 分支清理
#[test]
fn test_branch_cleanup_dry_run() {
    let repo = create_test_repo();
    // 创建并合并一个功能分支
    repo.run_git(&["checkout", "-b", "feature/merged"]);
    repo.create_file("test.txt", "test");
    repo.commit("feat: test commit");
    repo.run_git(&["checkout", "main"]);
    repo.run_git(&["merge", "feature/merged"]);

    // 执行 dry-run
    let output = run_gh(&["branch", "cleanup", "--dry-run", "--repo", &repo.path_str()]);
    assert!(output.contains("feature/merged"));
    assert!(output.contains("safe-delete"));  // ✅ 标记
}

// 集成测试: 安全拦截
#[test]
fn test_dangerous_op_without_dry_run() {
    let output = run_gh_expecting_error(&["branch", "cleanup", "--repo", "/tmp/test"]);
    assert!(output.contains("--dry-run"));  // 提示使用预演
}
```

---

## 13. 性能基准与优化

### 13.1 性能目标

| 场景 | 目标 | 测量方法 |
|------|------|---------|
| 冷启动（无参数） | < 50ms | `hyperfine 'gh --help'` |
| 小仓库统计（<1k commits） | < 1s | `hyperfine 'gh stats --repo <small>'` |
| 大仓库统计（10k commits） | < 5s | criterion bench |
| 超大仓库统计（100k commits） | < 30s | 进度条显示，非阻塞 |
| 10 仓库并行统计 | < 5× 单仓耗时 | rayon 缩放 |
| 多仓扫描（含 100 目录） | < 2s | walkdir filter_entry |

### 13.2 性能优化策略

```rust
// 1. gix object cache: 避免重复解析
let repo = gix::open(path)?
    .with_object_cache(1024 * 1024 * 128);  // 128MB 对象缓存

// 2. 并行 diff 分析: 每个 commit 的 diff 可并行
commits.par_iter()
    .with_min_len(10)  // 少于 10 个 commit 时不启用并行
    .map(|c| analyze_commit_diff(c))

// 3. 文件过滤预计算: 把 glob 模式编译为 Vec<Glob>
lazy_static! {
    static ref IGNORE_GLOBS: Vec<glob::Pattern> = IGNORE_PATTERNS
        .iter()
        .map(|p| glob::Pattern::new(p).unwrap())
        .collect();
}

// 4. 零拷贝字符串处理
// 对 commit message / author 使用 &str 引用而非 String clone

// 5. 增量统计 (可选，未来版本)
// 首次全量统计后缓存结果，后续只统计新增提交
```

### 13.3 内存控制

```rust
// 大仓库分批处理，避免 OOM
const BATCH_SIZE: usize = 1000;

fn analyze_large_repo(repo: &RepoHandle) -> Result<Vec<AuthorStats>> {
    let mut aggregator = HashMap::new();
    let total = repo.commit_count()?;

    for batch in repo.commits().chunks(BATCH_SIZE) {
        for commit in batch {
            aggregator.entry(commit.author.email.clone())
                .or_insert_with(AuthorStats::new)
                .add_commit(commit);
        }
    }

    Ok(aggregator.into_values().sorted().collect())
}
```

---

## 14. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| gix 对写入操作支持不完整 | 部分写入命令不可用 | 中 | 实现 fallback 机制调用系统 git；逐步跟进 gix 更新 |
| gix 对非标准 Git 仓库格式不兼容 | 部分仓库解析失败 | 低 | 增加仓库格式检测 + 友好报错；提供 `--use-system-git` 降级开关 |
| 超大仓库统计内存溢出 (OOM) | 进程被 kill | 低 | 分批处理 + 流式聚合 + 内存上限检查 |
| Windows 终端编码问题 | 输出乱码 | 中 | 使用 console crate 处理 UTF-8；检测终端能力降级 |
| 大版本升级 breaking changes | 编译失败 | 高 | Cargo.lock 锁定版本 + Renovate/Dependabot 渐进升级 + CI 矩阵 |
| 命令名冲突 (`gh` = GitHub CLI) | 用户混淆 | 中 | 默认二进制名 `gh`，同时提供 `git-helper` 别名；文档中说明 |
| 单文件二进制体积过大 | 分发不便 | 低 | release profile LTO + strip + panic=abort + opt-level=s |
| 用户误操作导致数据丢失 | 数据丢失 | 中 | 安全模块强制预演 + backup branch + audit log + 确认弹窗 |

---

## 15. 里程碑规划

### Phase 1: MVP (预计 4-6 周)

- [x] 项目骨架 + CLI 框架搭建
- [ ] 模块一核心: 单仓统计 + 表格输出
- [ ] 模块二核心: 分支清理 (dry-run + 白名单)
- [ ] 模块三核心: 精简日志
- [ ] 模块五核心: 安全守卫 + audit log
- [ ] CI 流水线建立

### Phase 2: 功能完善 (预计 3-4 周)

- [ ] 模块一扩展: 报表导出 (Markdown/CSV)
- [ ] 模块一扩展: 多仓批量统计
- [ ] 模块二扩展: 批量切换 + 休眠检测
- [ ] 模块三扩展: 安全重置 + stash 管理
- [ ] 模块四: 多仓库批量运维

### Phase 3: 体验优化 (预计 2-3 周)

- [ ] 进度条 + 彩色输出完善
- [ ] 错误友好提示全覆盖
- [ ] 补全脚本自动生成
- [ ] 性能基准建立 + 优化

### Phase 4: 发布推广 (预计 1-2 周)

- [ ] 全平台 Release 打包
- [ ] README + 截图 + 使用示例
- [ ] rustdoc 完整文档
- [ ] 包管理器发布 (crates.io, Homebrew, Scoop)

---

## 附录 A: 关键参考

- [gitoxide (gix) 官方文档](https://docs.rs/gix/latest/gix/)
- [clap v5 参考](https://docs.rs/clap/latest/clap/)
- [rayon 并行模式指南](https://docs.rs/rayon/latest/rayon/)
- [indicatif 多进度条示例](https://docs.rs/indicatif/latest/indicatif/)
- [tabled 表格样式](https://docs.rs/tabled/latest/tabled/)
- [miette 错误诊断](https://docs.rs/miette/latest/miette/)

## 附录 B: 配置文件完整示例

```toml
# ~/.config/git-helper/config.toml

[general]
# 默认作者过滤（用于统计和日志）
default_author = ""
# 默认扫描深度
scan_depth = 3
# 并行任务数（0 = 自动检测）
parallel_jobs = 0

[display]
# 终端彩色输出
color = true
# 默认表格样式: "sharp", "rounded", "psql", "markdown"
table_style = "rounded"

[audit]
# 启用操作日志
enabled = true
# 日志保留天数
retention_days = 90

[ignore]
# 自定义忽略文件模式
extra_patterns = [
    "*.generated.rs",
    "test-fixtures/",
]

[alias]
# 命令别名
st = "stats"
br = "branch"
co = "branch switch-all"
```
