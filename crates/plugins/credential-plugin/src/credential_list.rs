//! 凭证列表页面 - UI和业务逻辑整合

use crate::service::CredentialService;
use crate::state::{CredentialItem, CredentialPluginState};
use plugin_sdk::{UiNode, UiSchema};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// 凭证列表UI界面
// ---------------------------------------------------------------------------

/// 构建凭证列表页面的UI schema
pub fn schema_credential_list() -> UiSchema {
    let search_bar = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("align_items", serde_json::json!("center"))
        .child(UiNode::input("credential_list.search_query", "搜索凭证...").prop("width", serde_json::json!("100%")))
        .child(UiNode::button("搜索", "search_credentials").prop("variant", serde_json::json!("primary")));

    // 平台筛选按钮组 - 确保每个按钮都有明确的action
    let platform_buttons = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(4))
        .prop("margin_top", serde_json::json!(8))
        .child(UiNode::button("全部", "filter_platform_all").prop("variant", serde_json::json!("primary")))
        .child(UiNode::button("GitHub", "filter_platform_github"))
        .child(UiNode::button("GitLab", "filter_platform_gitlab"))
        .child(UiNode::button("Docker", "filter_platform_docker"))
        .child(UiNode::button("AWS", "filter_platform_aws"));

    // 分类筛选按钮组
    let category_buttons = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(4))
        .prop("margin_top", serde_json::json!(4))
        .child(UiNode::button("全部", "filter_category_all").prop("variant", serde_json::json!("primary")))
        .child(UiNode::button("开发", "filter_category_development"))
        .child(UiNode::button("生产", "filter_category_production"))
        .child(UiNode::button("测试", "filter_category_testing"))
        .child(UiNode::button("个人", "filter_category_personal"));

    let filter_row = UiNode::new("flex-col")
        .prop("gap", serde_json::json!(4))
        .prop("margin_top", serde_json::json!(12))
        .child(UiNode::label("平台筛选"))
        .child(platform_buttons)
        .child(UiNode::label("分类筛选"))
        .child(category_buttons)
        .child(
            UiNode::new("flex-row")
                .prop("margin_top", serde_json::json!(8))
                .child(UiNode::button("新建凭证", "create_credential")
                    .prop("variant", serde_json::json!("primary")))
        );

    let table = UiNode::table_mapped("credential_list.credentials", vec![
        ("名称", "name"),
        ("平台", "platform"),
        ("用户名", "username"),
        ("分类", "category"),
        ("状态", "is_active"),
        ("更新时间", "updated_at"),
        ("操作", ""),
    ])
        .prop("row_actions", serde_json::json!([
            {"label": "查看", "action": "view_credential"},
            {"label": "编辑", "action": "edit_credential"},
            {"label": "删除", "action": "delete_credential"}
        ]));

    UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![search_bar, filter_row, table],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 凭证列表业务处理器
// ---------------------------------------------------------------------------

/// 凭证列表业务处理器
pub struct CredentialListHandler {
    pub state: Arc<Mutex<CredentialPluginState>>,
    pub credential_service: Option<CredentialService>,
}

impl CredentialListHandler {
    pub fn new(
        state: Arc<Mutex<CredentialPluginState>>,
        credential_service: Option<CredentialService>,
    ) -> Self {
        Self {
            state,
            credential_service,
        }
    }

