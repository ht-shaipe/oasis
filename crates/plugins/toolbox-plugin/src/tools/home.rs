//! 主页 Schema

use plugin_sdk::{UiNode, UiSchema};

/// 主页：工具卡片网格
pub fn schema_home() -> UiSchema {
    let groups = vec![
        ("CSV 工具", vec![
            ("csv_stats", "📊", "CSV 统计", "CsvStats"),
            ("csv_split", "✂️", "CSV 分割", "CsvSplit"),
            ("csv_convert", "🔄", "CSV 转换", "CsvExcelConvert"),
        ]),
        ("文件工具", vec![
            ("batch_rename", "📝", "批量重命名", "BatchRename"),
            ("excel_move", "📁", "Excel 移动文件", "ExcelMoveFiles"),
        ]),
        ("API 工具", vec![
            ("api_request", "🌐", "API 请求", "ApiRequest"),
            ("batch_dl", "⬇️", "批量下载", "ApiBatchDownload"),
        ]),
        ("JSON 工具", vec![
            ("json_conv", "📋", "JSON 转换", "JsonToCsvExcel"),
            ("json_merge", "🔗", "JSON 合并", "JsonMerge"),
        ]),
        ("网络工具", vec![
            ("net_scan", "🔍", "网络扫描", "NetworkScan"),
        ]),
    ];

    let mut children: Vec<UiNode> = vec![
        UiNode::label("🧰 工具箱"),
    ];

    for (group_name, tools) in groups {
        children.push(UiNode::label(group_name));

        let cards: Vec<UiNode> = tools.iter().map(|(_id, icon, name, tool_key)| {
            make_button_row(&[(*icon, *name, tool_key)])
        }).collect();

        let row_node = UiNode::with_props("container", serde_json::json!({
            "style": "flex-row",
            "gap": "12px",
            "flex_wrap": true
        }));
        let mut row_node = row_node;
        for card in cards {
            row_node = row_node.child(card);
        }
        children.push(row_node);
    }

    UiSchema {
        layout: "flex-col".into(),
        children,
        ..Default::default()
    }
}

/// 创建水平按钮行
pub fn make_button_row(buttons: &[(&str, &str, &str)]) -> UiNode {
    let mut row = UiNode::split("row");
    for (icon, label, action) in buttons {
        row = row.child(UiNode::button(&format!("{} {}", *icon, *label), *action));
    }
    row
}

/// 解析工具 ID 字符串
pub fn parse_tool_id(s: &str) -> crate::state::ToolId {
    use crate::state::ToolId;
    match s {
        "CsvStats" => ToolId::CsvStats,
        "CsvSplit" => ToolId::CsvSplit,
        "CsvExcelConvert" => ToolId::CsvExcelConvert,
        "BatchRename" => ToolId::BatchRename,
        "ExcelMoveFiles" => ToolId::ExcelMoveFiles,
        "ApiRequest" => ToolId::ApiRequest,
        "ApiBatchDownload" => ToolId::ApiBatchDownload,
        "JsonToCsvExcel" => ToolId::JsonToCsvExcel,
        "JsonMerge" => ToolId::JsonMerge,
        "NetworkScan" => ToolId::NetworkScan,
        _ => ToolId::Home,
    }
}
