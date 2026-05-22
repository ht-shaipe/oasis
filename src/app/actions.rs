use gpui::actions;
use gpui::Action;
use gpui::SharedString;
use gpui_component::ThemeMode;

actions!(
    app,
    [
        /// Quit the application
        Quit,
        /// Open file picker to set the window background image
        SetBackground,
        /// Show about dialog
        About,
    ]
);

/// Switch theme mode (Light/Dark)
#[derive(Clone, PartialEq, Action)]
#[action(namespace = app, no_json)]
pub struct SwitchThemeMode {
    pub mode: ThemeMode,
}

/// Switch to a named theme
#[derive(Clone, PartialEq, Action)]
#[action(namespace = app, no_json)]
pub struct SwitchTheme {
    pub name: SharedString,
}

/// Select a locale
#[derive(Clone, PartialEq, Action)]
#[action(namespace = app, no_json)]
pub struct SelectLocale {
    pub locale: SharedString,
}