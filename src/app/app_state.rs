use gpui::{App, Global, SharedString};
use gpui_component::IconName;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use crate::core::credential_manager::CredentialService;
#[cfg(not(target_family = "wasm"))]
use crate::core::event_bus::EventHub;
#[cfg(not(target_family = "wasm"))]
use crate::core::services::CommentStyle;
#[cfg(not(target_family = "wasm"))]
use std::sync::Mutex;

/// Application-wide settings persisted to state file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_switch_theme: bool,
    pub font_family: SharedString,
    pub font_size: f64,
    #[serde(default = "default_locale")]
    pub locale: SharedString,
    pub line_height: f64,
    pub resettable: bool,
    pub group_variant: SharedString,
    #[serde(default)]
    pub show_settings: bool,
    #[serde(default = "default_true")]
    pub show_left_panel: bool,
    #[serde(default = "default_true")]
    pub show_right_panel: bool,
    #[serde(default = "default_true")]
    pub show_bottom_panel: bool,
    
    // Update settings
    #[serde(default)]
    pub auto_check_on_startup: bool,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_check_frequency")]
    pub check_frequency_days: f64,
    
    // Tool state (not persisted)
    #[serde(skip)]
    pub selected_tool: Option<String>,

    // Tool tabs (not persisted)
    #[serde(skip)]
    pub tool_tabs: HashMap<usize, ToolTabState>,
    #[serde(skip)]
    pub active_tool_tab_id: Option<usize>,

    // Terminal state for bottom panel (not persisted)
    #[serde(skip)]
    pub terminal_tabs: HashMap<usize, TerminalTabState>,
    #[serde(skip)]
    pub active_terminal_tab_id: usize,
    #[serde(skip)]
    pub next_terminal_tab_id: usize,
}

/// State for a single terminal tab (kept in memory, not persisted)
#[derive(Debug, Clone)]
pub struct TerminalTabState {
    pub id: usize,
    pub title: SharedString,
    pub output: Vec<SharedString>,
}

/// State for a tool tab (kept in memory, not persisted)
#[derive(Clone)]
pub struct ToolTabState {
    pub id: usize,              // 标签 ID (>= 1000)
    pub tool_id: String,        // 工具标识符
    pub title: SharedString,    // 标签标题
    pub icon: IconName,         // 标签图标
}

impl std::fmt::Debug for ToolTabState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolTabState")
            .field("id", &self.id)
            .field("tool_id", &self.tool_id)
            .field("title", &self.title)
            .field("icon", &"<icon>")  // IconName doesn't implement Debug
            .finish()
    }
}

/// Tool metadata for creating tabs
pub struct ToolMetadata {
    pub id: &'static str,
    pub name_key: &'static str,
    pub icon: IconName,
    pub tab_id: usize,
}

pub const TOOL_METADATA: &[ToolMetadata] = &[
    ToolMetadata { id: "csv_stats", name_key: "toolbox.tools.csv_stats", icon: IconName::Folder, tab_id: 1001 },
    ToolMetadata { id: "csv_split", name_key: "toolbox.tools.csv_split", icon: IconName::File, tab_id: 1002 },
    ToolMetadata { id: "csv_convert", name_key: "toolbox.tools.csv_convert", icon: IconName::File, tab_id: 1003 },
    ToolMetadata { id: "batch_rename", name_key: "toolbox.tools.batch_rename", icon: IconName::File, tab_id: 1004 },
    ToolMetadata { id: "excel_move", name_key: "toolbox.tools.excel_move_files", icon: IconName::Folder, tab_id: 1005 },
    ToolMetadata { id: "api_request", name_key: "toolbox.tools.api_request", icon: IconName::Globe, tab_id: 1006 },
    ToolMetadata { id: "api_batch_download", name_key: "toolbox.tools.api_batch_download", icon: IconName::ArrowRight, tab_id: 1007 },
    ToolMetadata { id: "json_convert", name_key: "toolbox.tools.json_convert", icon: IconName::File, tab_id: 1008 },
    ToolMetadata { id: "json_merge", name_key: "toolbox.tools.json_merge", icon: IconName::File, tab_id: 1009 },
    ToolMetadata { id: "network_scan", name_key: "toolbox.tools.network_scan", icon: IconName::Globe, tab_id: 1010 },
    ToolMetadata { id: "credential_manager", name_key: "credential.manager", icon: IconName::Settings, tab_id: 1011 },
    ToolMetadata { id: "code_editor", name_key: "code_editor.title", icon: IconName::File, tab_id: 1012 },
    ToolMetadata { id: "markdown_editor", name_key: "markdown_editor.title", icon: IconName::File, tab_id: 1013 },
];

