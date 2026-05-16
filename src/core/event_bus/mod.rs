// Event bus for code editor events
// Stub implementation

use gpui::SharedString;

/// Event fired when code is selected
#[derive(Debug, Clone)]
pub struct CodeSelectionEvent {
    pub file_path: SharedString,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub content: SharedString,
    pub selection: SharedString,
}

/// Event fired when a toolbox tool is selected
#[derive(Debug, Clone)]
pub struct ToolSelectedEvent(pub String); // tool identifier string

/// Event hub for publishing events
#[derive(Default)]
pub struct EventHub;

impl EventHub {
    pub fn publish_code_selection(&self, _event: CodeSelectionEvent) {
        // Stub: no-op
    }

    pub fn publish_tool_selected(&self, _event: ToolSelectedEvent) {
        // Stub: no-op
    }
}