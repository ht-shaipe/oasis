//! 局域网设备扫描工具:扫描指定IP段和端口的在线设备

use gpui::{
    AppContext as _, Context, Entity, IntoElement,
    ParentElement as _, Styled, Window,  px,
};
use gpui_component::{
    ActiveTheme, checkbox::Checkbox, Disableable,
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
use std::sync::mpsc;
use std::thread;
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
            s.set_placeholder(t!("toolbox.network_scan.placeholder_timeout").to_string(), window, cx);
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
            format!("{} {}/{}({}%)", t!("toolbox.network_scan.scanning"), state.scanned, state.total, pct)
        } else if state.loading {
            t!("toolbox.network_scan.scanning").to_string()
        } else {
            t!("toolbox.network_scan.scan_button").to_string()
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
            .label(t!("toolbox.network_scan.label_clear"))
            .outline()
            .disabled(state.loading || results_is_empty)
            .on_click(move |_, _, cx| {
                clear_entity.update(cx, |this, cx| {
                    this.network_scan_clear(cx);
                });
            });

        // 渲染结果表格
        let results_guard = state.results.lock().unwrap();
        let results_data: Vec<_> = results_guard.iter().enumerate().map(|(idx, result)| {
            let status_key = result.status.clone();
            let status_color = if result.status == "开放" {
                theme.green
            } else {
                theme.muted_foreground
            };
            let row_entity = entity.clone();
            let checked = result.selected;
            let row_idx = idx;
            let ip = result.ip.clone();
            let port = result.port;
            let latency_ms = result.latency_ms;
            (idx, status_key, status_color, row_entity, checked, row_idx, ip, port, latency_ms)
        }).collect();
        drop(results_guard); // Release the lock before building UI

        let results_rows: Vec<gpui::AnyElement> = results_data
            .into_iter()
            .map(|(idx, status_key, status_color, row_entity, checked, row_idx, ip, port, latency_ms)| {
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
                                        Label::new(&ip)
                                            .text_xs()
                                            .flex_1()
                                            .font_weight(gpui::FontWeight::MEDIUM),
                                    )
                                    .child(
                                        Label::new(format!("{}", port))
                                            .text_xs()
                                            .w(px(60.))
                                            .text_color(theme.muted_foreground),
                                    )
                                    .child(
                                        Label::new(if status_key == "toolbox.network_scan.status_open" { t!("toolbox.network_scan.status_open") } else { t!("toolbox.network_scan.status_closed") })
                                            .text_xs()
                                            .w(px(50.))
                                            .text_color(status_color)
                                            .font_weight(gpui::FontWeight::MEDIUM),
                                    )
                                    .child(
                                        Label::new(
                                            latency_ms
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

        let select_all_entity = entity.clone();
        let results_for_select_all = state.results.lock().unwrap();
        let select_all_checked = state.select_all || (!results_for_select_all.is_empty() && results_for_select_all.iter().all(|r| r.selected));
        let _has_any_selected = results_for_select_all.iter().any(|r| r.selected);
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
                        Label::new(t!("toolbox.network_scan.table_ip"))
                            .text_xs()
                            .flex_1()
                            .font_weight(gpui::FontWeight::MEDIUM),
                    )
                    .child(
                        Label::new(t!("toolbox.network_scan.table_port"))
                            .text_xs()
                            .w(px(60.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Label::new(t!("toolbox.network_scan.table_status"))
                            .text_xs()
                            .w(px(50.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        Label::new(t!("toolbox.network_scan.table_latency"))
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
            .label(format!("{} ({})", t!("toolbox.network_scan.open_in_browser"), selected_count))
            .outline()
            .disabled(is_loading || !has_any_selected)
            .on_click(move |_, _, _cx| {
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
                        Label::new(t!("toolbox.network_scan.label_ip_range"))
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
                        Label::new(t!("toolbox.network_scan.ip_range_placeholder"))
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new(t!("toolbox.network_scan.label_ports"))
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
                        Label::new(t!("toolbox.network_scan.ports_format_hint"))
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new(t!("toolbox.network_scan.label_timeout_ms"))
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
                                    "{} {}",
                                    t!("toolbox.network_scan.open_ports_count", count = state.results.lock().unwrap().len()),
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
            self.network_scan.message = Some(t!("toolbox.network_scan.please_input_ip_port").to_string());
            self.network_scan.message_ok = false;
            cx.notify();
            return;
        }

        let timeout_ms: u64 = match timeout_str.parse() {
            Ok(t) if t > 0 => t,
            _ => {
                self.network_scan.message =
                    Some(t!("toolbox.network_scan.timeout_must_be_positive").to_string());
                self.network_scan.message_ok = false;
                cx.notify();
                return;
            }
        };

        let ips = match parse_ip_range(&ip_range) {
            Ok(ips) => ips,
            Err(e) => {
                self.network_scan.message = Some(t!("toolbox.network_scan.ip_parse_failed", error = e.as_str()).to_string());
                self.network_scan.message_ok = false;
                cx.notify();
                return;
            }
        };

        let ports = match parse_ports(&ports_str) {
            Ok(ports) => ports,
            Err(e) => {
                self.network_scan.message = Some(t!("toolbox.network_scan.port_parse_failed", error = e.as_str()).to_string());
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
            format!("{} {} 个目标...", t!("toolbox.network_scan.scan_button"), total)
        );
        self.network_scan.message_ok = true;
        cx.notify();

        let _entity = cx.entity().downgrade();

        // 使用标准库channel进行跨线程通信
        let (tx, rx) = mpsc::channel::<ScanMsg>();

        // 启动后台扫描线程
        thread::spawn(move || {
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
                            status: status.to_string(),
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
                // 尝试接收消息，使用短超时避免阻塞
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(msg) => {
                        match msg {
                            ScanMsg::Found(result) => {
                                // 直接在当前线程更新实体，避免嵌套cx.update()
                                if let Some(ent) = entity.upgrade() {
                                    let _ = ent.update(cx, |this, cx| {
                                        this.network_scan.results.lock().unwrap().push(result);
                                        this.network_scan.message = Some(
                                            format!(
                                                "{} {}/{}({}个开放)",
                                                t!("toolbox.network_scan.scanning"),
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
                                                "{} {}/{}({}个开放)",
                                                t!("toolbox.network_scan.scanning"),
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
                                            t!(
                                                "toolbox.network_scan.scan_complete_message",
                                                scanned = this.network_scan.total,
                                                open = open_count
                                            ).to_string()
                                        );
                                        this.network_scan.message_ok = true;
                                        cx.notify();
                                    });
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout, continue loop
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed
                        done = true;
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
        self.network_scan.message = Some(t!("toolbox.network_scan.cleared_results").to_string());
        self.network_scan.message_ok = true;
        cx.notify();
    }
}

/// 解析IP范围
fn parse_ip_range(range: &str) -> Result<Vec<IpAddr>, String> {
    let mut ips = vec![];

    if range.contains('/') {
        return Err(t!("toolbox.network_scan.cidr_not_supported").to_string());
    }

    if range.contains('-') {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err(t!("toolbox.network_scan.invalid_range_format").to_string());
        }
        let start: Ipv4Addr = parts[0]
            .parse()
            .map_err(|_| t!("toolbox.network_scan.invalid_start_ip").to_string())?;
        let end_octet: u8 = parts[1]
            .trim()
            .parse()
            .map_err(|_| t!("toolbox.network_scan.invalid_end_ip_segment").to_string())?;
        let mut current = start;
        loop {
            ips.push(IpAddr::V4(current));
            let octets = current.octets();
            if octets[3] == end_octet {
                break;
            }
            if octets[3] == 255 {
                return Err(t!("toolbox.network_scan.ip_out_of_range").to_string());
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
            .map_err(|e| format!("{} {}: {}", t!("toolbox.network_scan.invalid_start_ip"), part, e))?;
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
                return Err(t!("toolbox.network_scan.port_invalid_range", port = part).to_string());
            }
            let start: u16 = range[0]
                .parse()
                .map_err(|_| t!("toolbox.network_scan.invalid_start_port").to_string())?;
            let end: u16 = range[1]
                .trim()
                .parse()
                .map_err(|_| t!("toolbox.network_scan.invalid_end_port").to_string())?;
            if start > end {
                return Err(t!("toolbox.network_scan.port_range_order").to_string());
            }
            for port in start..=end {
                ports.push(port);
            }
        } else {
            let port: u16 = part.parse().map_err(|_| t!("toolbox.network_scan.invalid_port").to_string())?;
            ports.push(port);
        }
    }

    if ports.is_empty() {
        return Err(t!("toolbox.network_scan.at_least_one_port").to_string());
    }

    Ok(ports)
}

/// 扫描单个端口
fn scan_port(addr: &SocketAddr, timeout_ms: u64) -> &'static str {
    if StdTcpStream::connect_timeout(addr, Duration::from_millis(timeout_ms)).is_ok() {
        "toolbox.network_scan.status_open"
    } else {
        "toolbox.network_scan.status_closed"
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
