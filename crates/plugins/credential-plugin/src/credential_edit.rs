//! 凭证编辑/创建功能 - UI和业务逻辑整合

use crate::models::Credential;
use crate::service::CredentialService;
use crate::state::{CredentialPluginState, CredentialType, ToolId};
use anyhow::Result;
use plugin_sdk::{UiNode, UiSchema};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// 凭证编辑UI界面
// ---------------------------------------------------------------------------

/// 构建凭证编辑/创建页面的UI schema
pub fn schema_credential_edit(is_new: bool, cred_type: &CredentialType) -> UiSchema {
    let title = if is_new { "新建凭证" } else { "编辑凭证" };

    // 通用字段：名称
    let name_field = UiNode::input("credential_edit.credential.name", "名称")
        .prop("required", serde_json::json!(true))
        .prop("width", serde_json::json!("100%"));

    // 类型选择按钮组
    let type_buttons: Vec<UiNode> = CredentialType::all()
        .iter()
        .map(|(label, _value)| {
            let is_active = cred_type.label() == *label;
            let variant = if is_active {
                serde_json::json!("outline")    // 选中状态使用 outline
            } else {
                serde_json::json!("primary")    // 未选中状态使用 primary
            };
            UiNode::button(*label, &format!("change_type_to_{}", *label))
                .prop("variant", variant)
        })
        .collect();

    let type_button_group = UiNode::new("button_group")
        .prop("gap", serde_json::json!(0))
        .prop("margin_top", serde_json::json!(12))
        .children(type_buttons);

    let common_fields = UiNode::new("card")
        .prop("title", serde_json::json!(title))
        .prop("gap", serde_json::json!(16))
        .child(name_field)
        .child(
            UiNode::new("flex-col")
                .prop("gap", serde_json::json!(8))
                .prop("margin_top", serde_json::json!(12))
                .child(UiNode::label("类型："))
                .child(type_button_group)
        );

    // 根据类型生成不同的字段
    let type_specific_fields = match cred_type {
        CredentialType::ApiKey => UiNode::new("card")
            .prop("title", serde_json::json!("接口密钥"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(UiNode::input("credential_edit.credential.api_key_value", "API Key").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.api_secret", "API Secret").prop("type", serde_json::json!("password")))
            .child(UiNode::input("credential_edit.credential.api_endpoint", "接口地址"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::WebsiteUser => UiNode::new("card")
            .prop("title", serde_json::json!("网站用户"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(12))
                    .child(UiNode::input("credential_edit.credential.platform", "网站/平台").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.category", "分类").prop("width", serde_json::json!("50%")))
            )
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(12))
                    .child(UiNode::input("credential_edit.credential.username", "用户名").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.password_masked", "密码").prop("required", serde_json::json!(true)).prop("type", serde_json::json!("password")).prop("width", serde_json::json!("50%")))
            )
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::SshKey => UiNode::new("card")
            .prop("title", serde_json::json!("SSH 密钥"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(UiNode::input("credential_edit.credential.ssh_private_key", "私钥").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.ssh_public_key", "公钥"))
            .child(UiNode::input("credential_edit.credential.username", "用户名"))
            .child(UiNode::input("credential_edit.credential.api_endpoint", "主机地址"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Database => UiNode::new("card")
            .prop("title", serde_json::json!("数据库"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(12))
                    .child(UiNode::input("credential_edit.credential.db_host", "主机").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("60%")))
                    .child(UiNode::input("credential_edit.credential.db_port", "端口").prop("width", serde_json::json!("20%")))
                    .child(UiNode::input("credential_edit.credential.db_name", "数据库名").prop("width", serde_json::json!("20%")))
            )
            .child(
                UiNode::new("flex-row")
                    .prop("gap", serde_json::json!(12))
                    .child(UiNode::input("credential_edit.credential.username", "用户名").prop("required", serde_json::json!(true)).prop("width", serde_json::json!("50%")))
                    .child(UiNode::input("credential_edit.credential.password_masked", "密码").prop("required", serde_json::json!(true)).prop("type", serde_json::json!("password")).prop("width", serde_json::json!("50%")))
            )
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Certificate => UiNode::new("card")
            .prop("title", serde_json::json!("证书"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(UiNode::input("credential_edit.credential.cert_path", "证书路径").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.password_masked", "证书密码").prop("type", serde_json::json!("password")))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),

        CredentialType::Token => UiNode::new("card")
            .prop("title", serde_json::json!("令牌"))
            .prop("gap", serde_json::json!(16))
            .prop("margin_top", serde_json::json!(16))
            .child(UiNode::input("credential_edit.credential.token_value", "Token").prop("required", serde_json::json!(true)))
            .child(UiNode::input("credential_edit.credential.platform", "平台/服务"))
            .child(UiNode::input("credential_edit.credential.tags", "标签（逗号分隔）"))
            .child(UiNode::input("credential_edit.credential.notes", "备注")),
    };

    // 启用状态和操作按钮放在同一个区域
    let status_row = UiNode::new("flex-row")
        .prop("gap", serde_json::json!(12))
        .prop("align_items", serde_json::json!("center"))
        .prop("margin_top", serde_json::json!(16))
        .child(UiNode::label("启用状态"))
        .child(
            UiNode::new("switch")
                .bind("credential_edit.credential.is_active")
                .on_action("toggle_active_status")
        )
        .child(
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .prop("margin_left", serde_json::json!("auto"))
                .child(UiNode::button("取消", "cancel_edit"))
                .child(UiNode::button("保存", "save_credential").prop("variant", serde_json::json!("primary")))
        );

    UiSchema {
        layout: "flex-col".into(),
        gap: 16,
        children: vec![common_fields, type_specific_fields, status_row],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 凭证编辑业务处理器
// ---------------------------------------------------------------------------

/// 凭证编辑业务处理器
pub struct CredentialEditHandler {
    pub state: Arc<Mutex<CredentialPluginState>>,
    pub credential_service: Option<CredentialService>,
}

impl CredentialEditHandler {
    pub fn new(
        state: Arc<Mutex<CredentialPluginState>>,
        credential_service: Option<CredentialService>,
    ) -> Self {
        Self {
            state,
            credential_service,
        }
    }

    /// 初始化新建凭证表单
    pub fn init_create(&self) {
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = ToolId::CredentialCreate;
            s.credential_edit.is_new = true;
            s.credential_edit.credential = Default::default();
            s.credential_edit.type_display = CredentialType::default().label().to_string();
            s.credential_edit.validation_errors = Vec::new();
        });
    }

    /// 初始化编辑凭证表单
    pub fn init_edit(&self, credential_id: &str) {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                log::error!("凭证服务未初始化");
                return;
            }
        };

        // 从数据库加载凭证数据
        let credential = match service.read(credential_id) {
            Ok(cred) => cred,
            Err(e) => {
                log::error!("加载凭证失败: {}", e);
                let error_msg = format!("加载凭证失败: {}", e);
                let _ = self.state.lock().map(|mut s| {
                    s.selected_tool = ToolId::CredentialEdit;
                    s.credential_edit.validation_errors = vec![error_msg];
                });
                return;
            }
        };

        // 将数据库凭证转换为UI凭证项
        let credential_item = crate::state::CredentialItem {
            id: credential.id,
            name: credential.name,
            platform: credential.platform,
            category: credential.category,
            username: credential.username,
            password_masked: "********".to_string(),
            notes: credential.notes,
            is_active: credential.is_active,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
            expires_at: credential.expires_at,
            tags: credential.tags,
            extra_fields: credential.extra_fields,
            credential_type: CredentialType::from_value(&credential.credential_type),
            ..Default::default()
        };

        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = ToolId::CredentialEdit;
            s.credential_edit.is_new = false;
            s.credential_edit.credential = credential_item;
            s.credential_edit.type_display = s.credential_edit.credential.credential_type.label().to_string();
            s.credential_edit.validation_errors = Vec::new();
        });
    }

    /// 切换凭证类型
    pub fn change_type(&self, type_label: &str) {
        let new_type = CredentialType::from_label(type_label);
        let _ = self.state.lock().map(|mut s| {
            s.credential_edit.credential.credential_type = new_type.clone();
            s.credential_edit.type_display = new_type.label().to_string();
            // 清空之前的保存消息
            s.credential_edit.validation_errors = Vec::new();
        });
    }

    /// 切换启用状态
    pub fn toggle_active_status(&self) {
        let _ = self.state.lock().map(|mut s| {
            s.credential_edit.credential.is_active = !s.credential_edit.credential.is_active;
            s.credential_edit.is_active_display =
                if s.credential_edit.credential.is_active { "已启用" } else { "已禁用" }.to_string();
        });
    }

    /// 保存凭证（新建或更新）
    pub fn save_credential(&self) -> serde_json::Value {
        let service = match &self.credential_service {
            Some(s) => s,
            None => {
                let error_msg = "凭证服务未初始化".to_string();
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.validation_errors = vec![error_msg.clone()];
                });
                return serde_json::json!({"success": false, "message": error_msg});
            }
        };

        // 获取当前编辑的凭证数据
        let (credential_data, is_new) = match self.state.lock().ok().map(|s| {
            let data = s.credential_edit.credential.clone();
            let is_new = s.credential_edit.is_new;
            (data, is_new)
        }) {
            Some((data, is_new)) => (data, is_new),
            None => {
                let error_msg = "无法获取凭证数据".to_string();
                return serde_json::json!({"success": false, "message": error_msg});
            }
        };

        // 验证必填字段
        if credential_data.name.trim().is_empty() {
            let error_msg = "凭证名称不能为空".to_string();
            let _ = self.state.lock().map(|mut s| {
                s.credential_edit.validation_errors = vec![error_msg.clone()];
            });
            return serde_json::json!({"success": false, "message": error_msg});
        }

        // 根据不同类型验证特定字段
        if let Err(error) = self.validate_credential_fields(&credential_data) {
            let error_msg = error.to_string();
            let _ = self.state.lock().map(|mut s| {
                s.credential_edit.validation_errors = vec![error_msg.clone()];
            });
            return serde_json::json!({"success": false, "message": error_msg});
        }

        // 转换为数据库凭证格式
        let credential = self.convert_to_credential(credential_data);

        // 执行保存操作
        let result = if is_new {
            // 新建凭证
            service.create(credential).map(|_| ())
        } else {
            // 更新凭证
            let credential_id = credential.id.clone();
            service.update(&credential_id, credential)
        };

        match result {
            Ok(_) => {
                let success_msg = if is_new {
                    "凭证创建成功".to_string()
                } else {
                    "凭证更新成功".to_string()
                };

                log::info!("{}", success_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.validation_errors = Vec::new();
                    s.selected_tool = ToolId::CredentialList;
                });

                serde_json::json!({"success": true, "message": success_msg})
            }
            Err(e) => {
                let error_msg = format!("保存失败: {}", e);
                log::error!("{}", error_msg);
                let _ = self.state.lock().map(|mut s| {
                    s.credential_edit.validation_errors = vec![error_msg.clone()];
                });
                serde_json::json!({"success": false, "message": error_msg})
            }
        }
    }

    /// 取消编辑
    pub fn cancel_edit(&self) {
        let _ = self.state.lock().map(|mut s| {
            s.selected_tool = ToolId::CredentialList;
            s.credential_edit.validation_errors = Vec::new();
        });
    }

    /// 验证凭证字段
    fn validate_credential_fields(&self, credential: &crate::state::CredentialItem) -> Result<()> {
        match &credential.credential_type {
            CredentialType::ApiKey => {
                if credential.api_key_value.trim().is_empty() {
                    anyhow::bail!("API Key不能为空");
                }
            }
            CredentialType::WebsiteUser => {
                if credential.platform.trim().is_empty() {
                    anyhow::bail!("网站/平台不能为空");
                }
                if credential.username.trim().is_empty() {
                    anyhow::bail!("用户名不能为空");
                }
                if credential.password_masked.trim().is_empty() || credential.password_masked == "********" {
                    anyhow::bail!("密码不能为空");
                }
            }
            CredentialType::SshKey => {
                if credential.ssh_private_key.trim().is_empty() {
                    anyhow::bail!("私钥不能为空");
                }
            }
            CredentialType::Database => {
                if credential.db_host.trim().is_empty() {
                    anyhow::bail!("数据库主机不能为空");
                }
                if credential.username.trim().is_empty() {
                    anyhow::bail!("数据库用户名不能为空");
                }
                if credential.password_masked.trim().is_empty() || credential.password_masked == "********" {
                    anyhow::bail!("数据库密码不能为空");
                }
            }
            CredentialType::Certificate => {
                if credential.cert_path.trim().is_empty() {
                    anyhow::bail!("证书路径不能为空");
                }
            }
            CredentialType::Token => {
                if credential.token_value.trim().is_empty() {
                    anyhow::bail!("Token不能为空");
                }
            }
        }

        Ok(())
    }

    /// 将UI凭证项转换为数据库凭证格式
    fn convert_to_credential(&self, item: crate::state::CredentialItem) -> Credential {
        // 根据凭证类型构建extra_fields
        let extra_fields = match &item.credential_type {
            CredentialType::ApiKey => serde_json::json!({
                "api_key_value": item.api_key_value,
                "api_secret": item.api_secret,
                "api_endpoint": item.api_endpoint,
            }).to_string(),
            CredentialType::WebsiteUser => serde_json::json!({
                "platform": item.platform,
                "category": item.category,
            }).to_string(),
            CredentialType::SshKey => serde_json::json!({
                "ssh_private_key": item.ssh_private_key,
                "ssh_public_key": item.ssh_public_key,
                "api_endpoint": item.api_endpoint,
            }).to_string(),
            CredentialType::Database => serde_json::json!({
                "db_host": item.db_host,
                "db_port": item.db_port,
                "db_name": item.db_name,
            }).to_string(),
            CredentialType::Certificate => serde_json::json!({
                "cert_path": item.cert_path,
            }).to_string(),
            CredentialType::Token => serde_json::json!({
                "token_value": item.token_value,
                "platform": item.platform,
            }).to_string(),
        };

        // 获取密码（处理掩码情况）
        let password = if item.password_masked == "********" {
            // 如果是编辑模式且密码未更改，需要从原数据获取
            // 这里简化处理，实际应该从数据库或加密存储中获取
            item.password_masked.clone()
        } else {
            item.password_masked.clone()
        };

        Credential {
            id: if item.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                item.id
            },
            name: item.name,
            credential_type: item.credential_type.value().to_string(),
            platform: item.platform,
            category: item.category,
            username: item.username,
            password_encrypted: password, // TODO: 实际应该使用加密
            extra_fields,
            notes: item.notes,
            is_active: item.is_active,
            created_at: if item.created_at == 0 {
                chrono::Local::now().timestamp()
            } else {
                item.created_at
            },
            updated_at: chrono::Local::now().timestamp(),
            expires_at: item.expires_at,
            tags: item.tags,
        }
    }

    /// 加载凭证数据到编辑表单（用于编辑模式）
    pub fn load_credential_for_edit(&self, credential_id: &str) -> Result<()> {
        let service = self.credential_service.as_ref()
            .ok_or_else(|| anyhow::anyhow!("凭证服务未初始化"))?;

        let credential = service.read(credential_id)?;

        // 转换为UI凭证项
        let credential_item = crate::state::CredentialItem {
            id: credential.id.clone(),
            name: credential.name.clone(),
            platform: credential.platform.clone(),
            category: credential.category.clone(),
            username: credential.username.clone(),
            password_masked: "********".to_string(), // 密码显示掩码
            notes: credential.notes.clone(),
            is_active: credential.is_active,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
            expires_at: credential.expires_at,
            tags: credential.tags.clone(),
            extra_fields: credential.extra_fields.clone(),
            credential_type: CredentialType::from_value(&credential.credential_type),
            ..Default::default()
        };

        // 解析extra_fields
        if let Ok(extra) = serde_json::from_str::<serde_json::Value>(&credential.extra_fields) {
            let _ = self.state.lock().map(|mut s| {
                s.credential_edit.credential = credential_item.clone();

                // 根据类型设置特定字段
                match s.credential_edit.credential.credential_type {
                    CredentialType::ApiKey => {
                        s.credential_edit.credential.api_key_value = extra["api_key_value"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.api_secret = extra["api_secret"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.api_endpoint = extra["api_endpoint"].as_str().unwrap_or("").to_string();
                    }
                    CredentialType::WebsiteUser => {
                        s.credential_edit.credential.platform = extra["platform"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.category = extra["category"].as_str().unwrap_or("").to_string();
                    }
                    CredentialType::SshKey => {
                        s.credential_edit.credential.ssh_private_key = extra["ssh_private_key"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.ssh_public_key = extra["ssh_public_key"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.api_endpoint = extra["api_endpoint"].as_str().unwrap_or("").to_string();
                    }
                    CredentialType::Database => {
                        s.credential_edit.credential.db_host = extra["db_host"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.db_port = extra["db_port"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.db_name = extra["db_name"].as_str().unwrap_or("").to_string();
                    }
                    CredentialType::Certificate => {
                        s.credential_edit.credential.cert_path = extra["cert_path"].as_str().unwrap_or("").to_string();
                    }
                    CredentialType::Token => {
                        s.credential_edit.credential.token_value = extra["token_value"].as_str().unwrap_or("").to_string();
                        s.credential_edit.credential.platform = extra["platform"].as_str().unwrap_or("").to_string();
                    }
                }
            });
        } else {
            let _ = self.state.lock().map(|mut s| {
                s.credential_edit.credential = credential_item;
            });
        }

        Ok(())
    }
}
