use ai_llm_kit::{LlmClient, LlmFactory, LlmProvider, LlmService, StreamCallback};
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
    #[serde(default = "default_model_type")]
    pub model_type: String,
}

fn default_model_type() -> String {
    "chat".to_string()
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

    // LlmService trait uses `#[async_trait(?Send)]`, so the returned future
    // is not `Send`. Wrap it in a single-threaded runtime on a blocking thread
    // to satisfy Tauri's `Send` requirement for async commands.
    let resp = tauri::async_runtime::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Runtime error: {}", e))?;
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, client.chat(&body))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Chat failed: {}", e))?;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    pub is_over: bool,
    pub usage: Option<ChatUsage>,
}

async fn local_chat_stream(
    app: AppHandle,
    request: ChatRequest,
    channel: Channel<StreamChunk>,
    _model: &LlmModel,
) -> std::result::Result<(), String> {
    oasis_local_llm::inference::load_model(&app, &request.model_id).await?;

    let local_messages: Vec<oasis_local_llm::inference::ChatMessageForLocal> = request
        .messages
        .iter()
        .map(|m| oasis_local_llm::inference::ChatMessageForLocal {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let channel_clone = channel.clone();
    tauri::async_runtime::spawn_blocking(move || {
        oasis_local_llm::inference::chat_stream_blocking(local_messages, move |content, is_over| {
            let _ = channel_clone.send(StreamChunk {
                content,
                reasoning_content: None,
                is_over,
                usage: None,
            });
        })
    })
    .await
    .map_err(|e| format!("Local inference task error: {}", e))?
}

/// SSE 流式对话。前端通过 `Channel<StreamChunk>` 逐块接收增量内容。
#[tauri::command]
pub async fn ai_chat_stream(
    app: AppHandle,
    request: ChatRequest,
    channel: Channel<StreamChunk>,
) -> std::result::Result<(), String> {
    let config = load_llm_config(&app)?;
    let model = config
        .models
        .iter()
        .find(|m| m.model_id == request.model_id)
        .cloned();

    if let Some(ref m) = model {
        if m.provider == "local" {
            return local_chat_stream(app, request, channel, m).await;
        }
    }

    let (client, model_cfg) = get_client_for_model(&app, &request.model_id)?;

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
        "temperature": request.temperature.unwrap_or(model_cfg.temperature),
        "max_tokens": request.max_tokens.unwrap_or(model_cfg.max_tokens),
        "stream": true,
    });

    // LlmService trait uses `#[async_trait(?Send)]`, so the returned future
    // is not `Send`. Wrap it in a single-threaded runtime on a blocking thread
    // to satisfy Tauri's `Send` requirement for async commands.
    let channel_for_blocking = channel.clone();
    let resp = tauri::async_runtime::spawn_blocking(move || {
        let callback: Arc<StreamCallback> = {
            let channel = channel_for_blocking.clone();
            Arc::new(move |content: String, is_over: bool| {
                let channel = channel.clone();
                Box::pin(async move {
                    if content.is_empty() {
                        let _ = channel.send(StreamChunk {
                            content: String::new(),
                            reasoning_content: None,
                            is_over,
                            usage: None,
                        });
                        return Ok(Value::Null);
                    }

                    let decoded = if content.starts_with("data: ") {
                        &content[6..]
                    } else {
                        &content
                    };

                    if decoded.trim() == "[DONE]" {
                        let _ = channel.send(StreamChunk {
                            content: String::new(),
                            reasoning_content: None,
                            is_over: true,
                            usage: None,
                        });
                        return Ok(Value::Null);
                    }

                    let (chunk_content, chunk_reasoning) = match serde_json::from_str::<serde_json::Value>(decoded) {
                        Ok(json) => {
                            let delta = json
                                .get("choices")
                                .and_then(|c| c.as_array())
                                .and_then(|arr| arr.get(0))
                                .and_then(|choice| choice.get("delta"));

                            let text = delta
                                .and_then(|d| d.get("content"))
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_default();

                            let reasoning = delta
                                .and_then(|d| d.get("reasoning_content"))
                                .and_then(|r| r.as_str())
                                .map(|s| s.to_string());

                            (text, reasoning)
                        }
                        Err(_) => (content.clone(), None),
                    };

                    let _ = channel.send(StreamChunk {
                        content: chunk_content,
                        reasoning_content: chunk_reasoning,
                        is_over,
                        usage: None,
                    });
                    Ok(Value::Null)
                }) as Pin<Box<dyn Future<Output = TubeResult<Value>> + Send>>
            })
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Runtime error: {}", e))?;
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, client.chat_stream(&body, callback))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Stream chat failed: {}", e))?;

    // 流结束后将最终的 usage 信息作为最后一个消息发送
    if let Some(usage) = resp.get("usage") {
        let _ = channel.send(StreamChunk {
            content: String::new(),
            reasoning_content: None,
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

// ── Tauri Commands: 平商列表 & 远程模型拉取 ─────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProviderOption {
    pub code: String,
    pub name: String,
    pub base_url: String,
}

const PROVIDER_INFO: &[(&str, &str, &str)] = &[
    ("deepseek", "DeepSeek", "https://api.deepseek.com"),
    ("chatgpt", "ChatGPT", "https://api.openai.com/v1"),
    ("ollama", "Ollama", "http://localhost:11434"),
    ("kimi", "Kimi", "https://api.moonshot.cn/v1"),
    ("hunyuan", "腾讯混元", "https://api.hunyuan.cloud.tencent.com/v1"),
    ("doubao", "豆包", "https://ark.cn-beijing.volces.com/api/v3"),
    ("mimo", "小米MiMo", "https://xiaomi.com/api/v1"),
    ("qwen", "阿里千问", "https://dashscope.aliyuncs.com/compatible-mode/v1"),
    ("zhipu", "智谱", "https://open.bigmodel.cn/api/paas/v4"),
    ("wenxin", "文心一言", "https://qianfan.baidubce.com/v2"),
    ("xunfei", "讯飞", "https://spark-api-open.xf-yun.com/v1"),
];

#[tauri::command]
pub fn get_llm_providers() -> Vec<ProviderOption> {
    PROVIDER_INFO
        .iter()
        .map(|(code, name, base_url)| ProviderOption {
            code: code.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemoteModel {
    pub id: String,
    pub name: String,
    pub owned_by: String,
}

#[tauri::command]
pub async fn fetch_provider_models(
    provider: String,
    base_url: String,
    api_key: String,
) -> std::result::Result<Vec<RemoteModel>, String> {
    let access_token = format!("Bearer {}", api_key);

    let client = LlmClient::new(&base_url, "", &access_token);

    let resp = tauri::async_runtime::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Runtime error: {}", e))?;
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, client.models())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
    .map_err(|e| format!("Fetch models failed: {}", e))?;

    let model_list: Vec<Value> = resp
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let remote_models: Vec<RemoteModel> = model_list
        .iter()
        .filter_map(|m| {
            let id = m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
            if id.is_empty() {
                return None;
            }
            let name = m.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.clone());
            let owned_by = m.get("owned_by")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| provider.clone());
            Some(RemoteModel { id, name, owned_by })
        })
        .collect();

    Ok(remote_models)
}
