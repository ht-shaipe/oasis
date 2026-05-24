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
    pub is_active_display: String,
    pub type_display: String,
    pub validation_errors: Vec<String>,
    pub saving: bool,
}

/// 导入导出状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportExportState {
    pub import_path: String,
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

/// 凭证类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    ApiKey,
    WebsiteUser,
    SshKey,
    Database,
    Certificate,
    Token,
}

impl Default for CredentialType {
    fn default() -> Self {
        Self::ApiKey
    }
}

impl CredentialType {
    pub fn label(&self) -> &str {
        match self {
            Self::ApiKey => "接口密钥",
            Self::WebsiteUser => "网站用户",
            Self::SshKey => "SSH 密钥",
            Self::Database => "数据库",
            Self::Certificate => "证书",
            Self::Token => "令牌",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::ApiKey => "api_key",
            Self::WebsiteUser => "website_user",
            Self::SshKey => "ssh_key",
            Self::Database => "database",
            Self::Certificate => "certificate",
            Self::Token => "token",
        }
    }

    pub fn from_value(v: &str) -> Self {
        match v {
            "website_user" => Self::WebsiteUser,
            "ssh_key" => Self::SshKey,
            "database" => Self::Database,
            "certificate" => Self::Certificate,
            "token" => Self::Token,
            _ => Self::ApiKey,
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "接口密钥" => Self::ApiKey,
            "网站用户" => Self::WebsiteUser,
            "SSH 密钥" => Self::SshKey,
            "数据库" => Self::Database,
            "证书" => Self::Certificate,
            "令牌" => Self::Token,
            _ => Self::ApiKey,
        }
    }

    pub fn all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("接口密钥", "api_key"),
            ("网站用户", "website_user"),
            ("SSH 密钥", "ssh_key"),
            ("数据库", "database"),
            ("证书", "certificate"),
            ("令牌", "token"),
        ]
    }
}

/// 凭证项（用于 UI 展示）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialItem {
    pub id: String,
    pub name: String,
    pub credential_type: CredentialType,
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
    // 类型特有字段
    pub api_key_value: String,
    pub api_secret: String,
    pub api_endpoint: String,
    pub ssh_private_key: String,
    pub ssh_public_key: String,
    pub db_host: String,
    pub db_port: String,
    pub db_name: String,
    pub cert_path: String,
    pub token_value: String,
    pub token_expiry: Option<i64>,
}

impl Default for CredentialItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            credential_type: CredentialType::default(),
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
            api_key_value: String::new(),
            api_secret: String::new(),
            api_endpoint: String::new(),
            ssh_private_key: String::new(),
            ssh_public_key: String::new(),
            db_host: String::new(),
            db_port: String::new(),
            db_name: String::new(),
            cert_path: String::new(),
            token_value: String::new(),
            token_expiry: None,
        }
    }
}

impl Default for CredentialEditState {
    fn default() -> Self {
        Self {
            credential: CredentialItem::default(),
            is_new: true,
            is_active_display: "已启用".to_string(),
            type_display: CredentialType::default().label().to_string(),
            validation_errors: Vec::new(),
            saving: false,
        }
    }
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
                credential: CredentialItem::default(),
                is_new: true,
                is_active_display: "启用".to_string(),
                type_display: CredentialType::default().label().to_string(),
                validation_errors: vec![],
                saving: false,
            },
            import_export: ImportExportState {
                import_path: String::new(),
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