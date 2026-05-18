//! 接口请求工具：参照 Postman 式布局，方法+URL+发送、Params/Body/Headers 标签、返回响应区。

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _,
    Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    label::Label,
    scroll::ScrollableElement,
    v_flex,
};
use rust_i18n::t;

use super::super::ToolboxPanel;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTab {
    Params,
    Body,
    Headers,
}

pub struct ApiRequestState {
    pub method: HttpMethod,
    pub active_tab: RequestTab,
    pub url_input: Entity<gpui_component::input::InputState>,
    pub params_input: Entity<gpui_component::input::InputState>,
    pub headers_input: Entity<gpui_component::input::InputState>,
    pub body_input: Entity<gpui_component::input::InputState>,
    pub loading: bool,
    pub response_status: Option<u16>,
    pub response_body: String,
    pub response_error: Option<String>,
}

impl ApiRequestState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        let url_input = cx.new(|cx| {
            gpui_component::input::InputState::new(window, cx).placeholder("/api/...".to_string())
        });
        let params_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder(t!("toolbox.api.params_placeholder").to_string(), window, cx);
            s
        });
        let headers_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("Content-Type: application/json".to_string(), window, cx);
            s
        });
        let body_input = cx.new(|cx| gpui_component::input::InputState::new(window, cx));

        Self {
            method: HttpMethod::Get,
            active_tab: RequestTab::Params,
            url_input,
            params_input,
            headers_input,
            body_input,
            loading: false,
            response_status: None,
            response_body: String::new(),
            response_error: None,
        }
    }

    pub fn render(
        state: &mut ApiRequestState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let url_input = state.url_input.clone();
        let method_get = state.method == HttpMethod::Get;
        let method_post = state.method == HttpMethod::Post;
        let method_put = state.method == HttpMethod::Put;
        let method_del = state.method == HttpMethod::Delete;
        let method_patch = state.method == HttpMethod::Patch;
        let e_get = entity.clone();
        let e_post = entity.clone();
        let e_put = entity.clone();
        let e_del = entity.clone();
        let e_patch = entity.clone();

        let method_btns = h_flex().gap_1().child(
            gpui::div()
                .flex()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .child(
                    h_flex()
                        .child(
                            Button::new("m-get")
                                .label("GET")
                                .when(method_get, |b| b.primary())
                                .when(!method_get, |b| {
                                    b.outline().text_color(theme.muted_foreground)
                                })
                                .on_click(move |_, _, cx| {
                                    e_get.update(cx, |this, cx| {
                                        this.api_request.method = HttpMethod::Get;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            Button::new("m-post")
                                .label("POST")
                                .when(method_post, |b| b.primary())
                                .when(!method_post, |b| {
                                    b.outline().text_color(theme.muted_foreground)
                                })
                                .on_click(move |_, _, cx| {
                                    e_post.update(cx, |this, cx| {
                                        this.api_request.method = HttpMethod::Post;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            Button::new("m-put")
                                .label("PUT")
                                .when(method_put, |b| b.primary())
                                .when(!method_put, |b| {
                                    b.outline().text_color(theme.muted_foreground)
                                })
                                .on_click(move |_, _, cx| {
                                    e_put.update(cx, |this, cx| {
                                        this.api_request.method = HttpMethod::Put;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            Button::new("m-del")
                                .label("DELETE")
                                .when(method_del, |b| b.primary())
                                .when(!method_del, |b| {
                                    b.outline().text_color(theme.muted_foreground)
                                })
                                .on_click(move |_, _, cx| {
                                    e_del.update(cx, |this, cx| {
                                        this.api_request.method = HttpMethod::Delete;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            Button::new("m-patch")
                                .label("PATCH")
                                .when(method_patch, |b| b.primary())
                                .when(!method_patch, |b| {
                                    b.outline().text_color(theme.muted_foreground)
                                })
                                .on_click(move |_, _, cx| {
                                    e_patch.update(cx, |this, cx| {
                                        this.api_request.method = HttpMethod::Patch;
                                        cx.notify();
                                    });
                                }),
                        ),
                ),
        );

        let entity_send = entity.clone();
        let send_btn = Button::new("api-send")
            .label(t!("toolbox.api.send").to_string())
            .icon(Icon::new(IconName::ArrowRight).text_color(theme.primary))
            .primary()
            .disabled(state.loading)
            .on_click(move |_, _, cx| {
                entity_send.update(cx, |this, cx| this.execute_api_request(cx));
            });

        let top_bar = h_flex()
            .gap_2()
            .items_center()
            .w_full()
            .child(method_btns)
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(px(0.))
                    .child(Input::new(&url_input)),
            )
            .child(send_btn);

        let tab_params = state.active_tab == RequestTab::Params;
        let tab_body = state.active_tab == RequestTab::Body;
        let tab_headers = state.active_tab == RequestTab::Headers;
        let e_tp = entity.clone();
        let e_tb = entity.clone();
        let e_th = entity.clone();

        let tab_row = h_flex()
            .gap_0()
            .border_b_1()
            .border_color(theme.border)
            .child(
                gpui::div()
                    .px(px(12.))
                    .py(px(8.))
                    .border_b_2()
                    .border_color(if tab_params {
                        theme.primary
                    } else {
                        theme.border
                    })
                    .child(
                        Button::new("tab-params")
                            .label(t!("toolbox.api.tab_params").to_string())
                            .when(tab_params, |b| b.primary())
                            .when(!tab_params, |b| {
                                b.outline().text_color(theme.muted_foreground)
                            })
                            .on_click(move |_, _, cx| {
                                e_tp.update(cx, |this, cx| {
                                    this.api_request.active_tab = RequestTab::Params;
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .child(
                gpui::div()
                    .px(px(12.))
                    .py(px(8.))
                    .border_b_2()
                    .border_color(if tab_body {
                        theme.primary
                    } else {
                        theme.border
                    })
                    .child(
                        Button::new("tab-body")
                            .label(t!("toolbox.api.tab_body").to_string())
                            .when(tab_body, |b| b.primary())
                            .when(!tab_body, |b| {
                                b.outline().text_color(theme.muted_foreground)
                            })
                            .on_click(move |_, _, cx| {
                                e_tb.update(cx, |this, cx| {
                                    this.api_request.active_tab = RequestTab::Body;
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .child(
                gpui::div()
                    .px(px(12.))
                    .py(px(8.))
                    .border_b_2()
                    .border_color(if tab_headers {
                        theme.primary
                    } else {
                        theme.border
                    })
                    .child(
                        Button::new("tab-headers")
                            .label(t!("toolbox.api.tab_headers").to_string())
                            .when(tab_headers, |b| b.primary())
                            .when(!tab_headers, |b| {
                                b.outline().text_color(theme.muted_foreground)
                            })
                            .on_click(move |_, _, cx| {
                                e_th.update(cx, |this, cx| {
                                    this.api_request.active_tab = RequestTab::Headers;
                                    cx.notify();
                                });
                            }),
                    ),
            );

        let params_input = state.params_input.clone();
        let headers_input = state.headers_input.clone();
        let body_input = state.body_input.clone();
        let tab_content = match state.active_tab {
            RequestTab::Params => gpui::div()
                .flex_1()
                .min_h(px(100.))
                .p_3()
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .child(Input::new(&params_input).min_h(px(100.))),
            RequestTab::Body => gpui::div()
                .flex_1()
                .min_h(px(100.))
                .p_3()
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .child(Input::new(&body_input).min_h(px(100.))),
            RequestTab::Headers => gpui::div()
                .flex_1()
                .min_h(px(100.))
                .p_3()
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .child(Input::new(&headers_input).min_h(px(100.))),
        };

        let has_response = state.response_status.is_some()
            || !state.response_body.is_empty()
            || state.response_error.is_some();
        let status = state.response_status;
        let body_preview = state.response_body.clone();
        let err = state.response_error.clone();

        let status_label = if let Some(ref e) = err {
            e.clone()
        } else if let Some(s) = status {
            format!("{} ({})", t!("toolbox.api.success_status"), s)
        } else {
            String::new()
        };
        let body_display = if let Some(ref e) = err {
            e.clone()
        } else if body_preview.len() > 8000 {
            format!("{}...", &body_preview[..8000])
        } else {
            body_preview
        };

        let response_section = v_flex()
            .gap_2()
            .flex_1()
            .min_h(px(0.))
            .border_t_1()
            .border_color(theme.border)
            .pt_3()
            .child(
                Label::new(t!("toolbox.api.response_title").to_string())
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.foreground),
            )
            .when(has_response, |v| {
                let mono = theme.mono_font_family.clone();
                let mut body_lines = v_flex().gap_1().w_full();
                for line in body_display.lines() {
                    body_lines = body_lines.child(
                        Label::new(line.to_string())
                            .text_xs()
                            .text_color(theme.foreground)
                            .font_family(mono.clone()),
                    );
                }
                v.child(
                    Label::new(status_label)
                        .text_sm()
                        .text_color(if err.is_some() {
                            theme.danger
                        } else {
                            theme.green
                        }),
                )
                .child(
                    gpui::div()
                        .flex_1()
                        .min_h(px(120.))
                        .max_h(px(320.))
                        .overflow_y_scrollbar()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .p_3()
                        .text_sm()
                        .text_color(theme.foreground)
                        .child(body_lines),
                )
            })
            .when(!has_response, |v| {
                v.child(
                    gpui::div()
                        .flex_1()
                        .min_h(px(160.))
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.muted_foreground)
                        .child(
                            v_flex()
                                .gap_3()
                                .items_center()
                                .child(Icon::new(IconName::Globe).size_24())
                                .child(
                                    Label::new(t!("toolbox.api.response_placeholder").to_string())
                                        .text_sm(),
                                ),
                        ),
                )
            });

        v_flex()
            .size_full()
            .overflow_hidden()
            .gap_3()
            .child(top_bar)
            .child(tab_row)
            .child(tab_content)
            .child(response_section)
    }
}
