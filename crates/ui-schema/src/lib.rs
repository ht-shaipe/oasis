//! copyright © ecdata.cn 2026 - present
//! 插件 UI Schema 类型定义
//!
//! 设计原则：
//! - 通用 component schema：插件通过 `component` 字段声明要渲染的 gpui-component 组件
//! - 宿主渲染器按 component 名分发，未实现的组件优雅降级
//! - `props` 为自由 JSON，由渲染器按 component 类型解析
//! - `bind` 从插件 state 取值，`on_action` 触发回调

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 插件清单
// ---------------------------------------------------------------------------

/// 插件清单（含 UI schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmManifest {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub description: String,
    pub version: String,
    /// UI schema JSON
    pub ui: serde_json::Value,
}

// ---------------------------------------------------------------------------
// UI Schema（声明式 UI 描述）
// ---------------------------------------------------------------------------

/// UI 布局定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSchema {
    /// 布局模式：flex-col / flex-row / grid 等
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default)]
    pub children: Vec<UiNode>,
}

fn default_layout() -> String {
    "flex-col".into()
}

/// UI 节点 — 通用 component schema
///
/// 每个节点声明一个组件类型（对应 gpui-component 模块名），
/// 宿主渲染器根据 `component` 字段分发到对应的渲染函数。
/// 未识别的组件渲染为占位符。
///
/// # 示例
///
/// ```json
/// {
///   "component": "input",
///   "props": { "placeholder": "Enter host IP" },
///   "bind": "host"
/// }
/// ```
///
/// ```json
/// {
///   "component": "tab",
///   "props": { "active": 0 },
///   "children": [
///     { "component": "table", "bind": "results", "props": { "columns": ["port","status"] } },
///     { "component": "form", "children": [...] }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNode {
    /// 组件类型名，对应 gpui-component 模块名
    /// 常用值：display, label, button, button_row, input, select,
    ///         table, tree, tab, form, progress, switch, info,
    ///         divider, card, collapsible, skeleton
    pub component: String,

    /// 组件属性（自由 JSON，渲染器按 component 类型解析）
    #[serde(default)]
    pub props: serde_json::Value,

    /// 子节点（容器类组件使用：tab, form, card, collapsible 等）
    #[serde(default)]
    pub children: Vec<UiNode>,

    /// 数据绑定：从插件 state 中取值的 key 路径
    /// 支持 dot 路径如 "scan_results.0.port"
    #[serde(default)]
    pub bind: Option<String>,

    /// 事件回调：用户交互时宿主回调给插件的 action 名
    /// 用于 button 的 click、input 的 change 等
    #[serde(default)]
    pub on_action: Option<String>,

    /// 节点 ID（用于交互元素，gpui 需要 id 来创建 Stateful<Div>）
    #[serde(default)]
    pub id: Option<String>,
}

// ---------------------------------------------------------------------------
// 便捷构造函数
// ---------------------------------------------------------------------------

