//! UI DSL Widget 定义
//!
//! 定义声明式 UI 描述格式

use serde::{Deserialize, Serialize};

/// 间距对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

/// 间距对齐方式（垂直方向）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

/// 动作/事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// 自定义动作标识符
    Custom(String),
    /// 预设动作
    Increment,
    Decrement,
    Reset,
    Submit,
    Cancel,
    Delete,
    Edit,
    Close,
    Toggle,
}

/// 可点击的 Widget 包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clickable<T> {
    pub widget: T,
    pub action: Action,
    pub disabled: Option<bool>,
}

impl<T> Clickable<T> {
    pub fn new(widget: T, action: impl Into<Action>) -> Self {
        Self {
            widget,
            action: action.into(),
            disabled: Some(false),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

impl Action {
    /// 从字符串转换为 Action
    pub fn from_str(s: &str) -> Self {
        match s {
            "increment" => Action::Increment,
            "decrement" => Action::Decrement,
            "reset" => Action::Reset,
            "submit" => Action::Submit,
            "cancel" => Action::Cancel,
            "delete" => Action::Delete,
            "edit" => Action::Edit,
            "close" => Action::Close,
            "toggle" => Action::Toggle,
            other => Action::Custom(other.to_string()),
        }
    }

    /// 获取动作标识符字符串
    pub fn as_str(&self) -> &str {
        match self {
            Action::Custom(s) => s,
            Action::Increment => "increment",
            Action::Decrement => "decrement",
            Action::Reset => "reset",
            Action::Submit => "submit",
            Action::Cancel => "cancel",
            Action::Delete => "delete",
            Action::Edit => "edit",
            Action::Close => "close",
            Action::Toggle => "toggle",
        }
    }
}

impl<T> From<T> for Clickable<T> {
    fn from(widget: T) -> Self {
        Self {
            widget,
            action: Action::Custom("".to_string()),
            disabled: Some(false),
        }
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action::from_str(s)
    }
}

impl From<String> for Action {
    fn from(s: String) -> Self {
        Action::from_str(&s)
    }
}

/// 文字样式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextStyle {
    pub size: Option<f32>,
    pub bold: Option<bool>,
    pub color: Option<String>,
}

/// 按钮样式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ButtonStyle {
    pub bg: Option<String>,
    pub color: Option<String>,
    pub size: Option<f32>,
    pub rounded: Option<bool>,
}

/// 背景样式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackgroundStyle {
    pub color: Option<String>,
    pub opacity: Option<f32>,
}

/// Widget 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Widget {
    /// 列布局（垂直）
    Column {
        #[serde(default = "default_gap")]
        gap: f32,
        #[serde(default)]
        align: Align,
        #[serde(default)]
        padding: Option<f32>,
        #[serde(default)]
        bg: Option<BackgroundStyle>,
        #[serde(default)]
        rounded: Option<f32>,
        #[serde(default)]
        children: Vec<Widget>,
    },
    /// 行布局（水平）
    Row {
        #[serde(default = "default_gap")]
        gap: f32,
        #[serde(default)]
        align: Align,
        #[serde(default)]
        padding: Option<f32>,
        #[serde(default)]
        bg: Option<BackgroundStyle>,
        #[serde(default)]
        rounded: Option<f32>,
        #[serde(default)]
        children: Vec<Widget>,
    },
    /// 文本
    Text {
        value: String,
        #[serde(default)]
        style: Option<TextStyle>,
    },
    /// 按钮
    Button {
        label: String,
        action: Action,
        #[serde(default)]
        style: Option<ButtonStyle>,
    },
    /// 图标
    Icon {
        value: String,
        #[serde(default)]
        size: Option<f32>,
    },
    /// 间距
    Spacer {
        #[serde(default = "default_spacer")]
        size: f32,
    },
    /// 进度条
    Progress {
        value: f32,
        max: f32,
        #[serde(default)]
        height: Option<f32>,
        #[serde(default)]
        bg_color: Option<String>,
        #[serde(default)]
        fill_color: Option<String>,
    },
    /// 分割线
    Divider {
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        thickness: Option<f32>,
    },
    /// 输入框（只读显示）
    Input {
        value: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        style: Option<InputStyle>,
    },
    /// 图片
    Image {
        src: String,
        #[serde(default)]
        width: Option<f32>,
        #[serde(default)]
        height: Option<f32>,
        #[serde(default)]
        rounded: Option<f32>,
    },
    /// 徽章/标签
    Badge {
        label: String,
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        text_color: Option<String>,
        #[serde(default)]
        style: Option<BadgeStyle>,
    },
    /// 开关
    Toggle {
        checked: bool,
        action: Action,
        #[serde(default)]
        label: Option<String>,
    },
    /// 滑块
    Slider {
        value: f32,
        min: f32,
        max: f32,
        step: Option<f32>,
        action: Action,
        #[serde(default)]
        show_value: Option<bool>,
    },
}

/// 输入框样式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputStyle {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub border_color: Option<String>,
    pub bg_color: Option<String>,
}

/// 徽章样式
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BadgeStyle {
    pub padding_h: Option<f32>,
    pub padding_v: Option<f32>,
    pub rounded: Option<f32>,
}

fn default_gap() -> f32 {
    8.0
}

fn default_spacer() -> f32 {
    16.0
}

/// 插件返回的完整 UI 描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUI {
    /// 根 Widget
    pub root: Widget,
}

/// 插件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// 状态数据（JSON 格式）
    pub data: serde_json::Value,
    /// 可用的动作列表
    #[serde(default)]
    pub actions: Vec<String>,
}

impl PluginState {
    /// 从 JSON 创建
    pub fn from_json(data: serde_json::Value) -> Self {
        Self {
            data,
            actions: vec![],
        }
    }

    /// 获取状态值
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// 获取字符串值
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// 获取数值
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.data.get(key).and_then(|v| v.as_f64())
    }

    /// 格式化文本，替换 {key} 为状态值
    pub fn format_value(&self, value: &str) -> String {
        let mut result = value.to_string();
        if let Some(obj) = self.data.as_object() {
            for (key, val) in obj {
                let placeholder = format!("{{{}}}", key);
                let val_str = match val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "null".to_string(),
                    _ => serde_json::to_string(val).unwrap_or_default(),
                };
                result = result.replace(&placeholder, &val_str);
            }
        }
        result
    }
}
