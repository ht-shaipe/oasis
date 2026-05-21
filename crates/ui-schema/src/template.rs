//! copyright © ecdata.cn 2026 - present
//! UI Schema — 视图模板
//!
//! 提供常见布局的快捷构造方法，返回 `UiSchema`。
//! 插件可直接在 `ui_schema()` 返回值中使用这些模板。

use crate::{UiNode, UiSchema};

// ---------------------------------------------------------------------------
// 树 + 表格
// ---------------------------------------------------------------------------

/// TreeTableOpts — 树+表格布局配置
#[derive(Debug, Clone, Default)]
pub struct TreeTableOpts {
    pub tree_bind: String,
    pub tree_on_select: String,
    pub tree_title: Option<String>,
    pub table_bind: String,
    pub table_columns: Vec<String>,
    pub table_on_action: Option<String>,
    pub left_width: i64,
    pub toolbar: Vec<(String, String)>,
    pub status_bind: Option<String>,
}

/// 树+表格布局：左侧树形导航 + 右侧数据表格
///
/// # state 数据格式
/// ```json
/// {
///   "tree": [{ "name": "src", "is_dir": true, "path": "/src", "children": [] }],
///   "rows": [{ "name": "main.rs", "size": "1.2KB" }],
///   "status": "3 files"
/// }
/// ```
pub fn tree_table(
    tree_bind: &str,
    tree_on_select: &str,
    table_bind: &str,
    table_columns: &[&str],
) -> UiSchema {
    tree_table_with(TreeTableOpts {
        tree_bind: tree_bind.into(),
        tree_on_select: tree_on_select.into(),
        table_bind: table_bind.into(),
        table_columns: table_columns.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    })
}

