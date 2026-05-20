use std::ops::Range;

/// Represents a single edit operation that can be undone/redone
#[derive(Clone)]
pub struct EditOperation {
    pub old_text: String,
    pub new_text: String,
    pub old_cursor: usize,
    pub new_cursor: usize,
    pub old_selection: Option<Range<usize>>,
    pub new_selection: Option<Range<usize>>,
}

/// Manages undo/redo history for document editing
#[derive(Clone)]
pub struct UndoHistory {
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
    max_history: usize,
}

impl UndoHistory {
    pub fn new(max_history: usize) -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_history }
    }

    pub fn push(&mut self, op: EditOperation) {
        self.undo_stack.push(op);
        self.redo_stack.clear();
        while self.undo_stack.len() > self.max_history { self.undo_stack.remove(0); }
    }

    pub fn undo(&mut self) -> Option<EditOperation> {
        if let Some(op) = self.undo_stack.pop() { self.redo_stack.push(op.clone()); Some(op) } else { None }
    }

    pub fn redo(&mut self) -> Option<EditOperation> {
        if let Some(op) = self.redo_stack.pop() { self.undo_stack.push(op.clone()); Some(op) } else { None }
    }

    pub fn clear(&mut self) { self.undo_stack.clear(); self.redo_stack.clear(); }
    #[allow(dead_code)] pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    #[allow(dead_code)] pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
}

impl Default for UndoHistory {
    fn default() -> Self { Self::new(100) }
}