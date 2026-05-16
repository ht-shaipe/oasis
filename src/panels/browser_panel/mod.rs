// Browser/WebView panel - stub implementation
// TODO: Implement actual webview functionality

use std::path::PathBuf;
use std::sync::mpsc::Sender;

/// Message type for webview log communication
#[derive(Debug, Clone)]
pub struct WebViewLogMessage {
    pub level: String,
    pub message: String,
    pub raw: Option<String>,
}

/// Create a webview entity with the given URL
/// This is a stub - actual implementation would create a real webview
#[allow(dead_code)]
pub fn create_webview_entity(
    _window: &mut gpui::Window,
    _cx: &mut impl gpui::AppContext,
    _url: &str,
    _log_tx: Sender<WebViewLogMessage>,
) -> Option<gpui::Entity<gpui_wry::WebView>> {
    // Stub: no-op
    // TODO: Implement actual webview creation
    None
}

/// Register a download with the webview
/// This is a stub - actual implementation would handle download registration
#[allow(dead_code)]
pub fn register_webview_download(
    _url: &str,
    _slot_index: usize,
    _save_path: PathBuf,
) {
    // Stub: no-op
    // TODO: Implement actual download registration
}
