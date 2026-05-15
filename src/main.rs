#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{Application, App};

fn main() {
    #[cfg(target_os = "windows")]
    std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");

    Application::new()
        .with_assets(oasis::Assets)
        .run(move |cx: &mut App| {
            oasis::init(cx);

            // System tray
            if let Err(e) = oasis::system_tray::init_platform() {
                log::error!("Failed to init tray platform: {}", e);
            }
            match oasis::system_tray::SystemTray::new() {
                Ok(tray) => {
                    oasis::system_tray::setup_tray_event_handler(tray, cx);
                }
                Err(e) => log::error!("Failed to init system tray: {}", e),
            }

            cx.on_action(|_: &oasis::Quit, cx| {
                cx.quit();
            });

            oasis::open_new("oasis", |window, cx| oasis::Workspace::new(window, cx), cx);
        });
}
