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

pub use plugin::CredentialPlugin;

/// 创建插件实例
#[unsafe(no_mangle)]
pub extern "C" fn create_plugin() -> *mut std::ffi::c_void {
    let plugin = Box::new(CredentialPlugin::new());
    Box::into_raw(plugin) as *mut _
}

/// 销毁插件实例
#[unsafe(no_mangle)]
pub extern "C" fn destroy_plugin(ptr: *mut std::ffi::c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut CredentialPlugin);
    }
}
