//! 凭证详情页面 - UI和业务逻辑

use crate::state::CredentialPluginState;
use plugin_sdk::{UiNode, UiSchema};
use std::sync::{Arc, Mutex};

/// 构建凭证详情页面的UI schema
pub fn schema_credential_detail() -> UiSchema {
    let info_card = UiNode::new("card")
        .prop("title", serde_json::json!("凭证详情"))
        .child(UiNode::info(&[
            ("名称", "credential_detail.credential.name"),
            ("平台", "credential_detail.credential.platform"),
            ("分类", "credential_detail.credential.category"),
            ("用户名", "credential_detail.credential.username"),
            ("密码", "credential_detail.credential.password_masked"),
            ("标签", "credential_detail.credential.tags"),
            ("备注", "credential_detail.credential.notes"),
            ("状态", "credential_detail.credential.is_active"),
            ("创建时间", "credential_detail.credential.created_at"),
            ("更新时间", "credential_detail.credential.updated_at"),
        ]));

    let actions = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("margin_top", serde_json::json!(16))
        .child(UiNode::button("编辑", "edit_current_credential").prop("variant", serde_json::json!("primary")))
        .child(UiNode::button("删除", "delete_current_credential").prop("variant", serde_json::json!("danger")))
        .child(UiNode::button("显示/隐藏密码", "toggle_password_visibility"));

    let audit_logs = UiNode::new("card")
        .prop("title", serde_json::json!("审计日志"))
        .prop("margin_top", serde_json::json!(16))
        .child(UiNode::table_mapped("audit_logs", vec![
            ("时间", "timestamp"),
            ("操作", "action"),
            ("IP地址", "ip_address"),
            ("结果", "result"),
        ]));

    UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![info_card, actions, audit_logs],
        ..Default::default()
    }
}

/// 凭证详情业务逻辑
pub struct CredentialDetailHandler {
    state: Arc<Mutex<CredentialPluginState>>,
    credential_service: Option<crate::service::CredentialService>,
}

impl CredentialDetailHandler {
    pub fn new(
        state: Arc<Mutex<CredentialPluginState>>,
        credential_service: Option<crate::service::CredentialService>,
    ) -> Self {
        Self {
            state,
            credential_service,
        }
    }

    /// 加载凭证详情
    pub fn load_credential_detail(&self, credential_id: &str) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => return,
        };

        let Ok(credential) = service.read(credential_id) else { return };

        // 转换为CredentialItem
        let credential_item = crate::state::CredentialItem {
            id: credential.id,
            name: credential.name,
            platform: credential.platform,
            category: credential.category,
            username: credential.username,
            password_masked: "********".to_string(),
            notes: credential.notes,
            is_active: credential.is_active,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
            expires_at: credential.expires_at,
            tags: credential.tags,
            extra_fields: credential.extra_fields,
            credential_type: crate::state::CredentialType::from_value(&credential.credential_type),
            ..Default::default()
        };

        let _ = self.state.lock().map(|mut s| {
            s.credential_detail.credential = Some(credential_item);
            s.credential_detail.show_password = false;
        });
    }

    /// 切换密码可见性
    pub fn toggle_password_visibility(&self) {
        let _ = self.state.lock().map(|mut s| {
            s.credential_detail.show_password = !s.credential_detail.show_password;
        });
    }

    /// 删除当前凭证
    pub fn delete_current_credential(&self) {
        if let Some(service) = &self.credential_service {
            let id = self.state.lock().ok()
                .and_then(|s| s.credential_detail.credential.as_ref().map(|c| c.id.clone()))
                .unwrap_or_default();
            if !id.is_empty() {
                let _ = service.delete(&id);
            }
        }
        let _ = self.state.lock().map(|mut s| s.selected_tool = crate::state::ToolId::CredentialList);
    }

    /// 编辑当前凭证
    pub fn edit_current_credential(&self) {
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = crate::state::ToolId::CredentialEdit;
            s.credential_edit.is_new = false;
            s.credential_edit.type_display = s.credential_edit.credential.credential_type.label().to_string();
        });
    }
}
