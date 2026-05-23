//! 网络工具：局域网扫描
//!
//! 基于 crawler scanner.rs 的功能，支持：
//! - IP 范围和端口扫描
//! - 自动检测本机网段
//! - 异步后台扫描（不阻塞 UI）
//! - 结果选择/全选
//! - 在浏览器打开选中端口

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream as StdTcpStream};
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Duration;

use plugin_sdk::{UiNode, UiSchema};
use crate::state::ScanResultItem;

// ---------------------------------------------------------------------------
// 网络扫描 Schema
// ---------------------------------------------------------------------------
pub fn schema_network_scan() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        gap: 12,
        children: vec![
            // 标题
            UiNode::new("card")
                .prop("title", serde_json::json!("局域网设备扫描"))
                .child(
                    UiNode::label("扫描指定 IP 段的开放端口，快速发现局域网内设备和服务。")
                        .prop("type", serde_json::json!("secondary")),
                ),

            // IP 范围
            UiNode::new("card")
                .prop("title", serde_json::json!("IP 范围"))
                .child(
                    UiNode::new("flex-col")
                        .prop("gap", serde_json::json!(8))
                        .child(
                            UiNode::input("network_scan.ip_range", "如 192.168.1.1-254")
                                .prop("label", serde_json::json!("IP 范围"))
                                .on_action("net_scan:set_ip_range"),
                        )
                        .child(
                            UiNode::label("支持格式: 192.168.1.1-10, 192.168.1.1-254, 192.168.1.1,192.168.1.2")
                                .prop("type", serde_json::json!("muted"))
                                .prop("size", serde_json::json!("xs")),
                        ),
                ),

            // 端口
            UiNode::new("card")
                .prop("title", serde_json::json!("端口"))
                .child(
                    UiNode::new("flex-col")
                        .prop("gap", serde_json::json!(8))
                        .child(
                            UiNode::input("network_scan.ports", "如 80,443,22 或 80-100")
                                .prop("label", serde_json::json!("端口列表"))
                                .on_action("net_scan:set_ports"),
                        )
                        .child(
                            UiNode::label("支持格式: 80,443,22 或 80-100（范围）")
                                .prop("type", serde_json::json!("muted"))
                                .prop("size", serde_json::json!("xs")),
                        ),
                ),

            // 超时
            UiNode::new("card")
                .prop("title", serde_json::json!("超时设置"))
                .child(
                    UiNode::input("network_scan.timeout", "500")
                        .prop("label", serde_json::json!("超时时间 (毫秒)"))
                        .prop("type", serde_json::json!("number"))
                        .on_action("net_scan:set_timeout"),
                ),

            // 操作按钮行
            UiNode::new("flex-row")
                .prop("gap", serde_json::json!(8))
                .child(UiNode::button("开始扫描", "net_scan:start").prop("variant", serde_json::json!("primary")))
                .child(UiNode::button("清空结果", "net_scan:clear"))
                .child(UiNode::button("重置", "net_scan:reset")),

            // 消息提示
            UiNode::display("network_scan.message")
                .prop("type", serde_json::json!("info")),

            // 结果区域
            UiNode::new("card")
                .prop("title", serde_json::json!("扫描结果"))
                .child(
                    UiNode::new("flex-col")
                        .prop("gap", serde_json::json!(8))
                        .child(
                            // 全选 + 操作按钮
                            UiNode::new("flex-row")
                                .prop("gap", serde_json::json!(8))
                                .prop("align_items", serde_json::json!("center"))
                                .child(
                                    UiNode::button("全选/取消", "net_scan:toggle_select_all")
                                        .prop("variant", serde_json::json!("ghost")),
                                )
                                .child(
                                    UiNode::button("在浏览器打开", "net_scan:open_selected")
                                        .prop("variant", serde_json::json!("ghost")),
                                ),
                        )
                        .child(
                            UiNode::table("network_scan.results", &["IP 地址", "端口", "状态", "延迟(ms)", "选中"])
                                .prop("empty_text", serde_json::json!("暂无扫描结果，请先输入 IP 范围和端口后点击\"开始扫描\""))
                                .on_action("net_scan:toggle_select"),
                        ),
                ),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 扫描逻辑
// ---------------------------------------------------------------------------

/// 自动检测本机局域网网段
pub fn detect_local_subnet() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:53").ok()?;
    let local_ip = socket.local_addr().ok()?.ip();

    if local_ip.is_unspecified() || local_ip.is_loopback() {
        return None;
    }

    match local_ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            Some(format!("{}.{}.{}.1-254", octets[0], octets[1], octets[2]))
        }
        IpAddr::V6(_) => None,
    }
}

/// 解析端口字符串
pub fn parse_ports(s: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() != 2 {
                return Err(format!("无效端口范围: {part}"));
            }
            let start: u16 = range[0].parse().map_err(|_| "无效起始端口")?;
            let end: u16 = range[1].trim().parse().map_err(|_| "无效结束端口")?;
            if start > end {
                return Err("起始端口不能大于结束端口".to_string());
            }
            for port in start..=end {
                ports.push(port);
            }
        } else {
            let port: u16 = part.parse().map_err(|_| "无效端口")?;
            ports.push(port);
        }
    }
    if ports.is_empty() {
        return Err("至少需要一个端口".to_string());
    }
    Ok(ports)
}

