//! System config templates — built-in presets for common configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
}

pub fn list_config_templates() -> Vec<ConfigTemplate> {
    vec![
        ConfigTemplate {
            id: "default".into(),
            name: "默认配置 (Default)".into(),
            description: "空配置，使用 Claude CLI 默认行为。".into(),
            config: serde_json::json!({}),
        },
        ConfigTemplate {
            id: "sandbox".into(),
            name: "沙箱模式 (Sandbox)".into(),
            description: "启用沙箱，严格权限控制，适合不信任的代码库。".into(),
            config: serde_json::json!({
                "permissions": { "defaultMode": "default" },
                "sandbox": { "enabled": true }
            }),
        },
        ConfigTemplate {
            id: "bypass".into(),
            name: "跳过权限 (Bypass)".into(),
            description: "跳过所有权限检查，适合受信任的项目。".into(),
            config: serde_json::json!({
                "permissions": { "defaultMode": "bypassPermissions" }
            }),
        },
        ConfigTemplate {
            id: "native-api".into(),
            name: "原生 API (Native)".into(),
            description: "使用 Anthropic 官方 API，直接触发原生授权引导。".into(),
            config: anthropic_official_config(),
        },
        ConfigTemplate {
            id: "proxy-config".into(),
            name: "中转配置 (Proxy)".into(),
            description: "使用国内主流模型供应商进行中转。".into(),
            config: third_party_proxy_config(),
        },
        ConfigTemplate {
            id: "deepseek-api".into(),
            name: "DeepSeek API".into(),
            description: "使用 DeepSeek API 代理。".into(),
            config: deepseek_config(),
        },
        ConfigTemplate {
            id: "zhipu-api".into(),
            name: "智谱 API (Zhipu)".into(),
            description: "使用智谱 AI BigModel API 代理。".into(),
            config: zhipu_config(),
        },
    ]
}

fn anthropic_official_config() -> serde_json::Value {
    let env: HashMap<String, String> = HashMap::from([
        ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
    ]);
    serde_json::json!({
        "apiProvider": "anthropic",
        "model": "claude-sonnet-4-6",
        "env": env
    })
}

fn third_party_proxy_config() -> serde_json::Value {
    let env: HashMap<String, String> = HashMap::from([
        ("ANTHROPIC_BASE_URL".into(), String::new()),
        ("ANTHROPIC_AUTH_TOKEN".into(), String::new()),
        ("ANTHROPIC_MODEL".into(), String::new()),
    ]);
    serde_json::json!({
        "apiProvider": "anthropic",
        "env": env
    })
}

fn deepseek_config() -> serde_json::Value {
    let env: HashMap<String, String> = HashMap::from([
        ("DEEPSEEK_API_KEY".into(), String::new()),
    ]);
    serde_json::json!({
        "apiProvider": "deepseek",
        "model": "deepseek-chat",
        "env": env
    })
}

fn zhipu_config() -> serde_json::Value {
    let env: HashMap<String, String> = HashMap::from([
        ("ZHIPU_API_KEY".into(), String::new()),
    ]);
    serde_json::json!({
        "apiProvider": "zhipu",
        "model": "glm-4",
        "env": env
    })
}
