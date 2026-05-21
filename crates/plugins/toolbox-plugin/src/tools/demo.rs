//! UiSchema 模板与组件 Demo

use plugin_sdk::{UiNode, UiSchema};
use ui_schema::{
    dashboard, dialog, empty_state, form_page, tree_table, DashboardCard, FormField, FormPageOpts,
};

use crate::state::ToolboxState;
use crate::tools::home::make_button_row;

pub fn schema_ui_schema_demo() -> UiSchema {
    let overview = UiNode::new("card")
        .prop("title", serde_json::json!("总览"))
        .children(vec![
            UiNode::label("这页把 ui-schema 的模板函数和宿主已支持的组件放在一起，方便直接看渲染效果。"),
            UiNode::info(&[("模板状态", "status_message"), ("总用户", "total_users"), ("今日活跃", "active_today")]),
            UiNode::display("total_users").prop("style", serde_json::json!("large_number")),
            UiNode::progress("completion"),
            make_button_row(&[("返回主页", "Home"), ("重置样例", "demo:refresh"), ("切换文案", "demo:toggle")]),
        ]);

    let components = UiNode::new("card")
        .prop("title", serde_json::json!("基础组件"))
        .children(vec![
            UiNode::label("label / display / progress / info / input / button_row / table / tree / card / split"),
            UiNode::new("flex-row").prop("gap", serde_json::json!(12)).children(vec![
                UiNode::new("card")
                    .prop("title", serde_json::json!("指标卡"))
                    .children(vec![UiNode::label("活跃用户"), UiNode::display("active_today").prop("style", serde_json::json!("large_number"))]),
                UiNode::new("card")
                    .prop("title", serde_json::json!("进度条"))
                    .children(vec![UiNode::label("当前完成度"), UiNode::progress("completion")]),
            ]),
            UiNode::info(&[("姓名", "name"), ("邮箱", "email"), ("角色", "role")]),
            UiNode::input("name", "姓名输入框"),
            UiNode::input("email", "邮箱输入框"),
            UiNode::new("table").bind("rows").prop("columns", serde_json::json!(["name", "kind", "owner", "status"])),
            UiNode::split("row")
                .prop("left_width", serde_json::json!(260))
                .prop("gap", serde_json::json!(1))
                .child(UiNode::tree("tree"))
                .child(
                    UiNode::new("card")
                        .prop("title", serde_json::json!("卡片容器"))
                        .children(vec![
                            UiNode::label("卡片里可以继续嵌套任意支持的组件。"),
                            UiNode::new("button_row").children(vec![UiNode::button("主按钮", "demo:toggle"), UiNode::button("返回", "Home")]),
                        ]),
                ),
        ]);

    let template_gallery = UiNode::new("card")
        .prop("title", serde_json::json!("模板库"))
        .children(vec![
            template_block(
                "dashboard",
                "看板模板",
                dashboard(vec![
                    DashboardCard { title: "总用户".into(), value_bind: "total_users".into(), value_style: "large_number".into(), subtitle: Some("示例指标".into()), width: 220, on_click: Some("demo:toggle".into()) },
                    DashboardCard { title: "今日活跃".into(), value_bind: "active_today".into(), value_style: "large_number".into(), subtitle: Some("实时快照".into()), width: 220, on_click: None },
                    DashboardCard { title: "收入".into(), value_bind: "revenue".into(), value_style: "large_number".into(), subtitle: Some("单位：元".into()), width: 220, on_click: None },
                ]),
            ),
            template_block(
                "tree_table",
                "树 + 表格模板",
                tree_table(
                    "tree",
                    "demo:select_path",
                    "rows",
                    &["name", "kind", "owner", "status"],
                ),
            ),
            template_block(
                "form_page",
                "标准表单模板",
                form_page(FormPageOpts {
                    title: Some("项目设置".into()),
                    fields: vec![
                        FormField::input("项目名称", "name").placeholder("请输入项目名称"),
                        FormField::input("负责人邮箱", "email").placeholder("name@example.com"),
                        FormField::input("标签", "role").placeholder("design / product / engineering"),
                    ],
                    actions: vec![("返回主页".into(), "Home".into()), ("重新加载".into(), "demo:refresh".into())],
                    bordered: true,
                }),
            ),
            template_block(
                "empty_state",
                "空状态模板",
                empty_state("", "没有更多模板了", "目前先展示宿主已支持的核心模板，后续可继续补充 settings/search/list/wizard。"),
            ),
            template_block(
                "dialog",
                "确认弹窗模板",
                dialog(
                    "返回主页",
                    vec![UiNode::label("确认后会切回工具箱主页。")],
                    "确认",
                    "Home",
                    "取消",
                    "UiSchemaDemo",
                ),
            ),
        ]);

    UiSchema {
        layout: "flex-col".into(),
        gap: 16,
        children: vec![
            UiNode::label("UiSchema 模板与组件 Demo"),
            UiNode::label("直接在工具箱里查看模板函数与基础组件的组合效果。"),
            overview,
            components,
            template_gallery,
        ],
        ..Default::default()
    }
}

fn template_block(title: &str, summary: &str, schema: UiSchema) -> UiNode {
    UiNode::new("card")
        .prop("title", serde_json::json!(title))
        .children(vec![UiNode::label(summary), UiNode::new("flex-col").prop("gap", serde_json::json!(12)).children(schema.children)])
}

pub fn update_demo_state(state: &mut ToolboxState, refresh: bool) {
    if refresh {
        state.demo = ToolboxState::default().demo;
        return;
    }

    if state.demo.status_message == "模板库已就绪" {
        state.demo.status_message = "正在浏览组件示例".into();
        state.demo.completion = 92;
        state.demo.role = "engineering".into();
    } else {
        state.demo.status_message = "模板库已就绪".into();
        state.demo.completion = 76;
        state.demo.role = "design".into();
    }
}