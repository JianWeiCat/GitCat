//! Git 底层抽象层
//!
//! 基于 gix (gitoxide) 纯 Rust 实现的 Git 仓库操作封装。
//! 当 gix 对某些写入操作支持不完整时，通过 `fallback` 模块调用系统 git。

pub mod repo;
pub mod commit;
pub mod diff;
pub mod branch;
pub mod reference;
pub mod fallback;
