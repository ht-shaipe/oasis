use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::dock::Panel;
use gpui_component::scroll::ScrollableElement;
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use crate::app::app_state::AppState;
#[cfg(not(target_family = "wasm"))]
use crate::core::credential_manager::{Credential, CredentialService};

pub struct CredentialManagerPanel {
    focus_handle: FocusHandle,
    #[cfg(not(target_family = "wasm"))]
    service: Option<Arc<CredentialService>>,
    #[cfg(not(target_family = "wasm"))]
    credentials: Vec<Credential>,
    selected_credential: Option<usize>,
    show_add_form: bool,
    search_query: SharedString,
}

impl CredentialManagerPanel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Self {
        let mut panel = Self {
            focus_handle: cx.focus_handle(),
            #[cfg(not(target_family = "wasm"))]
            service: None,
            #[cfg(not(target_family = "wasm"))]
            credentials: Vec::new(),
            selected_credential: None,
            show_add_form: false,
            search_query: SharedString::new(""),
        };

        #[cfg(not(target_family = "wasm"))]
        {
            if let Some(service) = AppState::global(cx).credential_service() {
                panel.service = Some(service.clone());
                if let Ok(creds) = service.list_all() {
                    panel.credentials = creds;
                }
            }
        }

        panel
    }

    #[cfg(not(target_family = "wasm"))]
    fn refresh_credentials(&mut self, _cx: &mut Context<Self>) {
        if let Some(service) = &self.service {
            if let Ok(creds) = service.list_all() {
                self.credentials = creds;
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn add_credential(&mut self, cred: Credential, cx: &mut Context<Self>) {
        if let Some(service) = &self.service {
            match service.create(cred) {
                Ok(_) => {
                    self.refresh_credentials(cx);
                    self.show_add_form = false;
                }
                Err(e) => log::error!("Failed to add credential: {}", e),
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn delete_credential(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(service) = &self.service {
            match service.delete(id) {
                Ok(_) => {
                    self.refresh_credentials(cx);
                    self.selected_credential = None;
                }
                Err(e) => log::error!("Failed to delete credential: {}", e),
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn search_credentials(&mut self, query: &str, cx: &mut Context<Self>) {
        self.search_query = SharedString::from(query.to_string());
        if let Some(service) = &self.service {
            if query.is_empty() {
                if let Ok(creds) = service.list_all() {
                    self.credentials = creds;
                }
            } else {
                if let Ok(creds) = service.search(query) {
                    self.credentials = creds;
                }
            }
        }
        cx.notify();
    }
}

impl gpui::Focusable for CredentialManagerPanel {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_component::dock::Panel for CredentialManagerPanel {
    fn panel_name(&self) -> &'static str {
        "Credentials"
    }
}

impl gpui::EventEmitter<gpui_component::dock::PanelEvent> for CredentialManagerPanel {}

impl Render for CredentialManagerPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        div()
            .id("credential-manager-panel")
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .border_l_1()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .child("🔐 Credentials")
                    .child(
                        div().flex_1()
                    )
            )
            .child(
                if self.show_add_form {
                    div()
                        .id("add-credential-form")
                        .flex()
                        .flex_col()
                        .gap_2()
                        .px_4()
                        .py_3()
                        .border_b_1()
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .child("Search credentials...")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .children(
                        self.credentials.iter().enumerate().map(|(idx, cred)| {
                            let id = SharedString::from(format!("cred-{}", idx));
                            Button::new(id)
                                .ghost()
                                .w_full()
                                .on_click(move |_ev, _window, cx| {
                                    // This would need to be handled differently - emit a panel event instead
                                    log::info!("Credential {} clicked", idx);
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .px_4()
                                        .py_3()
                                        .border_b_1()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .child(
                                                    div()
                                                        .child(cred.name.clone())
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .child(format!("{} • {}", &cred.platform, &cred.username))
                                                )
                                        )
                                )
                                .into_any_element()
                        }).collect::<Vec<_>>()
                    )
            )
    }
}

