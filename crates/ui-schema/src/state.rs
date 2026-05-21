//! copyright © ecdata.cn 2026 - present
//! State 解析与插值辅助
//!
//! 提供从插件 state 中取值、解析路径的辅助函数。

/// 从 state 中按 bind 路径取值（支持 dot notation）
pub fn state_get<'a>(state: &'a serde_json::Value, bind: &str) -> Option<&'a serde_json::Value> {
    let mut current = state;
    for key in bind.split('.') {
        if let Ok(idx) = key.parse::<usize>() {
            current = current.as_array().and_then(|a| a.get(idx))?;
        } else {
            current = current.get(key)?;
        }
    }
    Some(current)
}

/// 从 state 中按 bind 路径取字符串
pub fn state_get_str(state: &serde_json::Value, bind: &str) -> String {
    state_get(state, bind)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 从 state 中按 bind 路径取整数
pub fn state_get_i64(state: &serde_json::Value, bind: &str) -> i64 {
    state_get(state, bind)
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// 简单插值：将 `{field}` 替换为状态中的值
pub fn state_interpolate(state: &serde_json::Value, template: &str) -> String {
    let mut result = template.to_string();
    loop {
        let start = result.find('{');
        let end = result.find('}');
        match (start, end) {
            (Some(s), Some(e)) if s < e => {
                let key = &result[s + 1..e];
                let value = state
                    .get(key)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default();
                result = result.replacen(&format!("{{{}}}", key), &value, 1);
            }
            _ => break,
        }
    }
    result
}
