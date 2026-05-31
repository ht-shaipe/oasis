use std::path::{Path, PathBuf};

/// JSON 转换输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonOutputFormat {
    Csv,
    Excel,
}

// ── 内部辅助函数 ──────────────────────────────────────────────────────────────────

/// 解析 dot-separated JSON path
fn resolve_json_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(root);
    }
    let mut current = root;
    for key in path.split('.') {
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(key)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// 从 JSON 数组中提取 (headers, rows)。
/// 如果 `fields` 非空，则只保留指定的字段（按 fields 顺序排列）。
/// 如果 `fields` 为空，headers 取所有对象的 key 并集（保持首次出现顺序）。
fn extract_table(arr: &[serde_json::Value], fields: &[String]) -> (Vec<String>, Vec<Vec<String>>) {
    let headers: Vec<String> = if fields.is_empty() {
        let mut headers_ordered: Vec<String> = Vec::new();
        let mut header_set = std::collections::HashSet::new();
        for item in arr {
            if let serde_json::Value::Object(map) = item {
                for key in map.keys() {
                    if header_set.insert(key.clone()) {
                        headers_ordered.push(key.clone());
                    }
                }
            }
        }
        headers_ordered
    } else {
        fields.to_vec()
    };

    if headers.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            headers
                .iter()
                .map(|key| match item.get(key) {
                    Some(serde_json::Value::Null) => String::new(),
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Bool(b)) => b.to_string(),
                    Some(v) => serde_json::to_string(v).unwrap_or_default(),
                    None => String::new(),
                })
                .collect()
        })
        .collect();

    (headers, rows)
}

fn write_csv(output_path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<(), String> {
    let mut writer = csv::WriterBuilder::new()
        .from_path(output_path)
        .map_err(|e| e.to_string())?;
    writer.write_record(headers).map_err(|e| e.to_string())?;
    for row in rows {
        writer.write_record(row).map_err(|e| e.to_string())?;
    }
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn write_excel(output_path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<(), String> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();

    for (col, header) in headers.iter().enumerate() {
        let col_u16 = u16::try_from(col).map_err(|_| "列数超出限制")?;
        worksheet
            .write_string_with_format(0, col_u16, header, &header_format)
            .map_err(|e| e.to_string())?;
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let row_u32 = u32::try_from(row_idx + 1).map_err(|_| "行数超出限制")?;
        for (col, val) in row.iter().enumerate() {
            let col_u16 = u16::try_from(col).map_err(|_| "列数超出限制")?;
            worksheet
                .write_string(row_u32, col_u16, val)
                .map_err(|e| e.to_string())?;
        }
    }

    workbook.save(output_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ── 公开 API ──────────────────────────────────────────────────────────────────────

/// 执行 JSON → CSV/Excel 转换（单个文件）
pub fn do_json_convert(
    input_path: &Path,
    output_path: &Path,
    json_path: &str,
    fields: &[String],
    format: JsonOutputFormat,
) -> Result<(), String> {
    let raw = std::fs::read_to_string(input_path).map_err(|e| e.to_string())?;
    let root: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("无效 JSON: {}", e))?;

    let target = resolve_json_path(&root, json_path).ok_or_else(|| {
        format!(
            "路径未找到: {}",
            if json_path.is_empty() {
                "(root)"
            } else {
                json_path
            }
        )
    })?;

    let arr = match target {
        serde_json::Value::Array(a) => a,
        other => {
            return Err(format!("目标不是数组 (got {})", json_type_name(other)));
        }
    };

    if arr.is_empty() {
        return Err("数组为空".to_string());
    }

    let (headers, rows) = extract_table(arr, fields);
    if headers.is_empty() {
        return Err("无有效表头".to_string());
    }

    match format {
        JsonOutputFormat::Csv => write_csv(output_path, &headers, &rows),
        JsonOutputFormat::Excel => write_excel(output_path, &headers, &rows),
    }
}

/// 扫描目录下所有 .json 文件并批量转换。
/// 返回 (成功数, 失败文件列表)
pub fn do_batch_json_convert(
    input_dir: &Path,
    output_dir: &Path,
    json_path: &str,
    fields: &[String],
    format: JsonOutputFormat,
) -> Result<(usize, Vec<String>), String> {
    if !input_dir.exists() {
        return Err("输入目录不存在".to_string());
    }
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }

    let entries = std::fs::read_dir(input_dir).map_err(|e| e.to_string())?;
    let json_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
        .collect();

    if json_files.is_empty() {
        return Err("目录下无 JSON 文件".to_string());
    }

    let ext = match format {
        JsonOutputFormat::Csv => "csv",
        JsonOutputFormat::Excel => "xlsx",
    };

    let mut ok = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for input_path in &json_files {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.{}", stem, ext));

        match do_json_convert(input_path, &output_path, json_path, fields, format) {
            Ok(()) => ok += 1,
            Err(e) => {
                let file_name = input_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                errors.push(format!("{}: {}", file_name, e));
            }
        }
    }

    Ok((ok, errors))
}