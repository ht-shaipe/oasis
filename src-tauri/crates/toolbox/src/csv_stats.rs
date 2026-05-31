use std::io::{BufRead, BufReader};
use std::path::Path;

/// CSV 文件扫描条目
#[derive(Debug, Clone)]
pub struct CsvEntry {
    pub path: std::path::PathBuf,
    pub lines: usize,
}

/// 统计单个 CSV 文件的行数（正确处理字段内换行符，与 csv crate 对齐）
pub fn count_lines(path: &Path) -> Result<usize, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let mut count = 0usize;
    for result in reader.records() {
        result.map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// 扫描目录下所有 .csv 文件，统计每个文件行数，返回条目列表和总行数
pub fn scan_csv_in_dir(dir: &Path) -> Result<(Vec<CsvEntry>, usize), String> {
    let mut entries = Vec::new();
    let mut total = 0usize;

    let dir_entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in dir_entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        match count_lines(&path) {
            Ok(n) => {
                entries.push(CsvEntry { path, lines: n });
                total += n;
            }
            Err(e) => {
                log::warn!("跳过 {}：{}", path.display(), e);
            }
        }
    }

    entries.sort_by_key(|e| e.path.clone());
    Ok((entries, total))
}

/// 统计纯文本文件的行数（行缓冲方式，不处理 CSV 转义）
pub fn count_lines_raw(path: &Path) -> Result<usize, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut count = 0usize;
    for line in reader.lines() {
        line.map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}