//! 安全守卫 — 危险操作拦截

/// 危险操作列表（模块名, 命令名）
const DANGEROUS_OPS: &[(&str, &str)] = &[
    ("branch", "cleanup"),
    ("reset", "hard"),
    ("stash", "clean"),
    ("stash", "clear"),
    ("multi", "gc"),
];

/// 检查是否为危险操作
pub fn is_dangerous(module: &str, command: Option<&str>) -> bool {
    if let Some(cmd) = command {
        DANGEROUS_OPS.iter().any(|(m, c)| *m == module && *c == cmd)
    } else {
        false
    }
}

/// 危险操作安全校验
pub fn check_safety(
    module: &str,
    command: Option<&str>,
    dry_run: bool,
    force: bool,
) -> Result<SafetyDecision, crate::error::GhError> {
    if !is_dangerous(module, command) {
        return Ok(SafetyDecision::Proceed);
    }

    if dry_run {
        return Ok(SafetyDecision::DryRun);
    }

    if force {
        return Ok(SafetyDecision::ForceProceed);
    }

    // 无 --dry-run 也无 --force → 拦截
    Err(crate::error::GhError::Blocked(format!(
        "⚠️ 危险操作 '{} {}' 被拦截!\n\n\
        安全规则: 此操作将修改/删除仓库数据, 请先使用 --dry-run 预览。\n\
        \n  gh {} {} --dry-run    预览将要发生的变更\n  gh {} {} --force      确认后强制执行\n\
        \n💡 建议: 先用 --dry-run 查看效果, 确认无误再加 --force。",
        module, command.unwrap_or(""),
        module, command.unwrap_or(""),
        module, command.unwrap_or(""),
    )))
}

/// 安全检查结果
pub enum SafetyDecision {
    /// 直接执行
    Proceed,
    /// 预演模式
    DryRun,
    /// 强制执行
    ForceProceed,
}