pub fn tree_table_with(opts: TreeTableOpts) -> UiSchema {
    let mut children: Vec<UiNode> = Vec::new();

    if !opts.toolbar.is_empty() {
        let btns: Vec<UiNode> = opts.toolbar
            .iter()
            .map(|(label, action)| UiNode::button(label, action))
            .collect();
        children.push(
            UiNode::new("container")
                .prop("direction", serde_json::json!("row"))
                .prop("gap", serde_json::json!(8))
                .id("toolbar")
                .children(btns),
        );
    }

    let mut tree_node = UiNode::tree(&opts.tree_bind)
        .on_action(&opts.tree_on_select)
        .id("tree");
    if let Some(ref title) = opts.tree_title {
        tree_node = tree_node.prop("title", serde_json::json!(title));
    }

    let mut table_node = UiNode::table(&opts.table_bind, &opts.table_columns);
    if let Some(ref action) = opts.table_on_action {
        table_node = table_node.on_action(action);
    }

    children.push(
        UiNode::split("row")
            .prop("left_width", serde_json::json!(opts.left_width))
            .prop("gap", serde_json::json!(1))
            .child(tree_node)
            .child(table_node),
    );

    if let Some(ref status_bind) = opts.status_bind {
        children.push(
            UiNode::label("").bind(status_bind).id("status-bar")
                .prop("style", serde_json::json!("status")),
        );
    }

    UiSchema { layout: "flex-col".into(), children, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 表单页面
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct FormPageOpts {
    pub title: Option<String>,
    pub fields: Vec<FormField>,
    pub actions: Vec<(String, String)>,
    pub bordered: bool,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub bind: String,
    pub component: String,
    pub props: serde_json::Value,
    pub readonly: bool,
}

impl FormField {
    pub fn input(label: &str, bind: &str) -> Self {
        Self { label: label.into(), bind: bind.into(), component: "input".into(), props: serde_json::json!({}), readonly: false }
    }
    pub fn textarea(label: &str, bind: &str) -> Self {
        Self { label: label.into(), bind: bind.into(), component: "textarea".into(), props: serde_json::json!({}), readonly: false }
    }
    pub fn select(label: &str, bind: &str, options: Vec<(&str, &str)>) -> Self {
        Self { label: label.into(), bind: bind.into(), component: "select".into(), props: serde_json::json!({ "options": options }), readonly: false }
    }
    pub fn switch(label: &str, bind: &str) -> Self {
        Self { label: label.into(), bind: bind.into(), component: "switch".into(), props: serde_json::json!({}), readonly: false }
    }
    pub fn date(label: &str, bind: &str) -> Self {
        Self { label: label.into(), bind: bind.into(), component: "date".into(), props: serde_json::json!({}), readonly: false }
    }
    pub fn readonly(mut self) -> Self { self.readonly = true; self }
    pub fn placeholder(mut self, v: &str) -> Self {
        if let Some(map) = self.props.as_object_mut() { map.insert("placeholder".into(), serde_json::json!(v)); }
        self
    }
}

/// 标准表单页面：标题 + 字段列表 + 底部按钮
///
/// # state 格式
/// ```json
/// { "name": "张三", "email": "z@example.com", "role": "admin" }
/// ```
pub fn form_page(opts: FormPageOpts) -> UiSchema {
    let mut children: Vec<UiNode> = Vec::new();

    if let Some(title) = opts.title {
        children.push(UiNode::new("label").prop("text", serde_json::json!(title)).prop("size", serde_json::json!(20)).prop("style", serde_json::json!("heading")));
        children.push(UiNode::new("divider"));
    }

    for field in &opts.fields {
        let mut field_props = field.props.clone();
        if let Some(map) = field_props.as_object_mut() {
            map.insert("label".into(), serde_json::json!(&field.label));
        } else {
            field_props = serde_json::json!({ "label": &field.label });
        }
        if field.readonly {
            if let Some(map) = field_props.as_object_mut() { map.insert("readonly".into(), serde_json::json!(true)); }
        }
        let node = UiNode { component: field.component.clone(), props: field_props, children: Vec::new(), bind: Some(field.bind.clone()), on_action: None, id: None };
        children.push(UiNode::new("form").prop("layout", serde_json::json!("field-row")).child(node));
    }

    let actions = if opts.actions.is_empty() {
        vec![
            UiNode::button("取消", "cancel"),
            UiNode::button("提交", "submit").prop("variant", serde_json::json!("primary")),
        ]
    } else {
        opts.actions.iter().map(|(label, action)| UiNode::button(label, action)).collect()
    };
    children.push(UiNode::new("button_row").children(actions));

    UiSchema { layout: "flex-col".into(), gap: 16, children, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 数据看板
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DashboardCard {
    pub title: String,
    pub value_bind: String,
    pub value_style: String,
    pub subtitle: Option<String>,
    pub width: i64,
    pub on_click: Option<String>,
}

/// 数据看板：指标卡片网格，每行最多 3 张
///
/// # state 格式
/// ```json
/// { "total_users": 12800, "active_today": 3842, "revenue": 128500 }
/// ```
pub fn dashboard(cards: Vec<DashboardCard>) -> UiSchema {
    let rows: Vec<UiNode> = cards.chunks(3).map(|row| {
        let children: Vec<UiNode> = row.iter().map(|card| {
            let style = if card.value_style.is_empty() { "large_number".to_string() } else { card.value_style.clone() };
            let mut node = UiNode::new("card");
            node = node.child(UiNode::new("label").prop("text", serde_json::json!(&card.title)).prop("size", serde_json::json!(12)).prop("style", serde_json::json!("muted")));
            node = node.child(UiNode::new("display").bind(&card.value_bind).prop("style", serde_json::json!(&style)));
            if let Some(ref subtitle) = card.subtitle {
                node = node.child(UiNode::new("label").prop("text", serde_json::json!(subtitle)).prop("size", serde_json::json!(11)).prop("style", serde_json::json!("muted")));
            }
            if let Some(ref action) = card.on_click { node = node.on_action(action); }
            node.prop("width", serde_json::json!(card.width))
        }).collect();
        UiNode::new("flex-row").children(children)
    }).collect();

    UiSchema { layout: "flex-col".into(), gap: 16, children: rows, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 列表 + 详情
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ListDetailOpts {
    pub list_bind: String,
    pub list_display_field: String,
    pub detail_bind: String,
    pub on_select: String,
    pub detail_fields: Vec<(String, String)>,
    pub detail_actions: Vec<(String, String)>,
    pub list_width: i64,
    pub empty_text: Option<String>,
}

/// 列表+详情布局：左侧可选择列表，右侧详情面板
///
/// # state 格式
/// ```json
/// {
///   "servers": [{ "id": 1, "name": "Web Server 1", "status": "running" }],
///   "selected": { "id": 1, "name": "Web Server 1", "ip": "10.0.0.1" }
/// }
/// ```
pub fn list_detail(opts: ListDetailOpts) -> UiSchema {
    let list_width = if opts.list_width > 0 { opts.list_width } else { 240 };

    let list_node = UiNode::new("list")
        .bind(&opts.list_bind)
        .prop("display_field", serde_json::json!(&opts.list_display_field))
        .on_action(&opts.on_select)
        .id("list-panel");

    let mut detail_children: Vec<UiNode> = Vec::new();
    for (label, bind) in &opts.detail_fields {
        detail_children.push(UiNode::new("info").prop("fields", serde_json::json!([{ "label": label, "field": bind }])));
    }
    if !opts.detail_actions.is_empty() {
        let btns: Vec<UiNode> = opts.detail_actions.iter().map(|(l, a)| UiNode::button(l, a)).collect();
        detail_children.push(UiNode::new("button_row").children(btns));
    }
    let empty_text = opts.empty_text.unwrap_or_else(|| "请选择一个项目".to_string());
    if opts.detail_fields.is_empty() {
        detail_children.push(UiNode::new("label").prop("text", serde_json::json!(&empty_text)).prop("style", serde_json::json!("muted")));
    }

    let detail_node = UiNode::new("flex-col").prop("gap", serde_json::json!(16)).children(detail_children).id("detail-panel");

    let split = UiNode::split("row")
        .prop("direction", serde_json::json!("row"))
        .prop("left_width", serde_json::json!(list_width))
        .prop("gap", serde_json::json!(1))
        .child(list_node)
        .child(detail_node);

    UiSchema { layout: "flex-col".into(), children: vec![split], ..Default::default() }
}

// ---------------------------------------------------------------------------
// 多步骤向导
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WizardStep {
    pub title: String,
    pub content: UiNode,
}

/// 多步骤向导：步骤指示器 + 内容区 + 上一步/下一步导航
///
/// # state 格式
/// ```json
/// { "current_step": 0, "steps": [{ "title": "基本信息" }], "form_data": {} }
/// ```
pub fn wizard(steps: Vec<WizardStep>) -> UiSchema {
    let indicators: Vec<UiNode> = steps.iter().enumerate().map(|(i, step)| {
        let is_last = i == steps.len() - 1;
        UiNode::new("wizard-step").prop("index", serde_json::json!(i)).prop("title", serde_json::json!(&step.title)).prop("is_last", serde_json::json!(is_last))
    }).collect();

    let nav = UiNode::new("button_row").children(vec![
        UiNode::button("上一步", "wizard_prev").prop("id", serde_json::json!("btn-prev")),
        UiNode::button("下一步", "wizard_next").prop("variant", serde_json::json!("primary")).prop("id", serde_json::json!("btn-next")),
    ]);

    let mut children: Vec<UiNode> = Vec::new();
    children.push(UiNode::new("wizard-indicator").children(indicators));
    children.push(UiNode::new("divider"));
    if let Some(first) = steps.first() {
        children.push(first.content.clone());
    }
    children.push(UiNode::new("divider"));
    children.push(nav);

    UiSchema { layout: "flex-col".into(), gap: 16, children, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 设置页面
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SettingsGroup {
    pub title: String,
    pub items: Vec<SettingsItem>,
}

#[derive(Debug, Clone)]
pub struct SettingsItem {
    pub label: String,
    pub description: Option<String>,
    pub component: String,
    pub bind: String,
    pub props: serde_json::Value,
}

/// 设置页面：分组展示配置项，适合偏好设置/系统设置场景
///
/// # state 格式
/// ```json
/// { "general.theme": "dark", "network.proxy_enabled": true }
/// ```
pub fn settings_page(groups: Vec<SettingsGroup>) -> UiSchema {
    let mut children: Vec<UiNode> = Vec::new();

    for group in &groups {
        children.push(UiNode::new("label").prop("text", serde_json::json!(&group.title)).prop("size", serde_json::json!(14)).prop("style", serde_json::json!("section-header")).prop("weight", serde_json::json!("semibold")));
        children.push(UiNode::new("divider"));

        for item in &group.items {
            let mut node_props = item.props.clone();
            if let Some(map) = node_props.as_object_mut() {
                map.insert("label".into(), serde_json::json!(&item.label));
            } else {
                node_props = serde_json::json!({ "label": &item.label });
            }
            let node = UiNode { component: item.component.clone(), props: node_props, children: Vec::new(), bind: Some(item.bind.clone()), on_action: None, id: None };
            let mut row = UiNode::new("settings-row").child(node);
            if let Some(ref desc) = item.description {
                row = row.child(UiNode::new("label").prop("text", serde_json::json!(desc)).prop("size", serde_json::json!(11)).prop("style", serde_json::json!("muted")));
            }
            children.push(row);
        }
        children.push(UiNode::new("divider"));
    }

    children.push(UiNode::new("button_row").children(vec![
        UiNode::button("恢复默认", "settings_reset"),
        UiNode::button("保存设置", "settings_save").prop("variant", serde_json::json!("primary")),
    ]));

    UiSchema { layout: "flex-col".into(), gap: 12, children, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 空状态
// ---------------------------------------------------------------------------

/// 空状态占位：列表为空时显示友好提示和操作入口
pub fn empty_state(icon: &str, title: &str, description: &str) -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        align_items: Some("center".to_string()),
        justify_content: Some("center".to_string()),
        gap: 12,
        children: vec![
            UiNode::new("display").prop("style", serde_json::json!("icon-large")).prop("text", serde_json::json!(icon)),
            UiNode::new("label").prop("text", serde_json::json!(title)).prop("size", serde_json::json!(16)).prop("weight", serde_json::json!("semibold")),
            UiNode::new("label").prop("text", serde_json::json!(description)).prop("size", serde_json::json!(12)).prop("style", serde_json::json!("muted")),
        ],
    }
}

// ---------------------------------------------------------------------------
// 模态对话框
// ---------------------------------------------------------------------------

/// 模态对话框：居中弹窗，适合确认操作/简易表单
pub fn dialog(
    title: &str,
    content: Vec<UiNode>,
    confirm_label: &str,
    confirm_action: &str,
    cancel_label: &str,
    cancel_action: &str,
) -> UiSchema {
    let mut children: Vec<UiNode> = vec![
        UiNode::new("label").prop("text", serde_json::json!(title)).prop("size", serde_json::json!(16)).prop("weight", serde_json::json!("semibold")).prop("style", serde_json::json!("heading")),
        UiNode::new("divider"),
    ];
    children.extend(content);
    children.push(UiNode::new("button_row").children(vec![
        UiNode::button(cancel_label, cancel_action),
        UiNode::button(confirm_label, confirm_action).prop("variant", serde_json::json!("primary")),
    ]));
    UiSchema { layout: "flex-col".into(), gap: 16, children, ..Default::default() }
}

// ---------------------------------------------------------------------------
// 搜索 + 列表
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SearchListOpts {
    pub search_bind: String,
    pub search_action: String,
    pub list_bind: String,
    pub display_field: String,
    pub select_action: String,
    pub toolbar: Vec<(String, String)>,
}

/// 搜索 + 列表布局：顶部搜索框 + 工具栏 + 数据列表
///
/// # state 格式
/// ```json
/// { "query": "", "results": [{ "name": "产品 A", "price": 128 }] }
/// ```
pub fn search_list(opts: SearchListOpts) -> UiSchema {
    let mut children: Vec<UiNode> = Vec::new();

    let mut top_bar: Vec<UiNode> = vec![
        UiNode::new("input").bind(&opts.search_bind).prop("placeholder", serde_json::json!("搜索...")).on_action(&opts.search_action).id("search-input"),
    ];
    for (label, action) in &opts.toolbar {
        top_bar.push(UiNode::button(label, action));
    }

    children.push(UiNode::new("flex-row").prop("gap", serde_json::json!(8)).children(top_bar));
    children.push(UiNode::new("divider"));
    children.push(UiNode::new("list").bind(&opts.list_bind).prop("display_field", serde_json::json!(&opts.display_field)).on_action(&opts.select_action).id("result-list"));

    UiSchema { layout: "flex-col".into(), gap: 12, children, ..Default::default() }
}
