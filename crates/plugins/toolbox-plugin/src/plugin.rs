//! ToolboxPlugin 结构体 + Plugin trait 实现

use std::sync::RwLock;

use plugin_sdk::{Plugin, PluginMeta, UiSchema};

use crate::state::*;
use crate::tools::*;

/// 工具箱插件
pub struct ToolboxPlugin {
    state: RwLock<ToolboxState>,
}

impl ToolboxPlugin {
    pub fn new() -> Self {
        Self { state: RwLock::new(ToolboxState::default()) }
    }

    fn selected_tool(&self) -> ToolId {
        self.state.read().unwrap().selected_tool.clone()
    }

    /// 构建左侧边栏菜单
    fn build_sidebar(current_tool: ToolId) -> plugin_sdk::UiNode {
        use plugin_sdk::UiNode;

        let menu_items = vec![
            ("主页", ToolId::Home),
            ("CSV 统计", ToolId::CsvStats),
            ("CSV 分割", ToolId::CsvSplit),
            ("CSV 转换", ToolId::CsvExcelConvert),
            ("批量重命名", ToolId::BatchRename),
            ("Excel 移动文件", ToolId::ExcelMoveFiles),
            ("API 请求", ToolId::ApiRequest),
            ("批量下载", ToolId::ApiBatchDownload),
            ("JSON 转换", ToolId::JsonToCsvExcel),
            ("JSON 合并", ToolId::JsonMerge),
            ("网络扫描", ToolId::NetworkScan),
            ("UI 测试", ToolId::UiSchemaDemo),
        ];

        let items: Vec<UiNode> = menu_items
            .into_iter()
            .map(|(label, tool_id)| {
                let action = match tool_id {
                    ToolId::Home => "Home",
                    ToolId::CsvStats => "CsvStats",
                    ToolId::CsvSplit => "CsvSplit",
                    ToolId::CsvExcelConvert => "CsvExcelConvert",
                    ToolId::BatchRename => "BatchRename",
                    ToolId::ExcelMoveFiles => "ExcelMoveFiles",
                    ToolId::ApiRequest => "ApiRequest",
                    ToolId::ApiBatchDownload => "ApiBatchDownload",
                    ToolId::JsonToCsvExcel => "JsonToCsvExcel",
                    ToolId::JsonMerge => "JsonMerge",
                    ToolId::NetworkScan => "NetworkScan",
                    ToolId::UiSchemaDemo => "UiSchemaDemo",
                };

                let is_active = tool_id == current_tool;

                UiNode::new("nav-item")
                    .prop("label", serde_json::json!(label))
                    .prop("active", serde_json::json!(is_active))
                    .on_action(action)
            })
            .collect();

        // 返回带内边距的菜单容器
        UiNode::new("flex-col")
            .prop("gap", serde_json::json!(4))
            .prop("padding", serde_json::json!(12))
            .children(items)
    }

    /// 根据当前选中工具返回对应 UiSchema
    fn tool_schema(&self) -> UiSchema {
        match self.selected_tool() {
            ToolId::Home => home::schema_home(),
            ToolId::UiSchemaDemo => demo::schema_ui_schema_demo(),
            ToolId::CsvStats => csv::schema_csv_stats(),
            ToolId::CsvSplit => csv::schema_csv_split(),
            ToolId::CsvExcelConvert => csv::schema_csv_convert(),
            ToolId::BatchRename => file::schema_batch_rename(),
            ToolId::ExcelMoveFiles => file::schema_excel_move(),
            ToolId::ApiRequest => api::schema_api_request(),
            ToolId::ApiBatchDownload => api::schema_batch_download(),
            ToolId::JsonToCsvExcel => json::schema_json_convert(),
            ToolId::JsonMerge => json::schema_json_merge(),
            ToolId::NetworkScan => network::schema_network_scan(),
        }
    }
}

impl Default for ToolboxPlugin {
    fn default() -> Self { Self::new() }
}

impl Plugin for ToolboxPlugin {
    fn id(&self) -> &str { "toolbox" }

    fn meta(&self) -> PluginMeta {
        PluginMeta::new(
            "toolbox",
            "🧰",
            "工具箱",
            "实用工具集：CSV 统计/分割、批量重命名、网络扫描等",
            "1.0.0",
        )
    }

    fn state(&self) -> serde_json::Value {
        serde_json::to_value(&*self.state.read().unwrap()).unwrap_or(serde_json::Value::Null)
    }

