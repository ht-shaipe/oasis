use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

/// 扫描结果
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub open: bool,
    pub latency_ms: Option<u64>,
}

/// 解析 IP 范围字符串（如 "192.168.1.1", "192.168.1.1-254", "192.168.1.1-192.168.1.50"）
pub fn parse_ip_range(range: &str) -> Result<Vec<Ipv4Addr>, String> {
    let range = range.trim();
    if range.is_empty() {
        return Err("IP 范围为空".to_string());
    }

    // 单个 IP
    if !range.contains('-') {
        let ip: Ipv4Addr = range.parse().map_err(|e| format!("无效 IP: {}", e))?;
        return Ok(vec![ip]);
    }

    let parts: Vec<&str> = range.split('-').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err("IP 范围格式错误".to_string());
    }

    let start = parts[0].parse::<Ipv4Addr>().map_err(|e| {
        format!("无效起始 IP: {}", e)
    })?;

    let end_str = parts[1];
    let end_addr: Ipv4Addr;

    // Try full IP first: "192.168.1.1-192.168.1.50"
    if end_str.contains('.') {
        end_addr = end_str.parse::<Ipv4Addr>().map_err(|e| {
            format!("无效结束 IP: {}", e)
        })?;
    }
    // Else try just last octet: "192.168.1.1-254"
    else {
        let last_octet: u8 = end_str
            .parse()
            .map_err(|e| format!("无效 IP 结束值: {}", e))?;
        let mut octets = start.octets();
        octets[3] = last_octet;
        end_addr = Ipv4Addr::from(octets);
    }

    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end_addr);
    if start_u32 > end_u32 {
        return Err("起始 IP 大于结束 IP".to_string());
    }

    let mut addrs = Vec::new();
    for ip_u32 in start_u32..=end_u32 {
        addrs.push(Ipv4Addr::from(ip_u32));
    }
    Ok(addrs)
}

/// 解析端口列表（如 "80", "80,443,22", "80-100", "80,443,8080-8090"）
pub fn parse_ports(ports_str: &str) -> Result<Vec<u16>, String> {
    let s = ports_str.trim();
    if s.is_empty() {
        return Err("端口列表为空".to_string());
    }

    let mut ports = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').map(|p| p.trim()).collect();
            if range_parts.len() != 2 {
                return Err(format!("端口范围格式错误: {}", part));
            }
            let start: u16 = range_parts[0]
                .parse()
                .map_err(|e| format!("无效端口: {}", e))?;
            let end: u16 = range_parts[1]
                .parse()
                .map_err(|e| format!("无效端口: {}", e))?;
            if start > end {
                return Err(format!("端口范围起始大于结束: {}", part));
            }
            for p in start..=end {
                ports.push(p);
            }
        } else {
            let port: u16 = part.parse().map_err(|e| format!("无效端口: {}", e))?;
            ports.push(port);
        }
    }

    Ok(ports)
}

/// TCP 端口扫描（Connect 方式）
pub fn scan_port(addr: SocketAddr, timeout_ms: u64) -> ScanResult {
    let duration = Duration::from_millis(timeout_ms);
    let start = std::time::Instant::now();

    match TcpStream::connect_timeout(&addr, duration) {
        Ok(_) => {
            let latency = start.elapsed().as_millis() as u64;
            ScanResult {
                ip: match addr.ip() {
                    IpAddr::V4(v4) => v4,
                    IpAddr::V6(_) => return ScanResult {
                        ip: Ipv4Addr::UNSPECIFIED,
                        port: addr.port(),
                        open: false,
                        latency_ms: None,
                    },
                },
                port: addr.port(),
                open: true,
                latency_ms: Some(latency),
            }
        }
        Err(_) => ScanResult {
            ip: match addr.ip() {
                IpAddr::V4(v4) => v4,
                IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
            },
            port: addr.port(),
            open: false,
            latency_ms: None,
        },
    }
}

/// 自动检测本机所在子网（从活跃网卡中选取第一个私有 IPv4 地址）
pub fn detect_local_subnet() -> Result<(Ipv4Addr, u8), String> {
    use std::net::UdpSocket;

    // 使用 UDP socket 获取默认路由的本地地址
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .connect("8.8.8.8:80")
        .map_err(|e| e.to_string())?;
    let local_addr = socket.local_addr().map_err(|e| e.to_string())?;
    drop(socket);

    match local_addr.ip() {
        IpAddr::V4(v4) => {
            // 假设 /24 子网
            Ok((v4, 24))
        }
        IpAddr::V6(_) => Err("未检测到 IPv4 地址".to_string()),
    }
}

/// 格式化扫描结果为表格文本
pub fn format_scan_results(results: &[ScanResult], show_closed: bool) -> String {
    let mut out = String::new();
    out.push_str("地址\t\t端口\t状态\n");
    out.push_str("──\t\t──\t──\n");

    for r in results {
        if !r.open && !show_closed {
            continue;
        }
        let status = if r.open {
            match r.latency_ms {
                Some(lat) => format!("开放 ({}ms)", lat),
                None => "开放".to_string(),
            }
        } else {
            "关闭".to_string()
        };
        out.push_str(&format!("{}\t{}\t{}\n", r.ip, r.port, status));
    }

    out
}