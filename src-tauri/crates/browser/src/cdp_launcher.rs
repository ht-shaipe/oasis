//! CDP Chrome 启动模块
//!
//! 负责通过 CDP (Chrome DevTools Protocol) 启动并连接 Chrome 浏览器。
//! 使用独立的 std::process::Command 管理 Chrome 进程，避免 chromiumoxide
//! runtime 导致的进程意外退出问题。

use std::path::PathBuf;

use log::{error, info};

/// CDP 调试端口
pub const CDP_DEBUG_PORT: u16 = 9222;

/// macOS 上 Google Chrome 的标准路径
#[cfg(target_os = "macos")]
const MACOS_CHROME_PATH: &str = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

/// Chrome 用户数据目录（持久化登录态）
pub fn chrome_user_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crawler-chrome-profile")
}

/// 查找系统已安装的 Chrome/Chromium 可执行文件
pub fn find_system_chrome() -> Option<PathBuf> {
    // 1. 环境变量
    if let Ok(path) = std::env::var("CHROME") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. macOS 标准路径
    #[cfg(target_os = "macos")]
    {
        let candidates = [
            MACOS_CHROME_PATH,
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }
    }

    // 3. Linux 标准路径
    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }
    }

    // 4. Windows 标准路径
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

/// 轮询等待 Chrome DevTools 端口就绪，返回 WebSocket URL
pub fn wait_for_devtools_ws(debug_url: &str) -> Option<String> {
    for i in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Ok(resp) = reqwest::blocking::get(debug_url) {
            if let Ok(body) = resp.text() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(url) = json.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
                        info!("Chrome DevTools 已就绪 (尝试 {} 次)", i + 1);
                        return Some(url.to_string());
                    }
                }
            }
        }
    }
    None
}

/// 通过 CDP 启动 Chrome 浏览器并打开指定页面
#[allow(dead_code)]
pub fn spawn_cdp_open_chrome(
    target_url: Option<String>,
    username: Option<String>,
    password: Option<String>,
) {
    info!("开始执行 CDP 打开流程");
    std::thread::spawn(move || {
        let target_url = target_url.unwrap_or_else(|| "https://google.com".to_string());
        let debug_url = format!("http://127.0.0.1:{}/json/version", CDP_DEBUG_PORT);
        let ws_url = match resolve_cdp_ws_url(&debug_url, &target_url) {
            Ok(url) => url,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                error!("创建 Tokio 运行时失败: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            if let Err(e) = open_page_via_cdp(
                &ws_url,
                &target_url,
                username.as_deref(),
                password.as_deref(),
            )
            .await
            {
                error!("{}", e);
            }
        });
    });
}

fn resolve_cdp_ws_url(debug_url: &str, target_url: &str) -> Result<String, String> {
    if let Some(ws_url) = wait_for_devtools_ws(debug_url) {
        info!(
            "检测到已有 CDP 连接，直接复用现有 Chrome，新开标签: {}",
            target_url
        );
        return Ok(ws_url);
    }

    info!("未检测到可用的 CDP 连接，准备启动 Chrome");
    let chrome_path =
        find_system_chrome().ok_or_else(|| "未找到系统 Chrome 或 Chromium".to_string())?;
    info!("使用 Chrome: {}", chrome_path.display());

    let user_data_str = chrome_user_data_dir().to_string_lossy().to_string();

    let child = std::process::Command::new(&chrome_path)
        .arg(format!("--remote-debugging-port={}", CDP_DEBUG_PORT))
        .arg(format!("--user-data-dir={}", user_data_str))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--disable-infobars")
        .arg("--window-size=1920,1080")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("启动 Chrome 进程失败: {}", e))?;

    info!(
        "Chrome 进程已启动 (PID={:?}, debug_port={})",
        child.id(),
        CDP_DEBUG_PORT
    );

    #[cfg(target_os = "macos")]
    {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Google Chrome")
            .spawn();
        info!("已发送 macOS 前台激活命令");
    }

    let ws_url = wait_for_devtools_ws(debug_url)
        .ok_or_else(|| "Chrome DevTools 端口超时未就绪".to_string())?;
    info!("Chrome DevTools 已就绪: {}", ws_url);
    Ok(ws_url)
}

async fn open_page_via_cdp(
    ws_url: &str,
    target_url: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), String> {
    info!("通过 CDP 连接到 Chrome: {}", ws_url);
    use chromiumoxide::browser::Browser;
    use futures_util::StreamExt;

    let (browser, mut handler) = Browser::connect(ws_url)
        .await
        .map_err(|e| format!("CDP 连接失败: {}", e))?;

    tokio::spawn(async move {
        while handler.next().await.is_some() {}
        info!("CDP handler stream 已结束");
    });

    let page = browser
        .new_page(target_url)
        .await
        .map_err(|e| format!("通过 CDP 打开页面失败: {}", e))?;

    info!("已通过 CDP 成功打开页面: {}", target_url);

    activate_page(&page).await;

    inject_stealth_scripts(&page).await;
    fill_login_form(&page, username, password).await;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    save_page_html(&page).await;
    Ok(())
}

