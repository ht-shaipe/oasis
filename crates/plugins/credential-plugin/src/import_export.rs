//! 凭证管理插件 - 导入导出服务（UI + 业务逻辑）

use crate::models::Credential;
use crate::state::CredentialPluginState;
use anyhow::Result;
use plugin_sdk::{UiNode, UiSchema};
use serde_json::json;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// 导入导出业务服务
// ---------------------------------------------------------------------------

/// 导入导出服务
pub struct ImportExportService;

impl ImportExportService {
    /// 导出为 JSON 格式
    pub fn export_credentials(credentials: &[Credential]) -> Result<String> {
        let mut export_data = Vec::new();
        for cred in credentials {
            export_data.push(json!({
                "id": cred.id,
                "name": cred.name,
                "platform": cred.platform,
                "category": cred.category,
                "username": cred.username,
                "password_encrypted": cred.password_encrypted,
                "extra_fields": cred.extra_fields,
                "notes": cred.notes,
                "is_active": cred.is_active,
                "created_at": cred.created_at,
                "updated_at": cred.updated_at,
                "expires_at": cred.expires_at,
                "tags": cred.tags,
            }));
        }

        let export_json = json!({
            "version": "1.0",
            "exported_at": chrono::Local::now().timestamp(),
            "credentials": export_data,
        });

        Ok(serde_json::to_string_pretty(&export_json)?)
    }

    /// 导出为 CSV 格式
    pub fn export_credentials_csv(credentials: &[Credential]) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("ID,Name,Platform,Category,Username,PasswordEncrypted,ExtraFields,Notes,IsActive,CreatedAt,UpdatedAt,ExpiresAt,Tags\n");

        for cred in credentials {
            let expires_at = cred
                .expires_at
                .map(|ts| ts.to_string())
                .unwrap_or_default();

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},{},\"{}\"\n",
                Self::escape_csv(&cred.id),
                Self::escape_csv(&cred.name),
                Self::escape_csv(&cred.platform),
                Self::escape_csv(&cred.category),
                Self::escape_csv(&cred.username),
                Self::escape_csv(&cred.password_encrypted),
                Self::escape_csv(&cred.extra_fields),
                Self::escape_csv(&cred.notes),
                cred.is_active as u8,
                cred.created_at,
                cred.updated_at,
                expires_at,
                Self::escape_csv(&cred.tags),
            ));
        }

        Ok(csv)
    }

    /// 从 JSON 导入凭证
    pub fn import_credentials(json_str: &str) -> Result<Vec<Credential>> {
        let data: serde_json::Value = serde_json::from_str(json_str)?;

        let credentials_arr = data["credentials"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid import format: missing 'credentials' array"))?;

        let mut credentials = Vec::new();
        for cred_json in credentials_arr {
            let cred = Credential {
                id: cred_json["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing id"))?
                    .to_string(),
                name: cred_json["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing name"))?
                    .to_string(),
                platform: cred_json["platform"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing platform"))?
                    .to_string(),
                credential_type: cred_json["credential_type"]
                    .as_str()
                    .unwrap_or("api_key")
                    .to_string(),
                category: cred_json["category"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                username: cred_json["username"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing username"))?
                    .to_string(),
                password_encrypted: cred_json["password_encrypted"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing password_encrypted"))?
                    .to_string(),
                extra_fields: cred_json["extra_fields"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                notes: cred_json["notes"].as_str().unwrap_or("").to_string(),
                is_active: cred_json["is_active"].as_bool().unwrap_or(true),
                created_at: cred_json["created_at"]
                    .as_i64()
                    .ok_or_else(|| anyhow::anyhow!("Missing created_at"))?,
                updated_at: cred_json["updated_at"]
                    .as_i64()
                    .ok_or_else(|| anyhow::anyhow!("Missing updated_at"))?,
                expires_at: cred_json["expires_at"].as_i64(),
                tags: cred_json["tags"].as_str().unwrap_or("").to_string(),
            };
            credentials.push(cred);
        }

        Ok(credentials)
    }

    /// 从 CSV 导入凭证
    pub fn import_credentials_csv(csv_str: &str) -> Result<Vec<Credential>> {
        let mut credentials = Vec::new();
        let mut lines = csv_str.lines();

        // Skip header
        lines.next();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let fields = Self::parse_csv_line(line)?;
            if fields.len() < 14 {
                anyhow::bail!("Invalid CSV format: expected at least 14 fields");
            }

            let cred = Credential {
                id: fields[0].clone(),
                name: fields[1].clone(),
                platform: fields[2].clone(),
                category: fields[3].clone(),
                credential_type: fields[12].clone(),
                username: fields[4].clone(),
                password_encrypted: fields[5].clone(),
                extra_fields: fields[6].clone(),
                notes: fields[7].clone(),
                is_active: fields[8].parse().unwrap_or(true),
                created_at: fields[9].parse()?,
                updated_at: fields[10].parse()?,
                expires_at: if fields[11].is_empty() {
                    None
                } else {
                    Some(fields[11].parse()?)
                },
                tags: fields[12].clone(),
            };
            credentials.push(cred);
        }

        Ok(credentials)
    }

    fn escape_csv(s: &str) -> String {
        s.replace('"', "\"\"")
    }

    fn parse_csv_line(line: &str) -> Result<Vec<String>> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => current_field.push(c),
            }
        }

        fields.push(current_field.trim().to_string());
        Ok(fields)
    }
}