/// 扫描单个端口，返回 (状态, 延迟)
pub fn scan_port(addr: SocketAddr, timeout_ms: u64) -> (String, Option<u64>) {
    let start = std::time::Instant::now();
    match StdTcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)) {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ("开放".to_string(), Some(elapsed))
        }
        Err(_) => ("关闭".to_string(), None),
    }
}

/// 解析 IP 范围，支持以下格式：
/// - 单个 IP: `192.168.1.1`
/// - IP 范围: `192.168.1.1-254`（按最后一个 octet 递增）
/// - 逗号分隔: `192.168.1.1,192.168.1.2`
pub fn parse_ip_range(s: &str) -> Result<Vec<IpAddr>, String> {
    let mut ips = Vec::new();

    // 不支持 CIDR
    if s.contains('/') {
        return Err("暂不支持 CIDR 格式，请使用范围格式如 192.168.1.1-254".to_string());
    }

    // 范围格式: 192.168.1.1-254
    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err("无效 IP 范围格式".to_string());
        }
        let start: Ipv4Addr = parts[0].trim().parse().map_err(|_| "无效起始 IP")?;
        let end_octet: u8 = parts[1].trim().parse().map_err(|_| "无效结束 octet")?;
        let octets = start.octets();
        // 从 .1 开始而不是 .0（避免网络地址和广播地址）
        let start_octet = octets[3].max(1);
        for i in start_octet..=end_octet {
            ips.push(IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], i)));
        }
        return Ok(ips);
    }

    // 逗号分隔的多个 IP
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let ip: IpAddr = part.parse().map_err(|e| format!("无效 IP 地址 {part}: {e}"))?;
        ips.push(ip);
    }

    if ips.is_empty() {
        return Err("请至少输入一个 IP 地址".to_string());
    }

    Ok(ips)
}

/// 执行网络扫描（后台线程，结果直接写入共享 state 的 network_scan 字段）
///
/// 扫描完成后自动写入 results/scanned/total/message/loading，
/// 宿主侧下次 handle_action 时读取最新状态即可。
pub fn spawn_network_scan(
    state: Arc<StdRwLock<crate::state::ToolboxState>>,
    ip_range: String,
    ports_str: String,
    timeout_ms: u64,
) {
    // 解析参数（提前失败）
    let ips = match parse_ip_range(&ip_range) {
        Ok(i) => i,
        Err(e) => {
            if let Ok(mut s) = state.write() {
                s.network_scan.message = Some(format!("IP 范围解析失败: {e}"));
                s.network_scan.message_ok = false;
                s.network_scan.loading = false;
            }
            return;
        }
    };

    let ports = match parse_ports(&ports_str) {
        Ok(p) => p,
        Err(e) => {
            if let Ok(mut s) = state.write() {
                s.network_scan.message = Some(format!("端口解析失败: {e}"));
                s.network_scan.message_ok = false;
                s.network_scan.loading = false;
            }
            return;
        }
    };

    let total = ips.len() * ports.len();

    // 后台线程执行扫描
    std::thread::spawn(move || {
        let mut results = Vec::new();
        let mut scanned = 0usize;

        for ip in &ips {
            for &port in &ports {
                let addr = SocketAddr::new(*ip, port);
                let (status, latency) = scan_port(addr, timeout_ms);
                scanned += 1;

                if status == "开放" {
                    results.push(ScanResultItem {
                        ip: ip.to_string(),
                        port,
                        status,
                        latency_ms: latency,
                        selected: false,
                        hostname: None,
                    });
                }

                // 每 50 个目标更新一次进度
                if scanned % 50 == 0 {
                    if let Ok(mut s) = state.write() {
                        s.network_scan.scanned = scanned;
                        s.network_scan.message = Some(format!(
                            "扫描中... {}/{} ({}个开放)",
                            scanned, total, results.len()
                        ));
                    }
                }
            }
        }

        // 按 IP + 端口排序
        results.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.port.cmp(&b.port)));

        // 写入最终结果
        if let Ok(mut s) = state.write() {
            let open_count = results.len();
            s.network_scan.loading = false;
            s.network_scan.results = results;
            s.network_scan.scanned = scanned;
            s.network_scan.message = Some(format!(
                "扫描完成！共扫描 {} 个目标，发现 {} 个开放端口",
                total, open_count
            ));
            s.network_scan.message_ok = true;
        }
    });
}

/// 在浏览器中打开选中的结果
pub fn open_selected_in_browser(results: &[ScanResultItem]) -> Result<usize, String> {
    let selected: Vec<&ScanResultItem> = results.iter().filter(|r| r.selected).collect();
    if selected.is_empty() {
        return Err("没有选中的结果".to_string());
    }

    let mut opened = 0;
    for r in &selected {
        let url = if r.port == 443 || r.port == 8443 {
            format!("https://{}:{}", r.ip, r.port)
        } else {
            format!("http://{}:{}", r.ip, r.port)
        };

        match open::that(&url) {
            Ok(_) => opened += 1,
            Err(e) => log::warn!("打开 URL 失败 {}: {}", url, e),
        }
    }

    Ok(opened)
}