/// Get tool metadata by tool ID
pub fn get_tool_metadata(tool_id: &str) -> Option<&'static ToolMetadata> {
    TOOL_METADATA.iter().find(|m| m.id == tool_id)
}

/// Get tool ID from tab ID
pub fn tool_id_from_tab(tab_id: usize) -> Option<&'static str> {
    TOOL_METADATA.iter().find(|m| m.tab_id == tab_id).map(|m| m.id)
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut this = Self {
            auto_switch_theme: false,
            font_family: "Arial".into(),
            font_size: 14.0,
            locale: default_locale(),
            line_height: 12.0,
            resettable: true,
            group_variant: "Fill".into(),
            show_settings: false,
            show_left_panel: true,
            show_right_panel: true,
            show_bottom_panel: true,
            auto_check_on_startup: false,
            notifications_enabled: true,
            auto_update: false,
            check_frequency_days: 7.0,
            selected_tool: None,
            tool_tabs: HashMap::new(),
            active_tool_tab_id: None,
            terminal_tabs: HashMap::new(),
            active_terminal_tab_id: 0,
            next_terminal_tab_id: 1,
        };
        // Create initial terminal tab
        this.add_terminal_tab();
        this
    }
}

impl Global for AppSettings {}

impl AppSettings {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
    
    pub fn add_terminal_tab(&mut self) -> usize {
        let id = self.next_terminal_tab_id;
        self.next_terminal_tab_id += 1;
        self.terminal_tabs.insert(id, TerminalTabState {
            id,
            title: format!("Terminal {}", id).into(),
            output: vec!["$ ".into()],
        });
        self.active_terminal_tab_id = id;
        id
    }
    
    pub fn close_terminal_tab(&mut self, id: usize) {
        if self.terminal_tabs.len() <= 1 {
            return;
        }
        self.terminal_tabs.remove(&id);
        if self.active_terminal_tab_id == id {
            self.active_terminal_tab_id = self.terminal_tabs.keys().last().copied().unwrap_or(0);
        }
    }
    
    pub fn set_active_terminal_tab(&mut self, id: usize) {
        if self.terminal_tabs.contains_key(&id) {
            self.active_terminal_tab_id = id;
        }
    }
    
    pub fn set_selected_tool(&mut self, tool: Option<String>) {
        self.selected_tool = tool;
    }

    /// 打开或切换到工具标签
    pub fn open_tool_tab(&mut self, tool_id: String) -> usize {
        let metadata = get_tool_metadata(&tool_id);
        let tab_id = metadata.map(|m| m.tab_id).unwrap_or(1000 + tool_id.len());

        // 如果标签已存在，切换到它
        if self.tool_tabs.contains_key(&tab_id) {
            self.active_tool_tab_id = Some(tab_id);
            return tab_id;
        }

        // 创建新标签
        let title = metadata.map(|m| t!(m.name_key).to_string())
            .unwrap_or_else(|| tool_id.clone());
        let icon = metadata.map(|m| m.icon.clone()).unwrap_or(IconName::File);

        self.tool_tabs.insert(tab_id, ToolTabState {
            id: tab_id,
            tool_id: tool_id.clone(),
            title: title.into(),
            icon,
        });

        self.active_tool_tab_id = Some(tab_id);
        tab_id
    }

    /// 关闭工具标签
    pub fn close_tool_tab(&mut self, tab_id: usize) {
        if tab_id < 1000 {
            return; // 不是工具标签
        }
        self.tool_tabs.remove(&tab_id);
        if self.active_tool_tab_id == Some(tab_id) {
            self.active_tool_tab_id = None;
        }
    }

