use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use std::fs;
use std::path::PathBuf;

const WORKSPACE_CONFIG_FILE: &str = "workspace.json";
const DEFAULT_WORKSPACE: &str = ".oasis";

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn workspace_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join(WORKSPACE_CONFIG_FILE))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub workspace_dir: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        let home = dirs_home();
        Self {
            workspace_dir: format!("{}/{}", home, DEFAULT_WORKSPACE),
        }
    }
}

fn dirs_home() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
}

fn load_workspace_config(app: &AppHandle) -> Result<WorkspaceConfig, String> {
    let path = workspace_config_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let config: WorkspaceConfig =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        Ok(WorkspaceConfig::default())
    }
}

fn save_workspace_config(app: &AppHandle, config: &WorkspaceConfig) -> Result<(), String> {
    let path = workspace_config_path(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

/// Get the current workspace directory path (expanded absolute path).
#[tauri::command]
pub fn get_workspace_dir(app: AppHandle) -> Result<String, String> {
    let config = load_workspace_config(&app)?;
    let expanded = if config.workspace_dir.starts_with('~') {
        config.workspace_dir.replacen('~', &dirs_home(), 1)
    } else {
        config.workspace_dir
    };
    Ok(expanded)
}

/// Set the workspace directory path. Creates the directory if it doesn't exist.
/// Returns the expanded absolute path.
#[tauri::command]
pub fn set_workspace_dir(app: AppHandle, path: String) -> Result<String, String> {
    let expanded = if path.starts_with('~') {
        path.replacen('~', &dirs_home(), 1)
    } else {
        path
    };

    fs::create_dir_all(&expanded).map_err(|e| format!("Failed to create directory: {}", e))?;

    let config = WorkspaceConfig {
        workspace_dir: expanded.clone(),
    };
    save_workspace_config(&app, &config)?;

    Ok(expanded)
}

/// Read directory entries at the given path. Returns sorted list with file metadata.
/// Directories first, then files, both alphabetically.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub extension: String,
}

#[tauri::command]
pub fn read_dir_entries(dir_path: String) -> Result<Vec<DirEntry>, String> {
    let path = if dir_path.starts_with('~') {
        dir_path.replacen('~', &dirs_home(), 1)
    } else {
        dir_path
    };

    let dir = std::fs::read_dir(&path).map_err(|e| format!("Cannot read directory '{}': {}", path, e))?;

    let mut entries: Vec<DirEntry> = Vec::new();
    for entry in dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let extension = if is_dir {
            String::new()
        } else {
            std::path::Path::new(&file_name)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        entries.push(DirEntry {
            name: file_name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
            extension,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Oasis 🏜️", name)
}

#[derive(Serialize, Deserialize)]
pub struct TrayLocale {
    pub show: String,
    pub hide: String,
    pub about: String,
    pub quit: String,
}

#[tauri::command]
pub fn update_tray_locale(app: AppHandle, locale: TrayLocale) -> Result<(), String> {
    let ids_and_texts = [
        ("show", &locale.show),
        ("hide", &locale.hide),
        ("about", &locale.about),
        ("quit", &locale.quit),
    ];
    for (id, text) in &ids_and_texts {
        if let Some(menu) = app.menu() {
            if let Some(item) = menu.get(&tauri::menu::MenuId::new(*id)) {
                if let Some(mi) = item.as_menuitem() {
                    let _ = mi.set_text(text);
                }
            }
        }
    }
    Ok(())
}

// ─── 嵌入式 WebView 管理 ─────────────────────────────────────────

/// 创建嵌入主窗口的子 WebView（参照 crawler 的 wry WebViewBuilder::build_as_child 模式）
#[tauri::command]
pub fn create_embedded_webview(
    app: AppHandle,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<String, String> {
    use tauri::Manager;

    let window = app
        .get_window("main")
        .ok_or("main window not found")?;

    // 关闭已有的嵌入式 webview
    if let Some(existing) = app.webviews().get("safari-embedded") {
        existing.close().map_err(|e| e.to_string())?;
    }

    let parsed = url.parse().map_err(|e| format!("invalid url: {}", e))?;
    window
        .add_child(
            tauri::webview::WebviewBuilder::new("safari-embedded", tauri::WebviewUrl::External(parsed)),
            tauri::LogicalPosition::new(x, y),
            tauri::LogicalSize::new(width, height),
        )
        .map_err(|e| e.to_string())?;

    Ok("safari-embedded".to_string())
}

/// 关闭嵌入主窗口的子 WebView
#[tauri::command]
pub fn close_embedded_webview(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.webviews().get("safari-embedded") {
        webview.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 更新嵌入式 WebView 的位置和大小（窗口 resize 时调用）
#[tauri::command]
pub fn update_embedded_webview_bounds(
    app: AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(webview) = app.webviews().get("safari-embedded") {
        webview
            .set_position(tauri::LogicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
        webview
            .set_size(tauri::LogicalSize::new(width, height))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ─── 应用更新检查 ────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: Vec<ReleaseNoteSection>,
    pub published_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReleaseNoteSection {
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CheckUpdateResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub update_info: Option<UpdateInfo>,
}

fn get_current_version(app: &AppHandle) -> String {
    app.config()
        .version
        .clone()
        .unwrap_or_else(|| "0.1.0".to_string())
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };
    let cur = parse(current);
    let lat = parse(latest);
    for i in 0..lat.len().max(cur.len()) {
        let c = cur.get(i).copied().unwrap_or(0);
        let l = lat.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<CheckUpdateResult, String> {
    let current_version = get_current_version(&app);

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/repos/ht-shaipe/oasis/releases/latest")
        .header("User-Agent", "Oasis-App")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Ok(CheckUpdateResult {
            has_update: false,
            current_version: current_version.clone(),
            latest_version: current_version,
            update_info: None,
        });
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse response failed: {}", e))?;

    let latest_version = body["tag_name"]
        .as_str()
        .unwrap_or(&current_version)
        .trim_start_matches('v')
        .to_string();

    let has_update = compare_versions(&current_version, &latest_version);

    if !has_update {
        return Ok(CheckUpdateResult {
            has_update: false,
            current_version,
            latest_version,
            update_info: None,
        });
    }

    let download_url = body["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let name = a["name"].as_str().unwrap_or("");
                if name.ends_with(".dmg") {
                    a["browser_download_url"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            body["html_url"].as_str().map(|s| s.to_string())
        })
        .unwrap_or_default();

    let body_md = body["body"].as_str().unwrap_or("");

    let release_notes = parse_release_notes(body_md);

    let published_at = body["published_at"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(CheckUpdateResult {
        has_update: true,
        current_version,
        latest_version,
        update_info: Some(UpdateInfo {
            version: body["tag_name"].as_str().unwrap_or("").to_string(),
            download_url,
            release_notes,
            published_at,
        }),
    })
}

fn parse_release_notes(md: &str) -> Vec<ReleaseNoteSection> {
    let mut sections: Vec<ReleaseNoteSection> = Vec::new();
    let mut current_title = String::new();
    let mut current_items: Vec<String> = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### ") || trimmed.starts_with("## ") {
            if !current_title.is_empty() || !current_items.is_empty() {
                sections.push(ReleaseNoteSection {
                    title: current_title.clone(),
                    items: current_items.clone(),
                });
            }
            current_title = trimmed.trim_start_matches('#').trim().to_string();
            current_items.clear();
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            current_items.push(trimmed[2..].trim().to_string());
        } else if !trimmed.is_empty() && (current_title.is_empty() && sections.is_empty()) {
            current_title = "更新内容".to_string();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                current_items.push(trimmed[2..].trim().to_string());
            } else {
                current_items.push(trimmed.to_string());
            }
        } else if !trimmed.is_empty() {
            current_items.push(trimmed.to_string());
        }
    }

    if !current_title.is_empty() || !current_items.is_empty() {
        sections.push(ReleaseNoteSection {
            title: current_title,
            items: current_items,
        });
    }

    sections
}

#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    url: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Oasis-App")
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let file_name = url.split('/').last().unwrap_or("update.dmg");
    let download_dir = dirs_next::download_dir()

        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let file_path = download_dir.join(file_name);

    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| format!("Create file failed: {}", e))?;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Read chunk failed: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| format!("Write file failed: {}", e))?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            ((downloaded as f64 / total_size as f64) * 100.0) as u32
        } else {
            0
        };

        let _ = app.emit("update-download-progress", progress);
    }

    Ok(file_path.to_string_lossy().to_string())
}
