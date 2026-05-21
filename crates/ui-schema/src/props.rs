//! copyright © ecdata.cn 2026 - present
//! Props 解析辅助
//!
//! 提供从 serde_json::Value 中提取常用类型值的辅助函数。

/// 从 props JSON 中取字符串字段
pub fn prop_str<'a>(props: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    props.get(key).and_then(|v| v.as_str())
}

/// 从 props JSON 中取字符串字段，带默认值
pub fn prop_str_or<'a>(props: &'a serde_json::Value, key: &str, default: &'a str) -> &'a str {
    prop_str(props, key).unwrap_or(default)
}

/// 从 props JSON 中取整数
pub fn prop_i64(props: &serde_json::Value, key: &str) -> Option<i64> {
    props.get(key).and_then(|v| v.as_i64())
}

/// 从 props JSON 中取布尔值
pub fn prop_bool(props: &serde_json::Value, key: &str) -> Option<bool> {
    props.get(key).and_then(|v| v.as_bool())
}

/// 从 props JSON 中取数组
pub fn prop_array<'a>(props: &'a serde_json::Value, key: &str) -> Option<&'a Vec<serde_json::Value>> {
    props.get(key).and_then(|v| v.as_array())
}
