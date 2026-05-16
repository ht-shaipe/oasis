// Utility functions module

use gpui::{App, AppContext, Task};
use std::path::PathBuf;

/// Pick a file using native file dialog
pub async fn pick_file(
    _title: &str,
    _filter_name: Option<&str>,
    _extensions: Option<&[&str]>,
) -> Option<PathBuf> {
    // Stub implementation - should use native file dialog
    None
}

/// Pick a folder using native folder dialog
pub async fn pick_folder(_title: &str) -> Option<PathBuf> {
    // Stub implementation - should use native folder dialog
    None
}

/// Pick a save file using native save dialog
pub async fn pick_save_file(
    _title: &str,
    _default_name: Option<&str>,
    _filter_name: Option<&str>,
    _extensions: Option<&[&str]>,
) -> Option<PathBuf> {
    // Stub implementation - should use native save dialog
    None
}