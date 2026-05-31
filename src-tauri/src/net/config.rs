//! copyright © ecdata.cn 2024 - present
//! 代理配置模块
//! created by shaipe 20241223

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;
use toml;

// 默认加载静态全局
lazy_static! {
    pub static ref PROXY_CONFIG_CACHE: Mutex<Option<ProxyConfig>> = Mutex::new(None);
}

/// 代理配置结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    /// 默认代理域名
    pub default: String,
    /// 特殊配置 - 特定API路径的代理域名
    pub specific: HashMap<String, String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            default: "http://127.0.0.1:8099".to_string(),
            specific: HashMap::new(),
        }
    }
}

impl ProxyConfig {
    /// 创建新的代理配置
    pub fn new(conf_path: &str) -> Self {
        let config = Self::load_toml(conf_path).unwrap_or_default();

        // 写入缓存中
        Self::set(config.clone());

        log!("✅ 代理配置加载成功: {:?}", config);

        config
    }

    /// 从配置文件加载配置
    pub fn load_toml(conf_path: &str) -> tube::Result<ProxyConfig> {
        match Self::load_string(conf_path) {
            Ok(content) => {
                let config: ProxyConfig = match toml::de::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        err_log!("⚠️ 从配置文件加载失败: {}, 使用默认配置", e);
                        Self::default()
                    }
                };
                log!("✅ 从配置文件加载代理配置成功");
                Ok(config)
            }
            Err(e) => {
                err_log!("⚠️ 从配置文件加载失败: {}, 使用默认配置", e);
                Ok(Self::default())
            }
        }
    }

    /// 加载配置文件的字符串
    fn load_string(conf_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut f = File::open(conf_path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        Ok(content)
    }

    /// 保存配置到缓存
    pub fn set(config: ProxyConfig) {
        if let Ok(mut cache) = PROXY_CONFIG_CACHE.lock() {
            *cache = Some(config);
        }
    }

    /// 获取缓存中的配置
    pub fn get() -> ProxyConfig {
        if let Ok(cache) = PROXY_CONFIG_CACHE.lock() {
            if let Some(config) = cache.as_ref() {
                return config.clone();
            }
        }
        // 如果没有缓存，则加载配置文件
        Self::new("proxy.toml")
    }
}
