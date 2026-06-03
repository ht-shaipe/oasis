use ai_llm_kit::{LlmClient, LlmService, StreamCallback};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Manager;
use tauri::ipc::Channel;
use tube::{value, Result as TubeResult, Value};

const LLM_CONFIG_FILE: &str = "llm_config.json";

fn config_dir(app: &AppHandle) -> std::result::Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn llm_config_path(app: &AppHandle) -> std::result::Result<PathBuf, String> {
    Ok(config_dir(app)?.join(LLM_CONFIG_FILE))
}

// ── 数据模型 ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    pub base_url: String,
    pub auth_type: String,
    pub api_key: String,
    pub token_plan: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub description: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmConfig {
    pub models: Vec<LlmModel>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatRequest {
    pub model_id: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatResponse {
    pub content: String,
    pub usage: Option<ChatUsage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── 配置读写 ──────────────────────────────────────────────────────

fn load_llm_config(app: &AppHandle) -> std::result::Result<LlmConfig, String> {
    let path = llm_config_path(app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let config: LlmConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        Ok(LlmConfig { models: Vec::new() })
    }
}

fn save_llm_config(app: &AppHandle, config: &LlmConfig) -> std::result::Result<(), String> {
    let path = llm_config_path(app)?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

// ── Tauri Commands: 模型管理 ───────────────────────────────────────

#[tauri::command]
pub fn get_llm_models(app: AppHandle) -> std::result::Result<Vec<LlmModel>, String> {
    let config = load_llm_config(&app)?;
    Ok(config.models)
}

#[tauri::command]
pub fn save_llm_model(app: AppHandle, model: LlmModel) -> std::result::Result<LlmModel, String> {
    let mut config = load_llm_config(&app)?;
    if let Some(existing) = config.models.iter_mut().find(|m| m.id == model.id) {
        *existing = model.clone();
    } else {
        config.models.push(model.clone());
    }
    save_llm_config(&app, &config)?;
    Ok(model)
}

#[tauri::command]
pub fn delete_llm_model(app: AppHandle, id: String) -> std::result::Result<(), String> {
    let mut config = load_llm_config(&app)?;
    config.models.retain(|m| m.id != id);
    save_llm_config(&app, &config)
}

#[tauri::command]
pub fn toggle_llm_model(app: AppHandle, id: String, enabled: bool) -> std::result::Result<(), String> {
    let mut config = load_llm_config(&app)?;
    if let Some(model) = config.models.iter_mut().find(|m| m.id == id) {
        model.enabled = enabled;
        save_llm_config(&app, &config)?;
        Ok(())
    } else {
        Err(format!("model '{}' not found", id))
    }
}

// ── Tauri Commands: 对话 ──────────────────────────────────────────

fn get_client_for_model(app: &AppHandle, model_id: &str) -> std::result::Result<(LlmClient, LlmModel), String> {
    let config = load_llm_config(app)?;
    let model = config
        .models
        .iter()
        .find(|m| m.model_id == model_id)
        .cloned()
        .ok_or(format!("model '{}' not found in config", model_id))?;

    let access_token = match model.auth_type.as_str() {
        "token_plan" => format!("Bearer {}", model.token_plan),
        _ => format!("Bearer {}", model.api_key),
    };

    let client = LlmClient::new(&model.base_url, "", &access_token).set_model(&model.model_id);

    Ok((client, model))
}

#[tauri::command]
pub async fn ai_chat(app: AppHandle, request: ChatRequest) -> std::result::Result<ChatResponse, String> {
    let (client, model) = get_client_for_model(&app, &request.model_id)?;

    let msgs: Vec<tube::Value> = request
        .messages
        .iter()
        .map(|m| {
            value!({
                "role": m.role.clone(),
                "content": m.content.clone(),
            })
        })
        .collect();

    let body = value!({
        "model": request.model_id.clone(),
        "messages": msgs,
        "temperature": request.temperature.unwrap_or(model.temperature),
        "max_tokens": request.max_tokens.unwrap_or(model.max_tokens),
        "stream": false,
    });

    let resp = client.chat(&body).await.map_err(|e| format!("Chat failed: {}", e))?;

    let content = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first().cloned())
        .and_then(|choice| choice.get("message").cloned())
        .and_then(|msg| msg.get("content").cloned())
        .and_then(|c| c.as_str().map(String::from))
        .unwrap_or_default();

    let usage = resp.get("usage").map(|u| ChatUsage {
        prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    });

    Ok(ChatResponse { content, usage })
}

// ── Tauri Commands: 流式对话 ───────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    pub content: String,
    pub is_over: bool,
    pub usage: Option<ChatUsage>,
}

/// SSE 流式对话。前端通过 `Channel<StreamChunk>` 逐块接收增量内容。
#[tauri::command]
pub async fn ai_chat_stream(
    app: AppHandle,
    request: ChatRequest,
    channel: Channel<StreamChunk>,
) -> std::result::Result<(), String> {
    let (client, model) = get_client_for_model(&app, &request.model_id)?;

    let msgs: Vec<Value> = request
        .messages
        .iter()
        .map(|m| {
            value!({
                "role": m.role.clone(),
                "content": m.content.clone(),
            })
        })
        .collect();

    let body = value!({
        "model": request.model_id.clone(),
        "messages": msgs,
        "temperature": request.temperature.unwrap_or(model.temperature),
        "max_tokens": request.max_tokens.unwrap_or(model.max_tokens),
        "stream": true,
    });

    let callback: Arc<StreamCallback> = {
        let channel = channel.clone();
        Arc::new(move |content: String, is_over: bool| {
            let channel = channel.clone();
            Box::pin(async move {
                let _ = channel.send(StreamChunk {
                    content,
                    is_over,
                    usage: None,
                });
                Ok(Value::Null)
            }) as Pin<Box<dyn Future<Output = TubeResult<Value>> + Send>>
        })
    };

    let resp = client
        .chat_stream(&body, callback)
        .await
        .map_err(|e| format!("Stream chat failed: {}", e))?;

    // 流结束后将最终的 usage 信息作为最后一个消息发送
    if let Some(usage) = resp.get("usage") {
        let _ = channel.send(StreamChunk {
            content: String::new(),
            is_over: true,
            usage: Some(ChatUsage {
                prompt_tokens: usage
                    .get("prompt_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                completion_tokens: usage
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: usage
                    .get("total_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            }),
        });
    }

    Ok(())
}
