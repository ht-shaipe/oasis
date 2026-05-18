#![allow(dead_code)]
use gpui::prelude::*;
use gpui::*;
use gpui::InteractiveElement;
use gpui_component::{Sizable, WindowExt, scroll::ScrollableElement, StyledExt};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputState};
use gpui_component::ActiveTheme;
use rust_i18n::t;
use std::sync::Arc;
use std::collections::HashSet;

#[cfg(not(target_family = "wasm"))]
use crate::app::app_state::AppState;
#[cfg(not(target_family = "wasm"))]
use crate::core::credential_manager::{Credential, CredentialService};

#[derive(Debug, Clone)]
struct TreeNode {
    name: String,
    full_path: String,
    children: Vec<TreeNode>,
    credential_count: usize,
}

impl TreeNode {
    fn new(name: String, full_path: String) -> Self {
        Self {
            name,
            full_path,
            children: Vec::new(),
            credential_count: 0,
        }
    }

    fn add_credential(&mut self, path_parts: &[&str], increment: bool) {
        if increment {
            self.credential_count += 1;
        }
        if path_parts.is_empty() {
            return;
        }
        let child_name = path_parts[0].to_string();
        let child_full_path = if self.full_path.is_empty() {
            child_name.clone()
        } else {
            format!("{}/{}", self.full_path, child_name)
        };

        let child = self.children
            .iter_mut()
            .find(|c| c.name == child_name);

        match child {
            Some(c) => c.add_credential(&path_parts[1..], false),
            None => {
                let mut new_child = TreeNode::new(child_name, child_full_path);
                new_child.add_credential(&path_parts[1..], false);
                self.children.push(new_child);
            }
        }
    }
}

pub struct CredentialManagerPanel {
    focus_handle: FocusHandle,
    #[cfg(not(target_family = "wasm"))]
    service: Option<Arc<CredentialService>>,
    #[cfg(not(target_family = "wasm"))]
    credentials: Vec<Credential>,
    #[cfg(not(target_family = "wasm"))]
    categories: Vec<String>,
    #[cfg(not(target_family = "wasm"))]
    selected_category: Option<String>,
    #[cfg(not(target_family = "wasm"))]
    category_tree: TreeNode,
    #[cfg(not(target_family = "wasm"))]
    expanded_nodes: HashSet<String>,
    selected_credential: Option<usize>,
    show_add_dialog: bool,
    form_name: Entity<InputState>,
    form_platform: Entity<InputState>,
    form_category: Entity<InputState>,
    form_username: Entity<InputState>,
    form_password: Entity<InputState>,
    form_notes: Entity<InputState>,
}

