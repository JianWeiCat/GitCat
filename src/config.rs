//! 配置文件加载与管理
//!
//! 配置文件位置: `~/.config/git-helper/config.toml`

use serde::{Deserialize, Serialize};

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub alias: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// 默认作者过滤
    #[serde(default)]
    pub default_author: String,
    /// 默认扫描深度
    #[serde(default = "default_scan_depth")]
    pub scan_depth: usize,
    /// 并行任务数（0 = 自动检测）
    #[serde(default)]
    pub parallel_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 终端彩色输出
    #[serde(default = "default_true")]
    pub color: bool,
    /// 默认表格样式
    #[serde(default = "default_table_style")]
    pub table_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 启用操作日志
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 日志保留天数
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// 自定义忽略文件模式
    #[serde(default)]
    pub extra_patterns: Vec<String>,
}

// 默认值函数
fn default_scan_depth() -> usize { 3 }
fn default_true() -> bool { true }
fn default_table_style() -> String { "rounded".into() }
fn default_retention_days() -> u32 { 90 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            display: DisplayConfig::default(),
            audit: AuditConfig::default(),
            ignore: IgnoreConfig::default(),
            alias: std::collections::HashMap::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_author: String::new(),
            scan_depth: default_scan_depth(),
            parallel_jobs: 0,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            table_style: default_table_style(),
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: default_retention_days(),
        }
    }
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            extra_patterns: Vec::new(),
        }
    }
}

impl AppConfig {
    /// 从默认路径加载配置
    pub fn load() -> Result<Self, crate::error::GhError> {
        let config_path = config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| crate::error::GhError::Config(format!("无法读取配置文件: {}", e)))?;
            toml::from_str(&content)
                .map_err(|e| crate::error::GhError::Config(format!("配置文件解析失败: {}", e)))
        } else {
            Ok(Self::default())
        }
    }

    /// 保存配置到默认路径
    pub fn save(&self) -> Result<(), crate::error::GhError> {
        let config_path = config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::error::GhError::Config(format!("无法创建配置目录: {}", e)))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::GhError::Config(format!("配置序列化失败: {}", e)))?;
        std::fs::write(&config_path, content)
            .map_err(|e| crate::error::GhError::Config(format!("无法写入配置文件: {}", e)))?;
        Ok(())
    }
}

/// 获取配置文件路径: ~/.config/git-helper/config.toml
pub fn config_path() -> Result<std::path::PathBuf, crate::error::GhError> {
    dirs::config_dir()
        .map(|d| d.join("git-helper").join("config.toml"))
        .ok_or_else(|| crate::error::GhError::Config("无法获取系统配置目录".into()))
}

/// 获取数据目录路径: ~/.local/share/git-helper/
pub fn data_dir() -> Result<std::path::PathBuf, crate::error::GhError> {
    dirs::data_dir()
        .map(|d| d.join("git-helper"))
        .ok_or_else(|| crate::error::GhError::Config("无法获取系统数据目录".into()))
}