impl UiNode {
    /// 创建简单组件节点
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            props: serde_json::Value::Null,
            children: Vec::new(),
            bind: None,
            on_action: None,
            id: None,
        }
    }

    /// 创建带 props 的组件
    pub fn with_props(component: impl Into<String>, props: serde_json::Value) -> Self {
        Self {
            component: component.into(),
            props,
            children: Vec::new(),
            bind: None,
            on_action: None,
            id: None,
        }
    }

    /// 创建按钮节点
    pub fn button(label: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            component: "button".into(),
            props: serde_json::json!({ "label": label.into() }),
            children: Vec::new(),
            bind: None,
            on_action: Some(action.into()),
            id: None,
        }
    }

    /// 创建输入框节点
    pub fn input(bind: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            component: "input".into(),
            props: serde_json::json!({ "placeholder": placeholder.into() }),
            children: Vec::new(),
            bind: Some(bind.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建标签节点
    pub fn label(text: impl Into<String>) -> Self {
        Self {
            component: "label".into(),
            props: serde_json::json!({ "text": text.into() }),
            children: Vec::new(),
            bind: None,
            on_action: None,
            id: None,
        }
    }

    /// 创建数据展示节点
    pub fn display(field: impl Into<String>) -> Self {
        Self {
            component: "display".into(),
            props: serde_json::Value::Null,
            children: Vec::new(),
            bind: Some(field.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建进度条节点
    pub fn progress(field: impl Into<String>) -> Self {
        Self {
            component: "progress".into(),
            props: serde_json::Value::Null,
            children: Vec::new(),
            bind: Some(field.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建表格节点
    pub fn table(bind: impl Into<String>, columns: &[&str]) -> Self {
        Self {
            component: "table".into(),
            props: serde_json::json!({ "columns": columns }),
            children: Vec::new(),
            bind: Some(bind.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建分栏容器（左右/上下分割）
    /// `direction`: "row"（左右）或 "col"（上下）
    /// `sizes`: 各栏宽度比例，如 &[300, 700] 或 &[1, 2]
    pub fn split(direction: impl Into<String>) -> Self {
        Self {
            component: "split".into(),
            props: serde_json::json!({ "direction": direction.into() }),
            children: Vec::new(),
            bind: None,
            on_action: None,
            id: None,
        }
    }

    /// 创建文件树节点
    /// `bind`: state 中树数据的 key（数组，每项含 name/children/path/is_dir 等）
    /// `on_select`: 选中文件/目录时的 action 名
    pub fn tree(bind: impl Into<String>) -> Self {
        Self {
            component: "tree".into(),
            props: serde_json::json!({}),
            children: Vec::new(),
            bind: Some(bind.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建信息字段列表节点
    pub fn info(fields: &[(&str, &str)]) -> Self {
        let fields_json: Vec<serde_json::Value> = fields
            .iter()
            .map(|(label, field)| {
                serde_json::json!({ "label": label, "field": field })
            })
            .collect();
        Self {
            component: "info".into(),
            props: serde_json::json!({ "fields": fields_json }),
            children: Vec::new(),
            bind: None,
            on_action: None,
            id: None,
        }
    }

    /// 设置 props 中的单个字段
    pub fn prop(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        if self.props.is_null() {
            self.props = serde_json::json!({});
        }
        if let Some(map) = self.props.as_object_mut() {
            map.insert(key.into(), value);
        }
        self
    }

    /// 设置 bind
    pub fn bind(mut self, bind: impl Into<String>) -> Self {
        self.bind = Some(bind.into());
        self
    }

    /// 设置 on_action
    pub fn on_action(mut self, action: impl Into<String>) -> Self {
        self.on_action = Some(action.into());
        self
    }

    /// 设置 id
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 添加子节点
    pub fn child(mut self, child: UiNode) -> Self {
        self.children.push(child);
        self
    }

    /// 添加多个子节点
    pub fn children(mut self, children: impl IntoIterator<Item = UiNode>) -> Self {
        self.children.extend(children);
        self
    }
}

// ---------------------------------------------------------------------------
// 视图模板 — 预定义常用布局，减少插件手动拼 schema 的工作量
// ---------------------------------------------------------------------------

/// 视图模板工厂
///
/// 提供常见布局的快捷构造方法，返回 `UiSchema`。
/// 插件可直接在 `ui_schema()` 返回值中使用这些模板。
pub mod template {
    use super::UiNode;

    /// 文件管理器布局：左侧文件树 + 右侧表格
    ///
    /// # 参数
    /// - `tree_bind`: state 中文件树数据的 key（数组，每项含 name/children/path/is_dir）
    /// - `tree_on_select`: 点击树节点时的 action 名
    /// - `table_bind`: state 中表格数据的 key
    /// - `table_columns`: 表头列名
    ///
    /// # state 数据格式示例
    ///
    /// ```json
    /// {
    ///   "files": [
    ///     { "name": "src", "is_dir": true, "path": "/src", "children": [
    ///       { "name": "main.rs", "is_dir": false, "path": "/src/main.rs" }
    ///     ]},
    ///     { "name": "Cargo.toml", "is_dir": false, "path": "/Cargo.toml" }
    ///   ],
    ///   "records": [
    ///     { "name": "main.rs", "size": "1.2KB", "modified": "2026-05-21" }
    ///   ]
    /// }
    /// ```
    ///
    /// # 生成的 schema 结构
    ///
    /// ```text
    /// ┌─────────────┬──────────────────────────┐
    /// │  文件树      │  表格                    │
    /// │  (300px)    │  (flex-1)               │
    /// │             │                          │
    /// │  📁 src     │  name  | size  | modified│
    /// │  📄 main.rs │  main.rs | 1.2KB | 05-21 │
    /// └─────────────┴──────────────────────────┘
    /// ```
    pub fn file_manager(
        tree_bind: &str,
        tree_on_select: &str,
        table_bind: &str,
        table_columns: &[&str],
    ) -> super::UiSchema {
        super::UiSchema {
            layout: "flex-col".into(),
            children: vec![
                // 分栏容器
                UiNode::split("row")
                    .prop("left_width", serde_json::json!(300))
                    .prop("gap", serde_json::json!(1))
                    .child(
                        // 左侧：文件树
                        UiNode::tree(tree_bind)
                            .on_action(tree_on_select)
                            .id("file-tree"),
                    )
                    .child(
                        // 右侧：表格
                        UiNode::table(table_bind, table_columns),
                    ),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// 向后兼容类型别名
// ---------------------------------------------------------------------------

/// 按钮定义（向后兼容，新代码直接用 UiNode::button）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonDef {
    pub label: String,
    pub action: String,
    #[serde(default)]
    pub variant: String,
}

/// 信息字段（向后兼容，新代码直接用 UiNode::info）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoField {
    pub label: String,
    pub field: String,
}

// ---------------------------------------------------------------------------
// 宿主环境 API（Host Imports，WASM 专用）
// ---------------------------------------------------------------------------

/// 宿主注入给 WASM 插件的环境函数
pub struct HostEnv;

impl HostEnv {
    pub const MODULE_NAME: &'static str = "env";
    pub const FN_GET_CONTEXT: &'static str = "host_get_context";
    pub const FN_LOG: &'static str = "host_log";
    pub const FN_READ_FILE: &'static str = "host_read_file";
    pub const FN_WRITE_FILE: &'static str = "host_write_file";
    pub const FN_SHOW_NOTIFICATION: &'static str = "host_show_notification";
}

/// 宿主传给 WASM 插件的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContext {
    pub current_file: Option<String>,
    pub selected_text: Option<String>,
    pub work_dir: Option<String>,
    pub locale: String,
    #[serde(default)]
    pub extra: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Props 解析辅助
// ---------------------------------------------------------------------------

/// 从 props JSON 中取字符串字段
pub fn prop_str<'a>(props: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    props.get(key).and_then(|v| v.as_str())
}

/// 从 props JSON 中取字符串字段，带默认值
pub fn prop_str_or<'a>(props: &'a serde_json::Value, key: &str, default: &'a str) -> &'a str {
    prop_str(props, key).unwrap_or(default)
}

/// 从 props JSON 中取整数
pub fn prop_i64(props: &serde_json::Value, key: &str) -> Option<i64> {
    props.get(key).and_then(|v| v.as_i64())
}

/// 从 props JSON 中取布尔值
pub fn prop_bool(props: &serde_json::Value, key: &str) -> Option<bool> {
    props.get(key).and_then(|v| v.as_bool())
}

/// 从 props JSON 中取数组
pub fn prop_array<'a>(props: &'a serde_json::Value, key: &str) -> Option<&'a Vec<serde_json::Value>> {
    props.get(key).and_then(|v| v.as_array())
}

/// 从 state 中按 bind 路径取值（支持 dot notation）
pub fn state_get<'a>(state: &'a serde_json::Value, bind: &str) -> Option<&'a serde_json::Value> {
    let mut current = state;
    for key in bind.split('.') {
        if let Ok(idx) = key.parse::<usize>() {
            current = current.as_array().and_then(|a| a.get(idx))?;
        } else {
            current = current.get(key)?;
        }
    }
    Some(current)
}

/// 从 state 中按 bind 路径取字符串
pub fn state_get_str(state: &serde_json::Value, bind: &str) -> String {
    state_get(state, bind)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 从 state 中按 bind 路径取整数
pub fn state_get_i64(state: &serde_json::Value, bind: &str) -> i64 {
    state_get(state, bind)
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// 简单插值：将 `{field}` 替换为状态中的值
pub fn state_interpolate(state: &serde_json::Value, template: &str) -> String {
    let mut result = template.to_string();
    loop {
        let start = result.find('{');
        let end = result.find('}');
        match (start, end) {
            (Some(s), Some(e)) if s < e => {
                let key = &result[s + 1..e];
                let value = state
                    .get(key)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default();
                result = result.replacen(&format!("{{{}}}", key), &value, 1);
            }
            _ => break,
        }
    }
    result
}
