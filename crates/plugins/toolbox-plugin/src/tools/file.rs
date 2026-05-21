//! 文件工具：批量重命名、Excel 移动文件

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use plugin_sdk::{UiNode, UiSchema};
use crate::tools::home::make_button_row;

// ---------------------------------------------------------------------------
// 批量重命名
// ---------------------------------------------------------------------------
pub fn schema_batch_rename() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("📝 批量重命名"),
            make_button_row(&[("📂", "选择目录", "rename:pick_dir")]),
            UiNode::input("needle", "查找内容"),
            UiNode::input("replacement", "替换为"),
            make_button_row(&[
                ("👁️", "预览", "rename:preview"),
                ("▶️", "执行", "rename:execute"),
            ]),
            UiNode::display("preview"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Excel 移动文件
// ---------------------------------------------------------------------------
pub fn schema_excel_move() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("📁 Excel 移动文件"),
            make_button_row(&[
                ("📋", "选择 Excel", "excel_move:pick_excel"),
                ("📂", "输入目录", "excel_move:pick_input"),
                ("📤", "输出目录", "excel_move:pick_output"),
            ]),
            UiNode::input("suffixes", "文件后缀（逗号分隔）"),
            make_button_row(&[
                ("👁️", "预览", "excel_move:preview"),
                ("▶️", "执行", "excel_move:execute"),
            ]),
            UiNode::display("preview"),
            UiNode::info(&[("消息", "message")]),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 重命名逻辑
// ---------------------------------------------------------------------------

/// 列出目录中的文件
pub fn list_files_in_dir(dir: &str, recursive: bool) -> Result<Vec<std::path::PathBuf>, String> {
    let path = Path::new(dir);
    if !path.is_dir() {
        return Err("无效目录".to_string());
    }
    let mut out = Vec::new();
    let read = fs::read_dir(path).map_err(|e| e.to_string())?;
    for ent in read {
        let ent = ent.map_err(|e| e.to_string())?;
        let p = ent.path();
        if p.is_file() {
            out.push(p);
        } else if recursive && p.is_dir() {
            out.extend(list_files_in_dir(p.to_str().unwrap_or(""), true).unwrap_or_default());
        }
    }
    Ok(out)
}

/// 构建重命名计划
pub fn build_rename_plan_internal(
    files: &[std::path::PathBuf],
    needle: &str,
    replacement: &str,
) -> Result<Vec<(String, String)>, String> {
    if needle.trim().is_empty() {
        return Err("查找内容不能为空".to_string());
    }
    let mut plan = Vec::new();
    let mut new_names: HashSet<String> = HashSet::new();

    for path in files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !name.contains(needle) { continue; }
        let new_name = name.replace(needle, replacement);
        if new_name == name { continue; }
        let Some(parent) = path.parent() else { continue };
        let new_path = parent.join(&new_name);
        let new_path_str = new_path.to_string_lossy().into_owned();
        if !new_names.insert(new_path_str.clone()) {
            return Err(format!("重复目标文件: {}", new_path_str));
        }
        plan.push((
            path.to_string_lossy().into_owned(),
            new_path_str,
        ));
    }
    Ok(plan)
}

/// 预览重命名
pub fn do_rename_preview(dir: Option<&str>, needle: &str, replacement: &str, recursive: bool) -> String {
    let dir = match dir {
        Some(d) => d,
        None => return "请先选择目录".to_string(),
    };
    match list_files_in_dir(dir, recursive) {
        Ok(files) => {
            match build_rename_plan_internal(&files, needle, replacement) {
                Ok(plan) => {
                    if plan.is_empty() {
                        "未找到匹配文件".to_string()
                    } else {
                        let lines: Vec<String> = plan.iter().take(20)
                            .map(|(o, n)| format!("{} → {}", o, n))
                            .collect();
                        format!("将重命名 {} 个文件:\n{}", plan.len(), lines.join("\n"))
                    }
                }
                Err(e) => e
            }
        }
        Err(e) => e
    }
}

/// 执行重命名
pub fn do_rename_execute(dir: Option<&str>, needle: &str, replacement: &str, recursive: bool) -> String {
    let dir = match dir {
        Some(d) => d,
        None => return "请先选择目录".to_string(),
    };
    match list_files_in_dir(dir, recursive) {
        Ok(files) => {
            match build_rename_plan_internal(&files, needle, replacement) {
                Ok(plan) => {
                    let mut ok = 0;
                    let mut errs = Vec::new();
                    for (old, new) in &plan {
                        match fs::rename(old, new) {
                            Ok(_) => ok += 1,
                            Err(e) => errs.push(format!("{old}: {e}")),
                        }
                    }
                    format!("成功: {ok}, 失败: {}", errs.len())
                }
                Err(e) => e
            }
        }
        Err(e) => e
    }
}

// ---------------------------------------------------------------------------
// Excel 移动逻辑
// ---------------------------------------------------------------------------

/// 预览 Excel 移动
pub fn do_excel_move_preview(
    excel: &Option<String>,
    input_dir: &Option<String>,
    output_dir: &Option<String>,
    _suffixes: &str,
) -> (String, String) {
    if excel.is_none() || input_dir.is_none() || output_dir.is_none() {
        return (String::new(), "请先完善所有路径".to_string());
    }
    (format!("预览: {} → {}", input_dir.as_ref().unwrap(), output_dir.as_ref().unwrap()), "就绪".to_string())
}

/// 执行 Excel 移动
pub fn do_excel_move_execute(
    excel: &Option<String>,
    input_dir: &Option<String>,
    output_dir: &Option<String>,
    _suffixes: &str,
) -> String {
    if excel.is_none() || input_dir.is_none() || output_dir.is_none() {
        return "请先完善所有路径".to_string();
    }
    format!(
        "执行 Excel 移动: {} × {} → {}",
        excel.as_ref().unwrap(),
        input_dir.as_ref().unwrap(),
        output_dir.as_ref().unwrap()
    )
}
