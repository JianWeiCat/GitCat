# GitCat

纯 Rust + Tauri 2 跨平台 Git 仓库增强桌面工具箱 —— 安全、高效的图形化 Git 管理体验。

## 为什么选择 GitCat？

| 日常痛点 | GitCat 方案 |
|---------|---------------|
| 手动 `git log` 拼统计，周报难写 | 一键统计 + Markdown/CSV 导出 |
| 逐个仓库 `cd && git pull` 效率低 | 目录扫描，批量并发执行（status / pull / gc） |
| `git branch -d` 怕误删 | 安全删除 + dry-run 预演 + 白名单保护 |
| `git reset --hard` 没后悔药 | 自动备份分支 + 追加式操作审计日志 |
| Shell 脚本 Windows 跑不了 | 纯 Rust + Tauri，全平台静态编译 |

## 功能区域

| 区域 | 功能 |
|------|------|
| 仓库发现 | 全盘扫描 / 目录扫描，筛选，批量勾选，最近使用 |
| 变更 | Staged / Unstaged / Untracked / 冲突分组，文件 Diff，暂存，提交 |
| 提交历史 | 多泳道拓扑图，搜索，引用标签，父提交，变更文件统计 |
| 分支 | 搜索，创建切换，安全删除，保护标识 |
| 同步 | Fetch / Pull / Push，ahead/behind 提示 |
| 仓库洞察 | 语言体积，贡献者统计 |
| 批量任务 | 多仓库 status / pull / gc，逐仓库结果报告 |
| Agent（设计阶段） | 仓库只读助手，Diff 审查，提交说明生成，Git 错误诊断 |

## 技术栈

| 层面 | 选型 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | React 19 + TypeScript，@lobehub/ui + Ant Design 6 |
| 图标 | Lucide React |
| 动画 | Motion |
| 构建 | Vite 8 + Rolldown |
| 后端语言 | Rust 2021 Edition |
| Git 底层 | gix (gitoxide) — 纯 Rust 实现，不依赖系统 git |
| 并发 | rayon |
| 平台 | Windows / macOS / Linux (x86_64 + aarch64) |

## 快速开始

### 环境要求

- Rust 1.85+
- Node.js 20+
- Windows / macOS / Linux

### 开发启动

```bash
# 克隆仓库
git clone https://github.com/JianWeiCat/Git-Helper.git
cd Git-Helper

# 安装前端依赖
cd git-helper-ui
npm install

# 启动 Tauri 开发模式
cd src-tauri
cargo tauri dev
```

### 生产构建

```bash
cd git-helper-ui/src-tauri
cargo tauri build
```

构建产物为单文件可执行程序，位于 `git-helper-ui/src-tauri/target/release/`。

## 项目文档

| 文档 | 说明 |
|------|------|
| [WORKFLOW.md](./git-helper-ui/WORKFLOW.md) | 产品工作流、信息架构与回归场景 |
| [PROJECT_CONSTRAINTS.md](./git-helper-ui/PROJECT_CONSTRAINTS.md) | 工程硬约束 —— 数据边界、安全边界、性能门禁、开发与发布门禁 |
| [AGENT_MODULE_DESIGN.md](./git-helper-ui/AGENT_MODULE_DESIGN.md) | Agent 模块技术方案 —— 架构、工具设计、审批模型、分阶段实施计划 |

## 开发

```bash
# 前端类型检查 + 构建
cd git-helper-ui
npm run build
npm run lint

# Rust 格式化 + 测试
cargo fmt --check
cargo test
cargo test --manifest-path git-helper-ui/src-tauri/Cargo.toml
```

## 项目结构

```
GitCat/
├── src/                          # Rust 核心库（Git 操作、统计、安全层）
│   ├── git_backend/              # gix-based 纯 Rust Git 实现
│   ├── branch/                   # 分支管理、清理、白名单
│   ├── log_ops/                  # 日志查看、重置、stash
│   ├── multi_repo/               # 多仓库扫描、并发执行
│   ├── stats/                    # 贡献统计、报表导出
│   └── safety/                   # 安全守护、审计日志、恢复
├── git-helper-ui/
│   ├── src/                      # React 前端（App.tsx + 样式）
│   └── src-tauri/src/            # Tauri Rust 后端（命令、状态管理）
└── tests/                        # 集成测试
```

## License

MIT OR Apache-2.0
