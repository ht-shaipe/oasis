mod credential;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            update_tray_locale,
            credential::commands::is_master_key_set,
            credential::commands::setup_master_key,
            credential::commands::verify_master_key,
            credential::commands::list_categories,
            credential::commands::create_category,
            credential::commands::delete_category,
            credential::commands::list_credentials,
            credential::commands::get_credential,
            credential::commands::create_credential,
            credential::commands::update_credential,
            credential::commands::delete_credential,
            credential::commands::change_master_key,
        ])
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide Window").build(app)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let about = MenuItemBuilder::with_id("about", "About Oasis").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show, &hide, &sep, &about, &quit])
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Oasis")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "about" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                    let _ = w.emit("tray-action", "about");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Oasis 🏜️", name)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TrayLocale {
    show: String,
    hide: String,
    about: String,
    quit: String,
}

#[tauri::command]
fn update_tray_locale(app: tauri::AppHandle, locale: TrayLocale) -> Result<(), String> {
    // 通过 ID 查找菜单项并更新文字
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
