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
    cdp_launcher::spawn_cdp_open_chrome();
    Ok("Chrome 已在后台启动，CDP 端口: 9222".to_string())
}
