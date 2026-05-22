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
            UiNode::label("JSON 转换"),
            make_button_row(&[("选择文件", "json_conv:pick_file")]),
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
            UiNode::label("JSON 合并"),
            make_button_row(&[
                ("添加文件", "json_merge:add_file"),
                ("执行合并", "json_merge:execute"),
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
    
    let mut merged: Vec<serde_json::Value> = Vec::new();
    let mut errors = Vec::new();
    
    for (i, file_path) in files.iter().enumerate() {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("文件 {}: 读取失败 - {}", file_path, e));
                continue;
            }
        };
        
        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("文件 {}: JSON 解析失败 - {}", file_path, e));
                continue;
            }
        };
        
        match value {
            serde_json::Value::Array(arr) => {
                merged.extend(arr);
            }
            _ => {
                errors.push(format!("文件 {}: 不是 JSON 数组", file_path));
            }
        }
    }
    
    if merged.is_empty() {
        return format!("合并失败: 无有效数据\n{}", errors.join("\n"));
    }
    
    // 写入合并后的文件
    let output_path = "merged.json";
    let output_content = match serde_json::to_string_pretty(&merged) {
        Ok(c) => c,
        Err(e) => {
            return format!("序列化合并数据失败: {}", e);
        }
    };
    
    match std::fs::write(output_path, &output_content) {
        Ok(_) => {
            let mut msg = format!("成功合并 {} 个 JSON 文件，共 {} 项数据，保存到 {}", 
                                 files.len() - errors.len(), merged.len(), output_path);
            if !errors.is_empty() {
                msg.push_str(&format!("\n部分错误:\n{}", errors.join("\n")));
            }
            msg
        }
        Err(e) => {
            format!("写入合并文件失败: {}", e)
        }
    }
}
