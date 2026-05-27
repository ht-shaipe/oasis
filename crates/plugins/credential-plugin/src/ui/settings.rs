//! 设置页面 - UI和业务逻辑

use crate::state::CredentialPluginState;
use plugin_sdk::{UiNode, UiSchema};
use std::sync::{Arc, Mutex};

/// 构建设置页面的UI schema
pub fn schema_settings() -> UiSchema {
    let form = UiNode::new("card")
        .prop("title", serde_json::json!("安全设置"))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(UiNode::input("settings.change_password", "新密码").prop("type", serde_json::json!("password")))
                .child(UiNode::input("settings.confirm_password", "确认密码").prop("type", serde_json::json!("password")))
                .child(UiNode::button("修改密码", "change_master_password").prop("variant", serde_json::json!("primary")))
                .child(UiNode::display("settings.password_error").prop("type", serde_json::json!("error")))
        );

    let info_card = UiNode::new("card")
        .prop("title", serde_json::json!("关于"))
        .prop("margin_top", serde_json::json!(16))
        .child(
            UiNode::info(&[
                ("版本", "1.0.0"),
                ("加密算法", "AES-256-GCM"),
                ("密钥派生", "PBKDF2-HMAC-SHA256"),
                ("数据库", "SQLite"),
            ])
        );

    UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![form, info_card],
        ..Default::default()
    }
}

/// 设置业务逻辑
pub struct SettingsHandler {
    state: Arc<Mutex<CredentialPluginState>>,
    credential_service: Option<crate::service::CredentialService>,
}

impl SettingsHandler {
    pub fn new(
        state: Arc<Mutex<CredentialPluginState>>,
        credential_service: Option<crate::service::CredentialService>,
    ) -> Self {
        Self {
            state,
            credential_service,
        }
    }

    /// 修改主密钥
    pub fn change_master_password(&self, new_password: &str, confirm_password: &str) -> serde_json::Value {
        if new_password != confirm_password {
            let _ = self.state.lock().map(|mut s| {
                s.settings.password_error = Some("两次输入的密码不一致".to_string());
            });
            return serde_json::json!({"success": false, "message": "两次输入的密码不一致"});
        }

        if new_password.len() < 8 {
            let _ = self.state.lock().map(|mut s| {
                s.settings.password_error = Some("密码长度不能少于8位".to_string());
            });
            return serde_json::json!({"success": false, "message": "密码长度不能少于8位"});
        }

        let service = match &self.credential_service {
            Some(s) => s,
            None => return serde_json::json!({"success": false, "message": "服务未初始化"}),
        };

        use crate::encryption::EncryptionService;

        // 创建新的主密钥配置
        let config = match EncryptionService::initialize_master_key(new_password) {
            Ok(config) => config,
            Err(e) => {
                log::error!("创建主密钥失败: {}", e);
                return serde_json::json!({"success": false, "message": format!("创建主密钥失败: {}", e)});
            }
        };

        // 保存新的主密钥配置
        match service.set_master_key(config) {
            Ok(_) => {
                log::info!("主密钥修改成功");
                let _ = self.state.lock().map(|mut s| {
                    s.settings.password_error = None;
                    s.settings.change_password = String::new();
                    s.settings.confirm_password = String::new();
                });
                serde_json::json!({"success": true, "message": "密码修改成功"})
            }
            Err(e) => {
                log::error!("保存主密钥失败: {}", e);
                serde_json::json!({"success": false, "message": format!("保存失败: {}", e)})
            }
        }
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self) -> serde_json::Value {
        let service = match &self.credential_service {
            Some(s) => s,
            None => return serde_json::json!({"success": false, "message": "服务未初始化"}),
        };

        let Ok(credentials) = service.list_all() else {
            return serde_json::json!({"success": false, "message": "获取统计信息失败"});
        };

        // 简化版本，暂时不实现审计日志统计
        serde_json::json!({
            "success": true,
            "data": {
                "total_credentials": credentials.len(),
                "total_audit_logs": 0,
                "active_credentials": credentials.iter().filter(|c| c.is_active).count(),
            }
        })
    }
}
