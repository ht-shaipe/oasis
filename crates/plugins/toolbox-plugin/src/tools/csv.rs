//! CSV 工具：统计、分割、转换

use plugin_sdk::{UiNode, UiSchema};
use crate::tools::home::make_button_row;

// ---------------------------------------------------------------------------
// 统计
// ---------------------------------------------------------------------------
pub fn schema_csv_stats() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("CSV 统计"),
            make_button_row(&[("选择文件", "csv_stats:pick_file")]),
            UiNode::display("stats"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 分割
// ---------------------------------------------------------------------------
pub fn schema_csv_split() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("CSV 分割"),
            make_button_row(&[("选择文件", "csv_split:pick_file")]),
            UiNode::info(&[
                ("分割列", "split_col"),
                ("输出前缀", "output_prefix"),
            ]),
            make_button_row(&[("执行分割", "csv_split:execute")]),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 转换
// ---------------------------------------------------------------------------
pub fn schema_csv_convert() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("CSV/Excel 转换"),
            make_button_row(&[("选择文件", "json_conv:pick_file")]),
            UiNode::display("status"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// CSV 数据逻辑
// ---------------------------------------------------------------------------

/// 读取 CSV 文件
pub fn read_csv(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)
        .map_err(|e| e.to_string())?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let rows: Vec<Vec<String>> = rdr
        .records()
        .map(|r| {
            r.map_err(|e| e.to_string())
                .map(|rec| rec.iter().map(|s| s.to_string()).collect())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((headers, rows))
}

/// 计算 CSV 统计摘要
pub fn compute_csv_stats(headers: &[String], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return "未加载文件".to_string();
    }
    format!(
        "文件已加载\n列数: {}\n行数: {}",
        headers.len(),
        rows.len()
    )
}

/// 执行 CSV 分割
pub fn do_csv_split(headers: &[String], _rows: &[Vec<String>], col: usize, prefix: &str) -> String {
    if col >= headers.len() {
        return format!("无效列索引: {col}");
    }
    format!(
        "将以列「{}」={} 分割文件（功能开发中）",
        headers[col], prefix
    )
}
