//! 网络工具：局域网扫描

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream as StdTcpStream};
use std::time::Duration;

use plugin_sdk::{UiNode, UiSchema};
use crate::state::ScanResultItem;
use crate::tools::home::make_button_row;

// ---------------------------------------------------------------------------
// 网络扫描 Schema
// ---------------------------------------------------------------------------
pub fn schema_network_scan() -> UiSchema {
    UiSchema {
        layout: "flex-col".into(),
        children: vec![
            UiNode::label("局域网设备扫描"),
            UiNode::input("ip_range", "IP 段 (如 192.168.1.1-254)"),
            UiNode::input("ports", "端口 (如 80,443,8080 或 1-1000)"),
            UiNode::input("timeout", "超时 (ms)"),
            UiNode::progress("scan_progress"),
            make_button_row(&[("开始扫描", "net_scan:start")]),
            UiNode::table("scan_results", &["IP", "端口", "状态", "延迟(ms)"]),
        ],
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 扫描逻辑
// ---------------------------------------------------------------------------

/// 解析端口字符串
pub fn parse_ports(s: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
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

/// 扫描单个端口
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

/// 解析 IP 范围
pub fn parse_ip_range(s: &str) -> Result<Vec<Ipv4Addr>, String> {
    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err("无效 IP 范围格式".to_string());
        }
        let base: Ipv4Addr = parts[0].trim().parse().map_err(|_| "无效 IP")?;
        let end_octet: u8 = parts[1].trim().parse().map_err(|_| "无效结束 octet")?;
        let octets = base.octets();
        let mut ips = Vec::new();
        for i in 1..=end_octet {
            if i > 0 {
                ips.push(Ipv4Addr::new(octets[0], octets[1], octets[2], i));
            }
        }
        Ok(ips)
    } else {
        let ip: Ipv4Addr = s.parse().map_err(|_| "无效 IP")?;
        Ok(vec![ip])
    }
}

/// 执行网络扫描
pub fn do_network_scan(ip_range: &str, ports_str: &str, timeout_ms: u64) -> Vec<ScanResultItem> {
    if ip_range.is_empty() || ports_str.is_empty() {
        return vec![];
    }

    let ports = match parse_ports(ports_str) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let ips = match parse_ip_range(ip_range) {
        Ok(i) => i,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();

    for ip in ips {
        for port in &ports {
            let addr = SocketAddr::new(IpAddr::V4(ip), *port);
            let (status, latency) = scan_port(addr, timeout_ms);
            results.push(ScanResultItem {
                ip: ip.to_string(),
                port: *port,
                status,
                latency_ms: latency,
            });
        }
    }

    results
}