    /// 加载凭证列表
    pub fn load_credential_list(&self) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                log::error!("凭证服务未初始化");
                return;
            }
        };

        let credentials = match service.list_all() {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("加载凭证列表失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.credential_list.loading = false;
                    s.credential_list.total_count = 0;
                    s.credential_list.credentials = Vec::new();
                });
                return;
            }
        };

        let ui_credentials: Vec<CredentialItem> = credentials
            .into_iter()
            .map(|cred| CredentialItem {
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
                credential_type: crate::state::CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        let _ = self.state.lock().map(|mut s| {
            s.credential_list.credentials = ui_credentials;
            s.credential_list.total_count = count;
            s.credential_list.loading = false;
            // 重置筛选状态
            s.credential_list.selected_platform = "all".to_string();
            s.credential_list.selected_category = "all".to_string();
            s.credential_list.search_query = String::new();
        });

        log::info!("成功加载 {} 个凭证", count);
    }

    /// 搜索凭证
    pub fn search_credentials(&self) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                log::error!("凭证服务未初始化");
                return;
            }
        };

        let query = self.state.lock()
            .ok()
            .map(|s| s.credential_list.search_query.clone())
            .unwrap_or_default();

        if query.trim().is_empty() {
            // 如果搜索框为空，显示所有凭证
            self.load_credential_list();
            return;
        }

        let credentials = match service.search(&query) {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("搜索凭证失败: {}", e);
                return;
            }
        };

        let ui_credentials: Vec<CredentialItem> = credentials
            .into_iter()
            .map(|cred| CredentialItem {
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
                credential_type: crate::state::CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        let _ = self.state.lock().map(|mut s| {
            s.credential_list.credentials = ui_credentials;
            s.credential_list.total_count = count;
        });

        log::info!("搜索找到 {} 个匹配的凭证", count);
    }

    /// 按平台筛选
    pub fn filter_by_platform(&self, platform: &str) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                log::error!("凭证服务未初始化");
                return;
            }
        };

        let credentials = if platform == "all" {
            // 显示所有凭证
            match service.list_all() {
                Ok(creds) => creds,
                Err(e) => {
                    log::error!("加载凭证列表失败: {}", e);
                    return;
                }
            }
        } else {
            // 按平台筛选
            match service.filter_by_platform(platform) {
                Ok(creds) => creds,
                Err(e) => {
                    log::error!("按平台筛选失败: {}", e);
                    return;
                }
            }
        };

        let ui_credentials: Vec<CredentialItem> = credentials
            .into_iter()
            .map(|cred| CredentialItem {
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
                credential_type: crate::state::CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        let _ = self.state.lock().map(|mut s| {
            s.credential_list.credentials = ui_credentials;
            s.credential_list.total_count = count;
            s.credential_list.selected_platform = platform.to_string();
            // 重置其他筛选
            s.credential_list.selected_category = "all".to_string();
        });

        log::info!("按平台 {} 筛选，找到 {} 个凭证", platform, count);
    }

    /// 按分类筛选
    pub fn filter_by_category(&self, category: &str) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                log::error!("凭证服务未初始化");
                return;
            }
        };

        let credentials = if category == "all" {
            // 显示所有凭证
            match service.list_all() {
                Ok(creds) => creds,
                Err(e) => {
                    log::error!("加载凭证列表失败: {}", e);
                    return;
                }
            }
        } else {
            // 按分类筛选
            match service.filter_by_category(category) {
                Ok(creds) => creds,
                Err(e) => {
                    log::error!("按分类筛选失败: {}", e);
                    return;
                }
            }
        };

        let ui_credentials: Vec<CredentialItem> = credentials
            .into_iter()
            .map(|cred| CredentialItem {
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
                credential_type: crate::state::CredentialType::from_value(&cred.credential_type),
                ..Default::default()
            })
            .collect();

        let count = ui_credentials.len();
        let _ = self.state.lock().map(|mut s| {
            s.credential_list.credentials = ui_credentials;
            s.credential_list.total_count = count;
            s.credential_list.selected_category = category.to_string();
            // 重置其他筛选
            s.credential_list.selected_platform = "all".to_string();
        });

        log::info!("按分类 {} 筛选，找到 {} 个凭证", category, count);
    }

    /// 查看凭证详情
    pub fn view_credential(&self, credential_id: &str) {
        log::info!("查看凭证详情: {}", credential_id);
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = crate::state::ToolId::CredentialDetail;
        });
    }

    /// 编辑凭证
    pub fn edit_credential(&self, credential_id: &str) {
        log::info!("编辑凭证: {}", credential_id);
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = crate::state::ToolId::CredentialEdit;
            // 标记为编辑模式
            s.credential_edit.is_new = false;
        });
    }

    /// 删除凭证
    pub fn delete_credential(&self, credential_id: &str) -> serde_json::Value {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                return serde_json::json!({"success": false, "message": "凭证服务未初始化"});
            }
        };

        match service.delete(credential_id) {
            Ok(_) => {
                log::info!("成功删除凭证: {}", credential_id);
                // 刷新列表
                self.load_credential_list();
                serde_json::json!({"success": true, "message": "凭证删除成功"})
            }
            Err(e) => {
                log::error!("删除凭证失败: {}", e);
                serde_json::json!({"success": false, "message": format!("删除失败: {}", e)})
            }
        }
    }

    /// 创建新凭证
    pub fn create_credential(&self) {
        log::info!("创建新凭证");
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = crate::state::ToolId::CredentialCreate;
            // 初始化新建凭证表单
            s.credential_edit.is_new = true;
            s.credential_edit.credential = CredentialItem::default();
            s.credential_edit.type_display = crate::state::CredentialType::default().label().to_string();
        });
    }
}
