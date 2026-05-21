//! copyright © ecdata.cn 2026 - present
//! UI Schema — 声明式 UI 描述

use serde::{Deserialize, Serialize};
use crate::UiNode;

// ---------------------------------------------------------------------------
// UI Schema（声明式 UI 描述）
// ---------------------------------------------------------------------------

/// UI 布局定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiSchema {
    /// 布局模式：flex-col / flex-row / grid 等
    #[serde(default = "default_layout")]
    pub layout: String,
    /// 子元素间距（px）
    #[serde(default)]
    pub gap: i64,
    /// 交叉轴对齐：start / center / end / stretch
    #[serde(default)]
    pub align_items: Option<String>,
    /// 主轴对齐：start / center / end / space-between / space-around
    #[serde(default)]
    pub justify_content: Option<String>,
    #[serde(default)]
    pub children: Vec<UiNode>,
}

fn default_layout() -> String {
    "flex-col".into()
}

impl UiSchema {
    /// 创建空布局
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建垂直布局
    pub fn flex_col() -> Self {
        Self {
            layout: "flex-col".into(),
            ..Default::default()
        }
    }

    /// 创建水平布局
    pub fn flex_row() -> Self {
        Self {
            layout: "flex-row".into(),
            ..Default::default()
        }
    }

    /// 添加子节点
    pub fn child(mut self, child: UiNode) -> Self {
        self.children.push(child);
        self
    }

    /// 添加多个子节点
    pub fn children(mut self, children: impl IntoIterator<Item = UiNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// 设置间距
    pub fn gap(mut self, gap: i64) -> Self {
        self.gap = gap;
        self
    }

    /// 设置对齐方式
    pub fn align(mut self, align: impl Into<String>) -> Self {
        self.align_items = Some(align.into());
        self
    }

    /// 设置主轴对齐
    pub fn justify(mut self, justify: impl Into<String>) -> Self {
        self.justify_content = Some(justify.into());
        self
    }
}