    /// 切换到指定工具标签
    pub fn set_active_tool_tab(&mut self, tab_id: usize) {
        if self.tool_tabs.contains_key(&tab_id) {
            self.active_tool_tab_id = Some(tab_id);
        }
    }

    /// 获取当前活动的工具ID
    pub fn get_active_tool_id(&self) -> Option<&str> {
        self.active_tool_tab_id
            .and_then(|id| self.tool_tabs.get(&id))
            .map(|state| state.tool_id.as_str())
    }

    /// 获取所有工具标签
    pub fn get_all_tool_tabs(&self) -> Vec<&ToolTabState> {
        let mut tabs: Vec<_> = self.tool_tabs.values().collect();
        tabs.sort_by_key(|t| t.id);
        tabs
    }

    pub fn get_active_terminal_tab(&self) -> Option<&TerminalTabState> {
        self.terminal_tabs.get(&self.active_terminal_tab_id)
    }
    
    pub fn get_all_terminal_tabs(&self) -> Vec<&TerminalTabState> {
        self.terminal_tabs.values().collect()
    }
}

fn default_true() -> bool {
    true
}

fn default_check_frequency() -> f64 {
    7.0
}

fn default_locale() -> SharedString {
    detect_system_locale().unwrap_or_else(|| "en".into())
}

fn detect_system_locale() -> Option<SharedString> {
    let raw = sys_locale::get_locale().or_else(|| std::env::var("LANG").ok())?;
    normalize_locale(&raw).map(SharedString::from)
}

fn normalize_locale(locale: &str) -> Option<&'static str> {
    let lower = locale.to_lowercase();
    if lower.starts_with("zh") {
        return Some("zh-CN");
    }
    if lower.starts_with("en") {
        return Some("en");
    }
    None
}

/// Minimal app state
pub struct AppState {
    app_title: SharedString,
    #[cfg(not(target_family = "wasm"))]
    credential_service: Option<Arc<CredentialService>>,
    #[cfg(not(target_family = "wasm"))]
    ai_service: Option<AiService>,
    #[cfg(not(target_family = "wasm"))]
    event_hub: Arc<Mutex<EventHub>>,
    #[cfg(not(target_family = "wasm"))]
    current_working_dir: SharedString,
}

/// AI service stub for future integration
#[derive(Clone)]
pub struct AiService;

impl AiService {
    pub async fn generate_comment(&self, _code: &str, _style: CommentStyle) -> Result<String, String> {
        Ok("// Generated comment placeholder".to_string())
    }

    pub async fn explain_code(&self, _code: &str) -> Result<String, String> {
        Ok("// Code explanation placeholder".to_string())
    }

    pub async fn suggest_improvements(&self, _code: &str) -> Result<String, String> {
        Ok("// Improvement suggestions placeholder".to_string())
    }
}

impl AppState {
    pub fn init(cx: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        let credential_service = {
            use crate::core::credential_manager::CredentialManagerInit;
            match CredentialManagerInit::initialize() {
                Ok(service) => {
                    log::info!("Credential manager initialized");
                    Some(Arc::new(service))
                }
                Err(e) => {
                    log::error!("Failed to initialize credential manager: {}", e);
                    None
                }
            }
        };

        #[cfg(target_family = "wasm")]
        let credential_service = ();

        #[cfg(not(target_family = "wasm"))]
        cx.set_global::<AppState>(Self {
            app_title: SharedString::from(""),
            credential_service,
            ai_service: None,
            event_hub: Arc::new(Mutex::new(EventHub::default())),
            current_working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .into(),
        });

        #[cfg(target_family = "wasm")]
        cx.set_global::<AppState>(Self {
            app_title: SharedString::from(""),
        });
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn set_app_title(&mut self, title: SharedString) {
        self.app_title = title;
    }

    pub fn app_title(&self) -> &SharedString {
        &self.app_title
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn credential_service(&self) -> Option<Arc<CredentialService>> {
        self.credential_service.clone()
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn ai_service(&self) -> Option<AiService> {
        self.ai_service.clone()
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn event_hub(&self) -> Arc<Mutex<EventHub>> {
        self.event_hub.clone()
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn current_working_dir(&self) -> &SharedString {
        &self.current_working_dir
    }
}

impl Global for AppState {}
