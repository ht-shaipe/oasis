//! 凭证管理插件 - 重构版实现

use crate::init::CredentialManagerInit;
use crate::service::CredentialService;
use crate::state::{CredentialPluginState, CredentialType, ToolId};
use crate::ui::{
    build_sidebar, schema_audit_logs, schema_credential_detail,
    schema_import_export, schema_settings,
    CredentialDetailHandler,
    ImportExportHandler, AuditLogsHandler, SettingsHandler,
};
use crate::credential_edit::{schema_credential_edit, CredentialEditHandler};
use crate::credential_list::{schema_credential_list, CredentialListHandler};
use plugin_sdk::{Plugin, PluginMeta, UiNode, UiSchema};
use std::sync::{Arc, Mutex};

pub struct CredentialPlugin {
    state: Arc<Mutex<CredentialPluginState>>,
    credential_service: Option<CredentialService>,
    // 功能处理器
    list_handler: Option<CredentialListHandler>,
    edit_handler: Option<CredentialEditHandler>,
    detail_handler: Option<CredentialDetailHandler>,
    import_export_handler: Option<ImportExportHandler>,
    audit_handler: Option<AuditLogsHandler>,
    settings_handler: Option<SettingsHandler>,
}

impl CredentialPlugin {
    pub fn new() -> Self {
        let service = CredentialManagerInit::initialize().ok();

        let state = Arc::new(Mutex::new(CredentialPluginState::default()));

        // 创建功能处理器
        let list_handler = service.as_ref().map(|_| CredentialListHandler::new(state.clone(), service.clone()));
        let edit_handler = service.as_ref().map(|_| CredentialEditHandler::new(state.clone(), service.clone()));
        let detail_handler = service.as_ref().map(|_| CredentialDetailHandler::new(state.clone(), service.clone()));
        let import_export_handler = service.as_ref().map(|_| ImportExportHandler::new(state.clone(), service.clone()));
        let audit_handler = service.as_ref().map(|_| AuditLogsHandler::new(state.clone(), service.clone()));
        let settings_handler = service.as_ref().map(|_| SettingsHandler::new(state.clone(), service.clone()));

        let plugin = Self {
            state,
            credential_service: service,
            list_handler,
            edit_handler,
            detail_handler,
            import_export_handler,
            audit_handler,
            settings_handler,
        };

        // 加载初始数据
        if let Some(handler) = &plugin.list_handler {
            handler.load_credential_list();
        }

        plugin
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
                if let Some(handler) = &self.list_handler {
                    handler.load_credential_list();
                }
                return self.state();
            }
            "select_credential_create" => {
                if let Some(handler) = &self.list_handler {
                    handler.create_credential();
                }
                return self.state();
            }
            "select_credential_edit" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialEdit);
                return self.state();
            }
            "select_import_export" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::ImportExport);
                return self.state();
            }
            "select_audit_logs" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::AuditLogs);
                return self.state();
            }
            "select_settings" => {
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::Settings);
                return self.state();
            }

            // ---- 凭证类型切换 ----
            a if a.starts_with("change_type_to_") => {
                let type_label = a.strip_prefix("change_type_to_").unwrap_or("");
                let new_type = CredentialType::from_label(type_label);
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.credential.credential_type = new_type.clone();
                    s.credential_edit.type_display = new_type.label().to_string();
                });
                return self.state();
            }

            // ---- 搜索 ----
            "search_credentials" => {
                if let Some(handler) = &self.list_handler {
                    handler.search_credentials();
                }
                return self.state();
            }

            // ---- 查看/编辑/删除 ----
            "view_credential" => {
                if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
                    if let Some(handler) = &self.list_handler {
                        handler.view_credential(id);
                    }
                }
                return self.state();
            }
            "edit_credential" => {
                if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
                    if let Some(handler) = &self.list_handler {
                        handler.edit_credential(id);
                    }
                }
                return self.state();
            }
            "delete_credential" => {
                if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
                    if let Some(handler) = &self.list_handler {
                        return handler.delete_credential(id);
                    }
                }
                return self.state();
            }
            "create_credential" => {
                if let Some(handler) = &self.list_handler {
                    handler.create_credential();
                }
                return self.state();
            }

            // ---- 保存/取消 ----
            "save_credential" => {
                if let Some(handler) = &self.edit_handler {
                    return handler.save_credential();
                }
                return serde_json::json!({"success": false, "message": "编辑处理器未初始化"});
            }
            "cancel_edit" => {
                if let Some(handler) = &self.edit_handler {
                    handler.cancel_edit();
                }
                return self.state();
            }

            // ---- 启用状态切换 ----
            "toggle_active_status" => {
                if let Some(handler) = &self.edit_handler {
                    handler.toggle_active_status();
                }
                return self.state();
            }

            // ---- 平台筛选 ----
            a if a.starts_with("filter_platform_") => {
                let platform = a.strip_prefix("filter_platform_").unwrap_or("");
                if let Some(handler) = &self.list_handler {
                    handler.filter_by_platform(platform);
                }
                return self.state();
            }

            // ---- 分类筛选 ----
            a if a.starts_with("filter_category_") => {
                let category = a.strip_prefix("filter_category_").unwrap_or("");
                if let Some(handler) = &self.list_handler {
                    handler.filter_by_category(category);
                }
                return self.state();
            }

            // ---- 审计日志筛选 ----
            a if a.starts_with("filter_logs_") => {
                let filter = a.strip_prefix("filter_logs_").unwrap_or("");
                let _ = self.state.lock().map(|mut s| {
                    s.audit_logs.filter_action = filter.to_string();
                });
                return self.state();
            }

            // ---- 详情页操作 ----
            "edit_current_credential" => {
                let _ = self.state.lock().map(|mut s| {
                    s.selected_tool = ToolId::CredentialEdit;
                    s.credential_edit.is_new = false;
                    s.credential_edit.type_display = s.credential_edit.credential.credential_type.label().to_string();
                });
                return self.state();
            }
            "delete_current_credential" => {
                if let Some(id) = self.state.lock().ok()
                    .and_then(|s| s.credential_detail.credential.as_ref().map(|c| c.id.clone())) {
                    if !id.is_empty() {
                        if let Some(handler) = &self.list_handler {
                            let result = handler.delete_credential(&id);
                            let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialList);
                            return result;
                        }
                    }
                }
                let _ = self.state.lock().map(|mut s| s.selected_tool = ToolId::CredentialList);
                return self.state();
            }
            "toggle_password_visibility" => {
                let _ = self.state.lock().map(|mut s| {
                    s.credential_detail.show_password = !s.credential_detail.show_password;
                });
                return self.state();
            }

            // ---- 导入导出 ----
            "import_json" | "import_csv" | "export_all_json" | "export_all_csv"
            | "export_selected_json" | "export_selected_csv" => {
                log::info!("{} called (TODO: implement)", action);
                return self.state();
            }

            // ---- 审计日志 ----
            "clear_old_logs" => {
                log::info!("clear_old_logs called (TODO: implement)");
                return self.state();
            }

            // ---- 设置 ----
            "change_master_password" => {
                log::info!("change_master_password called (TODO: implement)");
                return self.state();
            }

            _ => {
                log::warn!("Unknown action: {}", action);
                return self.state();
            }
        }
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
        if let Some(handler) = &self.list_handler {
            handler.load_credential_list();
        }
    }

    fn on_unload(&self) {
        log::info!("Credential plugin unloaded");
    }
}
