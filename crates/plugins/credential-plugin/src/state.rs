//! 凭证管理插件 - 状态类型定义

use serde::{Deserialize, Serialize};

/// 工具 ID
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    CredentialList,
    CredentialDetail,
    CredentialEdit,
    CredentialCreate,
    ImportExport,
    AuditLogs,
    Settings,
}

/// 凭证列表状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialListState {
    pub credentials: Vec<CredentialItem>,
    pub search_query: String,
    pub selected_platform: String,
    pub selected_category: String,
    pub selected_tags: Vec<String>,
    pub loading: bool,
    pub total_count: usize,
}

/// 凭证详情状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialDetailState {
    pub credential: Option<CredentialItem>,
    pub audit_logs: Vec<AuditLogItem>,
    pub show_password: bool,
    pub loading: bool,
}

/// 凭证编辑状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialEditState {
    pub credential: CredentialItem,
    pub is_new: bool,
    pub validation_errors: Vec<String>,
    pub saving: bool,
}

/// 导入导出状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportExportState {
    pub import_file: Option<String>,
    pub export_format: String,
    pub import_result: String,
    pub export_result: String,
    pub processing: bool,
}

/// 审计日志状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogsState {
    pub logs: Vec<AuditLogItem>,
    pub filter_action: String,
    pub filter_date_range: Option<(i64, i64)>,
    pub loading: bool,
    pub total_count: usize,
}

/// 设置状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsState {
    pub master_key_configured: bool,
    pub change_password: String,
    pub confirm_password: String,
    pub password_error: Option<String>,
    pub saving: bool,
}

/// 凭证项（用于 UI 展示）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialItem {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub category: String,
    pub username: String,
    pub password_masked: String,
    pub notes: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: Option<i64>,
    pub tags: String,
    pub extra_fields: String,
}

/// 审计日志项
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogItem {
    pub id: String,
    pub credential_id: String,
    pub credential_name: String,
    pub action: String,
    pub timestamp: i64,
    pub ip_address: String,
    pub result: bool,
    pub details: String,
}

/// 插件完整状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialPluginState {
    pub selected_tool: ToolId,
    pub credential_list: CredentialListState,
    pub credential_detail: CredentialDetailState,
    pub credential_edit: CredentialEditState,
    pub import_export: ImportExportState,
    pub audit_logs: AuditLogsState,
    pub settings: SettingsState,
}

impl Default for CredentialPluginState {
    fn default() -> Self {
        Self {
            selected_tool: ToolId::CredentialList,
            credential_list: CredentialListState {
                credentials: vec![],
                search_query: String::new(),
                selected_platform: String::new(),
                selected_category: String::new(),
                selected_tags: vec![],
                loading: false,
                total_count: 0,
            },
            credential_detail: CredentialDetailState {
                credential: None,
                audit_logs: vec![],
                show_password: false,
                loading: false,
            },
            credential_edit: CredentialEditState {
                credential: CredentialItem {
                    id: String::new(),
                    name: String::new(),
                    platform: String::new(),
                    category: String::new(),
                    username: String::new(),
                    password_masked: String::new(),
                    notes: String::new(),
                    is_active: true,
                    created_at: 0,
                    updated_at: 0,
                    expires_at: None,
                    tags: String::new(),
                    extra_fields: String::new(),
                },
                is_new: true,
                validation_errors: vec![],
                saving: false,
            },
            import_export: ImportExportState {
                import_file: None,
                export_format: "json".to_string(),
                import_result: String::new(),
                export_result: String::new(),
                processing: false,
            },
            audit_logs: AuditLogsState {
                logs: vec![],
                filter_action: String::new(),
                filter_date_range: None,
                loading: false,
                total_count: 0,
            },
            settings: SettingsState {
                master_key_configured: false,
                change_password: String::new(),
                confirm_password: String::new(),
                password_error: None,
                saving: false,
            },
        }
    }
}