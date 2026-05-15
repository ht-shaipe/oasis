//! 局域网设备扫描工具:扫描指定IP段和端口的在线设备

use gpui::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled, Window, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, checkbox::Checkbox, Disableable, Icon, IconName,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    label::Label,
    scroll::ScrollableElement,
    v_flex,
};
use rust_i18n::t;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream as StdTcpStream};
use std::sync::Mutex;
use std::time::Duration;

use super::super::ToolboxPanel;

/// 扫描结果
#[derive(Clone, Debug)]
pub struct ScanResult {
    pub ip: String,
    pub port: u16,
    pub status: String,
    pub hostname: Option<String>,
    pub latency_ms: Option<u64>,
    /// 是否选中
    pub selected: bool,
}

/// 后台线程发送的进度消息
enum ScanMsg {
    /// 发现一个开放端口
    Found(ScanResult),
    /// 进度更新: (已扫描数, 总数)
    Progress(usize, usize),
    /// 扫描完成
    Done,
}

/// 网络扫描器状态
pub struct NetworkScanState {
    pub ip_range_input: Entity<gpui_component::input::InputState>,
    pub ports_input: Entity<gpui_component::input::InputState>,
    pub timeout_input: Entity<gpui_component::input::InputState>,
    pub loading: bool,
    pub results: Mutex<Vec<ScanResult>>,
    pub message: Option<String>,
    pub message_ok: bool,
    /// 已扫描数量
    pub scanned: usize,
    /// 总扫描目标数
    pub total: usize,
    /// 是否全选
    pub select_all: bool,
}

