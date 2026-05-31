use std::path::Path;

/// 输出格式：CSV、JSON（每行一条 JSON）、SQL INSERT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertFormat {
    Csv,
    Json,
    Sql,
}

/// 读取 CSV 或 Excel 文件（自动识别扩展名），返回 (headers, rows)
pub fn read_sheet(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "csv" => read_csv(path),
        "xlsx" | "xls" | "xlsm" => read_excel(path),
        _ => Err(format!("不支持的文件格式: {}", ext)),
    }
}

fn read_csv(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
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
        .collect::<Result<_, _>>()?;

    Ok((headers, rows))
}

fn read_excel(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use calamine::{Reader as _, open_workbook_auto};

    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let sheet_names = workbook.sheet_names().to_vec();
    let name = sheet_names.first().ok_or("工作簿无工作表")?;
    let range = workbook
        .worksheet_range(name)
        .map_err(|e| format!("{:?}", e))?;

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect())
        .collect();

    if rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let headers = rows[0].clone();
    let data = rows[1..].to_vec();
    Ok((headers, data))
}

fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data;
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{}", f),
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(d) => format!("{}", d),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(_) => String::new(),
    }
}

/// 将 (headers, rows) 转为 JSON 行格式（每行一个 JSON 对象）
pub fn to_json(headers: &[String], rows: &[Vec<String>]) -> Result<String, String> {
    let mut out = String::new();
    for row in rows {
        let mut obj = serde_json::Map::new();
        for (i, val) in row.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
            obj.insert(key, serde_json::Value::String(val.clone()));
        }
        let line = serde_json::to_string(&serde_json::Value::Object(obj))
            .map_err(|e| e.to_string())?;
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// 将 (headers, rows) 转为 SQL INSERT 语句
pub fn to_sql(
    headers: &[String],
    rows: &[Vec<String>],
    table: &str,
) -> Result<String, String> {
    let mut out = String::new();
    for row in rows {
        let cols = headers.join(", ");
        let vals: Vec<String> = row.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect();
        out.push_str(&format!(
            "INSERT INTO {} ({}) VALUES ({});\n",
            table,
            cols,
            vals.join(", ")
        ));
    }
    Ok(out)
}

/// 执行格式转换：自动识别输入格式，输出指定格式
pub fn do_convert(
    input_path: &Path,
    output_path: &Path,
    format: ConvertFormat,
) -> Result<(), String> {
    let (headers, rows) = read_sheet(input_path)?;
    if headers.is_empty() {
        return Err("表头为空".to_string());
    }

    match format {
        ConvertFormat::Csv => {
            // 直接 copy（或重写）CSV
            let out_file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
            let mut writer = csv::Writer::from_writer(out_file);
            writer
                .write_record(&headers)
                .map_err(|e| e.to_string())?;
            for row in &rows {
                writer.write_record(row).map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
        }
        ConvertFormat::Json => {
            let content = to_json(&headers, &rows)?;
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(output_path, content).map_err(|e| e.to_string())?;
        }
        ConvertFormat::Sql => {
            let table = output_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("my_table");
            let content = to_sql(&headers, &rows, table)?;
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(output_path, content).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}