//! UI DSL 渲染器
//!
//! 将 Widget 描述转换为 GPUI 组件，支持事件处理

use gpui::{div, px, IntoElement, ParentElement as _, Styled as _};

use super::widgets::{Align, BackgroundStyle, ButtonStyle, PluginState, Widget, Widget:: *};

/// 解析颜色字符串
pub fn parse_color(color: &str) -> u32 {
    let color = color.trim_start_matches('#');
    match color.len() {
        6 => u32::from_str_radix(color, 16).unwrap_or(0x000000),
        3 => {
            let r = u32::from_str_radix(&color[0..1], 16).unwrap_or(0);
            let g = u32::from_str_radix(&color[1..2], 16).unwrap_or(0);
            let b = u32::from_str_radix(&color[2..3], 16).unwrap_or(0);
            (r * 16 + r) << 16 | (g * 16 + g) << 8 | (b * 16 + b)
        }
        _ => 0x000000,
    }
}

/// 将颜色 u32 转换为 gpui::Hsla
pub fn rgb_to_hsla(rgb: u32) -> gpui::Hsla {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return gpui::hsla(0.0, 0.0, l, 1.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    gpui::hsla(h, s, l, 1.0)
}

/// 按钮样式构建器
fn build_button_style<'a>(
    mut button: gpui::Div,
    style: &'a Option<ButtonStyle>,
    theme: &'a gpui_component::Theme,
) -> gpui::Div {
    match style {
        Some(s) => {
            if let Some(bg) = &s.bg {
                button = button.bg(rgb_to_hsla(parse_color(bg)));
            }
            if let Some(color) = &s.color {
                button = button.text_color(rgb_to_hsla(parse_color(color)));
            }
            if let Some(size) = s.size {
                button = button.size(px(size));
            }
            if s.rounded.unwrap_or(true) {
                button = button.rounded_lg();
            }
            button
        }
        None => button.bg(theme.colors.muted).rounded_lg(),
    }
}