impl NetworkScanState {
    pub fn new(window: &mut Window, cx: &mut Context<ToolboxPanel>) -> Self {
        // 自动检测本机网段
        let default_subnet = detect_local_subnet()
            .unwrap_or_else(|| "192.168.1.1-254".to_string());

        let ip_range_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("192.168.1.1-254".to_string(), window, cx);
            s.set_value(gpui::SharedString::from(default_subnet), window, cx);
            s
        });
        let ports_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("80,443,22,3389".to_string(), window, cx);
            s.set_value(gpui::SharedString::from("80,443,22"), window, cx);
            s
        });
        let timeout_input = cx.new(|cx| {
            let mut s = gpui_component::input::InputState::new(window, cx);
            s.set_placeholder("超时时间(毫秒)".to_string(), window, cx);
            s.set_value(gpui::SharedString::from("500"), window, cx);
            s
        });

        Self {
            ip_range_input,
            ports_input,
            timeout_input,
            loading: false,
            results: Mutex::new(vec![]),
            message: None,
            message_ok: true,
            scanned: 0,
            total: 0,
            select_all: false,
        }
    }

    pub fn render(
        state: &mut NetworkScanState,
        entity: Entity<ToolboxPanel>,
        _window: &mut Window,
        cx: &mut Context<ToolboxPanel>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let ip_range_input = state.ip_range_input.clone();
        let ports_input = state.ports_input.clone();
        let timeout_input = state.timeout_input.clone();

        // 进度信息
        let progress_text = if state.loading && state.total > 0 {
            let pct = if state.total > 0 {
                state.scanned * 100 / state.total
            } else {
                0
            };
            format!("扫描中... {}/{} ({}%)", state.scanned, state.total, pct)
        } else if state.loading {
            "扫描中...".to_string()
        } else {
            "开始扫描".to_string()
        };

        let start_entity = entity.clone();
        let start_btn = Button::new("network-scan-start")
            .label(progress_text)
            .primary()
            .disabled(state.loading)
            .on_click(move |_, _, cx| {
                start_entity.update(cx, |this, cx| {
                    this.start_network_scan(cx);
                });
            });

        let clear_entity = entity.clone();
        let results_is_empty = state.results.lock().unwrap().is_empty();
        let clear_btn = Button::new("network-scan-clear")
            .label("清空结果")
            .outline()
            .disabled(state.loading || results_is_empty)
            .on_click(move |_, _, cx| {
                clear_entity.update(cx, |this, cx| {
                    this.network_scan_clear(cx);
                });
            });

        // 渲染结果表格
        let results_guard = state.results.lock().unwrap();
        let results_rows: Vec<gpui::AnyElement> = results_guard
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let status_color = if result.status == "开放" {
                    theme.green
                } else {
                    theme.muted_foreground
                };

                let row_entity = entity.clone();
                let checked = result.selected;
                let row_idx = idx;

                v_flex()
                    .w_full()
                    .child(
                        h_flex()
                            .w_full()
                            .border_1()
                            .border_color(theme.border)
                            .p_2()
                            .child(
                                Checkbox::new(("scan-result-check", row_idx))
                                    .checked(checked)
                                    .on_click(move |_, _, cx| {
                                        row_entity.update(cx, |this, cx| {
                                            let mut results = this.network_scan.results.lock().unwrap();
                                            if let Some(r) = results.get_mut(row_idx) {
                                                r.selected = !r.selected;
                                            }
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .gap_4()
                                    .child(
                                        Label::new(format!("{}", idx + 1))
                                            .text_xs()
                                            .w(px(30.))
                                            .text_color(theme.muted_foreground),
                                    )
                                    .child(
                                        Label::new(&result.ip)
                                            .text_xs()
                                            .flex_1()
                                            .font_weight(gpui::FontWeight::MEDIUM),
                                    )
                                    .child(
                                        Label::new(format!("{}", result.port))
                                            .text_xs()
                                            .w(px(60.))
                                            .text_color(theme.muted_foreground),
                                    )
                                    .child(
                                        Label::new(&result.status)
                                            .text_xs()
                                            .w(px(50.))
                                            .text_color(status_color)
                                            .font_weight(gpui::FontWeight::MEDIUM),
                                    )
                                    .child(
                                        Label::new(
                                            result
                                                .latency_ms
                                                .map(|ms| format!("{}ms", ms))
                                                .unwrap_or_else(|| "-".to_string()),
                                        )
                                        .text_xs()
                                        .flex_1()
                                        .text_color(theme.muted_foreground),
                                    ),
                            ),
                    )
                    .into_any_element()
            })
            .collect();
        drop(results_guard); // Release the lock before accessing results again

        let select_all_entity = entity.clone();
        let results_for_select_all = state.results.lock().unwrap();
        let select_all_checked = state.select_all || (!results_for_select_all.is_empty() && results_for_select_all.iter().all(|r| r.selected));
        let has_any_selected = results_for_select_all.iter().any(|r| r.selected);
        drop(results_for_select_all);

        let results_header = h_flex()
            .w_full()
            .bg(theme.secondary)
            .p_2()
            .child(
                Checkbox::new("scan-select-all")
                    .checked(select_all_checked)
                    .on_click(move |_, _, cx| {
                        select_all_entity.update(cx, |this, cx| {
                            let new_state = !this.network_scan.select_all;
                            this.network_scan.select_all = new_state;
                            let mut results = this.network_scan.results.lock().unwrap();
                            for r in results.iter_mut() {
                                r.selected = new_state;
                            }
                            cx.notify();
                        });
                    }),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .child(
                        Label::new("#")
                            .text_xs()
                            .w(px(30.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Label::new("IP地址")
                            .text_xs()
                            .flex_1()
                            .font_weight(gpui::FontWeight::MEDIUM),
                    )
                    .child(
                        Label::new("端口")
                            .text_xs()
                            .w(px(60.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Label::new("状态")
                            .text_xs()
                            .w(px(50.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Label::new("延迟")
                            .text_xs()
                            .flex_1()
                            .text_color(theme.muted_foreground),
                    ),
            );

        // 提取按钮所需的状态值(避免在闭包中移动 state)
        let results_for_buttons = state.results.lock().unwrap();
        let selected_count = results_for_buttons.iter().filter(|r| r.selected).count();
        let has_any_selected = results_for_buttons.iter().any(|r| r.selected);
        let is_loading = state.loading;

        // 提取要打开的URL列表(用于闭包)
        let open_urls: Vec<String> = results_for_buttons
            .iter()
            .filter(|r| r.selected)
            .map(|r| {
                if r.port == 443 || r.port == 8443 {
                    format!("https://{}:{}", r.ip, r.port)
                } else {
                    format!("http://{}:{}", r.ip, r.port)
                }
            })
            .collect();
        drop(results_for_buttons); // Release the lock

        let open_btn = Button::new("network-scan-open-browser")
            .label(format!("在浏览器打开 ({})", selected_count))
            .outline()
            .disabled(is_loading || !has_any_selected)
            .on_click(move |_, _, cx| {
                for url in &open_urls {
                    if let Err(e) = open::that(url) {
                        log::error!("Failed to open URL {}: {}", url, e);
                    }
                }
            });
        
        v_flex()
            .gap_4()
            .size_full()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new("IP范围")
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground),
                    )
                    .child(
                        v_flex().child(
                            h_flex().w_full().child(
                                Input::new(&ip_range_input).w_full(),
                            ),
                        ),
                    )
                    .child(
                        Label::new("支持格式: 192.168.1.1-10, 192.168.1.1-254, 192.168.1.1,192.168.1.2")
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new("端口")
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground),
                    )
                    .child(
                        v_flex().child(
                            h_flex().w_full().child(
                                Input::new(&ports_input).w_full(),
                            ),
                        ),
                    )
                    .child(
                        Label::new("支持格式: 80,443,22 或 80-100")
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new("超时(毫秒)")
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground),
                    )
                    .child(
                        v_flex().child(
                            h_flex().w_full().child(
                                Input::new(&timeout_input).w_full(),
                            ),
                        ),
                    ),
            )
            .child(h_flex().gap_2().child(start_btn).child(clear_btn).child(open_btn))
            .child(
                if let Some(msg) = &state.message {
                    let text_color = if state.message_ok {
                        theme.green
                    } else {
                        theme.red
                    };
                    Label::new(msg.clone())
                        .text_sm()
                        .text_color(text_color)
                        .into_any_element()
                } else {
                    Label::new("")
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .into_any_element()
                },
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Label::new(format!(
                                    "开放端口 ({})",
                                    state.results.lock().unwrap().len()
                                ))
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h(px(0.))
                            .overflow_hidden()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_lg()
                            .child(
                                v_flex()
                                    .size_full()
                                    .overflow_y_scrollbar()
                                    .child(results_header)
                                    .children(results_rows),
                            ),
                    ),
            )
    }
}

impl ToolboxPanel {
    pub fn start_network_scan(&mut self, cx: &mut Context<Self>) {
        if self.network_scan.loading {
            return;
        }

        let ip_range = self
            .network_scan
            .ip_range_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();

        let ports_str = self
            .network_scan
            .ports_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();

        let timeout_str = self
            .network_scan
            .timeout_input
            .read(cx)
            .value()
            .to_string()
            .trim()
            .to_string();

        if ip_range.is_empty() || ports_str.is_empty() {
            self.network_scan.message = Some("请输入IP范围和端口".to_string());
            self.network_scan.message_ok = false;
            cx.notify();
            return;
        }

        let timeout_ms: u64 = match timeout_str.parse() {
            Ok(t) if t > 0 => t,
            _ => {
                self.network_scan.message =
                    Some("超时时间必须是大于0的数字".to_string());
                self.network_scan.message_ok = false;
                cx.notify();
                return;
            }
        };

        let ips = match parse_ip_range(&ip_range) {
            Ok(ips) => ips,
            Err(e) => {
                self.network_scan.message = Some(format!("IP范围解析失败: {}", e));
                self.network_scan.message_ok = false;
                cx.notify();
                return;
            }
        };

        let ports = match parse_ports(&ports_str) {
            Ok(ports) => ports,
            Err(e) => {
                self.network_scan.message = Some(format!("端口解析失败: {}", e));
                self.network_scan.message_ok = false;
                cx.notify();
                return;
            }
        };

        let total = ips.len() * ports.len();
        self.network_scan.loading = true;
        self.network_scan.results.lock().unwrap().clear();
        self.network_scan.scanned = 0;
        self.network_scan.total = total;
        self.network_scan.message = Some(
            format!("开始扫描 {} 个目标...", total)
        );
        self.network_scan.message_ok = true;
        cx.notify();

        let entity = cx.entity().downgrade();

        // 使用tokio channel进行跨线程通信
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ScanMsg>();

        // 启动后台扫描线程
        std::thread::spawn(move || {
            let mut scanned = 0usize;

            for ip in &ips {
                for &port in &ports {
                    let addr = SocketAddr::new(*ip, port);
                    let start = std::time::Instant::now();
                    let status = scan_port(&addr, timeout_ms);
                    scanned += 1;
                    let is_open = status == "开放";

                    if is_open {
                        let latency = start.elapsed().as_millis() as u64;
                        let result = ScanResult {
                            ip: ip.to_string(),
                            port,
                            status,
                            hostname: None,
                            latency_ms: Some(latency),
                            selected: false,
                        };
                        let _ = tx.send(ScanMsg::Found(result));
                    }

                    // 每 10 个目标或每个开放端口发送一次进度
                    if scanned % 10 == 0 || is_open {
                        let _ = tx.send(ScanMsg::Progress(scanned, total));
                    }
                }
            }

            let _ = tx.send(ScanMsg::Done);
        });

        // 在后台线程中轮询channel，每收到消息就更新 UI
        cx.spawn(async move |entity, cx| {
            let mut done = false;

            while !done {
                // 等待下一条消息，使用tokio::time::timeout进行定期检查
                let msg_result = tokio::time::timeout(
                    Duration::from_millis(200),
                    rx.recv()
                ).await;

                match msg_result {
                    Ok(Some(msg)) => {
                        match msg {
                            ScanMsg::Found(result) => {
                                // 直接在当前线程更新实体，避免嵌套cx.update()
                                if let Some(ent) = entity.upgrade() {
                                    let _ = ent.update(cx, |this, cx| {
                                        this.network_scan.results.lock().unwrap().push(result);
                                        this.network_scan.message = Some(
                                            format!(
                                                "扫描中... {}/{} ({}个开放)",
                                                this.network_scan.scanned,
                                                this.network_scan.total,
                                                this.network_scan.results.lock().unwrap().len()
                                            )
                                        );
                                        cx.notify();
                                    });
                                }
                            }
                            ScanMsg::Progress(scanned, _total) => {
                                if let Some(ent) = entity.upgrade() {
                                    let _ = ent.update(cx, |this, cx| {
                                        this.network_scan.scanned = scanned;
                                        this.network_scan.message = Some(
                                            format!(
                                                "扫描中... {}/{} ({}个开放)",
                                                this.network_scan.scanned,
                                                this.network_scan.total,
                                                this.network_scan.results.lock().unwrap().len()
                                            )
                                        );
                                        cx.notify();
                                    });
                                }
                            }
                            ScanMsg::Done => {
                                done = true;
                                if let Some(ent) = entity.upgrade() {
                                    let _ = ent.update(cx, |this, cx| {
                                        this.network_scan.loading = false;
                                        this.network_scan.scanned = this.network_scan.total;
                                        let open_count = this.network_scan.results.lock().unwrap().len();
                                        this.network_scan.message = Some(
                                            format!(
                                                "扫描完成! 共扫描 {} 个目标, {} 个端口开放",
                                                this.network_scan.total, open_count
                                            )
                                        );
                                        this.network_scan.message_ok = true;
                                        cx.notify();
                                    });
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        // Channel closed
                        done = true;
                    }
                    Err(_) => {
                        // Timeout, continue loop
                        continue;
                    }
                }
            }
        })
        .detach();
    }

    pub fn network_scan_clear(&mut self, cx: &mut Context<Self>) {
        self.network_scan.results.lock().unwrap().clear();
        self.network_scan.scanned = 0;
        self.network_scan.total = 0;
        self.network_scan.message = Some("已清空结果".to_string());
        self.network_scan.message_ok = true;
        cx.notify();
    }
}

/// 解析IP范围
fn parse_ip_range(range: &str) -> Result<Vec<IpAddr>, String> {
    let mut ips = vec![];

    if range.contains('/') {
        return Err("暂不支持CIDR格式,请使用范围格式如 192.168.1.1-254".to_string());
    }

    if range.contains('-') {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err("无效的范围格式".to_string());
        }
        let start: Ipv4Addr = parts[0]
            .parse()
            .map_err(|_| "无效的起始IP".to_string())?;
        let end_octet: u8 = parts[1]
            .trim()
            .parse()
            .map_err(|_| "无效的结束段".to_string())?;
        let mut current = start;
        loop {
            ips.push(IpAddr::V4(current));
            let octets = current.octets();
            if octets[3] == end_octet {
                break;
            }
            if octets[3] == 255 {
                return Err("IP超出范围".to_string());
            }
            current = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3] + 1);
        }
        return Ok(ips);
    }

    let parts: Vec<&str> = range.split(',').collect();
    for part in parts {
        let ip: IpAddr = part
            .trim()
            .parse()
            .map_err(|e| format!("无效的IP地址 {}: {}", part, e))?;
        ips.push(ip);
    }

    Ok(ips)
}

/// 解析端口列表
fn parse_ports(ports_str: &str) -> Result<Vec<u16>, String> {
    let mut ports = vec![];

    for part in ports_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() != 2 {
                return Err(format!("无效的端口范围: {}", part));
            }
            let start: u16 = range[0]
                .parse()
                .map_err(|_| "无效的起始端口".to_string())?;
            let end: u16 = range[1]
                .trim()
                .parse()
                .map_err(|_| "无效的结束端口".to_string())?;
            if start > end {
                return Err("起始端口不能大于结束端口".to_string());
            }
            for port in start..=end {
                ports.push(port);
            }
        } else {
            let port: u16 = part.parse().map_err(|_| "无效的端口".to_string())?;
            ports.push(port);
        }
    }

    if ports.is_empty() {
        return Err("至少需要一个端口".to_string());
    }

    Ok(ports)
}

/// 扫描单个端口
fn scan_port(addr: &SocketAddr, timeout_ms: u64) -> String {
    match StdTcpStream::connect_timeout(addr, Duration::from_millis(timeout_ms)) {
        Ok(_) => "开放".to_string(),
        Err(_) => "关闭".to_string(),
    }
}

/// 自动检测本机局域网网段
fn detect_local_subnet() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:53").ok()?;
    let local_ip = socket.local_addr().ok()?.ip();

    if local_ip.is_unspecified() || local_ip.is_loopback() {
        return None;
    }

    if let IpAddr::V4(ipv4) = local_ip {
        let octets = ipv4.octets();
        Some(format!("{}.{}.{}.1-254", octets[0], octets[1], octets[2]))
    } else {
        None
    }
}
