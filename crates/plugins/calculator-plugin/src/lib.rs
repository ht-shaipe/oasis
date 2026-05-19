//! 计算器挂件 — 独立 cdylib 插件
//!
//! 编译为 `libwidget_calculator.dylib`，宿主运行时动态加载。

use gpui::{
    div, px, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled, Window,
};
use gpui_component::ActiveTheme as _;
use plugin_sdk::{Widget, WidgetManifest};

// ---------------------------------------------------------------------------
// CalculatorWidget
// ---------------------------------------------------------------------------

pub struct CalculatorWidget {
    display: String,
    previous: Option<f64>,
    operator: Option<char>,
    new_input: bool,
}

impl CalculatorWidget {
    fn input_digit(&mut self, digit: &str) {
        if self.new_input {
            self.display = digit.to_string();
            self.new_input = false;
        } else if self.display == "0" {
            self.display = digit.to_string();
        } else {
            self.display.push_str(digit);
        }
    }

    fn input_operator(&mut self, op: char) {
        let current: f64 = self.display.parse().unwrap_or(0.0);
        if let Some(prev_op) = self.operator {
            if let Some(prev) = self.previous {
                let result = Self::calculate(prev, current, prev_op);
                self.display = format_result(result);
                self.previous = Some(result);
            }
        } else {
            self.previous = Some(current);
        }
        self.operator = Some(op);
        self.new_input = true;
    }

    fn calculate(a: f64, b: f64, op: char) -> f64 {
        match op {
            '+' => a + b,
            '-' | '−' => a - b,
            '×' | '*' => a * b,
            '÷' | '/' => {
                if b != 0.0 { a / b } else { 0.0 }
            }
            _ => b,
        }
    }

    fn equals(&mut self) {
        let current: f64 = self.display.parse().unwrap_or(0.0);
        if let Some(op) = self.operator {
            if let Some(prev) = self.previous {
                let result = Self::calculate(prev, current, op);
                self.display = format_result(result);
                self.previous = None;
                self.operator = None;
                self.new_input = true;
            }
        }
    }

    fn clear(&mut self) {
        self.display = "0".to_string();
        self.previous = None;
        self.operator = None;
        self.new_input = true;
    }
}

fn format_result(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.8}", value).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ---------------------------------------------------------------------------
// Widget trait 实现
// ---------------------------------------------------------------------------

impl Widget for CalculatorWidget {
    fn widget_id() -> &'static str {
        "calculator"
    }

    fn manifest() -> WidgetManifest {
        WidgetManifest {
            id: "calculator".into(),
            display_name: "计算器".into(),
            description: "一个简易计算器插件".into(),
            icon_emoji: "🔢".into(),
            icon_svg: include_str!("../icon.svg").into(),
            window_width: 320.0,
            window_height: 480.0,
        }
    }

    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            display: "0".to_string(),
            previous: None,
            operator: None,
            new_input: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum CalcBtnKind {
    Digit,
    Op,
    Clear,
    Equals,
}

impl Render for CalculatorWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.mode.is_dark();

        let display_fg = theme.colors.foreground;
        let btn_bg = if is_dark {
            theme.colors.muted.opacity(0.5)
        } else {
            theme.colors.muted.opacity(0.35)
        };
        let btn_fg = theme.colors.foreground;

        let buttons: Vec<(&str, usize, usize, CalcBtnKind)> = vec![
            ("7", 0, 0, CalcBtnKind::Digit),
            ("8", 0, 1, CalcBtnKind::Digit),
            ("9", 0, 2, CalcBtnKind::Digit),
            ("÷", 0, 3, CalcBtnKind::Op),
            ("4", 1, 0, CalcBtnKind::Digit),
            ("5", 1, 1, CalcBtnKind::Digit),
            ("6", 1, 2, CalcBtnKind::Digit),
            ("×", 1, 3, CalcBtnKind::Op),
            ("1", 2, 0, CalcBtnKind::Digit),
            ("2", 2, 1, CalcBtnKind::Digit),
            ("3", 2, 2, CalcBtnKind::Digit),
            ("−", 2, 3, CalcBtnKind::Op),
            ("C", 3, 0, CalcBtnKind::Clear),
            ("0", 3, 1, CalcBtnKind::Digit),
            ("=", 3, 2, CalcBtnKind::Equals),
            ("+", 3, 3, CalcBtnKind::Op),
        ];

        let entity = cx.entity().downgrade();
        let rows = buttons.chunks(4).map(|row| {
            let entity = entity.clone();
            div()
                .flex()
                .flex_row()
                .gap(px(6.))
                .children(row.iter().map(|(label, r, c, kind)| {
                    let entity = entity.clone();
                    let label_str = label.to_string();
                    let kind_clone = kind.clone();
                    let bg = match kind {
                        CalcBtnKind::Op => gpui::hsla(30.0 / 360.0, 0.9, 0.55, 1.0),
                        CalcBtnKind::Clear => gpui::red().opacity(0.7),
                        _ => btn_bg,
                    };
                    let fg = match kind {
                        CalcBtnKind::Op | CalcBtnKind::Clear => gpui::white(),
                        _ => btn_fg,
                    };
                    div()
                        .id(SharedString::from(format!("calc-btn-{}-{}", r, c)))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_1()
                        .h(px(40.))
                        .rounded_md()
                        .bg(bg)
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            if let Some(e) = entity.upgrade() {
                                e.update(cx, |view, _cx| {
                                    match &kind_clone {
                                        CalcBtnKind::Digit => view.input_digit(&label_str),
                                        CalcBtnKind::Op => {
                                            view.input_operator(label_str.chars().next().unwrap())
                                        }
                                        CalcBtnKind::Clear => view.clear(),
                                        CalcBtnKind::Equals => view.equals(),
                                    }
                                });
                            }
                        })
                        .child(
                            div()
                                .text_size(px(16.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(fg)
                                .child(label.to_string()),
                        )
                }))
        });

        div()
            .flex()
            .flex_col()
            .h_full()
            .p(px(10.))
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .items_end()
                    .justify_end()
                    .px(px(12.))
                    .py(px(10.))
                    .rounded_md()
                    .bg(if is_dark {
                        theme.colors.muted.opacity(0.3)
                    } else {
                        theme.colors.muted.opacity(0.15)
                    })
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(display_fg)
                            .child(self.display.clone()),
                    ),
            )
            .children(rows)
    }
}

// ---------------------------------------------------------------------------
// FFI 导出
// ---------------------------------------------------------------------------


/// 导出清单 JSON — 宿主通过 libloading 读取此符号获取清单
#[unsafe(no_mangle)]
pub extern "C" fn widget_manifest_json() -> *const std::ffi::c_char {
    static MANIFEST_JSON: &str = r#"{"id":"calculator","display_name":"计算器","description":"一个简易计算器插件","icon_emoji":"🔢","icon_svg":"","window_width":320.0,"window_height":480.0}"#;
    MANIFEST_JSON.as_ptr() as *const std::ffi::c_char
}

/// 实际的 widget 创建函数（在调用端被调用）
#[unsafe(no_mangle)]
pub extern "C" fn widget_create_impl(
    _window: *mut gpui::Window,
    app: *mut gpui::App,
) -> gpui::AnyView {
    unsafe {
        let cx = &mut *app;
        // 关键：Entity 创建在调用端的上下文中完成
        cx.new(|cx| CalculatorWidget::new(cx)).into()
    }
}

/// Factory 函数：返回创建函数的指针
#[unsafe(no_mangle)]
pub extern "C" fn widget_factory() -> plugin_sdk::WidgetCreateFn {
    widget_create_impl
}
