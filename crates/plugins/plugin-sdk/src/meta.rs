//! copyright © ecdata.cn 2026 - present
//! 
//! created shaipe by 2026-05-18 17:06:03

use serde::{Deserialize, Serialize};

/// 插件元数据（进程 / 清单级信息）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMeta {
    // 插件唯一标识
    pub id: String,
    // 插件图标
    pub icon: String,
    // 插件名称
    pub name: String,
	// 插件描述
	pub description: String,
    // 插件版本
    pub version: String,
}

// 插件元数据实现
impl PluginMeta {
    // 创建插件元数据
    pub fn new(id: impl Into<String>, icon: impl Into<String>, name: impl Into<String>, description: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            name: name.into(),
            description: description.into(),
            version: version.into(),
        }
    }
}
