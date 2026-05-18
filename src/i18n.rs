use gpui::App;

use crate::app::actions::SelectLocale;
use crate::app_menus;
use crate::app_state::AppSettings;

/// Initialize i18n
pub fn init(cx: &mut App) {
    let locale = AppSettings::global(cx).locale.clone();
    // 兼容旧版 locale 名：zh-CN → zh_cn
    let locale = match locale.as_ref() {
        "zh-CN" | "zh_cn" => "zh_cn",
        other => other,
    };
    rust_i18n::set_locale(locale);

    cx.on_action(|action: &SelectLocale, cx| {
        change_locale(action.locale.as_ref());
        AppSettings::global_mut(cx).locale = action.locale.clone();
        crate::app::themes::save_state(cx);
        app_menus::refresh(cx);
        cx.refresh_windows();
    });
}

pub fn change_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}