// ---------------------------------------------------------------------------
// 导入导出UI界面
// ---------------------------------------------------------------------------

/// 构建导入导出页面的UI schema
pub fn schema_import_export() -> UiSchema {
    let import_section = UiNode::new("card")
        .prop("title", json!("导入凭证"))
        .child(
            UiNode::new("flex-col")
                .prop("gap", json!(12))
                .child(UiNode::input("import_export.import_path", "文件路径（.json / .csv）"))
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", json!(8))
                        .child(UiNode::button("导入 JSON", "import_json").prop("variant", json!("primary")))
                        .child(UiNode::button("导入 CSV", "import_csv"))
                )
                .child(UiNode::display("import_export.import_result").prop("type", json!("info")))
        );

    let export_section = UiNode::new("card")
        .prop("title", json!("导出凭证"))
        .prop("margin_top", json!(16))
        .child(
            UiNode::new("flex-col")
                .prop("gap", json!(12))
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", json!(8))
                        .child(UiNode::button("导出全部 JSON", "export_all_json").prop("variant", json!("primary")))
                        .child(UiNode::button("导出全部 CSV", "export_all_csv"))
                )
                .child(
                    UiNode::new("flex-row")
                        .prop("gap", json!(8))
                        .child(UiNode::button("导出选中 JSON", "export_selected_json"))
                        .child(UiNode::button("导出选中 CSV", "export_selected_csv"))
                )
                .child(UiNode::display("import_export.export_result").prop("type", json!("info")))
        );

    UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![import_section, export_section],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 导入导出业务处理器
// ---------------------------------------------------------------------------

/// 导入导出业务处理器
pub struct ImportExportHandler {
    pub state: Arc<Mutex<CredentialPluginState>>,
    pub credential_service: Option<crate::service::CredentialService>,
}

impl ImportExportHandler {
    pub fn new(
        state: Arc<Mutex<CredentialPluginState>>,
        credential_service: Option<crate::service::CredentialService>,
    ) -> Self {
        Self {
            state,
            credential_service,
        }
    }

