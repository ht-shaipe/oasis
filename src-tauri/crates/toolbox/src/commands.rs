//! Tauri command wrappers for toolbox functionality

use std::path::PathBuf;

use crate::csv_convert::{self, ConvertFormat};
use crate::excel_move::{self, MatchStatus};
use crate::json_convert::{self, JsonOutputFormat};
use crate::json_merge;
use crate::network_scan;
use crate::{csv_split, csv_stats};
use serde::{Deserialize, Serialize};

// ── CSV 统计 ──

#[derive(Debug, Serialize)]
pub struct CsvStatsEntry {
    pub path: String,
    pub lines: usize,
}

#[derive(Debug, Serialize)]
pub struct CsvStatsResult {
    pub entries: Vec<CsvStatsEntry>,
    pub total: usize,
}

#[tauri::command]
pub fn csv_scan_dir(dir: String) -> Result<CsvStatsResult, String> {
    let path = PathBuf::from(&dir);
    let (entries, total) = csv_stats::scan_csv_in_dir(&path)?;
    Ok(CsvStatsResult {
        entries: entries
            .into_iter()
            .map(|e| CsvStatsEntry {
                path: e.path.to_string_lossy().to_string(),
                lines: e.lines,
            })
            .collect(),
        total,
    })
}

// ── CSV 拆分 ──

#[tauri::command]
pub fn csv_split_file(input_path: String, output_dir: String, parts: usize) -> Result<(), String> {
    csv_split::do_split(
        &PathBuf::from(&input_path),
        &PathBuf::from(&output_dir),
        parts,
    )
}

// ── CSV/Excel 格式转换 ──

#[derive(Debug, Deserialize)]
pub struct ConvertParams {
    pub input_path: String,
    pub output_path: String,
    pub format: String, // "csv" | "json" | "sql"
}

#[tauri::command]
pub fn csv_convert_file(params: ConvertParams) -> Result<(), String> {
    let format = match params.format.as_str() {
        "csv" => ConvertFormat::Csv,
        "json" => ConvertFormat::Json,
        "sql" => ConvertFormat::Sql,
        f => return Err(format!("不支持的输出格式: {}", f)),
    };
    csv_convert::do_convert(
        &PathBuf::from(&params.input_path),
        &PathBuf::from(&params.output_path),
        format,
    )
}

// ── Excel 匹配移动 ──

