use serde::{Deserialize, Serialize};

///返回的结构封装
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response<T> {
    // #[serde(rename = "Code")]
    code: i32, // 输出代码
    // #[serde(rename = "Content")]
    result: T, // 输出内容
    // #[serde(rename = "Message")]
    message: String, // 消息
}

impl<T> Response<T>
where
    T: Default,
{
    pub fn new(code: i32, result: T, msg: &str) -> Response<T> {
        Response {
            code,
            result,
            message: msg.to_string(),
        }
    }

    pub fn ok(data: T) -> Self {
        Self::new(200, data, "")
    }

    /// 返回错误
    pub fn error(code: i32, msg: &str) -> Self {
        Self::new(code, T::default(), msg)
    }
}

/// 获取输出对象
pub fn get_response(result: tube::Result<serde_json::Value>) -> Response<serde_json::Value> {
    match result {
        Ok(content) => Response::ok(content),
        Err(err) => Response::error(err.get_code(), &err.get_message()),
    }
}

/// 获取正确的response
pub fn get_ok_response(value: serde_json::Value) -> Response<serde_json::Value> {
    Response::ok(value)
}
