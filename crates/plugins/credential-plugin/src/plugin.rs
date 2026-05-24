//! 凭证管理插件 - 简化版实现

use crate::init::CredentialManagerInit;
use crate::service::CredentialService;
use crate::state::{CredentialPluginState, CredentialType, ToolId};
use crate::ui::{build_sidebar, schema_audit_logs, schema_credential_detail, schema_credential_edit, schema_credential_list, schema_import_export, schema_settings};
use plugin_sdk::{Plugin, PluginMeta, UiNode, UiSchema};
use std::sync::{Arc, Mutex};

pub struct CredentialPlugin {
    state: Arc<Mutex<CredentialPluginState>>,
    credential_service: Option<CredentialService>,
}

impl CredentialPlugin {
    pub fn new() -> Self {
        let service = CredentialManagerInit::initialize().ok();
        
        let plugin = Self {
            state: Arc::new(Mutex::new(CredentialPluginState::default())),
            credential_service: service,
        };

        // 加载初始数据
        plugin.load_credential_list();

        plugin
    }

    fn load_credential_list(&self) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => return,
        };

        let Ok(credentials) = service.list_all() else { return };

        let ui_credentials: Vec<crate::state::CredentialItem> = credentials
            .into_iter()
            .map(|cred| crate::state::CredentialItem {
                id: cred.id,
                name: cred.name,
                platform: cred.platform,
                category: cred.category,
                username: cred.username,
                password_masked: "********".to_string(),
                notes: cred.notes,
                is_active: cred.is_active,
                created_at: cred.created_at,
                updated_at: cred.updated_at,
                expires_at: cred.expires_at,
                tags: cred.tags,
                extra_fields: cred.extra_fields,
                credential_type: CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        if let Ok(mut state) = self.state.lock() {
            state.credential_list.credentials = ui_credentials;
            state.credential_list.total_count = count;
            state.credential_list.loading = false;
        }
    }

    fn search_credentials(&self, query: &str) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => return,
        };

        let Ok(credentials) = service.search(query) else { return };

        let ui_credentials: Vec<crate::state::CredentialItem> = credentials
            .into_iter()
            .map(|cred| crate::state::CredentialItem {
                id: cred.id,
                name: cred.name,
                platform: cred.platform,
                category: cred.category,
                username: cred.username,
                password_masked: "********".to_string(),
                notes: cred.notes,
                is_active: cred.is_active,
                created_at: cred.created_at,
                updated_at: cred.updated_at,
                expires_at: cred.expires_at,
                tags: cred.tags,
                extra_fields: cred.extra_fields,
                credential_type: CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        if let Ok(mut state) = self.state.lock() {
            state.credential_list.credentials = ui_credentials;
            state.credential_list.total_count = count;
        }
    }
}

impl Plugin for CredentialPlugin {
    fn id(&self) -> &str {
        "credential-plugin"
    }

    fn meta(&self) -> PluginMeta {
        PluginMeta {
            id: "credential-plugin".into(),
            name: "凭证管理".into(),
            version: "1.0.0".into(),
            description: "安全的凭证管理插件，支持 CRUD、加密、导入导出和审计日志。".into(),
            icon: "key".into(),
        }
    }

