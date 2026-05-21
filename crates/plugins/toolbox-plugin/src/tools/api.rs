//! API 工具：HTTP 请求、批量下载

use plugin_sdk::{UiNode, UiSchema};
use crate::tools::home::make_button_row;

// ---------------------------------------------------------------------------
// API 请求
// ---------------------------------------------------------------------------
pub fn schema_api_request() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("🌐 API 请求"),
            UiNode::input("url", "URL"),
            UiNode::input("method", "方法 (GET/POST/PUT/DELETE)"),
            UiNode::input("headers", "请求头 (每行一个，格式: Key: Value)"),
            UiNode::input("body", "请求体 (JSON)"),
            make_button_row(&[("📤", "发送", "api:send")]),
            UiNode::info(&[("状态", "response_status")]),
            UiNode::display("response_body"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 批量下载
// ---------------------------------------------------------------------------
pub fn schema_batch_download() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("⬇️ 批量下载"),
            UiNode::input("template", "URL 模板 (支持 {0} 占位)"),
            UiNode::input("paths", "路径列表 (每行一个)"),
            UiNode::input("concurrency", "并发数"),
            make_button_row(&[
                ("📂", "输出目录", "batch_dl:pick_output"),
                ("▶️", "开始下载", "batch_dl:start"),
            ]),
            UiNode::display("message"),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// HTTP 请求逻辑
// ---------------------------------------------------------------------------

/// 执行 HTTP 请求（同步阻塞）
pub fn do_http_request(url: &str, method: &str, headers: &str, body: &str) -> (u16, String) {
    if url.is_empty() {
        return (0, "URL 不能为空".to_string());
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return (0, format!("创建 runtime 失败: {e}")),
    };

    let client = match reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return (0, format!("创建 client 失败: {e}")),
    };

    let mut req = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => return (0, format!("不支持的方法: {method}")),
    };

    for line in headers.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim();
            let v = v.trim();
            if !k.is_empty() {
                req = req.header(k, v);
            }
        }
    }

    if !body.is_empty() && matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
        req = req.body(body.to_string());
    }

    let result = rt.block_on(req.send());
    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body_text = rt.block_on(resp.text()).unwrap_or_else(|_| String::new());
            (status, body_text)
        }
        Err(e) => (0, format!("请求失败: {e}")),
    }
}
