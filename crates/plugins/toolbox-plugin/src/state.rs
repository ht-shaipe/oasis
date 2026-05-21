//! 工具箱插件 — 状态类型定义

use serde::{Deserialize, Serialize};

/// 工具 ID
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    Home,
    CsvStats,
    CsvSplit,
    CsvExcelConvert,
    BatchRename,
    ExcelMoveFiles,
    ApiRequest,
    ApiBatchDownload,
    JsonToCsvExcel,
    JsonMerge,
    NetworkScan,
    UiSchemaDemo,
}

/// CSV 统计状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvStatsState {
    pub input_file: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub stats: String,
}

/// CSV 分割状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvSplitState {
    pub input_file: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub split_col: usize,
    pub output_prefix: String,
}

/// CSV 转换状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvConvertState {
    pub input_file: Option<String>,
    pub output_format: String,
}

/// 批量重命名状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchRenameState {
    pub dir: Option<String>,
    pub needle: String,
    pub replacement: String,
    pub recursive: bool,
    pub preview: String,
    pub plan: Vec<(String, String)>,
}

/// Excel 移动文件状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcelMoveState {
    pub excel_path: Option<String>,
    pub input_dir: Option<String>,
    pub output_dir: Option<String>,
    pub suffixes: String,
    pub preview: String,
    pub message: Option<String>,
}

/// API 请求状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiRequestState {
    pub url: String,
    pub method: String,
    pub headers: String,
    pub body: String,
    pub loading: bool,
    pub response_status: Option<u16>,
    pub response_body: String,
}

/// 批量下载状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchDownloadState {
    pub template: String,
    pub paths: String,
    pub concurrency: usize,
    pub output_dir: Option<String>,
    pub message: Option<String>,
}

/// UiSchema Demo 树节点
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoTreeNode {
    pub name: String,
    pub is_dir: bool,
    pub path: String,
    pub children: Vec<DemoTreeNode>,
}

/// UiSchema Demo 表格行
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoTableRow {
    pub name: String,
    pub kind: String,
    pub owner: String,
    pub status: String,
}

/// UiSchema Demo 状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiSchemaDemoState {
    pub total_users: u64,
    pub active_today: u64,
    pub revenue: u64,
    pub completion: u64,
    pub status_message: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub notes: String,
    pub tree: Vec<DemoTreeNode>,
    pub rows: Vec<DemoTableRow>,
}

/// JSON 转换状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonConvertState {
    pub input_file: Option<String>,
    pub output_format: String,
}

/// JSON 合并状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonMergeState {
    pub files: Vec<String>,
    pub merged: String,
    pub message: Option<String>,
}

/// 网络扫描状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkScanState {
    pub ip_range: String,
    pub ports: String,
    pub timeout: u64,
    pub loading: bool,
    pub results: Vec<ScanResultItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultItem {
    pub ip: String,
    pub port: u16,
    pub status: String,
    pub latency_ms: Option<u64>,
}

/// 插件完整状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolboxState {
    pub selected_tool: ToolId,
    pub csv_stats: CsvStatsState,
    pub csv_split: CsvSplitState,
    pub csv_convert: CsvConvertState,
    pub batch_rename: BatchRenameState,
    pub excel_move: ExcelMoveState,
    pub api_request: ApiRequestState,
    pub api_batch_download: BatchDownloadState,
    pub json_convert: JsonConvertState,
    pub json_merge: JsonMergeState,
    pub network_scan: NetworkScanState,
    pub demo: UiSchemaDemoState,
}

impl Default for ToolboxState {
    fn default() -> Self {
        Self {
            selected_tool: ToolId::Home,
            csv_stats: CsvStatsState {
                input_file: None,
                headers: vec![],
                rows: vec![],
                stats: String::new(),
            },
            csv_split: CsvSplitState {
                input_file: None,
                headers: vec![],
                rows: vec![],
                split_col: 0,
                output_prefix: String::new(),
            },
            csv_convert: CsvConvertState {
                input_file: None,
                output_format: "xlsx".into(),
            },
            batch_rename: BatchRenameState {
                dir: None,
                needle: String::new(),
                replacement: String::new(),
                recursive: false,
                preview: String::new(),
                plan: vec![],
            },
            excel_move: ExcelMoveState {
                excel_path: None,
                input_dir: None,
                output_dir: None,
                suffixes: String::new(),
                preview: String::new(),
                message: None,
            },
            api_request: ApiRequestState {
                url: String::new(),
                method: "GET".into(),
                headers: String::new(),
                body: String::new(),
                loading: false,
                response_status: None,
                response_body: String::new(),
            },
            api_batch_download: BatchDownloadState {
                template: String::new(),
                paths: String::new(),
                concurrency: 3,
                output_dir: None,
                message: None,
            },
            json_convert: JsonConvertState {
                input_file: None,
                output_format: "csv".into(),
            },
            json_merge: JsonMergeState {
                files: vec![],
                merged: String::new(),
                message: None,
            },
            network_scan: NetworkScanState {
                ip_range: String::new(),
                ports: String::new(),
                timeout: 1000,
                loading: false,
                results: vec![],
            },
            demo: UiSchemaDemoState {
                total_users: 12_840,
                active_today: 3_842,
                revenue: 128_500,
                completion: 76,
                status_message: "模板库已就绪".into(),
                name: "Li Hua".into(),
                email: "li.hua@example.com".into(),
                role: "design".into(),
                notes: "展示 ui-schema 的模板和基础组件".into(),
                tree: vec![
                    DemoTreeNode {
                        name: "src".into(),
                        is_dir: true,
                        path: "/src".into(),
                        children: vec![
                            DemoTreeNode {
                                name: "main.rs".into(),
                                is_dir: false,
                                path: "/src/main.rs".into(),
                                children: vec![],
                            },
                            DemoTreeNode {
                                name: "plugins".into(),
                                is_dir: true,
                                path: "/src/plugins".into(),
                                children: vec![DemoTreeNode {
                                    name: "demo.rs".into(),
                                    is_dir: false,
                                    path: "/src/plugins/demo.rs".into(),
                                    children: vec![],
                                }],
                            },
                        ],
                    },
                    DemoTreeNode {
                        name: "crates".into(),
                        is_dir: true,
                        path: "/crates".into(),
                        children: vec![],
                    },
                ],
                rows: vec![
                    DemoTableRow { name: "Dashboard".into(), kind: "template".into(), owner: "UI".into(), status: "ready".into() },
                    DemoTableRow { name: "Tree Table".into(), kind: "template".into(), owner: "UI".into(), status: "ready".into() },
                    DemoTableRow { name: "Form Page".into(), kind: "template".into(), owner: "UI".into(), status: "ready".into() },
                    DemoTableRow { name: "Empty State".into(), kind: "template".into(), owner: "UI".into(), status: "ready".into() },
                ],
            },
        }
    }
}
