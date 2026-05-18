//! WASM 插件系统 - 完整版本
//! 包含图标、标题、ID 和 dock 集成

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// 插件元数据
#[wasm_bindgen]
#[derive(Deserialize, Serialize)]
pub struct PluginManifest {
    id: String,
    title: String,
    icon: String,
    description: String,
    version: String,
}

#[wasm_bindgen]
impl PluginManifest {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        title: String,
        icon: String,
        description: String,
        version: String,
    ) -> Self {
        Self {
            id,
            title,
            icon,
            description,
            version,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn icon(&self) -> String {
        self.icon.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// 转换为 JSON
    #[wasm_bindgen]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// 插件状态
#[wasm_bindgen]
pub struct PluginState {
    data: String,
}

#[wasm_bindgen]
impl PluginState {
    #[wasm_bindgen(constructor)]
    pub fn new(data: String) -> Self {
        Self { data }
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> String {
        self.data.clone()
    }
}

/// 计数器插件
#[wasm_bindgen]
pub struct CounterPlugin {
    count: i32,
    max: i32,
}

#[wasm_bindgen]
impl CounterPlugin {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { count: 0, max: 100 }
    }

    /// 获取插件元数据
    #[wasm_bindgen]
    pub fn manifest(&self) -> PluginManifest {
        PluginManifest::new(
            "counter".to_string(),
            "计数器".to_string(),
            "🔢".to_string(),
            "一个简单的计数器插件".to_string(),
            "1.0.0".to_string(),
        )
    }

    /// 增加计数
    #[wasm_bindgen]
    pub fn increment(&mut self) -> PluginState {
        self.count = (self.count + 1).min(self.max);
        PluginState::new(self.get_state_json())
    }

    /// 减少计数
    #[wasm_bindgen]
    pub fn decrement(&mut self) -> PluginState {
        self.count = (self.count - 1).max(0);
        PluginState::new(self.get_state_json())
    }

    /// 重置
    #[wasm_bindgen]
    pub fn reset(&mut self) -> PluginState {
        self.count = 0;
        PluginState::new(self.get_state_json())
    }

    /// 设置最大值
    #[wasm_bindgen]
    pub fn set_max(&mut self, max: i32) -> PluginState {
        self.max = max.max(1);
        if self.count > self.max {
            self.count = self.max;
        }
        PluginState::new(self.get_state_json())
    }

    /// 获取状态
    #[wasm_bindgen]
    pub fn get_state(&self) -> PluginState {
        PluginState::new(self.get_state_json())
    }

    fn get_state_json(&self) -> String {
        serde_json::json!({
            "count": self.count,
            "max": self.max,
            "percentage": if self.max > 0 {
                (self.count * 100 / self.max).max(0).min(100)
            } else {
                0
            }
        }).to_string()
    }

    /// 处理操作
    #[wasm_bindgen]
    pub fn handle_action(&mut self, action: String, value: i32) -> PluginState {
        match action.as_str() {
            "increment" => self.increment(),
            "decrement" => self.decrement(),
            "reset" => self.reset(),
            "set_max" => self.set_max(value),
            _ => self.get_state(),
        }
    }
}

/// 创建插件实例
#[wasm_bindgen]
pub fn create_plugin() -> CounterPlugin {
    CounterPlugin::new()
}

/// 获取所有可用插件
#[wasm_bindgen]
pub fn get_available_plugins() -> String {
    serde_json::json!([
        {
            "id": "counter",
            "title": "计数器",
            "icon": "🔢",
            "description": "一个简单的计数器插件",
            "version": "1.0.0"
        }
    ]).to_string()
}
