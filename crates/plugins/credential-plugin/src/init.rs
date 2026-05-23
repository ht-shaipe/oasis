//! 凭证管理插件 - 初始化服务

use crate::encryption::EncryptionService;
use crate::service::CredentialService;
use anyhow::Result;
use std::path::PathBuf;

/// 凭证管理器初始化
pub struct CredentialManagerInit;

impl CredentialManagerInit {
    /// 初始化凭证管理器
    pub fn initialize() -> Result<CredentialService> {
        let db_path = Self::get_db_path()?;
        let service = CredentialService::new(db_path)?;

        // 检查是否需要初始化主密钥
        if service.get_master_key()?.is_none() {
            log::info!("No master key found, initializing default master key...");
            let password = "default_password";
            let config = EncryptionService::initialize_master_key(password)?;
            service.set_master_key(config)?;
        }

        log::info!("Credential manager initialized successfully");
        Ok(service)
    }

    /// 获取数据库路径
    fn get_db_path() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let mut path = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
            path.push("Library");
            path.push("Application Support");
            path.push("oasis");
            std::fs::create_dir_all(&path)?;
            path.push("credentials.db");
            Ok(path)
        }

        #[cfg(target_os = "windows")]
        {
            let mut path = dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?;
            path.push("oasis");
            std::fs::create_dir_all(&path)?;
            path.push("credentials.db");
            Ok(path)
        }

        #[cfg(target_os = "linux")]
        {
            let mut path = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
            path.push("oasis");
            std::fs::create_dir_all(&path)?;
            path.push("credentials.db");
            Ok(path)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(anyhow::anyhow!("Unsupported platform"))
        }
    }
}