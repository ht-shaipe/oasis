//! 代理模块
//! 统一管理代理转发功能

use super::config::ProxyConfig;

use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

/// 代理管理器
/// 统一管理代理转发功能
pub struct ProxyManager {
    client: Client,
}

impl ProxyManager {
    /// 创建新的代理管理器
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 初始化代理管理器
    pub fn init() {
        ProxyConfig::new("proxy.toml");
    }

    /// 从 API 路径中提取第二段作为服务名
    /// 例如: "/api/pas/user/login" -> Some("pas")
    /// 注意: 基础API（如 /api/health, /api/config）不提取服务名，使用默认配置
    pub fn extract_service_from_path(path: &str) -> Option<String> {
        // 如果路径是完整 URL，先提取路径部分
        let path_only = if path.starts_with("http://") || path.starts_with("https://") {
            if let Some(path_start) = path.find('/') {
                if let Some(path_part) = path[path_start..].split('?').next() {
                    path_part
                } else {
                    path
                }
            } else {
                path
            }
        } else {
            // 提取路径部分（去除查询参数）
            if let Some(path_part) = path.split('?').next() {
                path_part
            } else {
                path
            }
        };

        let parts: Vec<&str> = path_only.split('/').filter(|s| !s.is_empty()).collect();

        // 基础API列表，这些不应该被提取为服务名
        let base_apis = ["health", "config", "chat", "mcp", "ollama", "llm", "user"];

        if parts.len() >= 2 && parts[0] == "api" {
            let service_name = parts[1].to_string();
            // 如果是基础API，不提取服务名
            if base_apis.contains(&service_name.as_str()) {
                None
            } else {
                Some(service_name)
            }
        } else {
            None
        }
    }

    /// 获取目标URL
    pub fn get_target_url(&self, path: &str) -> String {
        // 获取代理配置
        let proxy_config = ProxyConfig::get();

        // 如果路径已经是完整 URL，直接返回
        if path.starts_with("http://") || path.starts_with("https://") {
            log!("路径已经是完整URL，直接使用: {}", path);
            return path.to_string();
        }

        // 提取路径部分（去除查询参数）
        let path_only = if let Some(path_part) = path.split('?').next() {
            path_part
        } else {
            path
        };

        let module = Self::extract_service_from_path(path_only);

        let domain = if let Some(module) = module.clone() {
            proxy_config
                .specific
                .get(&module)
                .unwrap_or(&proxy_config.default)
        } else {
            &proxy_config.default
        };

        let target_url = format!("{}{}", domain, path);
        log!("代理目标URL: {} (模块: {:?})", target_url, module);
        target_url
    }

    /// 统一代理转发
    /// 根据API路径自动选择对应的域名进行代理转发
    pub async fn proxy_request(
        &self,
        path: String,
        method: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> tube::Result<serde_json::Value> {
        let target_url = self.get_target_url(&path);

        let mut request_builder = match method.as_str() {
            "GET" => self.client.get(&target_url),
            "POST" => self.client.post(&target_url),
            "PUT" => self.client.put(&target_url),
            "DELETE" => self.client.delete(&target_url),
            "PATCH" => self.client.patch(&target_url),
            _ => return Err(tube::Error::msg(format!("不支持的HTTP方法: {}", method))),
        };

        // 添加请求头
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        // 添加请求体
        if let Some(body_str) = body {
            request_builder = request_builder.body(body_str);
        }

        let response = request_builder
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| {
                err_log!("代理请求失败: {}", e);
                format!("代理请求失败: {}", e)
            })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            err_log!(
                "代理请求： {target_url} ----> 返回错误状态: {} 内容: {}",
                status,
                response_text
            );
            return Err(tube::Error::msg(format!(
                "代理请求： {target_url} ----> 返回错误状态: {} 内容: {}",
                status, response_text
            )));
        }

        // 解析响应
        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                err_log!("解析代理响应失败: {} 原始内容: {}", e, response_text);
                format!("解析代理响应失败: {}", e)
            })?;

        // 递归处理嵌套的业务状态码
        match Self::process_nested_response(response_json) {
            Ok(response) => Ok(response),
            Err(err) => {
                err_log!("代理请求： {target_url} ----> 业务处理失败: {}", err);
                Err(err)
            }
        }
    }

    /// 递归处理嵌套的业务状态码
    fn process_nested_response(response: serde_json::Value) -> tube::Result<serde_json::Value> {
        // 检查是否有 code 字段
        if let Some(code) = response.get("code").and_then(|v| v.as_u64()) {
            if code == 200 {
                // 成功，检查是否有 result 字段
                if let Some(result) = response.get("result") {
                    // 递归处理 result 中的嵌套结构
                    Self::process_nested_response(result.clone())
                } else {
                    // 没有 result 字段，返回整个响应
                    Ok(response)
                }
            } else {
                // 业务错误
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                Err(tube::Error::msg(format!("业务处理失败: code={}, message={}", code, message)))
            }
        } else {
            // 没有 code 字段，直接返回数据
            Ok(response)
        }
    }
}