    fn handle_action(&self, action: &str, params: serde_json::Value) -> serde_json::Value {
        let mut state = self.state.write().unwrap();

        match action {
            // 全局导航
            "Home" => {
                state.selected_tool = ToolId::Home;
            }
            "CsvStats" => state.selected_tool = ToolId::CsvStats,
            "CsvSplit" => state.selected_tool = ToolId::CsvSplit,
            "CsvExcelConvert" => state.selected_tool = ToolId::CsvExcelConvert,
            "BatchRename" => state.selected_tool = ToolId::BatchRename,
            "ExcelMoveFiles" => state.selected_tool = ToolId::ExcelMoveFiles,
            "ApiRequest" => state.selected_tool = ToolId::ApiRequest,
            "ApiBatchDownload" => state.selected_tool = ToolId::ApiBatchDownload,
            "JsonToCsvExcel" => state.selected_tool = ToolId::JsonToCsvExcel,
            "JsonMerge" => state.selected_tool = ToolId::JsonMerge,
            "NetworkScan" => state.selected_tool = ToolId::NetworkScan,
            "UiSchemaDemo" => state.selected_tool = ToolId::UiSchemaDemo,
            "select_tool" => {
                if let Some(tool) = params.get("tool").and_then(|t| t.as_str()) {
                    state.selected_tool = home::parse_tool_id(tool);
                }
            }

            "demo:refresh" => {
                crate::tools::demo::update_demo_state(&mut state, true);
            }
            "demo:toggle" => {
                crate::tools::demo::update_demo_state(&mut state, false);
            }

            // CSV 统计
            "csv_stats:pick_file" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    match csv::read_csv(path) {
                        Ok((headers, rows)) => {
                            let stats = csv::compute_csv_stats(&headers, &rows);
                            state.csv_stats.input_file = Some(path.to_string());
                            state.csv_stats.headers = headers;
                            state.csv_stats.rows = rows;
                            state.csv_stats.stats = stats;
                        }
                        Err(e) => {
                            state.csv_stats.stats = format!("读取失败: {e}");
                        }
                    }
                }
            }

