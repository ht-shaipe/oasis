//! copyright © ecdata.cn 2026 - present
//! 
//! created shaipe by 2026-05-18 17:10:44

use crate::{PluginContext, PluginError, PluginMeta};

pub trait Plugin: Send + Sync + 'static {
    // 获取插件元数据
    fn meta(&self) -> PluginMeta;
	
    // 加载插件
    fn on_load(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
	// 卸载插件
	fn on_unload(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
		Ok(())
	}
}

