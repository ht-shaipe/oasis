//! JSON 工具：转换、合并

use plugin_sdk::{UiNode, UiSchema};
use crate::tools::home::make_button_row;

// ---------------------------------------------------------------------------
// JSON 转换
// ---------------------------------------------------------------------------
pub fn schema_json_convert() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("📋 JSON 转换"),
            make_button_row(&[("📂", "选择文件", "json_conv:pick_file")]),
            UiNode::input("format", "输出格式 (csv/xlsx)"),
            UiNode::display("status"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// JSON 合并
// ---------------------------------------------------------------------------
pub fn schema_json_merge() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("🔗 JSON 合并"),
            make_button_row(&[
                ("➕", "添加文件", "json_merge:add_file"),
                ("▶️", "执行合并", "json_merge:execute"),
            ]),
            UiNode::info(&[("文件数", "file_count")]),
            UiNode::display("merged"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// JSON 合并逻辑
// ---------------------------------------------------------------------------

/// 合并 JSON 文件
pub fn do_json_merge(files: &[String]) -> String {
    if files.is_empty() {
        return "未添加文件".to_string();
    }
    format!("将合并 {} 个 JSON 文件（功能开发中）", files.len())
}
