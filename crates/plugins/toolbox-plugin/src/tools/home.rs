//! 主页 Schema

use plugin_sdk::{UiNode, UiSchema};

/// 主页：工具卡片网格
pub fn schema_home() -> UiSchema {
    let groups = vec![
        ("CSV 工具", vec![
            ("csv_stats", "CSV 统计", "CsvStats"),
            ("csv_split", "CSV 分割", "CsvSplit"),
            ("csv_convert", "CSV 转换", "CsvExcelConvert"),
        ]),
        ("文件工具", vec![
            ("batch_rename", "批量重命名", "BatchRename"),
            ("excel_move", "Excel 移动文件", "ExcelMoveFiles"),
        ]),
        ("API 工具", vec![
            ("api_request", "API 请求", "ApiRequest"),
            ("batch_dl", "批量下载", "ApiBatchDownload"),
        ]),
        ("JSON 工具", vec![
            ("json_conv", "JSON 转换", "JsonToCsvExcel"),
            ("json_merge", "JSON 合并", "JsonMerge"),
        ]),
        ("网络工具", vec![
            ("net_scan", "网络扫描", "NetworkScan"),
        ]),
    ];

    let mut children: Vec<UiNode> = vec![
        UiNode::new("card")
            .prop("title", serde_json::json!("工具箱"))
            .children(vec![
                UiNode::label("从这里进入各类工具，也可以直接查看模板与组件测试 demo。"),
                make_button_row(&[("打开测试 Demo", "UiSchemaDemo")]),
            ]),
    ];

    for (group_name, tools) in groups {
        children.push(UiNode::label(group_name));

        let cards: Vec<UiNode> = tools.iter().map(|(_id, name, tool_key)| {
            make_button_row(&[(*name, tool_key)])
        }).collect();

        children.push(UiNode::new("flex-row").prop("gap", serde_json::json!(12)).children(cards));
    }

    UiSchema {
        layout: "flex-col".into(),
        children,
        ..Default::default()
    }
}

/// 创建水平按钮行
pub fn make_button_row(buttons: &[(&str, &str)]) -> UiNode {
    UiNode::new("button_row").children(
        buttons
            .iter()
            .map(|(label, action)| UiNode::button(*label, *action))
            .collect::<Vec<_>>(),
    )
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
        "UiSchemaDemo" => ToolId::UiSchemaDemo,
        _ => ToolId::Home,
    }
}
