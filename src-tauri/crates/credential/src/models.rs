use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialView {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialDetail {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data: SensitiveData,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct NewCredential {
    pub category_id: i64,
    pub title: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data_json: String,
    #[serde(alias = "dekBase64")]
    pub dek_base64: String,
    #[serde(alias = "nonceBase64")]
    pub nonce_base64: String,
    pub tags: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateCredential {
    pub id: i64,
    pub category_id: Option<i64>,
    pub title: Option<String>,
    pub username: Option<String>,
    pub url: Option<String>,
    pub sensitive_data_json: Option<String>,
    #[serde(alias = "dekBase64")]
    pub dek_base64: Option<String>,
    #[serde(alias = "nonceBase64")]
    pub nonce_base64: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
}