    fn state(&self) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::to_value(&*state).unwrap_or_default()
    }

    fn handle_action(&self, action: &str, params: serde_json::Value) -> serde_json::Value {
        log::debug!("Credential plugin action: {} params: {:?}", action, params);

        match action {
            // ---- 侧边栏导航 ----
            "select_credential_list" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialList);
                self.load_credential_list();
            }
            "select_credential_create" => {
                let _ = self.state.lock().map(|mut s| {
                    s.selected_tool = ToolId::CredentialCreate;
                    s.credential_edit.is_new = true;
                    s.credential_edit.credential = Default::default();
                    s.credential_edit.type_display = CredentialType::default().label().to_string();
                });
            }
            "select_credential_edit" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialEdit);
            }
            "select_import_export" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::ImportExport);
            }
            "select_audit_logs" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::AuditLogs);
            }
            "select_settings" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::Settings);
            }

            // ---- 凭证类型切换 ----
            a if a.starts_with("change_type_to_") => {
                let type_label = a.strip_prefix("change_type_to_").unwrap_or("");
                let new_type = CredentialType::from_label(type_label);
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.credential.credential_type = new_type.clone();
                    s.credential_edit.type_display = new_type.label().to_string();
                });
            }

            // ---- 搜索 ----
            "search_credentials" => {
                let query = self
                    .state
                    .lock()
                    .ok()
                    .map(|s| s.credential_list.search_query.clone())
                    .unwrap_or_default();
                self.search_credentials(&query);
            }

            // ---- 查看/编辑/删除 ----
            "view_credential" => {
                if let Some(_id) = params.get("id").and_then(|v| v.as_str()) {
                    let _ = self.state.lock().map(|mut s| {
                        s.selected_tool = ToolId::CredentialDetail;
                    });
                }
            }
            "edit_credential" => {
                if let Some(_id) = params.get("id").and_then(|v| v.as_str()) {
                    let _ = self.state.lock().map(|mut s| {
                        s.selected_tool = ToolId::CredentialEdit;
                        s.credential_edit.is_new = false;
                        s.credential_edit.type_display = s.credential_edit.credential.credential_type.label().to_string();
                    });
                }
            }
            "delete_credential" => {
                if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
                    if let Some(service) = &self.credential_service {
                        let _ = service.delete(id);
                    }
                    self.load_credential_list();
                }
            }
            "create_credential" => {
                let _ = self.state.lock().map(|mut s| {
                    s.selected_tool = ToolId::CredentialCreate;
                    s.credential_edit.is_new = true;
                    s.credential_edit.credential = Default::default();
                    s.credential_edit.type_display = CredentialType::default().label().to_string();
                });
            }

            // ---- 保存/取消 ----
            "save_credential" => {
                // TODO: 将 CredentialItem 转为 Credential 并调用 service.create / update
                log::info!("save_credential called");
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialList);
                self.load_credential_list();
            }
            "cancel_edit" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialList);
            }

            // ---- 启用状态切换 ----
            "toggle_active_status" => {
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.credential.is_active = !s.credential_edit.credential.is_active;
                    s.credential_edit.is_active_display =
                        if s.credential_edit.credential.is_active { "已启用" } else { "已禁用" }.to_string();
                });
            }

            _ => {
                log::warn!("Unknown action: {}", action);
            }
        }

        self.state()
    }

    fn ui_schema(&self) -> UiSchema {
        let (current_tool, cred_type, is_new) = self.state.lock().map(|s| {
            let ct = s.credential_edit.credential.credential_type.clone();
            let is_new = s.credential_edit.is_new;
            (s.selected_tool.clone(), ct, is_new)
        }).unwrap_or((ToolId::CredentialList, CredentialType::default(), true));

        let sidebar = build_sidebar(current_tool.clone());

        let content = match &current_tool {
            ToolId::CredentialList => schema_credential_list(),
            ToolId::CredentialCreate => schema_credential_edit(true, &cred_type),
            ToolId::CredentialEdit => schema_credential_edit(is_new, &cred_type),
            ToolId::CredentialDetail => schema_credential_detail(),
            ToolId::ImportExport => schema_import_export(),
            ToolId::AuditLogs => schema_audit_logs(),
            ToolId::Settings => schema_settings(),
        };

        let mut right_container = UiNode::new("flex-col")
            .children(content.children);
        if content.gap > 0 {
            right_container = right_container.prop("gap", serde_json::json!(content.gap));
        }
        if let Some(ref align) = content.align_items {
            right_container = right_container.prop("align_items", serde_json::json!(align));
        }

        UiSchema {
            layout: "flex-row".into(),
            children: vec![
                UiNode::split("row")
                    .prop("left_width", serde_json::json!(220))
                    .prop("gap", serde_json::json!(1))
                    .child(sidebar)
                    .child(right_container),
            ],
            ..Default::default()
        }
    }

    fn on_load(&self) {
        log::info!("Credential plugin loaded");
        self.load_credential_list();
    }

    fn on_unload(&self) {
        log::info!("Credential plugin unloaded");
    }
}
