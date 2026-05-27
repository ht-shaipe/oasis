//! 侧边栏导航组件

use crate::state::ToolId;
use plugin_sdk::UiNode;

/// 构建侧边栏导航菜单
pub fn build_sidebar(current_tool: ToolId) -> UiNode {
    let menu_items = vec![
        ("凭证列表", ToolId::CredentialList, "select_credential_list"),
        ("新建凭证", ToolId::CredentialCreate, "select_credential_create"),
        ("导入导出", ToolId::ImportExport, "select_import_export"),
        ("审计日志", ToolId::AuditLogs, "select_audit_logs"),
        ("设置", ToolId::Settings, "select_settings"),
    ];

    let items: Vec<UiNode> = menu_items
        .into_iter()
        .map(|(label, tool_id, action)| {
            let is_active = tool_id == current_tool;
            UiNode::new("nav-item")
                .prop("label", serde_json::json!(label))
                .prop("active", serde_json::json!(is_active))
                .on_action(action)
        })
        .collect();

    UiNode::new("flex-col")
        .prop("gap", serde_json::json!(4))
        .prop("padding", serde_json::json!(12))
        .children(items)
}
