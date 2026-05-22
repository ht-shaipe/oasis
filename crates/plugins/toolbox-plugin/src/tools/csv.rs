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
pub fn do_csv_split(headers: &[String], rows: &[Vec<String>], col: usize, prefix: &str) -> String {
    if col >= headers.len() {
        return format!("无效列索引: {col}");
    }
    if rows.is_empty() {
        return "无数据行".to_string();
    }
    
    // col 参数用作分割份数
    let n_parts = col.max(1);
    let total = rows.len();
    let actual_parts = n_parts.min(total);
    let base = total / actual_parts;
    let remainder = total % actual_parts;
    let mut cursor = 0usize;
    
    let mut success = 0;
    let mut errors = Vec::new();
    
    for i in 0..actual_parts {
        let size = base + usize::from(i < remainder);
        let filename = format!("{}_{}.csv", prefix, i + 1);
        
        // 创建 CSV writer
        let mut writer = match csv::Writer::from_path(&filename) {
            Ok(w) => w,
            Err(e) => {
                errors.push(format!("创建文件 {} 失败: {}", filename, e));
                continue;
            }
        };
        
        // 写入标题行
        if let Err(e) = writer.write_record(headers) {
            errors.push(format!("写入标题到 {} 失败: {}", filename, e));
            continue;
        }
        
        // 写入数据行
        let mut row_ok = true;
        for row in &rows[cursor..cursor + size] {
            if let Err(e) = writer.write_record(row) {
                errors.push(format!("写入数据到 {} 失败: {}", filename, e));
                row_ok = false;
                break;
            }
        }
        
        if row_ok {
            if let Err(e) = writer.flush() {
                errors.push(format!("保存文件 {} 失败: {}", filename, e));
            } else {
                success += 1;
            }
        }
        
        cursor += size;
    }
    
    if success == actual_parts && errors.is_empty() {
        format!("成功分割为 {} 个文件: {}_{}.csv 到 {}_{}.csv", 
                actual_parts, prefix, 1, prefix, actual_parts)
    } else {
        let mut msg = format!("部分成功: {}/{} 个文件", success, actual_parts);
        if !errors.is_empty() {
            msg.push_str(&format!("\n错误:\n{}", errors.join("\n")));
        }
        msg
    }
}
