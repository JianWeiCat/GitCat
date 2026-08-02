//! 白名单管理

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhitelistConfig {
    pub protected: Vec<String>,
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        Self {
            protected: vec![
                "main".into(),
                "master".into(),
                "develop".into(),
                "release/*".into(),
                "hotfix/*".into(),
            ],
        }
    }
}

impl WhitelistConfig {
    /// 加载白名单
    pub fn load(path: Option<&str>) -> Result<Self, crate::error::GhError> {
        let config_path = if let Some(p) = path {
            std::path::PathBuf::from(p)
        } else {
            crate::config::data_dir()?.join("whitelist.toml")
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| crate::error::GhError::Config(format!("无法读取白名单: {}", e)))?;
            toml::from_str(&content)
                .map_err(|e| crate::error::GhError::Config(format!("白名单解析失败: {}", e)))
        } else {
            Ok(Self::default())
        }
    }

    /// 保存白名单
    pub fn save(&self) -> Result<(), crate::error::GhError> {
        let config_path = crate::config::data_dir()?.join("whitelist.toml");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::GhError::Config(format!("序列化失败: {}", e)))?;
        std::fs::write(&config_path, content)
            .map_err(|e| crate::error::GhError::Config(format!("无法保存白名单: {}", e)))?;
        Ok(())
    }

    /// 添加分支到白名单
    pub fn add(&mut self, branch: String) {
        if !self.protected.contains(&branch) {
            self.protected.push(branch);
        }
    }

    /// 从白名单移除分支
    pub fn remove(&mut self, branch: &str) {
        self.protected.retain(|b| b != branch);
    }
}
