/// 独立测试：验证 Rust FFI 调用 Firefox NSS 库
/// 运行: cargo run --example test_nss
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::os::raw::c_uchar;

const DLFLAGS: i32 = 0x2 | 0x8;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SECItem {
    type_: c_uint,
    data: *mut c_uchar,
    len: c_uint,
}

type NssInitFn = unsafe extern "C" fn(*const c_char) -> c_int;

fn main() {
    let lib_dir = "/Applications/Firefox.app/Contents/MacOS";

    // Step 1: 列出库文件
    println!("=== Step 1: 检查库文件 ===");
    match std::fs::read_dir(lib_dir) {
        Ok(entries) => {
            let names: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.starts_with("lib"))
                .collect();
            println!("找到 {} 个 lib* 文件: {:?}", names.len(), names);
        }
        Err(e) => {
            println!("无法读取目录 {}: {}", lib_dir, e);
            return;
        }
    }

    // Step 2: 逐个加载库
    println!("\n=== Step 2: 加载 NSS 库 (RTLD_NOW|RTLD_GLOBAL) ===");
    let libs = &["libmozglue.dylib", "libnss3.dylib", "libfreebl3.dylib", "libsoftokn3.dylib"];
    for name in libs {
        let path = std::path::Path::new(lib_dir).join(name);
        match unsafe { libloading::os::unix::Library::open(Some(&path), DLFLAGS) } {
            Ok(_) => println!("  ✅ 加载成功: {}", name),
            Err(e) => println!("  ❌ 加载失败: {} → {}", name, e),
        }
    }

    // Step 3: 加载 libnss3 并获取 NSS_Init
    println!("\n=== Step 3: 获取符号 ===");
    let nss_path = std::path::Path::new(lib_dir).join("libnss3.dylib");
    let nss_lib = match unsafe { libloading::os::unix::Library::open(Some(&nss_path), DLFLAGS) } {
        Ok(lib) => lib,
        Err(e) => { println!("❌ 加载 libnss3: {}", e); return; }
    };

    let lib: &libloading::Library = &nss_lib.into();

    // 尝试带下划线和不带下划线的符号名
    let sym_names = &["_NSS_Init", "NSS_Init"];
    for name in sym_names {
        match unsafe { lib.get::<*mut c_void>(name.as_bytes()) } {
            Ok(ptr) => println!("  ✅ 符号 {} → {:?}", name, ptr),
            Err(e) => println!("  ❌ 符号 {}: {}", name, e),
        }
    }

    // Step 4: 查找 Firefox Profile
    println!("\n=== Step 4: 查找 Profile ===");
    let home = match dirs_next::home_dir() {
        Some(h) => h,
        None => { println!("❌ 无法获取 home"); return; }
    };
    let profiles_root = home.join("Library/Application Support/Firefox/Profiles");
    println!("  Profiles 目录: {}", profiles_root.display());

    match std::fs::read_dir(&profiles_root) {
        Ok(entries) => {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    let has_logins = p.join("logins.json").exists();
                    let has_key4 = p.join("key4.db").exists();
                    let has_cert9 = p.join("cert9.db").exists();
                    println!("  {:30} logins:{} key4:{} cert9:{}",
                        p.file_name().unwrap().to_string_lossy(),
                        if has_logins { "✅" } else { "❌" },
                        if has_key4 { "✅" } else { "❌" },
                        if has_cert9 { "✅" } else { "❌" },
                    );
                }
            }
        }
        Err(e) => println!("  ❌ 无法读取: {}", e),
    }

    // Step 5: 尝试 logins.json
    let profile = home.join("Library/Application Support/Firefox/Profiles");
    let mut found = None;
    if let Ok(entries) = std::fs::read_dir(&profile) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() && p.join("logins.json").exists() && p.join("cert9.db").exists() {
                let is_default = p.file_name().map(|n| n.to_string_lossy().ends_with(".default-release")).unwrap_or(false);
                if found.is_none() || is_default {
                    found = Some(p);
                    if is_default {
                        break;
                    }
                }
            }
        }
    }

    let profile_dir = match found {
        Some(p) => {
            println!("\n使用 Profile: {}", p.display());
            p
        }
        None => {
            println!("❌ 未找到有效 Profile");
            return;
        }
    };

    // Step 6: 读取 logins.json 条目数
    let logins_path = profile_dir.join("logins.json");
    match std::fs::read_to_string(&logins_path) {
        Ok(json) => {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json) {
                let count = data["logins"].as_array().map(|a| a.len()).unwrap_or(0);
                println!("  logins.json 条目数: {}", count);
            }
        }
        Err(e) => println!("  ❌ 读取 logins.json: {}", e),
    }

    println!("\n=== Step 7: NSS_Init + PK11_GetInternalKeySlot + SDR解密 ===");

    // 提取更多符号
    let get_fn = |name: &str| -> Result<*mut c_void, String> {
        unsafe {
            lib.get::<*mut c_void>(name.as_bytes())
                .map(|b| *b)
                .map_err(|e| format!("未找到符号 {}: {}", name, e))
        }
    };

    let nss_init: unsafe extern "C" fn(*const c_char) -> c_int =
        unsafe { std::mem::transmute(get_fn("NSS_Init").unwrap()) };
    let nss_shutdown: unsafe extern "C" fn() -> c_int =
        unsafe { std::mem::transmute(get_fn("NSS_Shutdown").unwrap()) };
    let pk11_get_slot: unsafe extern "C" fn() -> *mut c_void =
        unsafe { std::mem::transmute(get_fn("PK11_GetInternalKeySlot").unwrap()) };
    let pk11_check_pw: unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int =
        unsafe { std::mem::transmute(get_fn("PK11_CheckUserPassword").unwrap()) };
    let pk11_free_slot: unsafe extern "C" fn(*mut c_void) =
        unsafe { std::mem::transmute(get_fn("PK11_FreeSlot").unwrap()) };
    let sdr_decrypt: unsafe extern "C" fn(*mut SECItem, *mut SECItem, *mut c_void) -> c_int =
        unsafe { std::mem::transmute(get_fn("PK11SDR_Decrypt").unwrap()) };
    let secitem_free: unsafe extern "C" fn(*mut SECItem, c_int) =
        unsafe { std::mem::transmute(get_fn("SECITEM_ZfreeItem").unwrap()) };

    // NSS_Init
    let init_path = CString::new(format!("sql:{}", profile_dir.display())).unwrap();
    let ret = unsafe { nss_init(init_path.as_ptr()) };
    let status_init = if ret == 0 { "✅ SECSuccess" } else { "❌ 非零返回" };
    println!("  NSS_Init(\"sql:...\") → {} (码={})", status_init, ret);

    if ret != 0 {
        println!("  ❌ NSS 初始化失败，跳过后续测试");
        return;
    }

    // PK11_GetInternalKeySlot + PK11_CheckUserPassword
    let slot = unsafe { pk11_get_slot() };
    if slot.is_null() {
        println!("  ❌ PK11_GetInternalKeySlot 返回 NULL");
        unsafe { nss_shutdown(); }
        return;
    }
    println!("  ✅ PK11_GetInternalKeySlot → {:?}", slot);

    let empty = CString::new("").unwrap();
    let pw_ret = unsafe { pk11_check_pw(slot, empty.as_ptr()) };
    let pw_status = if pw_ret == 0 { "✅ SECSuccess" } else { "❌ 非零返回" };
    println!("  PK11_CheckUserPassword(\"\") → {} (码={})", pw_status, pw_ret);
    unsafe { pk11_free_slot(slot); }

    if pw_ret != 0 {
        println!("  ❌ 可能需要 Master Password，跳过解密测试");
        unsafe { nss_shutdown(); }
        return;
    }

    // 读取 logins.json 并解密第一条
    let logins_path = profile_dir.join("logins.json");
    let json_str = std::fs::read_to_string(&logins_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let logins = data["logins"].as_array().unwrap();

    let mut ok = 0;
    let mut fail = 0;
    for (i, login) in logins.iter().enumerate() {
        let enc = login["encryptedPassword"].as_str().unwrap_or("");
        if enc.is_empty() { continue; }

        use base64::Engine;
        let raw = match base64::engine::general_purpose::STANDARD.decode(enc) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut input = SECItem { type_: 0, data: raw.as_ptr() as *mut c_uchar, len: raw.len() as c_uint };
        let mut output = SECItem { type_: 0, data: std::ptr::null_mut(), len: 0 };

        let ret = unsafe { sdr_decrypt(&mut input, &mut output, std::ptr::null_mut()) };
        if ret == 0 && !output.data.is_null() && output.len > 0 {
            let dec = unsafe {
                std::slice::from_raw_parts(output.data, output.len as usize)
            };
            let pass = String::from_utf8_lossy(dec);
            if i == 0 {
                let host = login["hostname"].as_str().unwrap_or("");
                println!("\n  示例: {} → {}", host, &*pass);
            }
            unsafe { secitem_free(&mut output, 0); }
            ok += 1;
        } else {
            if !output.data.is_null() {
                unsafe { secitem_free(&mut output, 0); }
            }
            fail += 1;
        }
    }

    println!("  解密结果: {}/{} 成功", ok, ok + fail);
    unsafe { nss_shutdown(); }
    println!("  ✅ NSS_Shutdown 完成");

    println!("\n=== 诊断完成 ===");
}