/// 渲染单个 Widget（所有逻辑内联，确保返回类型一致）
pub fn render_widget(w: &Widget, theme: &gpui_component::Theme) -> impl IntoElement {
    match w {
        Column {
            gap,
            align,
            padding,
            bg,
            rounded: _,
            children,
        } => {
            let mut column = div().flex().flex_col().gap(px(*gap));

            match align {
                Align::Start => column = column.items_start(),
                Align::Center => column = column.items_center(),
                Align::End => column = column.items_end(),
            }

            if let Some(p) = padding {
                column = column.p(px(*p));
            }

            if let Some(bg_style) = bg {
                if let Some(color) = &bg_style.color {
                    let rgb = parse_color(color);
                    column = column.bg(rgb_to_hsla(rgb));
                }
            }

            for child in children {
                column = column.child(render_widget(child, theme));
            }

            column
        }
        Row {
            gap,
            align,
            padding,
            bg,
            rounded: _,
            children,
        } => {
            let mut row = div().flex().flex_row().gap(px(*gap));

            match align {
                Align::Start => row = row.justify_start(),
                Align::Center => row = row.justify_center(),
                Align::End => row = row.justify_end(),
            }

            if let Some(p) = padding {
                row = row.p(px(*p));
            }

            if let Some(bg_style) = bg {
                if let Some(color) = &bg_style.color {
                    let rgb = parse_color(color);
                    row = row.bg(rgb_to_hsla(rgb));
                }
            }

            for child in children {
                row = row.child(render_widget(child, theme));
            }

            row
        }
        Text { value, style } => {
            let mut text = div().child(value.to_string());

            if let Some(s) = style {
                if let Some(size) = s.size {
                    text = text.text_size(px(size));
                }
                if s.bold.unwrap_or(false) {
                    text = text.font_weight(gpui::FontWeight::BOLD);
                }
                if let Some(color) = &s.color {
                    text = text.text_color(rgb_to_hsla(parse_color(color)));
                }
            }

            text
        }
        Button {
            label,
            action: _,
            style,
        } => {
            let button = build_button_style(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(64.0))
                    .text_size(px(24.0))
                    .text_color(theme.colors.foreground),
                style,
                theme,
            );

            button.child(label.to_string())
        }
        Icon { value, size } => {
            let size = size.unwrap_or(24.0);
            div().text_size(px(size)).child(value.to_string())
        }
        Spacer { size } => div().h(px(*size)),
        Progress {
            value,
            max,
            height,
            bg_color,
            fill_color,
        } => {
            let height = height.unwrap_or(12.0);
            let percentage = if *max > 0.0 {
                (value / max * 100.0).min(100.0).max(0.0) as f32
            } else {
                0.0
            };
            let bar_width = (240.0 * percentage / 100.0).max(0.0);

            div()
                .flex()
                .flex_row()
                .items_center()
                .w(px(240.0))
                .h(px(height))
                .bg(bg_color
                    .as_ref()
                    .map(|c| rgb_to_hsla(parse_color(c)))
                    .unwrap_or_else(|| theme.colors.muted))
                .rounded_full()
                .child(
                    div()
                        .h(px(height))
                        .bg(fill_color
                            .as_ref()
                            .map(|c| rgb_to_hsla(parse_color(c)))
                            .unwrap_or(theme.colors.primary))
                        .rounded_full()
                        .flex_shrink_0()
                        .w(px(bar_width)),
                )
        }
        Divider { color, thickness } => {
            let thickness = thickness.unwrap_or(1.0);
            let color = color
                .as_ref()
                .map(|c| rgb_to_hsla(parse_color(c)))
                .unwrap_or(rgb_to_hsla(0x333333));
            div().h(px(thickness)).bg(color).w_full()
        }
        Input {
            value,
            placeholder,
            style,
        } => {
            let width = style
                .as_ref()
                .and_then(|s| s.width)
                .unwrap_or(200.0);
            let height = style
                .as_ref()
                .and_then(|s| s.height)
                .unwrap_or(36.0);
            let border_color = style
                .as_ref()
                .and_then(|s| s.border_color.as_ref())
                .map(|c| rgb_to_hsla(parse_color(c)))
                .unwrap_or(theme.colors.border);
            let bg_color = style
                .as_ref()
                .and_then(|s| s.bg_color.as_ref())
                .map(|c| rgb_to_hsla(parse_color(c)))
                .unwrap_or(theme.colors.input);

            div()
                .w(px(width))
                .h(px(height))
                .border(px(1.0))
                .border_color(border_color)
                .bg(bg_color)
                .rounded_md()
                .px(px(8.))
                .text_size(px(14.))
                .flex()
                .items_center()
                .child(if value.is_empty() {
                    placeholder.clone().unwrap_or_default()
                } else {
                    value.clone()
                })
        }
        Image {
            src,
            width,
            height,
            rounded,
        } => {
            let w = width.unwrap_or(100.0);
            let h = height.unwrap_or(100.0);
            let r = rounded.unwrap_or(0.0);

            // 简单实现：显示图片路径作为文本
            // 完整实现需要加载图片资源
            div()
                .w(px(w))
                .h(px(h))
                .rounded_lg()
                .bg(theme.colors.muted)
                .flex()
                .items_center()
                .justify_center()
                .child(format!("🖼 {}", src))
        }
        Badge {
            label,
            color,
            text_color,
            style,
        } => {
            let bg = color
                .as_ref()
                .map(|c| rgb_to_hsla(parse_color(c)))
                .unwrap_or(theme.colors.primary);
            let fg = text_color
                .as_ref()
                .map(|c| rgb_to_hsla(parse_color(c)))
                .unwrap_or(gpui::hsla(0.0, 0.0, 1.0, 1.0));
            let h = style
                .as_ref()
                .and_then(|s| s.padding_v)
                .unwrap_or(4.0);
            let ph = style
                .as_ref()
                .and_then(|s| s.padding_h)
                .unwrap_or(8.0);
            let r = style
                .as_ref()
                .and_then(|s| s.rounded)
                .unwrap_or(9999.0);

            div()
                .px(px(ph))
                .py(px(h))
                .bg(bg)
                .rounded_full()
                .text_size(px(12.0))
                .text_color(fg)
                .flex()
                .items_center()
                .justify_center()
                .child(label.clone())
        }
        Toggle {
            checked,
            action: _,
            label,
        } => {
            let toggle_width: f32 = 48.0;
            let toggle_height: f32 = 24.0;
            let knob_size: f32 = 20.0;
            let knob_offset = (toggle_width - knob_size - 4.0).max(0.0);

            let mut toggle_div = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .w(px(toggle_width))
                        .h(px(toggle_height))
                        .bg(if *checked {
                            theme.colors.primary
                        } else {
                            theme.colors.muted
                        })
                        .rounded_full()
                        .relative()
                        .child(
                            div()
                                .absolute()
                                .top(px(2.0))
                                .w(px(knob_size))
                                .h(px(knob_size))
                                .bg(gpui::white())
                                .rounded_full()
                                .left(if *checked { px(knob_offset) } else { px(2.0) }),
                        ),
                );

            if let Some(label_text) = label {
                toggle_div = toggle_div.child(
                    div().text_size(px(14.0)).child(label_text.clone())
                );
            }

            toggle_div
        }
        Slider {
            value,
            min,
            max,
            step: _,
            action: _,
            show_value,
        } => {
            let range = max - min;
            let percentage = if range > 0.0 {
                ((value - min) / range * 100.0).clamp(0.0, 100.0) as f32
            } else {
                0.0
            };
            let track_width: f32 = 200.0;
            let fill_width = track_width * percentage / 100.0;

            let mut slider_div = div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .w(px(track_width))
                        .h(px(6.0))
                        .bg(theme.colors.muted)
                        .rounded_full()
                        .relative()
                        .child(
                            div()
                                .absolute()
                                .top(px(0.0))
                                .left(px(0.0))
                                .h(px(6.0))
                                .w(px(fill_width))
                                .bg(theme.colors.primary)
                                .rounded_full(),
                        ),
                );

            if show_value.unwrap_or(false) {
                slider_div = slider_div.child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.colors.muted_foreground)
                        .child(format!("{:.1}", value))
                );
            }

            slider_div
        }
    }
}

/// 渲染 Widget 列表
pub fn render_widgets<'a>(
    widgets: &'a [Widget],
    theme: &'a gpui_component::Theme,
) -> Vec<impl IntoElement + 'a> {
    widgets.iter().map(|w| render_widget(w, theme)).collect()
}

/// DSL 渲染上下文
pub struct RenderContext<'a> {
    pub theme: &'a gpui_component::Theme,
    pub state: &'a PluginState,
}

impl<'a> RenderContext<'a> {
    pub fn new(theme: &'a gpui_component::Theme, state: &'a PluginState) -> Self {
        Self { theme, state }
    }

    /// 渲染单个 Widget 并格式化文本
    pub fn render(&self, widget: &Widget) -> impl IntoElement {
        render_widget(widget, self.theme)
    }

    /// 格式化带插值的文本
    pub fn format(&self, text: &str) -> String {
        self.state.format_value(text)
    }
}
