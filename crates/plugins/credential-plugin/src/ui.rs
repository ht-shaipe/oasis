//! 凭证管理插件 - UI 界面定义（声明式 UiSchema）

use crate::state::{CredentialType, ToolId};
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

    let platform_buttons = UiNode::new("button_group")
        .prop("gap", serde_json::json!(4))
        .prop("margin_top", serde_json::json!(8))
        .child(UiNode::button("全部", "filter_platform_all").prop("variant", serde_json::json!("primary")))
        .child(UiNode::button("GitHub", "filter_platform_github"))
        .child(UiNode::button("GitLab", "filter_platform_gitlab"))
        .child(UiNode::button("Docker", "filter_platform_docker"))
        .child(UiNode::button("AWS", "filter_platform_aws"));

    let category_buttons = UiNode::new("button_group")
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

    let table = UiNode::table_mapped("credentials", vec![
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

pub fn schema_credential_edit(is_new: bool, cred_type: &CredentialType) -> plugin_sdk::UiSchema {
    let title = if is_new { "新建凭证" } else { "编辑凭证" };

    // 通用字段：名称
    let name_field = UiNode::input("credential_edit.credential.name", "名称")
        .prop("required", serde_json::json!(true))
        .prop("width", serde_json::json!("100%"));

    // 类型选择按钮组
    let type_buttons: Vec<UiNode> = CredentialType::all()
        .iter()
        .map(|(label, _value)| {
            let is_active = cred_type.label() == *label;
            let variant = if is_active {
                serde_json::json!("primary")
            } else {
                serde_json::json!("outline")
            };
            UiNode::button(*label, &format!("change_type_to_{}", *label))
                .prop("variant", variant)
        })
        .collect();

    let type_button_group = UiNode::new("button_group")
        .prop("gap", serde_json::json!(6))
        .prop("margin_top", serde_json::json!(8))
        .children(type_buttons);

    // 说明文字（显示当前类型）
    let type_hint = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("align_items", serde_json::json!("center"))
        .child(UiNode::label("当前类型："))
        .child(UiNode::display("credential_edit.type_display"));

    let common_fields = UiNode::new("card")
        .prop("title", serde_json::json!(title))
        .prop("gap", serde_json::json!(12))
        .child(name_field)
        .child(type_button_group)
        .child(type_hint);

    // 根据类型生成不同的字段
    let type_specific_fields = match cred_type {
        CredentialType::ApiKey => UiNode::new("card")
            .prop("title", serde_json::json!("接口密钥"))
            .prop("gap", serde_json::json!(12))
            .child(UiNode::input("credential_edit.credential.api_key_value", "API Key").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.api_secret", "API Secret").prop("type", serde_json::json!("password")))
            .child(UiNode::input("credential_edit.credential.api_endpoint", "接口地址"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::WebsiteUser => UiNode::new("card")
            .prop("title", serde_json::json!("网站用户"))
            .prop("gap", serde_json::json!(12))
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(8))
                    .child(UiNode::input("credential_edit.credential.platform", "网站/平台").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.category", "分类").prop("width", serde_json::json!("50%")))
            )
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(8))
                    .child(UiNode::input("credential_edit.credential.username", "用户名").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.password_masked", "密码").prop("required", serde_json::json!(true)).prop("type", serde_json::json!("password")).prop("width", serde_json::json!("50%")))
            )
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::SshKey => UiNode::new("card")
            .prop("title", serde_json::json!("SSH 密钥"))
            .prop("gap", serde_json::json!(12))
            .child(UiNode::input("credential_edit.credential.ssh_private_key", "私钥").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.ssh_public_key", "公钥"))
            .child(UiNode::input("credential_edit.credential.username", "用户名"))
            .child(UiNode::input("credential_edit.credential.api_endpoint", "主机地址"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Database => UiNode::new("card")
            .prop("title", serde_json::json!("数据库"))
            .prop("gap", serde_json::json!(12))
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(8))
                    .child(UiNode::input("credential_edit.credential.db_host", "主机").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("60%")))
                    .child(UiNode::input("credential_edit.credential.db_port", "端口").prop("width", serde_json::json!("20%")))
                    .child(UiNode::input("credential_edit.credential.db_name", "数据库名").prop("width", serde_json::json!("20%")))
            )
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(8))
                    .child(UiNode::input("credential_edit.credential.username", "用户名").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.password_masked", "密码").prop("required", serde_json::json!(true)).prop("type", serde_json::json!("password")).prop("width", serde_json::json!("50%")))
            )
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Certificate => UiNode::new("card")
            .prop("title", serde_json::json!("证书"))
            .prop("gap", serde_json::json!(12))
            .child(UiNode::input("credential_edit.credential.cert_path", "证书路径").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.password_masked", "证书密码").prop("type", serde_json::json!("password")))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Token => UiNode::new("card")
            .prop("title", serde_json::json!("令牌"))
            .prop("gap", serde_json::json!(12))
            .child(UiNode::input("credential_edit.credential.token_value", "Token").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.platform", "平台/服务"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),
    };

    // 启用状态
    let status_row = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("align_items", serde_json::json!("center"))
        .prop("margin_top", serde_json::json!(16))
        .child(UiNode::label("启用状态"))
        .child(
            UiNode::new("switch")
                .bind("credential_edit.is_active")
                .on_action("toggle_active_status")
        );

    // 操作按钮
    let action_buttons = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(8))
        .prop("justify_content", serde_json::json!("flex_end"))
        .prop("margin_top", serde_json::json!(12))
        .child(UiNode::button("取消", "cancel_edit"))
        .child(UiNode::button("保存", "save_credential").prop("variant", serde_json::json!("primary")));

    let bottom = UiNode::new("flex-col")
        .children(vec![status_row, action_buttons]);

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![common_fields, type_specific_fields, bottom],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 导入导出页
// ---------------------------------------------------------------------------

pub fn schema_import_export() -> plugin_sdk::UiSchema {
    let import_section = UiNode::new("card")
        .prop("title", serde_json::json!("导入凭证"))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(UiNode::input("import_export.import_path", "文件路径（.json / .csv）"))
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", serde_json::json!(8))
                        .child(UiNode::button("导入 JSON", "import_json").prop("variant", serde_json::json!("primary")))
                        .child(UiNode::button("导入 CSV", "import_csv"))
                )
                .child(UiNode::display("import_export.import_result").prop("type", serde_json::json!("info")))
        );

    let export_section = UiNode::new("card")
        .prop("title", serde_json::json!("导出凭证"))
        .prop("margin_top", serde_json::json!(16))
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(12))
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", serde_json::json!(8))
                        .child(UiNode::button("导出全部 JSON", "export_all_json").prop("variant", serde_json::json!("primary")))
                        .child(UiNode::button("导出全部 CSV", "export_all_csv"))
                )
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", serde_json::json!(8))
                        .child(UiNode::button("导出选中 JSON", "export_selected_json"))
                        .child(UiNode::button("导出选中 CSV", "export_selected_csv"))
                )
                .child(UiNode::display("import_export.export_result").prop("type", serde_json::json!("info")))
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
    let filter_row = UiNode::new("flex-col")
        .prop("gap", serde_json::json!(8))
        .child(UiNode::label("操作类型筛选"))
        .child(
            UiNode::new("button_group")
                .prop("gap", serde_json::json!(4))
                .child(UiNode::button("全部", "filter_logs_all").prop("variant", serde_json::json!("primary")))
                .child(UiNode::button("创建", "filter_logs_create"))
                .child(UiNode::button("读取", "filter_logs_read"))
                .child(UiNode::button("更新", "filter_logs_update"))
                .child(UiNode::button("删除", "filter_logs_delete"))
        )
        .child(
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .child(UiNode::button("清除过期日志", "clear_old_logs"))
        );

    let table = UiNode::table_mapped("logs", vec![
        ("时间", "timestamp"),
        ("凭证", "credential_name"),
        ("操作", "action"),
        ("IP地址", "ip_address"),
        ("结果", "result"),
    ]);

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

    plugin_sdk::UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![form, info_card],
        ..Default::default()
    }
}