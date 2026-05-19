use gpui::{px, Pixels, Point};
use std::sync::{Arc, Mutex};

/// 全局拖动状态
#[derive(Debug, Clone)]
pub struct GlobalDragState {
    /// 正在拖动的窗口实体 ID (None 表示没有拖动)
    pub dragging_window: Option<String>,
    /// 拖动开始时的鼠标位置
    pub drag_start: Point<Pixels>,
    /// 拖动开始时窗口位置
    pub window_start: Point<Pixels>,
    /// 是否正在调整大小
    pub resizing_window: Option<String>,
    /// 调整大小开始时的鼠标位置
    pub resize_start: Point<Pixels>,
    /// 调整大小开始时窗口尺寸
    pub window_start_size: (Pixels, Pixels),
}

impl Default for GlobalDragState {
    fn default() -> Self {
        Self {
            dragging_window: None,
            drag_start: Point::default(),
            window_start: Point::default(),
            resizing_window: None,
            resize_start: Point::default(),
            window_start_size: (px(0.), px(0.)),
        }
    }
}

impl GlobalDragState {
    /// 开始拖动窗口
    pub fn start_drag(&mut self, window_id: String, mouse_pos: Point<Pixels>, window_pos: Point<Pixels>) {
        self.dragging_window = Some(window_id);
        self.drag_start = mouse_pos;
        self.window_start = window_pos;
    }

    /// 开始调整窗口大小
    pub fn start_resize(&mut self, window_id: String, mouse_pos: Point<Pixels>, window_size: (Pixels, Pixels)) {
        self.resizing_window = Some(window_id);
        self.resize_start = mouse_pos;
        self.window_start_size = window_size;
    }

    /// 结束拖动或调整大小
    pub fn end(&mut self) {
        self.dragging_window = None;
        self.resizing_window = None;
    }

    /// 是否正在拖动或调整大小
    pub fn is_active(&self) -> bool {
        self.dragging_window.is_some() || self.resizing_window.is_some()
    }
}

// 使用 Arc<Mutex<>> 实现线程安全的可变全局状态
pub struct SharedGlobalDragState {
    inner: Arc<Mutex<GlobalDragState>>,
}

impl Clone for SharedGlobalDragState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for SharedGlobalDragState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(GlobalDragState::default())),
        }
    }
}

impl SharedGlobalDragState {
    /// 开始拖动窗口
    pub fn start_drag(&self, window_id: String, mouse_pos: Point<Pixels>, window_pos: Point<Pixels>) {
        if let Ok(mut state) = self.inner.lock() {
            state.start_drag(window_id, mouse_pos, window_pos);
        }
    }

    /// 开始调整窗口大小
    pub fn start_resize(&self, window_id: String, mouse_pos: Point<Pixels>, window_size: (Pixels, Pixels)) {
        if let Ok(mut state) = self.inner.lock() {
            state.start_resize(window_id, mouse_pos, window_size);
        }
    }

    /// 结束拖动或调整大小
    pub fn end(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.end();
        }
    }

    /// 获取当前拖动状态（返回窗口 ID 和偏移量）
    pub fn get_drag_state(&self) -> (Option<String>, Point<Pixels>, Point<Pixels>) {
        if let Ok(state) = self.inner.lock() {
            (state.dragging_window.clone(), state.drag_start, state.window_start)
        } else {
            (None, Point::default(), Point::default())
        }
    }

    /// 获取当前调整大小状态
    pub fn get_resize_state(&self) -> (Option<String>, Point<Pixels>, (Pixels, Pixels)) {
        if let Ok(state) = self.inner.lock() {
            (state.resizing_window.clone(), state.resize_start, state.window_start_size)
        } else {
            (None, Point::default(), (px(0.), px(0.)))
        }
    }

    /// 是否正在拖动或调整大小
    pub fn is_active(&self) -> bool {
        if let Ok(state) = self.inner.lock() {
            state.is_active()
        } else {
            false
        }
    }
}

impl gpui::Global for SharedGlobalDragState {}