#[derive(Debug, Serialize)]
pub struct MatchPreviewItem {
    pub status: String, // "found" | "missing" | "duplicate"
    pub source: String,
    pub file_name: String,
    pub base: String,
    pub candidates: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExcelMovePreview {
    pub found: usize,
    pub total: usize,
    pub missing: usize,
    pub duplicate: usize,
    pub items: Vec<MatchPreviewItem>,
    pub preview_text: String,
}

#[tauri::command]
pub fn excel_move_preview(
    excel_path: String,
    col_header: String,
    col_index: u32,
    input_dir: String,
    suffixes: Vec<String>,
    output_dir: String,
) -> Result<ExcelMovePreview, String> {
    let plan = excel_move::build_match_plan(
        &PathBuf::from(&excel_path),
        &col_header,
        col_index,
        &PathBuf::from(&input_dir),
        &suffixes,
    )?;

    let (found, preview_text) = excel_move::preview_text(&plan, &PathBuf::from(&output_dir));

    let mut items = Vec::new();
    let mut missing = 0usize;
    let mut duplicate = 0usize;

    for s in &plan {
        match s {
            MatchStatus::Found { source, file_name } => {
                items.push(MatchPreviewItem {
                    status: "found".to_string(),
                    source: source.to_string_lossy().to_string(),
                    file_name: file_name.clone(),
                    base: String::new(),
                    candidates: Vec::new(),
                });
            }
            MatchStatus::Missing { base } => {
                missing += 1;
                items.push(MatchPreviewItem {
                    status: "missing".to_string(),
                    source: String::new(),
                    file_name: String::new(),
                    base: base.clone(),
                    candidates: Vec::new(),
                });
            }
            MatchStatus::Duplicate { base, candidates } => {
                duplicate += 1;
                items.push(MatchPreviewItem {
                    status: "duplicate".to_string(),
                    source: String::new(),
                    file_name: String::new(),
                    base: base.clone(),
                    candidates: candidates.clone(),
                });
            }
        }
    }

    Ok(ExcelMovePreview {
        found,
        total: plan.len(),
        missing,
        duplicate,
        items,
        preview_text,
    })
}

#[tauri::command]
pub fn excel_move_apply(
    excel_path: String,
    col_header: String,
    col_index: u32,
    input_dir: String,
    suffixes: Vec<String>,
    output_dir: String,
) -> Result<String, String> {
    let plan = excel_move::build_match_plan(
        &PathBuf::from(&excel_path),
        &col_header,
        col_index,
        &PathBuf::from(&input_dir),
        &suffixes,
    )?;

    let output = PathBuf::from(&output_dir);
    let (ok, errs) = excel_move::apply_move(&plan, &output, &suffixes);

    let mut result = format!("成功移动 {} 个文件", ok);
    if !errs.is_empty() {
        result.push_str(&format!("\n错误:\n{}", errs.join("\n")));
    }
    Ok(result)
}

// ── JSON 转换 ──

#[derive(Debug, Deserialize)]
pub struct JsonConvertParams {
    pub input_path: String,
    pub output_path: String,
    pub output_format: String, // "csv" | "excel"
    pub json_path: String,     // dot-separated path to array
    pub fields: Vec<String>,   // empty = all fields
}

#[tauri::command]
pub fn json_convert_file(params: JsonConvertParams) -> Result<(), String> {
    let format = match params.output_format.as_str() {
        "csv" => JsonOutputFormat::Csv,
        "excel" => JsonOutputFormat::Excel,
        f => return Err(format!("不支持的输出格式: {}", f)),
    };
    json_convert::do_json_convert(
        &PathBuf::from(&params.input_path),
        &PathBuf::from(&params.output_path),
        &params.json_path,
        &params.fields,
        format,
    )
}

#[derive(Debug, Deserialize)]
pub struct BatchJsonConvertParams {
    pub input_dir: String,
    pub output_dir: String,
    pub output_format: String,
    pub json_path: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub ok: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
pub fn json_convert_batch(params: BatchJsonConvertParams) -> Result<BatchResult, String> {
    let format = match params.output_format.as_str() {
        "csv" => JsonOutputFormat::Csv,
        "excel" => JsonOutputFormat::Excel,
        f => return Err(format!("不支持的输出格式: {}", f)),
    };
    let (ok, errors) = json_convert::do_batch_json_convert(
        &PathBuf::from(&params.input_dir),
        &PathBuf::from(&params.output_dir),
        &params.json_path,
        &params.fields,
        format,
    )?;
    Ok(BatchResult { ok, errors })
}

// ── JSON 合并 ──

#[tauri::command]
pub fn json_merge_files(
    input_dir: String,
    output_path: String,
    json_path: String,
) -> Result<String, String> {
    let count = json_merge::do_json_merge(
        &PathBuf::from(&input_dir),
        &PathBuf::from(&output_path),
        &json_path,
    )?;
    Ok(format!("合并完成，共 {} 条记录 → {}", count, output_path))
}

// ── 网络扫描 ──

#[derive(Debug, Serialize)]
pub struct ScanResultItem {
    pub ip: String,
    pub port: u16,
    pub open: bool,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NetworkScanResult {
    pub results: Vec<ScanResultItem>,
    pub format_text: String,
}

#[tauri::command]
pub fn network_scan_ports(
    ip_range: String,
    ports_str: String,
    timeout_ms: u64,
    show_closed: bool,
) -> Result<NetworkScanResult, String> {
    let ips = network_scan::parse_ip_range(&ip_range)?;
    let ports = network_scan::parse_ports(&ports_str)?;

    let mut results = Vec::new();
    for &ip in &ips {
        for &port in &ports {
            let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(ip), port);
            let r = network_scan::scan_port(addr, timeout_ms);
            results.push(r);
        }
    }
    // 转换为可序列化的结构
    let items: Vec<ScanResultItem> = results
        .iter()
        .map(|r| ScanResultItem {
            ip: r.ip.to_string(),
            port: r.port,
            open: r.open,
            latency_ms: r.latency_ms,
        })
        .collect();
    let format_text = network_scan::format_scan_results(&results, show_closed);

    Ok(NetworkScanResult {
        results: items,
        format_text,
    })
}
