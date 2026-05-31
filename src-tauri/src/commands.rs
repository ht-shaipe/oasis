use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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
