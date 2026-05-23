//! 凭证管理插件
//!
//! 从 master 分支的 core/credential_manager 迁移而来

mod models;
mod db;
mod encryption;
mod audit;
mod import_export;
mod service;
mod init;
mod state;
mod ui;
mod plugin;

use std::sync::Arc;
use plugin_sdk::Plugin;
pub use plugin::CredentialPlugin;

/// 插件入口 — 供宿主 libloading 调用
#[unsafe(no_mangle)]
unsafe extern "C" fn plugin_entry() -> Arc<dyn Plugin> {
    Arc::new(CredentialPlugin::new())
}
