//! Background image management module
//!
//! Provides functionality for setting and clearing window background images.

use gpui::{App, Global};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Background settings persisted to state file
/// This is the single source of truth for background image state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSettings {
    /// Background image path (local filesystem path)
    pub background_image: Option<String>,
}

impl Default for BackgroundSettings {
    fn default() -> Self {
        tracing::info!("🆕 BackgroundSettings::default() 创建");
        // 设置默认背景图片
        Self {
            background_image: Some("/Users/shaipe/workspace/rust/tools/oasis/assets/backgroud/deault.jpg".to_string()),
        }
    }
}

impl Global for BackgroundSettings {}

impl BackgroundSettings {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    /// Set the background image path
    pub fn set_background_image(&mut self, path: Option<String>) {
        tracing::info!("💾 BackgroundSettings::set_background_image: {:?} -> {:?}", self.background_image, path);
        self.background_image = path;
    }

    /// Get the background image path as PathBuf
    pub fn get_path_buf(&self) -> Option<PathBuf> {
        let result = self.background_image.as_ref().map(PathBuf::from);
        result
    }

    /// Check if a background image is set
    pub fn has_background(&self) -> bool {
        self.background_image.is_some()
    }

    /// Get the background image path (convenience method for rendering)
    pub fn background_image(&self) -> Option<&PathBuf> {
        // 注意：这里返回的是临时引用，不能直接使用
        // 需要调用 get_path_buf() 获取拥有的 PathBuf
        None
    }
}

/// Convenience type alias for backward compatibility
pub type BackgroundManager = BackgroundSettings;

/// Initialize background settings
pub fn init(cx: &mut App) {
    tracing::info!("🚀 background::init() 初始化背景设置");
    let settings = BackgroundSettings::default();
    cx.set_global(settings);
    tracing::info!("✅ 背景设置初始化完成: {:?}", BackgroundSettings::global(cx).background_image);
}
