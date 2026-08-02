# Git Helper 学习 + 实现计划

> 你有软件开发经验，但 Rust 和 Git 底层操作不熟 —— 这份计划就是帮你从能看懂到能写出来的。

---

## 一、先搞清楚一个核心问题：用代码操作 Git 到底在做什么

### 你熟悉的 vs 项目要做的

```
你熟悉的（人在终端敲命令）              项目要做的（程序调用函数）
────────────────────────────────────────────────────────────────
git log --oneline                   遍历 .git/objects/ 里的 commit 对象
git branch                         读取 .git/refs/heads/ 目录下的文件
git checkout -b xxx                 在 .git/refs/heads/ 下新建一个文件
git diff HEAD~1                     对比两个 commit 里同一个文件的内容差异
git merge-base main dev             用一个算法算两个分支的分叉点
```

**一句话**：Git 就是一个文件数据库（`.git/` 目录），所有操作本质上都是读/写文件。把这一点理解了，整个项目就没有黑魔法了。

### 跟 Java/C#/Python 读写文件的区别

| 你之前写的 | Git 底层 |
|-----------|---------|
| `File.ReadAllText(path)` | 打开 `.git/objects/` 下某个文件 |
| 拿到字符串直接用 | 得用 zlib 解压，再按 Git 二进制格式解析出 author、message 等字段 |
| 普通文件系统 | Git 的对象存储是 content-addressed（文件名是内容的 hash） |

**这就是 gix 的作用**：帮你做 zlib 解压 + 二进制格式解析，你直接用结构体拿数据。

---

## 二、三个前置概念（半小时能理解，但必须理解）

你在接触 Rust 代码之前，先在终端做 3 个实验：

### 实验 1：commit 长什么样

```bash
# 进到任意 Git 仓库
cd 你的项目目录
# 看最近一个 commit 的原始内容（这是 Git 真正存的东西）
git log -1 --format="%H"           # 抄下这个 hash
git cat-file -p 刚才抄的hash        # 这就是一个 commit 的原始内容
```

你会看到：
```
tree abc123...          ← 这个 commit 对应的目录快照
parent def456...        ← 上一个 commit（这就是"历史"）
author Zhang 123@...    ← 作者信息
committer Zhang 123@... ← 提交者信息

feat: add stats         ← 提交信息
```

### 实验 2：分支就是一个指针

```bash
cat .git/refs/heads/main     # 里面就一行：某个 commit hash
cat .git/HEAD                 # 里面就一行：ref: refs/heads/main
```

删除分支 = 删除 `refs/heads/xxx` 这个文件。commit 本身还在。

### 实验 3：diff 的数据长什么样

```bash
git diff HEAD~1..HEAD --numstat    # 纯数字格式，程序就是解析这个
# 输出格式：added  deleted  filename
# 3       2       src/main.rs
# 120     8       src/stats.rs
```

---

## 三、学 Rust 的节奏（跟项目同步，不单独学）

### 原则：你不需要"学好 Rust 再开始写项目"

你需要的 Rust 知识，**全部出现在你的 git-helper 项目代码里**。遇到不懂的语法现查就行，比整本读完书快 10 倍。

### 对照表：项目文件 → 涉及的 Rust 知识点

按实现顺序排列，从简单到复杂：

| 顺序 | 文件 | 涉及的 Rust 知识点 | 对应你熟悉的概念 |
|------|------|-------------------|----------------|
| ① | `src/stats/filter.rs` | `&str`、`Vec`、闭包、`#[test]` | 字符串比较函数 + if 条件 |
| ② | `src/config.rs` | `struct`、`enum`、`Option<T>`、`serde` 宏 | Java 的 class + annotation |
| ③ | `src/error.rs` | `enum` 进阶、`thiserror` | Java 的异常类 |
| ④ | `src/cli.rs` | `clap` Derive 宏、子命令枚举 | Java 的命令行参数库 |
| ⑤ | `src/git_backend/repo.rs` | `impl` 方法、`Path`、`Result` 返回 | Java 的类方法 |
| ⑥ | `src/stats/engine.rs` | `HashMap`、迭代器链（`.filter().map().collect()`） | Java 的 Stream API |
| ⑦ | `src/stats/analyzer.rs` | 泛型、trait bound | Java 的 `<T extends Xxx>` |
| ⑧ | `src/stats/exporter.rs` | 文件 I/O、`serde` 序列化 | Java 的 JSON 序列化库 |
| ⑨ | `src/multi_repo/scanner.rs` | `walkdir` 迭代器、错误处理 | Java 目录遍历 |
| ⑩ | `src/multi_repo/runner.rs` | `rayon` 并行迭代器 | Java 线程池 |
| ⑪ | `src/branch/cleanup.rs` | `gix` API 调用 | 就是调库函数 |
| ⑫ | `src/safety/guard.rs` | 模式匹配（`match`） | Java 的 switch |
| ⑬ | `src/safety/audit.rs` | `serde_json` + 文件追加写 | Java 的日志 |

