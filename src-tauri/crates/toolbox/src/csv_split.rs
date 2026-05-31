use std::io::BufReader;
use std::path::Path;

/// 将 CSV 文件按份数切分，保留标题行。
/// 每份写入 `output_dir/stem_N.csv`。
pub fn do_split(input_path: &Path, output_dir: &Path, n_parts: usize) -> Result<(), String> {
    if n_parts == 0 {
        return Err("份数不能为零".to_string());
    }

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("split");

    // 读取全部记录
    let file = std::fs::File::open(input_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader
        .headers()
        .map_err(|e| e.to_string())?
        .clone();

    let mut records: Vec<csv::StringRecord> = Vec::new();
    for result in csv_reader.records() {
        let record = result.map_err(|e| e.to_string())?;
        records.push(record);
    }

    let total = records.len();
    if total == 0 {
        return Err("源 CSV 没有数据行".to_string());
    }

    let chunk_size = (total + n_parts - 1) / n_parts; // ceil division

    for (part, chunk) in records.chunks(chunk_size).enumerate() {
        let output_path = output_dir.join(format!("{}_{}.csv", stem, part + 1));
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let out_file = std::fs::File::create(&output_path).map_err(|e| e.to_string())?;
        let mut writer = csv::Writer::from_writer(out_file);
        writer.write_record(&headers).map_err(|e| e.to_string())?;
        for record in chunk {
            writer.write_record(record).map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
    }

    Ok(())
}