    /// 导入JSON文件
    pub fn import_json(&self, file_path: &str) -> serde_json::Value {
        use std::fs;

        // 读取文件内容
        let json_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                log::error!("读取文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = format!("读取文件失败: {}", e);
                });
                return json!({"success": false, "message": format!("读取文件失败: {}", e)});
            }
        };

        // 解析导入数据
        let credentials = match ImportExportService::import_credentials(&json_content) {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("解析JSON失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = format!("解析JSON失败: {}", e);
                });
                return json!({"success": false, "message": format!("解析JSON失败: {}", e)});
            }
        };

        // 批量导入到数据库
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                let error_msg = "凭证服务未初始化".to_string();
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = error_msg.clone();
                });
                return json!({"success": false, "message": error_msg});
            }
        };

        let mut imported_count = 0;
        for credential in credentials {
            if service.create(credential).is_ok() {
                imported_count += 1;
            }
        }

        let success_msg = format!("成功导入 {} 个凭证", imported_count);
        log::info!("{}", success_msg);
        let _ = self.state.lock().map(|mut s| {
            s.import_export.import_result = success_msg.clone();
        });

        json!({"success": true, "message": success_msg})
    }

    /// 导入CSV文件
    pub fn import_csv(&self, file_path: &str) -> serde_json::Value {
        use std::fs;

        // 读取文件内容
        let csv_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                log::error!("读取文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = format!("读取文件失败: {}", e);
                });
                return json!({"success": false, "message": format!("读取文件失败: {}", e)});
            }
        };

        // 解析导入数据
        let credentials = match ImportExportService::import_credentials_csv(&csv_content) {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("解析CSV失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = format!("解析CSV失败: {}", e);
                });
                return json!({"success": false, "message": format!("解析CSV失败: {}", e)});
            }
        };

        // 批量导入到数据库
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                let error_msg = "凭证服务未初始化".to_string();
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.import_result = error_msg.clone();
                });
                return json!({"success": false, "message": error_msg});
            }
        };

        let mut imported_count = 0;
        for credential in credentials {
            if service.create(credential).is_ok() {
                imported_count += 1;
            }
        }

        let success_msg = format!("成功导入 {} 个凭证", imported_count);
        log::info!("{}", success_msg);
        let _ = self.state.lock().map(|mut s| {
            s.import_export.import_result = success_msg.clone();
        });

        json!({"success": true, "message": success_msg})
    }

    /// 导出全部为JSON
    pub fn export_all_json(&self, file_path: &str) -> serde_json::Value {
        use std::fs;

        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                return json!({"success": false, "message": "凭证服务未初始化"});
            }
        };

        // 获取所有凭证
        let credentials = match service.list_all() {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("获取凭证列表失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("获取凭证列表失败: {}", e);
                });
                return json!({"success": false, "message": format!("获取凭证列表失败: {}", e)});
            }
        };

        // 导出为JSON格式
        let json_content = match ImportExportService::export_credentials(&credentials) {
            Ok(content) => content,
            Err(e) => {
                log::error!("导出JSON失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("导出JSON失败: {}", e);
                });
                return json!({"success": false, "message": format!("导出JSON失败: {}", e)});
            }
        };

        // 写入文件
        match fs::write(file_path, json_content) {
            Ok(_) => {
                let success_msg = format!("成功导出 {} 个凭证到JSON文件", credentials.len());
                log::info!("{}", success_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = success_msg.clone();
                });
                json!({"success": true, "message": success_msg})
            }
            Err(e) => {
                log::error!("写入文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("写入文件失败: {}", e);
                });
                json!({"success": false, "message": format!("写入文件失败: {}", e)})
            }
        }
    }

    /// 导出全部为CSV
    pub fn export_all_csv(&self, file_path: &str) -> serde_json::Value {
        use std::fs;

        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                return json!({"success": false, "message": "凭证服务未初始化"});
            }
        };

        // 获取所有凭证
        let credentials = match service.list_all() {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("获取凭证列表失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("获取凭证列表失败: {}", e);
                });
                return json!({"success": false, "message": format!("获取凭证列表失败: {}", e)});
            }
        };

        // 导出为CSV格式
        let csv_content = match ImportExportService::export_credentials_csv(&credentials) {
            Ok(content) => content,
            Err(e) => {
                log::error!("导出CSV失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("导出CSV失败: {}", e);
                });
                return json!({"success": false, "message": format!("导出CSV失败: {}", e)});
            }
        };

        // 写入文件
        match fs::write(file_path, csv_content) {
            Ok(_) => {
                let success_msg = format!("成功导出 {} 个凭证到CSV文件", credentials.len());
                log::info!("{}", success_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = success_msg.clone();
                });
                json!({"success": true, "message": success_msg})
            }
            Err(e) => {
                log::error!("写入文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("写入文件失败: {}", e);
                });
                json!({"success": false, "message": format!("写入文件失败: {}", e)})
            }
        }
    }

    /// 导出选中项为JSON
    pub fn export_selected_json(&self, file_path: &str, selected_ids: &[String]) -> serde_json::Value {
        use std::fs;

        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                return json!({"success": false, "message": "凭证服务未初始化"});
            }
        };

        // 获取选中的凭证
        let mut credentials = Vec::new();
        for id in selected_ids {
            if let Ok(cred) = service.read(id) {
                credentials.push(cred);
            }
        }

        if credentials.is_empty() {
            let error_msg = "没有找到选中的凭证".to_string();
            let _ = self.state.lock().map(|mut s| {
                s.import_export.export_result = error_msg.clone();
            });
            return json!({"success": false, "message": error_msg});
        }

        // 导出为JSON格式
        let json_content = match ImportExportService::export_credentials(&credentials) {
            Ok(content) => content,
            Err(e) => {
                log::error!("导出JSON失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("导出JSON失败: {}", e);
                });
                return json!({"success": false, "message": format!("导出JSON失败: {}", e)});
            }
        };

        // 写入文件
        match fs::write(file_path, json_content) {
            Ok(_) => {
                let success_msg = format!("成功导出 {} 个凭证到JSON文件", credentials.len());
                log::info!("{}", success_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = success_msg.clone();
                });
                json!({"success": true, "message": success_msg})
            }
            Err(e) => {
                log::error!("写入文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("写入文件失败: {}", e);
                });
                json!({"success": false, "message": format!("写入文件失败: {}", e)})
            }
        }
    }

    /// 导出选中项为CSV
    pub fn export_selected_csv(&self, file_path: &str, selected_ids: &[String]) -> serde_json::Value {
        use std::fs;

        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                return json!({"success": false, "message": "凭证服务未初始化"});
            }
        };

        // 获取选中的凭证
        let mut credentials = Vec::new();
        for id in selected_ids {
            if let Ok(cred) = service.read(id) {
                credentials.push(cred);
            }
        }

        if credentials.is_empty() {
            let error_msg = "没有找到选中的凭证".to_string();
            let _ = self.state.lock().map(|mut s| {
                s.import_export.export_result = error_msg.clone();
            });
            return json!({"success": false, "message": error_msg});
        }

        // 导出为CSV格式
        let csv_content = match ImportExportService::export_credentials_csv(&credentials) {
            Ok(content) => content,
            Err(e) => {
                log::error!("导出CSV失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("导出CSV失败: {}", e);
                });
                return json!({"success": false, "message": format!("导出CSV失败: {}", e)});
            }
        };

        // 写入文件
        match fs::write(file_path, csv_content) {
            Ok(_) => {
                let success_msg = format!("成功导出 {} 个凭证到CSV文件", credentials.len());
                log::info!("{}", success_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = success_msg.clone();
                });
                json!({"success": true, "message": success_msg})
            }
            Err(e) => {
                log::error!("写入文件失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.import_export.export_result = format!("写入文件失败: {}", e);
                });
                json!({"success": false, "message": format!("写入文件失败: {}", e)})
            }
        }
    }
}