**关键点**：前 5 个文件写完，Rust 里 80% 的常用语法你就碰全了。后面所有模块就是换着组合这些语法。

### 遇到什么不知道怎么写的？

| 你想写的 | Rust 怎么写 | 查什么关键词 |
|---------|-----------|------------|
| if 判断 | `if condition { ... }` | 一模一样 |
| 循环 | `for item in vec { ... }` | for in |
| 列表 | `Vec<String>` | rust vec |
| 字典/map | `HashMap<String, u32>` | rust hashmap |
| 函数返回空 | `Ok(())` 或 `Result<(), Error>` | rust result |
| 函数返回数据 | `-> Result<MyStruct, Error>` | rust result |
| 打印到屏幕 | `println!("hello {}", name)` | rust println |
| 字符串转数字 | `"42".parse::<u32>()` | rust parse |
| 拼接字符串 | `format!("{}-{}", a, b)` | rust format |
| null 检查 | `Option<T>` — `Some(x)` / `None` | rust option |
| 异常处理 | `Result<T, E>` — `Ok(x)` / `Err(e)` | rust result |

---

## 四、分阶段实现路线（这才是重点）

### 总览

```
Phase 1: 搭骨架（已完成 ✅）
Phase 2: Git 后端核心（你现在在这里 👈）
Phase 3: 统计模块
Phase 4: 分支模块
Phase 5: 多仓库模块
Phase 6: 安全模块
Phase 7: 打磨（进度条/颜色/报表导出）
```

### Phase 1：搭骨架 ✅ 已完成

```
src/
├── main.rs          ← 入口
├── cli.rs           ← 命令定义（占位）
├── config.rs        ← 配置文件读取
├── error.rs         ← 错误类型
├── git_backend/     ← Git 操作（空壳）
├── stats/           ← 统计（空壳）
├── branch/          ← 分支（空壳）
├── log_ops/         ← 日志（空壳）
├── multi_repo/      ← 多仓库（空壳）
└── safety/          ← 安全（空壳）
```

### Phase 2：Git 后端核心（预计 1-2 天）

**目标**：能在 Rust 代码里打开一个 Git 仓库，遍历 commit，读 diff

**要写的文件**：
```
src/git_backend/
├── repo.rs          ← 打开仓库（已写）
├── commit.rs        ← 遍历 commit 列表 ← 接下来写
├── diff.rs          ← 读 diff 数据       ← 接下来写
├── branch.rs        ← 读分支列表        ← 接下来写
├── reference.rs     ← 引用操作
└── fallback.rs      ← 调系统 git 命令兜底
```

**工作方式**：
1. 先用 `std::process::Command` 调系统 `git` 命令把功能跑通（比如 `git log --format=...`）
2. 验证逻辑对了之后，再换成 `gix` 纯 Rust 实现
3. 两个实现共用一个 trait，方便对比验证

> **为什么这样**：先不管 gix 的复杂 API，直接用你熟悉的 git 命令拿数据，功能跑通再说。后续想换 gix 的时候，只要改 backend 层的实现，上层模块完全不动。

### Phase 3：统计模块（预计 2-3 天）

**目标**：能统计一个仓库里谁写了多少代码

**先写的**：
```
src/stats/
├── filter.rs        ← 判断文件要不要忽略（纯逻辑，最简单）
├── engine.rs        ← 调 backend 拿 commits，按作者聚合
├── analyzer.rs      ← 判断一行是代码/注释/空行
├── report.rs        ← 把聚合结果排好序
└── exporter.rs      ← 输出 Markdown/CSV
```

**实现顺序**：
1. `filter.rs` — 纯条件判断，10 分钟搞定，先建立信心
2. `engine.rs` — 循环 commit，用 HashMap 按作者累加，核心就 30 行
3. `analyzer.rs` — 判断代码/注释/空行，每种语言 3-5 条规则
4. `report.rs` — 按提交数/新增行排序
5. `exporter.rs` — 生成 Markdown 表格字符串

### Phase 4：分支模块（预计 1-2 天）

**目标**：列出分支、找已合并的、安全删除

```
src/branch/
├── cleanup.rs       ← 清理已合并分支
├── whitelist.rs     ← 白名单过滤
├── stale.rs         ← 休眠分支检测
└── batch_ops.rs     ← 批量操作
```

### Phase 5：多仓库模块（预计 1 天）

**目标**：扫目录 → 找到所有 Git 仓库 → 并发执行操作

```
src/multi_repo/
├── scanner.rs       ← walkdir 扫目录找 .git
├── runner.rs        ← rayon 并行跑
└── ops.rs           ← pull/gc/status 具体操作
```

### Phase 6：安全模块（预计 1 天）

**目标**：危险操作拦截、操作日志

```
src/safety/
├── guard.rs         ← 检查 --dry-run 是否传了
├── audit.rs         ← 写操作日志到文件
└── recovery.rs      ← 基于日志尝试恢复
```

