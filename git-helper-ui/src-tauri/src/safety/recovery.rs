//! 简易恢复 — 基于审计日志尝试撤销

use crate::safety::audit::{AuditEntry, AuditLog};

/// 查找指定 ID 的审计条目
pub fn find_entry(log: &AuditLog, id: &str) -> Result<Option<AuditEntry>, crate::error::GhError> {
    let entries = log.read_all()?;
    Ok(entries.into_iter().find(|e| e.id.starts_with(id)))
}

/// 格式化审计日志
pub fn format_audit_log(entries: &[AuditEntry]) -> String {
    if entries.is_empty() {
        return "暂无操作记录。\n".into();
    }

    let mut output = String::from("=== 操作审计日志 ===\n\n");
    for entry in entries {
        output.push_str(&format!(
            "{} | {} | {} | {} | {}\n",
            &entry.timestamp[..entry.timestamp.len().min(19)],
            if entry.dry_run { "🔍 DRY-RUN" } else { "⚡ EXECUTED" },
            entry.operation,
            entry.repository,
            entry.result,
        ));
    }
    output
}