impl CredentialManagerPanel {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        let mk = |placeholder: &'static str, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                let mut s = InputState::new(window, cx);
                s.set_placeholder(SharedString::from(placeholder), window, cx);
                s
            })
        };

        let mut panel = Self {
            focus_handle: cx.focus_handle(),
            #[cfg(not(target_family = "wasm"))]
            service: None,
            #[cfg(not(target_family = "wasm"))]
            credentials: Vec::new(),
            #[cfg(not(target_family = "wasm"))]
            categories: Vec::new(),
            #[cfg(not(target_family = "wasm"))]
            selected_category: None,
            #[cfg(not(target_family = "wasm"))]
            category_tree: TreeNode::new("Root".to_string(), String::new()),
            #[cfg(not(target_family = "wasm"))]
            expanded_nodes: HashSet::new(),
            selected_credential: None,
            show_add_dialog: false,
            form_name: mk("Name / 名称", window, cx),
            form_platform: mk("Platform / 平台", window, cx),
            form_category: mk("Category / 分类", window, cx),
            form_username: mk("Username / 用户名", window, cx),
            form_password: mk("Password / 密码", window, cx),
            form_notes: mk("Notes / 备注", window, cx),
        };

        #[cfg(not(target_family = "wasm"))]
        {
            if let Some(service) = AppState::global(cx).credential_service() {
                panel.service = Some(service.clone());
                if let Ok(creds) = service.list_all() {
                    panel.credentials = creds;
                    panel.extract_categories();
                }
            }
        }

        panel
    }

    #[cfg(not(target_family = "wasm"))]
    fn extract_categories(&mut self) {
        let mut categories_set = HashSet::new();
        for cred in &self.credentials {
            if !cred.category.is_empty() {
                categories_set.insert(cred.category.clone());
            }
        }
        let mut categories: Vec<String> = categories_set.into_iter().collect();
        categories.sort();
        self.categories = categories;
    }

    #[cfg(not(target_family = "wasm"))]
    fn get_filtered_credentials(&self) -> Vec<Credential> {
        if let Some(ref selected_cat) = self.selected_category {
            self.credentials
                .iter()
                .filter(|c| &c.category == selected_cat)
                .cloned()
                .collect()
        } else {
            self.credentials.clone()
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn refresh_credentials(&mut self, _cx: &mut Context<Self>) {
        if let Some(service) = &self.service {
            if let Ok(creds) = service.list_all() {
                self.credentials = creds;
                self.extract_categories();
                self.build_category_tree();
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn build_category_tree(&mut self) {
        self.category_tree = TreeNode::new("Root".to_string(), String::new());
        for cred in &self.credentials {
            if !cred.category.is_empty() {
                let path_parts: Vec<&str> = cred.category.split('/').collect();
                self.category_tree.add_credential(&path_parts, true);
            }
        }
        sort_node(&mut self.category_tree);
    }

    #[cfg(not(target_family = "wasm"))]
    fn toggle_node_expanded(&mut self, path: &str) {
        if self.expanded_nodes.contains(path) {
            self.expanded_nodes.remove(path);
        } else {
            self.expanded_nodes.insert(path.to_string());
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn sort_node(node: &mut TreeNode) {
    node.children.sort_by(|a, b| a.name.cmp(&b.name));
    for child in &mut node.children {
        sort_node(child);
    }
}

impl CredentialManagerPanel {
    #[cfg(not(target_family = "wasm"))]
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.form_name.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
        self.form_platform.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
        self.form_category.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
        self.form_username.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
        self.form_password.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
        self.form_notes.update(cx, |s, cx| {
            s.set_value(SharedString::default(), window, cx);
        });
    }

    #[cfg(not(target_family = "wasm"))]
    fn save_new_credential(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.form_name.read(cx).value().to_string();
        let username = self.form_username.read(cx).value().to_string();
        let password = self.form_password.read(cx).value().to_string();
        if name.is_empty() || username.is_empty() || password.is_empty() {
            return;
        }
        let cred = Credential::new(
            name,
            self.form_platform.read(cx).value().to_string(),
            self.form_category.read(cx).value().to_string(),
            username,
            password,
        );
        self.add_credential(cred, window, cx);
    }

    #[cfg(not(target_family = "wasm"))]
    fn add_credential(&mut self, cred: Credential, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(service) = &self.service {
            match service.create(cred) {
                Ok(_) => {
                    self.refresh_credentials(cx);
                    self.show_add_dialog = false;
                    self.reset_form(window, cx);
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
}

impl Focusable for CredentialManagerPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_component::dock::Panel for CredentialManagerPanel {
    fn panel_name(&self) -> &'static str {
        "Credentials"
    }
}

impl EventEmitter<gpui_component::dock::PanelEvent> for CredentialManagerPanel {}

impl Render for CredentialManagerPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("credential-manager-panel")
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .overflow_hidden()
            .child(self.render_categories_sidebar(cx))
            .child(self.render_main_content(_window, cx))
    }
}

impl CredentialManagerPanel {
    fn render_categories_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_category = self.selected_category.clone();
        let expanded_nodes = self.expanded_nodes.clone();
        let category_tree = &self.category_tree;

        let panel = cx.entity().clone();
        let all_element = {
            let theme = cx.theme();
            let accent = theme.accent;
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_4()
                .py_2()
                .cursor_pointer()
                .bg(if selected_category.is_none() {
                    accent.opacity(0.1)
                } else {
                    gpui::transparent_black()
                })
                .on_mouse_down(MouseButton::Left, {
                    let panel = panel.clone();
                    move |_event, _window: &mut Window, cx: &mut App| {
                        panel.update(cx, |this, _cx| {
                            this.selected_category = None;
                            this.selected_credential = None;
                        });
                        cx.stop_propagation();
                    }
                })
                .child("All / 全部")
                .into_any_element()
        };

        let tree_elements = Self::render_tree_nodes_static(
            category_tree,
            0,
            selected_category,
            expanded_nodes,
            cx
        );

        let theme = cx.theme();
        div()
            .id("categories-sidebar")
            .flex()
            .flex_col()
            .w(px(250.))
            .h_full()
            .border_r_1()
            .bg(theme.background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child("📁 Categories / 分类"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .children(std::iter::once(all_element).chain(tree_elements))
                    )
            )
    }

    #[cfg(not(target_family = "wasm"))]
    fn render_tree_nodes_static(
        node: &TreeNode,
        depth: usize,
        selected_category: Option<String>,
        expanded_nodes: HashSet<String>,
        cx: &mut Context<Self>
    ) -> Vec<AnyElement> {
        let mut elements = Vec::new();
        let theme = cx.theme();
        let accent = theme.accent;
        let foreground = theme.foreground;
        let muted_foreground = theme.muted_foreground;

        for child in &node.children {
            let is_expanded = expanded_nodes.contains(&child.full_path);
            let is_selected = selected_category.as_ref() == Some(&child.full_path);
            let has_children = !child.children.is_empty();
            let panel = cx.entity().clone();
            let child_path = child.full_path.clone();
            let child_name = child.name.clone();
            let child_count = child.credential_count;
            let depth = depth;
            let accent = accent;
            let foreground = foreground;
            let muted_foreground = muted_foreground;

            let node_element = div()
                .flex()
                .items_center()
                .gap_1()
                .px(px(12. + depth as f32 * 16.))
                .py_2()
                .cursor_pointer()
                .bg(if is_selected {
                    accent.opacity(0.1)
                } else {
                    gpui::transparent_black()
                })
                .on_mouse_down(MouseButton::Left, {
                    let panel = panel.clone();
                    let child_path = child_path.clone();
                    move |_event, _window: &mut Window, cx: &mut App| {
                        panel.update(cx, |this, _cx| {
                            if has_children {
                                this.toggle_node_expanded(&child_path);
                            }
                            this.selected_category = Some(child_path.clone());
                            this.selected_credential = None;
                        });
                        cx.stop_propagation();
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .flex_1()
                        .children([
                            if has_children {
                                div()
                                    .text_color(muted_foreground)
                                    .child(if is_expanded { "▼" } else { "▶" })
                                    .into_any_element()
                            } else {
                                div().w(px(12.)).into_any_element()
                            },
                            div()
                                .flex_1()
                                .text_color(foreground)
                                .child(child_name.clone())
                                .into_any_element(),
                            if child_count > 0 {
                                div()
                                    .text_sm()
                                    .text_color(muted_foreground)
                                    .child(format!("({})", child_count))
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }
                        ])
                )
                .into_any_element();

            elements.push(node_element);

            if is_expanded && has_children {
                elements.extend(Self::render_tree_nodes_static(
                    child,
                    depth + 1,
                    selected_category.clone(),
                    expanded_nodes.clone(),
                    cx
                ));
            }
        }

        elements
    }

    #[cfg(not(target_family = "wasm"))]
    fn render_tree_nodes(
        &self,
        node: &TreeNode,
        depth: usize,
        selected_category: Option<String>,
        expanded_nodes: HashSet<String>,
        theme: &gpui_component::Theme,
        cx: &mut Context<Self>
    ) -> Vec<AnyElement> {
        let mut elements = Vec::new();

        for child in &node.children {
            let is_expanded = expanded_nodes.contains(&child.full_path);
            let is_selected = selected_category.as_ref() == Some(&child.full_path);
            let has_children = !child.children.is_empty();
            let panel = cx.entity().clone();
            let child_path = child.full_path.clone();
            let child_name = child.name.clone();
            let child_count = child.credential_count;
            let depth = depth;

            let node_element = div()
                .flex()
                .items_center()
                .gap_1()
                .px(px(12. + depth as f32 * 16.))
                .py_2()
                .cursor_pointer()
                .bg(if is_selected {
                    theme.accent.opacity(0.1)
                } else {
                    gpui::transparent_black()
                })
                .on_mouse_down(MouseButton::Left, {
                    let panel = panel.clone();
                    let child_path = child_path.clone();
                    move |_event, _window: &mut Window, cx: &mut App| {
                        panel.update(cx, |this, _cx| {
                            if has_children {
                                this.toggle_node_expanded(&child_path);
                            }
                            this.selected_category = Some(child_path.clone());
                            this.selected_credential = None;
                        });
                        cx.stop_propagation();
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .flex_1()
                        .children([
                            if has_children {
                                div()
                                    .text_color(theme.muted_foreground)
                                    .child(if is_expanded { "▼" } else { "▶" })
                                    .into_any_element()
                            } else {
                                div().w(px(12.)).into_any_element()
                            },
                            div()
                                .flex_1()
                                .text_color(theme.foreground)
                                .child(child_name.clone())
                                .into_any_element(),
                            if child_count > 0 {
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("({})", child_count))
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }
                        ])
                )
                .into_any_element();

            elements.push(node_element);

            if is_expanded && has_children {
                elements.extend(self.render_tree_nodes(
                    child,
                    depth + 1,
                    selected_category.clone(),
                    expanded_nodes.clone(),
                    theme,
                    cx
                ));
            }
        }

        elements
    }

    fn render_main_content(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let filtered_creds = self.get_filtered_credentials();
        let selected = self.selected_credential;

        div()
            .id("credential-main-content")
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme.background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("🔐 Credentials / 凭证")
                            .child(if let Some(ref cat) = self.selected_category {
                                cat.clone()
                            } else {
                                t!("credential.all").to_string()
                            })
                    )
                    .child(
                        Button::new("add-credential-btn")
                            .small()
                            .label("+ Add / 添加")
                            .on_click({
                                let panel = cx.entity().clone();
                                move |_ev, window: &mut Window, cx: &mut App| {
                                    // 获取表单引用
                                    let this_ref = panel.read(cx);
                                    let form_name = this_ref.form_name.clone();
                                    let form_platform = this_ref.form_platform.clone();
                                    let form_category = this_ref.form_category.clone();
                                    let form_username = this_ref.form_username.clone();
                                    let form_password = this_ref.form_password.clone();
                                    let form_notes = this_ref.form_notes.clone();
                                    let panel_clone = panel.clone();

                                    panel.update(cx, |this, _cx| {
                                        this.show_add_dialog = true;
                                    });

                                    window.open_dialog(cx, move |dialog, _window, cx| {
                                        let theme = cx.theme();
                                        let panel_clone_for_save = panel_clone.clone();
                                        dialog
                                            .title("Add New Credential / 添加新凭证")
                                            .w(px(600.))
                                            .min_h(px(400.))
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .h_full()
                                                    .gap_4()
                                                    .px_6()
                                                    .py_4()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_3()
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_2()
                                                                    .children([
                                                                        div()
                                                                            .text_sm()
                                                                            .font_semibold()
                                                                            .text_color(theme.foreground)
                                                                            .child("Basic Information / 基本信息"),
                                                                        div()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .gap_2()
                                                                            .child(
                                                                                div()
                                                                                    .flex()
                                                                                    .flex_col()
                                                                                    .gap_1()
                                                                                    .children([
                                                                                        div().text_xs().text_color(theme.muted_foreground).child("Name / 名称 *"),
                                                                                        div().w_full().child(Input::new(&form_name)),
                                                                                    ]),
                                                                            )
                                                                            .child(
                                                                                div()
                                                                                    .flex()
                                                                                    .flex_col()
                                                                                    .gap_1()
                                                                                    .children([
                                                                                        div().text_xs().text_color(theme.muted_foreground).child("Platform / 平台"),
                                                                                        div().w_full().child(Input::new(&form_platform)),
                                                                                    ]),
                                                                            )
                                                                            .child(
                                                                                div()
                                                                                    .flex()
                                                                                    .flex_col()
                                                                                    .gap_1()
                                                                                    .children([
                                                                                        div().text_xs().text_color(theme.muted_foreground).child("Category / 分类"),
                                                                                        div().w_full().child(Input::new(&form_category)),
                                                                                    ]),
                                                                            )
                                                                    ])
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_2()
                                                                    .children([
                                                                        div()
                                                                            .text_sm()
                                                                            .font_semibold()
                                                                            .text_color(theme.foreground)
                                                                            .child("Account Information / 账户信息"),
                                                                        div()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .gap_2()
                                                                            .child(
                                                                                div()
                                                                                    .flex()
                                                                                    .flex_col()
                                                                                    .gap_1()
                                                                                    .children([
                                                                                        div().text_xs().text_color(theme.muted_foreground).child("Username / 用户名 *"),
                                                                                        div().w_full().child(Input::new(&form_username)),
                                                                                    ]),
                                                                            )
                                                                            .child(
                                                                                div()
                                                                                    .flex()
                                                                                    .flex_col()
                                                                                    .gap_1()
                                                                                    .children([
                                                                                        div().text_xs().text_color(theme.muted_foreground).child("Password / 密码 *"),
                                                                                        div().w_full().child(Input::new(&form_password)),
                                                                                    ]),
                                                                            )
                                                                    ])
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_2()
                                                                    .children([
                                                                        div()
                                                                            .text_sm()
                                                                            .font_semibold()
                                                                            .text_color(theme.foreground)
                                                                            .child("Other / 其他"),
                                                                        div()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .gap_1()
                                                                            .children([
                                                                                div().text_xs().text_color(theme.muted_foreground).child("Notes / 备注"),
                                                                                div().w_full().child(Input::new(&form_notes)),
                                                                            ]),
                                                                    ])
                                                            )
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_end()
                                                    .gap_2()
                                                    .border_t_1()
                                                    .pt_4()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .gap_2()
                                                            .child(
                                                                Button::new("cancel-add-btn")
                                                                    .label("Cancel / 取消")
                                                                    .ghost()
                                                                    .on_click({
                                                                        let panel_clone = panel_clone.clone();
                                                                        move |_event, window, cx| {
                                                                            panel_clone.update(cx, |this, _cx| {
                                                                                this.show_add_dialog = false;
                                                                            });
                                                                            window.close_dialog(cx);
                                                                        }
                                                                    })
                                                            )
                                                            .child(
                                                                Button::new("confirm-add-btn")
                                                                    .label("Confirm / 确定")
                                                                    .primary()
                                                                    .on_click({
                                                                        let panel_clone = panel_clone_for_save.clone();
                                                                        move |_event, window, cx| {
                                                                            panel_clone.update(cx, |this, cx| {
                                                                                this.save_new_credential(window, cx);
                                                                                this.show_add_dialog = false;
                                                                            });
                                                                            window.close_dialog(cx);
                                                                        }
                                                                    })
                                                            )
                                                    )
                                            )
                                    });
                                }
                            }),
                    )
            )
            .child(
                div()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .children(
                        filtered_creds
                            .iter()
                            .enumerate()
                            .map(|(idx, cred)| {
                                let is_selected = selected == Some(idx);
                                let panel = cx.entity().clone();
                                let panel_for_delete = panel.clone();
                                let cred_name = cred.name.clone();
                                let cred_platform = cred.platform.clone();
                                let cred_username = cred.username.clone();
                                let cred_notes = cred.notes.clone();
                                let cred_id = cred.id.clone();

                                div()
                                    .id(ElementId::Name(SharedString::from(format!("cred-item-{}", idx))))
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .px_4()
                                    .py_3()
                                    .border_b_1()
                                    .bg(if is_selected {
                                        theme.accent.opacity(0.1)
                                    } else {
                                        theme.background
                                    })
                                    .cursor_pointer()
                                    .on_mouse_down(MouseButton::Left, move |_ev, _window: &mut Window, cx: &mut App| {
                                        panel.update(cx, |this, _cx| {
                                            this.selected_credential = Some(idx);
                                        });
                                    })
                                    .children([
                                        div()
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .children([
                                                div()
                                                    .flex()
                                                    .flex_1()
                                                    .flex_col()
                                                    .gap_1()
                                                    .children([
                                                        div()
                                                            .font_semibold()
                                                            .text_color(theme.foreground)
                                                            .child(cred_name.clone()),
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme.muted_foreground)
                                                            .child(format!("{} • {}", cred_platform, cred_username)),
                                                    ]),
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .child(
                                                        Button::new(("delete-cred-btn", idx))
                                                            .small()
                                                            .danger()
                                                            .ghost()
                                                            .label("Delete")
                                                            .on_mouse_down(MouseButton::Left, move |_ev, _window: &mut Window, cx: &mut App| {
                                                                let cred_id = cred_id.clone();
                                                                panel_for_delete.update(cx, |this, cx| {
                                                                    this.delete_credential(&cred_id, cx);
                                                                });
                                                                cx.stop_propagation();
                                                            })
                                                    )
                                            ]),
                                        if is_selected && !cred_notes.is_empty() {
                                            div()
                                                .mt_2()
                                                .p_3()
                                                .rounded_md()
                                                .bg(theme.accent.opacity(0.05))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.muted_foreground)
                                                        .child(cred_notes.clone())
                                                )
                                        } else {
                                            div()
                                        }
                                    ])
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>(),
                    )
            )
    }
}