### Phase 7：打磨（预计 2-3 天）

- CLI 输出加颜色（colored）
- 统计数据加进度条（indicatif）
- 错误信息替换为用户能看懂的提示（error.rs 里已有的）
- 补全脚本生成（clap_complete）

---

## 五、第一个要写的功能：`filter.rs`（现在就动手）

这是整个项目最简单、最能建立成就感的切入点。

```rust
// src/stats/filter.rs
// 判断文件要不要被统计忽略

const IGNORE_PATTERNS: &[&str] = &[
    // 编译中间文件
    "target/", "node_modules/", "dist/", "build/",
    ".next/", "__pycache__/",
    // 锁文件
    "Cargo.lock", "package-lock.json", "yarn.lock",
    // 二进制/资源文件
    "*.o", "*.exe", "*.dll", "*.so",
    "*.png", "*.jpg", "*.woff", "*.ttf",
];

pub fn should_ignore(path: &str) -> bool {
    IGNORE_PATTERNS.iter().any(|p| {
        if p.ends_with('/') { path.contains(p) }
        else if p.starts_with('*') { path.ends_with(&p[1..]) }
        else { path == *p }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_build_output() {
        assert!(should_ignore("target/debug/main.o"));
        assert!(should_ignore("node_modules/react/index.js"));
    }

    #[test]
    fn keeps_source_code() {
        assert!(!should_ignore("src/main.rs"));
        assert!(!should_ignore("README.md"));
    }

    #[test]
    fn ignores_lock_files() {
        assert!(should_ignore("Cargo.lock"));
        assert!(should_ignore("package-lock.json"));
    }
}
```

**这个文件包含了你需要的所有 Rust 基础**：
- `&str` — 字符串引用（不拷贝数据）
- `Vec` — 动态数组
- 闭包 `|p| { ... }` — 匿名函数，Java 的 lambda
- `.iter().any()` — 迭代器，Java 的 `.stream().anyMatch()`
- `#[test]` — 单元测试，写完就能跑 `cargo test`

---

## 六、常见"卡住了"场景速查

| 问题 | 原因 | 解决 |
|------|------|------|
| `cannot borrow as mutable` | Rust 不允许同时读写同一份数据 | 先 `.clone()`，后面再优化 |
| `use of moved value` | 所有权转移了，你还想用 | 用 `&` 引用传参，或者先 clone |
| `expected String, found &str` | 类型不匹配 | `&str` 是借用，加 `.to_string()` 变 String |
| `trait bound not satisfied` | 泛型参数没实现要求的接口 | 报错里会告诉你要加什么 trait |
| gix 文档看不懂 | gix 确实比较复杂 | 先用 `Command::new("git")` 调系统命令，功能跑通了再研究 gix |

---

## 七、推荐的学习资料

| 资源 | 用途 | 什么时候看 |
|------|------|-----------|
| [Rust Book 中文版](https://kaisery.github.io/trpl-zh-cn/) | 系统学习 | 前 4 章必读（所有权），其余当字典查 |
| [Rust By Example](https://rustwiki.org/zh-CN/rust-by-example/) | 边看例子边写 | 遇到不熟悉的语法时查 |
| [Rustlings](https://github.com/rust-lang/rustlings) | 100+ 个小练习 | 装一个，每天做 5 个，一周搞定常用语法 |
| [tabled 文档](https://docs.rs/tabled) | 终端表格输出 | 做 stats 输出表格时看 |
| [indicatif 文档](https://docs.rs/indicatif) | 进度条 | 做多仓库统计时加进度条 |

---

## 八、每日行动清单

| 天 | 做什么 | 预计耗时 |
|----|--------|---------|
| 1 | 做实验 1/2/3 理解 Git 底层 + 装 Rustlings 做前 10 个练习 | 1h |
| 2 | 写 `filter.rs`（复制上面的代码跑通 `cargo test`） | 30min |
| 3 | Rustlings 第 11-30（vec / string / struct） + 写 `config.rs` 的更多测试 | 1h |
| 4 | Rustlings 第 31-50（enum / match / Result）+ 读 `error.rs` 理解 enum 做错误类型 | 1h |
| 5 | 写 `engine.rs`（用 `Command::new("git")` 先跑通统计逻辑） | 1.5h |
| 6 | Rustlings 第 51-70（迭代器 / HashMap / 泛型） | 1h |
| 7 | 写 `analyzer.rs`（行分类）+ `report.rs`（排序输出） | 1.5h |
| 8 | 写 `scanner.rs`（walkdir 扫仓库列表） | 1h |
| 9 | 写 `runner.rs`（rayon 并发 + indicatif 进度条） | 1.5h |
| 10 | 写 `cleanup.rs`（分支清理核心逻辑） | 1.5h |
| 11 | 写 `guard.rs` + `audit.rs`（安全模块） | 1h |
| 12 | 全部连起来 + `cargo test` 保证测试全过 | 2h |

> 每天 1-1.5 小时，两周写完核心功能。
