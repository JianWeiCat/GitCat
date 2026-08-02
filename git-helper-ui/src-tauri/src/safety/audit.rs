//! 操作审计日志

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 审计条目
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub repository: String,
    pub details: String,
    pub dry_run: bool,
    pub result: String,
}

/// 审计日志管理器
pub struct AuditLog {
    path: PathBuf,
}

impl AuditLog {
    /// 打开审计日志
    pub fn open() -> Result<Self, crate::error::GhError> {
        let data_dir = crate::config::data_dir()?;
        std::fs::create_dir_all(&data_dir).ok();
        let path = data_dir.join("audit.jsonl");
        Ok(Self { path })
    }

    /// 记录一条操作
    pub fn record(
        &self,
        operation: &str,
        repository: &str,
        details: &str,
        dry_run: bool,
        result: &str,
    ) -> Result<(), crate::error::GhError> {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono_now(),
            operation: operation.to_string(),
            repository: repository.to_string(),
            details: details.to_string(),
            dry_run,
            result: result.to_string(),
        };

        let mut line = serde_json::to_string(&entry)
            .map_err(|e| crate::error::GhError::Config(format!("序列化审计日志失败: {}", e)))?;
        line.push('\n');

        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| crate::error::GhError::Config(format!("无法打开审计日志: {}", e)))?;

        std::fs::write(&self.path, &line)
            .map_err(|_| ())
            .or_else(|_| {
                // 追加写入
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                    .map_err(|e| crate::error::GhError::Config(format!("无法打开审计日志: {}", e)))?;
                file.write_all(line.as_bytes())
                    .map_err(|e| crate::error::GhError::Config(format!("写入审计日志失败: {}", e)))
            })?;

        Ok(())
    }

    /// 读取所有审计记录
    pub fn read_all(&self) -> Result<Vec<AuditEntry>, crate::error::GhError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| crate::error::GhError::Config(format!("无法读取审计日志: {}", e)))?;

        let entries: Vec<AuditEntry> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();

        Ok(entries)
    }
}

fn chrono_now() -> String {
    std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S%z"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}