#[allow(dead_code)]
async fn inject_stealth_scripts(page: &chromiumoxide::Page) {
    let stealth_js = r#"
        Object.defineProperty(Object.getPrototypeOf(navigator), 'webdriver', { get: () => false });
    "#;
    let _ = page
        .execute(
            chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams::new(
                stealth_js.to_string(),
            ),
        )
        .await;

    let _ = page
        .evaluate_expression(
            r#"
            (function() {
                var style = document.createElement('style');
                style.textContent = 'body>div:last-child{display:none!important}';
                document.head.appendChild(style);
            })();
        "#,
        )
        .await;
}

async fn activate_page(page: &chromiumoxide::Page) {
    if let Err(e) = page.bring_to_front().await {
        error!("切换标签到前台失败: {}", e);
    }

    if let Err(e) = page.activate().await {
        error!("激活标签失败: {}", e);
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "Google Chrome" to activate"#)
            .spawn();
        info!("已请求 macOS 激活 Chrome 前台窗口");
    }
}

#[allow(dead_code)]
async fn fill_login_form(
    page: &chromiumoxide::Page,
    username: Option<&str>,
    password: Option<&str>,
) {
    if username.is_none() && password.is_none() {
        return;
    }

    let username = match username {
        Some(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => return,
    };
    let password = match password {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => return,
    };

    let script = format!(
        r#"
        (function() {{
            const usernameValue = {username:?};
            const passwordValue = {password:?};
            const setValue = (element, value) => {{
                if (!element || element.disabled || element.readOnly) return false;
                const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
                if (descriptor && descriptor.set) {{
                    descriptor.set.call(element, value);
                }} else {{
                    element.value = value;
                }}
                element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }};

            const inputs = Array.from(document.querySelectorAll('input'))
                .filter((element) => !element.disabled && !element.readOnly && element.type !== 'hidden');
            const passwordInput = inputs.find((element) => {{
                const text = `${{element.type}} ${{element.name || ''}} ${{element.id || ''}} ${{element.placeholder || ''}} ${{element.autocomplete || ''}}`.toLowerCase();
                return element.type === 'password' || text.includes('password') || text.includes('passwd') || text.includes('pass');
            }});

            let usernameInput = inputs.find((element) => {{
                const text = `${{element.type}} ${{element.name || ''}} ${{element.id || ''}} ${{element.placeholder || ''}} ${{element.autocomplete || ''}}`.toLowerCase();
                return text.includes('username') || text.includes('email') || text.includes('account') || text.includes('login') || text.includes('user');
            }});

            if (!usernameInput && passwordInput && passwordInput.form) {{
                const formInputs = Array.from(passwordInput.form.querySelectorAll('input'))
                    .filter((element) => element !== passwordInput && !element.disabled && !element.readOnly && element.type !== 'hidden');
                usernameInput = formInputs.find((element) => {{
                    const text = `${{element.type}} ${{element.name || ''}} ${{element.id || ''}} ${{element.placeholder || ''}} ${{element.autocomplete || ''}}`.toLowerCase();
                    return element.type !== 'password' && (text.includes('username') || text.includes('email') || text.includes('account') || text.includes('login') || text.includes('user'));
                }}) || formInputs.find((element) => element.type !== 'password');
            }}

            const usernameFilled = usernameInput ? setValue(usernameInput, usernameValue) : false;
            const passwordFilled = passwordInput ? setValue(passwordInput, passwordValue) : false;

            if (usernameFilled && usernameInput) {{
                usernameInput.focus();
            }} else if (passwordFilled && passwordInput) {{
                passwordInput.focus();
            }}
        }})();
    "#,
        username = username,
        password = password,
    );

    let _ = page.evaluate_expression(&script).await;
}

#[allow(dead_code)]
async fn save_page_html(page: &chromiumoxide::Page) {
    match page.content().await {
        Ok(html) => {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let downloads_dir =
                dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let file_name = format!("page_google_{}.html", timestamp);
            let file_path = downloads_dir.join(&file_name);

            match std::fs::write(&file_path, &html) {
                Ok(_) => {
                    info!(
                        "页面 HTML 已保存: {} ({} bytes)",
                        file_path.display(),
                        html.len()
                    )
                }
                Err(e) => error!("保存 HTML 失败: {:?}", e),
            }
        }
        Err(e) => {
            error!("获取页面 HTML 失败: {}", e);
        }
    }
}