            // CSV 分割
            "csv_split:pick_file" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    match csv::read_csv(path) {
                        Ok((headers, rows)) => {
                            state.csv_split.input_file = Some(path.to_string());
                            state.csv_split.headers = headers;
                            state.csv_split.rows = rows;
                        }
                        Err(_e) => {}
                    }
                }
            }
            "csv_split:set_col" => {
                if let Some(col) = params.get("col").and_then(|c| c.as_u64()) {
                    state.csv_split.split_col = col as usize;
                }
            }
            "csv_split:set_prefix" => {
                if let Some(prefix) = params.get("prefix").and_then(|p| p.as_str()) {
                    state.csv_split.output_prefix = prefix.to_string();
                }
            }
            "csv_split:execute" => {
                let (headers, rows, col, prefix) = (
                    state.csv_split.headers.clone(),
                    state.csv_split.rows.clone(),
                    state.csv_split.split_col,
                    state.csv_split.output_prefix.clone(),
                );
                drop(state);
                let result = csv::do_csv_split(&headers, &rows, col, &prefix);
                self.state.write().unwrap().csv_split.input_file = Some(result);
                return self.state();
            }

            // 批量重命名
            "rename:pick_dir" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.batch_rename.dir = Some(path.to_string());
                }
            }
            "rename:set_needle" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.batch_rename.needle = v.to_string();
                }
            }
            "rename:set_replacement" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.batch_rename.replacement = v.to_string();
                }
            }
            "rename:set_recursive" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_bool()) {
                    state.batch_rename.recursive = v;
                }
            }
            "rename:preview" => {
                let dir = state.batch_rename.dir.clone();
                let needle = state.batch_rename.needle.clone();
                let replacement = state.batch_rename.replacement.clone();
                let recursive = state.batch_rename.recursive;
                if dir.is_none() {
                    state.batch_rename.preview = "请先选择目录".to_string();
                } else {
                    drop(state);
                    let preview = file::do_rename_preview(dir.as_deref(), &needle, &replacement, recursive);
                    self.state.write().unwrap().batch_rename.preview = preview;
                    return self.state();
                }
            }
            "rename:execute" => {
                let dir = state.batch_rename.dir.clone();
                let needle = state.batch_rename.needle.clone();
                let replacement = state.batch_rename.replacement.clone();
                let recursive = state.batch_rename.recursive;
                if dir.is_none() {
                    state.batch_rename.preview = "请先选择目录".to_string();
                } else {
                    drop(state);
                    let result = file::do_rename_execute(dir.as_deref(), &needle, &replacement, recursive);
                    self.state.write().unwrap().batch_rename.preview = result;
                    return self.state();
                }
            }

            // Excel 移动文件
            "excel_move:pick_excel" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.excel_move.excel_path = Some(path.to_string());
                }
            }
            "excel_move:pick_input" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.excel_move.input_dir = Some(path.to_string());
                }
            }
            "excel_move:pick_output" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.excel_move.output_dir = Some(path.to_string());
                }
            }
            "excel_move:set_suffixes" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.excel_move.suffixes = v.to_string();
                }
            }
            "excel_move:preview" => {
                let excel = state.excel_move.excel_path.clone();
                let input_dir = state.excel_move.input_dir.clone();
                let output_dir = state.excel_move.output_dir.clone();
                let suffixes = state.excel_move.suffixes.clone();
                drop(state);
                let preview = file::do_excel_move_preview(&excel, &input_dir, &output_dir, &suffixes);
                self.state.write().unwrap().excel_move.preview = preview.0;
                self.state.write().unwrap().excel_move.message = Some(preview.1);
                return self.state();
            }
            "excel_move:execute" => {
                let excel = state.excel_move.excel_path.clone();
                let input_dir = state.excel_move.input_dir.clone();
                let output_dir = state.excel_move.output_dir.clone();
                let suffixes = state.excel_move.suffixes.clone();
                drop(state);
                let result = file::do_excel_move_execute(&excel, &input_dir, &output_dir, &suffixes);
                self.state.write().unwrap().excel_move.message = Some(result);
                return self.state();
            }

            // API 请求
            "api:set_url" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_request.url = v.to_string();
                }
            }
            "api:set_method" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_request.method = v.to_string();
                }
            }
            "api:set_headers" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_request.headers = v.to_string();
                }
            }
            "api:set_body" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_request.body = v.to_string();
                }
            }
            "api:send" => {
                state.api_request.loading = true;
                let url = state.api_request.url.clone();
                let method = state.api_request.method.clone();
                let headers = state.api_request.headers.clone();
                let body = state.api_request.body.clone();
                drop(state);
                let result = api::do_http_request(&url, &method, &headers, &body);
                let mut state = self.state.write().unwrap();
                state.api_request.loading = false;
                state.api_request.response_status = Some(result.0);
                state.api_request.response_body = result.1;
                return serde_json::to_value(&*state).unwrap_or_default();
            }

            // 批量下载
            "batch_dl:set_template" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_batch_download.template = v.to_string();
                }
            }
            "batch_dl:set_paths" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.api_batch_download.paths = v.to_string();
                }
            }
            "batch_dl:set_concurrency" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_u64()) {
                    state.api_batch_download.concurrency = v as usize;
                }
            }
            "batch_dl:pick_output" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.api_batch_download.output_dir = Some(path.to_string());
                }
            }
            "batch_dl:start" => {
                state.api_batch_download.message = Some("下载功能开发中...".to_string());
            }

            // JSON 转换
            "json_conv:pick_file" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.json_convert.input_file = Some(path.to_string());
                }
            }
            "json_conv:set_format" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.json_convert.output_format = v.to_string();
                }
            }

            // JSON 合并
            "json_merge:add_file" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    state.json_merge.files.push(path.to_string());
                }
            }
            "json_merge:execute" => {
                let files = state.json_merge.files.clone();
                drop(state);
                let result = json::do_json_merge(&files);
                self.state.write().unwrap().json_merge.merged = result;
                return self.state();
            }

            // 网络扫描
            "net_scan:set_ip_range" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.network_scan.ip_range = v.to_string();
                }
            }
            "net_scan:set_ports" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_str()) {
                    state.network_scan.ports = v.to_string();
                }
            }
            "net_scan:set_timeout" => {
                if let Some(v) = params.get("value").and_then(|v| v.as_u64()) {
                    state.network_scan.timeout = v;
                }
            }
            "net_scan:start" => {
                let ip_range = state.network_scan.ip_range.clone();
                let ports_str = state.network_scan.ports.clone();
                let timeout = state.network_scan.timeout;
                state.network_scan.loading = true;
                state.network_scan.results.clear();
                drop(state);
                let results = network::do_network_scan(&ip_range, &ports_str, timeout);
                let mut state = self.state.write().unwrap();
                state.network_scan.loading = false;
                state.network_scan.results = results;
                return serde_json::to_value(&*state).unwrap_or_default();
            }

            _ => {}
        }

        // 不要在持有写锁的情况下调用 self.state()，直接序列化
        serde_json::to_value(&*state).unwrap_or_default()
    }

    fn ui_schema(&self) -> UiSchema {
        // 创建左右分栏布局：左侧菜单 + 右侧内容
        use plugin_sdk::UiNode;

        let sidebar = Self::build_sidebar(self.selected_tool());
        let content = self.tool_schema();

        // 使用 split 组件创建左右分栏（左侧固定宽度 200px，右侧自适应）
        // 将工具的 gap / align_items 等布局属性传递到右侧容器
        let mut right_container = UiNode::new("flex-col")
            .children(content.children);
        if content.gap > 0 {
            right_container = right_container.prop("gap", serde_json::json!(content.gap));
        }
        if let Some(ref align) = content.align_items {
            right_container = right_container.prop("align_items", serde_json::json!(align));
        }

        UiSchema {
            layout: "flex-row".into(),
            children: vec![
                UiNode::split("row")
                    .prop("left_width", serde_json::json!(200))
                    .prop("gap", serde_json::json!(1))
                    .child(sidebar)
                    .child(right_container),
            ],
            ..Default::default()
        }
    }
}
