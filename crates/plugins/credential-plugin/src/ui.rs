//! 凭证管理插件 - UI 界面定义（声明式 UiSchema）

use crate::state::{CredentialPluginState, ToolId};
use plugin_sdk::UiNode;

// ---------------------------------------------------------------------------
// 通用：侧边栏菜单
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 凭证列表页
// ---------------------------------------------------------------------------

pub fn schema_credential_list() -> plugin_sdk::UiSchema {
    let search_bar = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("align_items", serde_json::json!("center"))
        .child(UiNode::input("search_query", "搜索凭证...").prop("width", serde_json::json!("100%")))
        .child(UiNode::button("搜索", "search_credentials").prop("variant", serde_json::json!("primary")));

    let platform_options = serde_json::json!([
        {"label": "全部", "value": ""},
        {"label": "GitHub", "value": "github"},
        {"label": "GitLab", "value": "gitlab"},
        {"label": "Docker Hub", "value": "docker"},
        {"label": "AWS", "value": "aws"},
        {"label": "GCP", "value": "gcp"},
        {"label": "Azure", "value": "azure"}
    ]);

    let category_options = serde_json::json!([
        {"label": "全部", "value": ""},
        {"label": "开发", "value": "development"},
        {"label": "生产", "value": "production"},
        {"label": "测试", "value": "testing"},
        {"label": "个人", "value": "personal"}
    ]);

    let filter_row = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("margin_top", serde_json::json!(12))
        .child(
            UiNode::new("select")
                .prop("label", serde_json::json!("平台"))
                .bind("selected_platform")
                .on_action("filter_by_platform")
                .prop("options", platform_options)
        )
        .child(
            UiNode::new("select")
                .prop("label", serde_json::json!("分类"))
                .bind("selected_category")
                .on_action("filter_by_category")
                .prop("options", category_options)
        )
        .child(
            UiNode::button("新建凭证", "create_credential")
                .prop("variant", serde_json::json!("primary"))
        );

    let table = UiNode::table("credentials", vec!["名称", "平台", "用户名", "分类", "状态", "更新时间", "操作"])
        .prop("row_actions", serde_json::json!([
            {"label": "查看", "action": "view_credential"},
            {"label": "编辑", "action": "edit_credential"},
            {"label": "删除", "action": "delete_credential"}
        ]));

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![search_bar, filter_row, table],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 凭证详情页
// ---------------------------------------------------------------------------

pub fn schema_credential_detail() -> plugin_sdk::UiSchema {
    let info_card = UiNode::new("card")
        .prop("title", serde_json::json!("凭证详情"))
        .child(UiNode::info(&[
            ("名称", "credential.name"),
            ("平台", "credential.platform"),
            ("分类", "credential.category"),
            ("用户名", "credential.username"),
            ("密码", "credential.password_masked"),
            ("标签", "credential.tags"),
            ("备注", "credential.notes"),
            ("状态", "credential.is_active"),
            ("创建时间", "credential.created_at"),
            ("更新时间", "credential.updated_at"),
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
        .child(UiNode::table("audit_logs", vec!["时间", "操作", "IP地址", "结果"]));

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![info_card, actions, audit_logs],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 凭证编辑/创建页
// ---------------------------------------------------------------------------

pub fn schema_credential_edit(is_new: bool) -> plugin_sdk::UiSchema {
    let title = if is_new { "新建凭证" } else { "编辑凭证" };

    let form = UiNode::new("form")
        .prop("title", serde_json::json!(title))
        .prop("gap", serde_json::json!(12))
        .child(
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .child(UiNode::input("credential.name", "名称").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                .child(UiNode::input("credential.platform", "平台").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
        )
        .child(
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .child(UiNode::input("credential.category", "分类").prop("width", serde_json::json!("50%")))
                .child(UiNode::input("credential.username", "用户名").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
        )
        .child(UiNode::input("credential.password_masked", "密码").prop("required", serde_json::json!(true)).prop("type", serde_json::json!("password")))
        .child(UiNode::input("credential.tags", "标签（逗号分隔）"))
        .child(
            UiNode::new("textarea")
                .prop("label", serde_json::json!("备注"))
                .bind("credential.notes")
                .prop("rows", serde_json::json!(4))
        )
        .child(
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .child(UiNode::new("switch").prop("label", serde_json::json!("启用")).bind("credential.is_active"))
        );

    let action_buttons = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("justify_content", serde_json::json!("flex-end"))
        .prop("margin_top", serde_json::json!(16))
        .child(UiNode::button("取消", "cancel_edit"))
        .child(UiNode::button("保存", "save_credential").prop("variant", serde_json::json!("primary")));

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![form, action_buttons],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 导入导出页
// ---------------------------------------------------------------------------

pub fn schema_import_export() -> plugin_sdk::UiSchema {
    let format_options = serde_json::json!([
        {"label": "JSON", "value": "json"},
        {"label": "CSV", "value": "csv"}
    ]);

    let import_section = UiNode::new("card")
        .prop("title", serde_json::json!("导入凭证"))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(
                    UiNode::new("file-picker")
                        .prop("label", serde_json::json!("选择文件"))
                        .bind("import_file")
                        .on_action("select_import_file")
                        .prop("accept", serde_json::json!(".json,.csv"))
                )
                .child(
                    UiNode::new("select")
                        .prop("label", serde_json::json!("格式"))
                        .bind("export_format")
                        .prop("options", format_options.clone())
                )
                .child(UiNode::button("导入", "import_credentials").prop("variant", serde_json::json!("primary")))
                .child(UiNode::display("import_result").prop("type", serde_json::json!("info")))
        );

    let export_section = UiNode::new("card")
        .prop("title", serde_json::json!("导出凭证"))
        .prop("margin_top", serde_json::json!(16))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(
                    UiNode::new("select")
                        .prop("label", serde_json::json!("格式"))
                        .bind("export_format")
                        .prop("options", format_options)
                )
                .child(UiNode::button("导出全部", "export_all_credentials").prop("variant", serde_json::json!("primary")))
                .child(UiNode::button("导出选中", "export_selected_credentials"))
                .child(UiNode::display("export_result").prop("type", serde_json::json!("info")))
        );

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![import_section, export_section],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 审计日志页
// ---------------------------------------------------------------------------

pub fn schema_audit_logs() -> plugin_sdk::UiSchema {
    let action_options = serde_json::json!([
        {"label": "全部", "value": ""},
        {"label": "创建", "value": "CREATE"},
        {"label": "读取", "value": "READ"},
        {"label": "更新", "value": "UPDATE"},
        {"label": "删除", "value": "DELETE"}
    ]);

    let filter_row = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("align_items", serde_json::json!("center"))
        .child(
            UiNode::new("select")
                .prop("label", serde_json::json!("操作类型"))
                .bind("filter_action")
                .on_action("filter_logs_by_action")
                .prop("options", action_options)
        )
        .child(UiNode::button("清除过期日志", "clear_old_logs"));

    let table = UiNode::table("logs", vec!["时间", "凭证", "操作", "IP地址", "结果"]);

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![filter_row, table],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 设置页
// ---------------------------------------------------------------------------

pub fn schema_settings() -> plugin_sdk::UiSchema {
    let form = UiNode::new("card")
        .prop("title", serde_json::json!("安全设置"))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(UiNode::input("change_password", "新密码").prop("type", serde_json::json!("password")))
                .child(UiNode::input("confirm_password", "确认密码").prop("type", serde_json::json!("password")))
                .child(UiNode::button("修改密码", "change_master_password").prop("variant", serde_json::json!("primary")))
                .child(UiNode::display("password_error").prop("type", serde_json::json!("error")))
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

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![form, info_card],
        ..Default::default()
    }
}