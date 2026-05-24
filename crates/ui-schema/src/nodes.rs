//! copyright © ecdata.cn 2026 - present
//! UI Schema — 节点类型与便捷构造函数

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// UI 节点
// ---------------------------------------------------------------------------

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
    /// 注意：bind 参数是 state 中的字段名，用于显示动态值
    /// 如果要显示静态文本，使用 .prop("text", "内容")
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
    pub fn table(bind: impl Into<String>, columns: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let cols: Vec<String> = columns.into_iter().map(|c| c.as_ref().to_string()).collect();
        Self {
            component: "table".into(),
            props: serde_json::json!({ "columns": cols }),
            children: Vec::new(),
            bind: Some(bind.into()),
            on_action: None,
            id: None,
        }
    }

    /// 创建表格节点（带字段映射）
    /// `columns`: 每项为 (显示列名, JSON字段名)，如 ("名称", "name")
    /// 字段名为空字符串表示该列不取数据（用于操作列等）
    pub fn table_mapped(
        bind: impl Into<String>,
        columns: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let cols: Vec<serde_json::Value> = columns
            .into_iter()
            .map(|(label, field)| {
                serde_json::json!({"label": label.into(), "field": field.into()})
            })
            .collect();
        Self {
            component: "table".into(),
            props: serde_json::json!({ "columns": cols }),
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

    /// 创建下拉选择节点
    /// `bind`: state 中当前选中值的 key
    /// `options`: 选项列表，每项 { label, value }
    /// `on_action`: 选择变更时触发的 action
    pub fn select(
        bind: impl Into<String>,
        options: &[(impl ToString, impl ToString)],
    ) -> Self {
        let opts: Vec<serde_json::Value> = options
            .iter()
            .map(|(label, value)| {
                serde_json::json!({ "label": label.to_string(), "value": value.to_string() })
            })
            .collect();
        Self {
            component: "select".into(),
            props: serde_json::json!({ "options": opts }),
            children: Vec::new(),
            bind: Some(bind.into()),
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
