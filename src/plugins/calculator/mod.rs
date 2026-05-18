use gpui::{
    div, px, AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::ActiveTheme as _;

use crate::plugins::{Plugin, PluginEntry};

// ---------------------------------------------------------------------------
// CalculatorView
// ---------------------------------------------------------------------------

/// 简易计算器视图
pub struct CalculatorView {
    /// 当前显示内容
    display: String,
    /// 上一个操作数
    previous: Option<f64>,
    /// 当前运算符
    operator: Option<char>,
    /// 是否开始新输入
    new_input: bool,
}

impl CalculatorView {
    /// 输入数字
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

    /// 输入运算符
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

    /// 执行计算
    fn calculate(a: f64, b: f64, op: char) -> f64 {
        match op {
            '+' => a + b,
            '-' | '−' => a - b,
            '×' | '*' => a * b,
            '÷' | '/' => {
                if b != 0.0 {
                    a / b
                } else {
                    0.0
                }
            }
            _ => b,
        }
    }

    /// 等号
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

    /// 清除
    fn clear(&mut self) {
        self.display = "0".to_string();
        self.previous = None;
        self.operator = None;
        self.new_input = true;
    }
}

/// 格式化计算结果，去掉不必要的小数位
fn format_result(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.8}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

impl Plugin for CalculatorView {
    fn plugin_id() -> &'static str {
        "calculator"
    }

    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            display: "0".to_string(),
            previous: None,
            operator: None,
            new_input: true,
        }
    }
}

/// 按钮类型
#[derive(Clone)]
enum CalcBtnKind {
    Digit,
    Op,
    Clear,
    Equals,
}

impl Render for CalculatorView {
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

        // 按钮定义: (label, row_index, col_index, kind)
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
            // 显示屏
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
            // 按键区
            .children(rows)
    }
}

// ---------------------------------------------------------------------------
// inventory 提交
// ---------------------------------------------------------------------------

/// 创建 CalculatorView 并转为 AnyView
fn create_calculator_view(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| CalculatorView::new(window, cx)).into()
}

inventory::submit! {
    PluginEntry {
        id: "calculator",
        manifest_toml: include_str!("manifest.toml"),
        icon_svg: include_str!("icon.svg"),
        create_view: create_calculator_view,
    }
}
