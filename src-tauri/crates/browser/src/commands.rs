use crate::cdp_launcher;

#[tauri::command]
pub fn find_chrome_path() -> Result<String, String> {
    cdp_launcher::find_system_chrome()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "未找到系统 Chrome 或 Chromium".to_string())
}

#[tauri::command]
pub fn launch_chrome_cdp() -> Result<String, String> {
    // 在后台线程中启动
    cdp_launcher::spawn_cdp_open_chrome(None, None, None);
    Ok("Chrome 已在后台启动，CDP 端口: 9222".to_string())
}

#[tauri::command]
pub fn open_url_cdp(
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<String, String> {
    let normalized_url = if url.starts_with("http://") || url.starts_with("https://") {
        url
    } else {
        format!("https://{url}")
    };

    cdp_launcher::spawn_cdp_open_chrome(Some(normalized_url.clone()), username, password);
    Ok(format!("已请求通过 CDP 打开并尝试填充: {normalized_url}"))
}
