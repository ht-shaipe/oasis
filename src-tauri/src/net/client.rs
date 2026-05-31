//! copyright © ecdata.cn 2024 - present
//!
//! created shaipe by 2024-12-12 07:48:25

use tube::{Result, Value};

#[derive(Debug)]
pub struct Client {
    partner_id: String,

    secret: String,

    url: String,

    parse_response: fn(Result<String>) -> Result<Value>,
}

#[allow(dead_code)]
impl Client {
    /// 创建一个新的客户端请求对象
    pub fn new(conf: &Value) -> Self {
        Client {
            url: conf.get_def_string("url", "https://paas.ecdata.cn/api"),
            partner_id: conf.get_def_string("app_id", "htui_agent"),
            secret: conf.get_def_string("secret", "6dde207319663ebf9b2f1f9a48039e4b"),
            parse_response: Self::def_parse_response,
        }
    }

    /// 设置请求地址
    pub fn set_url(&mut self, url: &str) -> &mut Self {
        self.url = url.to_owned();
        self
    }

    /// 设置请求密钥
    pub fn set_secret(&mut self, secret: &str) -> &mut Self {
        self.secret = secret.to_owned();
        self
    }

    /// 设置应用id
    pub fn set_partner_id(&mut self, id: &str) -> &mut Self {
        self.partner_id = id.to_owned();
        self
    }

    /// 设置解析函数
    pub fn set_parse_response(
        &mut self,
        parse_response: fn(Result<String>) -> Result<Value>,
    ) -> &mut Self {
        self.parse_response = parse_response;
        self
    }

    /// 获取请求客户端
    fn get_client(&self) -> tube::net::Client {
        tube::net::Client::new("").timeout(10000)
    }

    /// 以Post的方式请求数据
    pub fn post(&self, service: &str, data: &serde_json::Value) -> Result<Value> {
        let url = format!("{}/{}", self.url, service);

        log!("request url {} , data {data:?}", url);
        let res = self.get_client().post(&url, &data);
        let parse_fn = self.parse_response;
        parse_fn(res)
    }

    /// 用get的方式获取数据
    pub fn get(&self, service: &str, data: &Value) -> Result<Value> {
        // 拼接参数
        let url_params = data.to_url_params();

        // 拼接url
        let url = format!("{}{service}?{}", self.url, url_params);
        let res = self.get_client().get(&url);
        let parse_fn = self.parse_response;
        parse_fn(res)
    }

    /// 对响应值进行解析
    /// 接口返回的数据结构如下：
    /// {
    /// 	data: {},
    /// 	extra: {
    /// 		error_code: 0,
    /// 		description: ""
    /// 	}
    /// }
    fn def_parse_response(res: Result<String>) -> Result<Value> {
        match res {
            Ok(s) => {
                log!("request res {}", s);
                let val = Value::from_str(&s).unwrap_or(Value::Null);
                // let code = val["resultCode"].to_string();
                // if code == "EXECUTE_SUCCESS" {
                //     Ok(val["resultData"].clone())
                // } else {
                //     let msg = val.get_string("resultMessage");
                //     Err(msg.into())
                // }
                Ok(val["result"].clone())
            }
            Err(err) => Err(err),
        }
    }
}
