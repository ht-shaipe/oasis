use gpui::{
    div, prelude::FluentBuilder, App, Context, Entity, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Render, SharedString, Styled as _,
    Window, px,
};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    menu::AppMenuBar,
    TitleBar,
};
use rust_i18n::t;

use crate::app_state::AppState;
use crate::app_menus;

/// App title bar
pub struct AppTitleBar {
    app_menu_bar: Entity<AppMenuBar>,
}

impl AppTitleBar {
    pub fn new(
        title: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title = title.into();
        app_menus::init(title.clone(), cx);
        AppState::global_mut(cx).set_app_title(title);

        let app_menu_bar = AppMenuBar::new(window, cx);

        Self { app_menu_bar }
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();
        let now = chrono::Local::now();
        let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        TitleBar::new()
            .bg(cx.theme().transparent)
            .border_color(gpui::rgba(0x00000000))
            .child(
                div()
                    .flex()
                    .items_center()
                    .when(
                        !cfg!(any(target_os = "macos", target_family = "wasm")),
                        |this| this.child(self.app_menu_bar.clone()),
                    )
                    .when(cfg!(target_os = "macos"), |this| {
                        this.flex_1().justify_center().child(
                            div()
                                .text_size(px(13.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors.muted_foreground)
                                .child(t!("app.title").to_string()),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_2()
                    .gap(px(10.))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    // Date and time
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.colors.muted_foreground)
                            .child(datetime_str),
                    )
                    // Theme toggle
                    .child(
                        Button::new("theme-toggle")
                            .small()
                            .ghost()
                            .icon(if is_dark {
                                IconName::Sun
                            } else {
                                IconName::Moon
                            })
                            .on_click(move |_ev, _window: &mut Window, cx: &mut App| {
                                let mode = if is_dark {
                                    ThemeMode::Light
                                } else {
                                    ThemeMode::Dark
                                };
                                Theme::change(mode, None, cx);
                                crate::app::themes::save_state(cx);
                            }),
                    ),
            )
    }
}