//! 统一错误类型定义

use thiserror::Error;

/// git-helper 全局错误类型
#[derive(Debug, Error)]
pub enum GhError {
    /// Git 仓库相关错误
    #[error("Git 仓库错误: {0}")]
    Git(String),

    /// IO 错误
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 参数错误
    #[error("参数错误: {0}")]
    Args(String),

    /// 操作被拦截（安全策略）
    #[error("操作被拦截: {0}")]
    Blocked(String),

    /// 未找到
    #[error("未找到: {0}")]
    NotFound(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialize(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

/// 将底层 Git 错误转换为用户友好的中文提示
pub fn into_friendly_error(err: &dyn std::error::Error, _context: &str) -> GhError {
    into_friendly_error_msg(&err.to_string())
        .map_or_else(|| GhError::Other(err.to_string()), GhError::Git)
}

/// 从错误消息字符串生成友好提示
pub fn into_friendly_error_msg(msg: &str) -> Option<String> {
    if msg.contains("not a git repository") || msg.contains("bare git repository") {
        Some("当前目录不是一个 Git 仓库。请进入 Git 仓库目录后重试。".into())
    } else if msg.contains("detached HEAD") {
        Some("当前处于 detached HEAD 状态。建议先切换到具体分支。".into())
    } else if msg.contains("unborn branch") {
        Some("当前仓库尚未创建任何提交，请先执行首次提交后再操作。".into())
    } else if msg.contains("permission denied") {
        Some("权限不足。请检查文件/目录权限，或确认仓库未被其他进程占用。".into())
    } else if msg.contains("uncommitted changes") {
        Some("检测到未提交更改，操作被阻止。请先 git commit 或 git stash 保存更改。".into())
    } else {
        None
    }
}
