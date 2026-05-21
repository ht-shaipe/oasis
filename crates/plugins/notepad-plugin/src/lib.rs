//! 记事本插件 — 实现 Plugin trait，无 gpui 依赖

use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use plugin_sdk::{Plugin, PluginMeta, UiSchema, UiNode};

// ---------------------------------------------------------------------------
// Notepad State
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct NotepadState {
    pub content: String,
    #[serde(rename = "charCount")]
    pub char_count: usize,
    #[serde(rename = "lineCount")]
    pub line_count: usize,
}

impl Default for NotepadState {
    fn default() -> Self {
        Self {
            content: "欢迎使用记事本！\n\n这是一个简易文本编辑器插件。".into(),
            char_count: 0,
            line_count: 0,
        }
    }
}

fn compute_state(content: &str) -> NotepadState {
    NotepadState {
        char_count: content.chars().count(),
        line_count: content.lines().count(),
        content: content.into(),
    }
}

// ---------------------------------------------------------------------------
// NotepadPlugin
// ---------------------------------------------------------------------------

pub struct NotepadPlugin {
    state: RwLock<NotepadState>,
}

impl NotepadPlugin {
    pub fn new() -> Self {
        let state = compute_state(
            "欢迎使用记事本！\n\n这是一个简易文本编辑器插件。\n你可以在未来的版本中编辑文本内容。",
        );
        Self { state: RwLock::new(state) }
    }
}

impl Default for NotepadPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Plugin Trait
// ---------------------------------------------------------------------------

impl Plugin for NotepadPlugin {
    fn id(&self) -> &str {
        "notepad"
    }

    fn meta(&self) -> PluginMeta {
        PluginMeta::new(
            "notepad",
            "📝",
            "记事本",
            "一个简易文本编辑器插件",
            "1.0.0",
        )
    }

    fn state(&self) -> serde_json::Value {
        serde_json::to_value(&*self.state.read().unwrap()).unwrap_or(serde_json::Value::Null)
    }

    fn handle_action(&self, action: &str, _params: serde_json::Value) -> serde_json::Value {
        if let Some(text) = action.strip_prefix("set_content:") {
            *self.state.write().unwrap() = compute_state(text);
        }
        self.state()
    }

    fn ui_schema(&self) -> UiSchema {
        UiSchema {
            layout: "flex-col".into(),
            children: vec![
                // 文本展示区
                UiNode::display("content")
                    .id("notepad-content")
                    .child(
                        UiNode::with_props("display", serde_json::json!({
                            "style": "large_text"
                        })).bind("content"),
                    ),
                // 信息栏
                UiNode::info(&[
                    ("字符数", "charCount"),
                    ("行数", "lineCount"),
                ]),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// cdylib 导出入口
// ---------------------------------------------------------------------------

/// 插件工厂函数 — 供宿主 libloading 调用
#[unsafe(no_mangle)]
unsafe extern "C" fn plugin_entry() -> Arc<dyn Plugin> {
    Arc::new(NotepadPlugin::new())
}
