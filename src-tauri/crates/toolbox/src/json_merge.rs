use std::path::{Path, PathBuf};

use serde_json::Value;

/// 按 dot-separated path 从 JSON Value 中提取子节点
fn resolve_json_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
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
            Value::Object(map) => {
                current = map.get(key)?;
            }
            Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// 扫描目录下所有 .json 文件，按 JSON Path 提取数组并合并为一个 JSON 文件。
/// 返回合并的数组总长度。
pub fn do_json_merge(
    input_dir: &Path,
    output_path: &Path,
    json_path: &str,
) -> Result<usize, String> {
    if !input_dir.exists() {
        return Err("输入目录不存在".to_string());
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

    let mut merged: Vec<Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for file_path in &json_files {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let raw = match std::fs::read_to_string(file_path) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let root: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let target = resolve_json_path(&root, json_path);
        match target {
            Some(Value::Array(arr)) => {
                merged.extend(arr.clone());
            }
            Some(_) => {
                errors.push(format!("{}: 目标不是数组", file_name));
            }
            None => {
                errors.push(format!(
                    "{}: 路径未找到 ({})",
                    file_name,
                    if json_path.is_empty() {
                        "(root)"
                    } else {
                        json_path
                    }
                ));
            }
        }
    }

    let content = serde_json::to_string_pretty(&merged).map_err(|e| e.to_string())?;
    std::fs::write(output_path, content).map_err(|e| e.to_string())?;

    if !errors.is_empty() {
        return Err(format!("部分文件失败:\n{}", errors.join("\n")));
    }

    Ok(merged.len())
}