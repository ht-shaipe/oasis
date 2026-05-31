use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

// ── 匹配状态 ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum MatchStatus {
    Found {
        source: PathBuf,
        file_name: String,
    },
    Missing {
        base: String,
    },
    Duplicate {
        base: String,
        candidates: Vec<String>,
    },
}

// ── 内部辅助函数 ──────────────────────────────────────────────────────────────────

fn file_ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn read_csv_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
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

fn read_excel_first_sheet(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use calamine::{Reader as _, open_workbook_auto};
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let sheet_names = workbook.sheet_names().to_vec();
    let name = sheet_names.first().ok_or("工作簿无工作表")?;
    let range = workbook
        .worksheet_range(name)
        .map_err(|e| format!("{:?}", e))?;

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect::<Vec<_>>())
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

fn read_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let ext = file_ext_lower(path);
    if ext == "csv" {
        return read_csv_table(path);
    }
    if ext == "xlsx" || ext == "xls" || ext == "xlsm" {
        return read_excel_first_sheet(path);
    }
    Err(format!("不支持的文件格式: {}", ext))
}

fn normalize_suffix(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('.') {
        Some(s.to_string())
    } else {
        Some(format!(".{s}"))
    }
}

fn normalize_base_name(raw: &str) -> String {
    let mut s = raw.trim();
    if s.is_empty() {
        return String::new();
    }

    // 去掉 URL query/fragment
    if let Some((left, _)) = s.split_once('?') {
        s = left;
    }
    if let Some((left, _)) = s.split_once('#') {
        s = left;
    }

    // DOI/URL 常见格式：取最后一个 path segment
    let s = s.trim_end_matches('/');
    if let Some(seg) = s.rsplit('/').find(|p| !p.trim().is_empty()) {
        return seg.trim().to_string();
    }

    s.to_string()
}

fn move_file_fallback(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 跨盘/跨设备 rename 失败，退化为 copy + remove
            fs::copy(src, dst).map_err(|e2| format!("{e}; copy failed: {e2}"))?;
            fs::remove_file(src).map_err(|e2| format!("{e}; remove failed: {e2}"))?;
            Ok(())
        }
    }
}

// ── 公开 API ──────────────────────────────────────────────────────────────────────

/// 读取 Excel/CSV 文件的表头
pub fn read_headers(path: &Path) -> Result<Vec<String>, String> {
    let (headers, _) = read_table(path)?;
    Ok(headers)
}

/// 根据 Excel 表数据构建文件匹配计划。
///
/// - `excel_path`：Excel/CSV 文件路径
/// - `col_header`：列名（为空时使用 `col_index_1based`）
/// - `col_index_1based`：1-based 列索引
/// - `input_dir`：待匹配文件所在目录
/// - `suffixes`：文件后缀列表（如 `[".pdf", ".docx"]`）
pub fn build_match_plan(
    excel_path: &Path,
    col_header: &str,
    col_index_1based: u32,
    input_dir: &Path,
    suffixes: &[String],
) -> Result<Vec<MatchStatus>, String> {
    let (headers, rows) = read_table(excel_path)?;
    if headers.is_empty() {
        return Err("表头为空".to_string());
    }

    let idx = if !col_header.trim().is_empty() {
        headers
            .iter()
            .position(|h| h.trim() == col_header.trim())
            .ok_or_else(|| format!("未找到列: {}", col_header.trim()))?
    } else {
        let i = col_index_1based.saturating_sub(1) as usize;
        if i >= headers.len() {
            return Err(format!("列索引超出范围: {}", col_index_1based));
        }
        i
    };

    let mut statuses = Vec::new();
    for row in rows {
        let raw = row.get(idx).map(|s| s.as_str()).unwrap_or("");
        let base = normalize_base_name(raw);
        if base.is_empty() {
            continue;
        }

        let mut hits = Vec::new();
        for suf in suffixes {
            let candidate = format!("{}{suf}", base);
            let p = input_dir.join(&candidate);
            if p.is_file() {
                hits.push(candidate);
            }
        }

        match hits.len() {
            0 => statuses.push(MatchStatus::Missing { base }),
            1 => {
                let file_name = hits[0].clone();
                statuses.push(MatchStatus::Found {
                    source: input_dir.join(&file_name),
                    file_name,
                })
            }
            _ => statuses.push(MatchStatus::Duplicate {
                base,
                candidates: hits,
            }),
        }
    }

    Ok(statuses)
}

/// 生成匹配预览文本，返回 (可移动数量, 预览文本)
pub fn preview_text(statuses: &[MatchStatus], output_dir: &Path) -> (usize, String) {
    let mut found = 0usize;
    let mut missing = 0usize;
    let mut dup = 0usize;
    let mut lines = Vec::new();

    for s in statuses {
        match s {
            MatchStatus::Found { source, file_name } => {
                found += 1;
                let dst = output_dir.join(file_name);
                lines.push(format!(
                    "[OK] {}  →  {}",
                    source.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    dst.file_name().and_then(|n| n.to_str()).unwrap_or("")
                ));
            }
            MatchStatus::Missing { .. } => missing += 1,
            MatchStatus::Duplicate { .. } => dup += 1,
        }
    }

    let head = format!(
        "找到 {} | 总计 {} | 缺失 {} | 重复 {}",
        found,
        statuses.len(),
        missing,
        dup
    );
    (found, format!("{head}\n{}", lines.join("\n")))
}

/// 执行文件移动：将匹配成功的文件移动到 `output_dir`
/// 返回 (成功数, 错误列表)
pub fn apply_move(
    statuses: &[MatchStatus],
    output_dir: &Path,
    suffixes: &[String],
) -> (usize, Vec<String>) {
    let mut ok = 0usize;
    let mut errs = Vec::new();
    let mut used_targets: HashSet<PathBuf> = HashSet::new();

    for s in statuses {
        let (src, file_name) = match s {
            MatchStatus::Found { source, file_name } => (source, file_name),
            _ => continue,
        };
        let dst = output_dir.join(file_name);
        if !used_targets.insert(dst.clone()) {
            errs.push(format!("重复输出: {}", dst.display()));
            continue;
        }
        if dst.exists() {
            errs.push(format!("输出已存在: {}", dst.display()));
            continue;
        }
        if !src.is_file() {
            errs.push(format!("源文件不存在: {}", src.display()));
            continue;
        }
        if let Err(e) = move_file_fallback(src, &dst) {
            errs.push(format!("{} → {}: {e}", src.display(), dst.display()));
        } else {
            ok += 1;
        }
    }

    if suffixes.is_empty() {
        errs.push("未指定后缀".to_string());
    }

    (ok, errs)
}