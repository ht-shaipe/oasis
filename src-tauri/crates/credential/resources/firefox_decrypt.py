#!/usr/bin/env python3
"""
Firefox 密码解密工具
通过 ctypes 调用 Firefox 自带的 libnss3 库，使用 NSS SDR 解密 logins.json 中的加密密码。

要求：
  - macOS，已安装 Firefox
  - 支持 Python 3.8+，仅依赖标准库（ctypes/json/os/base64/sys/glob）

使用方式：
  python3 firefox_decrypt.py                # 输出 JSON 数组到 stdout
  python3 firefox_decrypt.py --profile /path/to/profile  # 指定 Profile 路径

输出格式（JSON 数组）：
  [{"id": 0, "url": "https://...", "username": "...", "password": "...", "browser": "firefox"}, ...]
"""

import ctypes
import json
import os
import sys
import base64
import glob as glob_mod
import platform
from ctypes import c_char_p, c_int, c_uint, c_void_p, POINTER, Structure, byref
from typing import Optional


# ── 常量 ────────────────────────────────────────────────────────────────────
FIREFOX_LIB_DIR = "/Applications/Firefox.app/Contents/MacOS"


# ── NSS 结构体定义 ──────────────────────────────────────────────────────────

class SECItem(Structure):
    """NSS SECItem 结构体，对应 C:
    typedef struct SECItemStr {
        SECItemType type;       // unsigned int
        unsigned char *data;    // char* 指针
        unsigned int len;
    } SECItem;
    
    注意：data 必须声明为 c_char_p（不能是 c_void_p），否则
    在 ARM64 macOS 上会导致 ctypes 对齐错误，PK11SDR_Decrypt 会 segfault。
    """
    _fields_ = [
        ("type", c_uint),
        ("data", c_char_p),
        ("len", c_uint),
    ]


# ── 辅助函数 ────────────────────────────────────────────────────────────────

def find_firefox_profile(profile_path: Optional[str] = None) -> str:
    """查找 Firefox Profile 目录"""
    if profile_path:
        if os.path.isdir(profile_path):
            return profile_path
        raise FileNotFoundError(f"Profile 不存在: {profile_path}")

    # 自动查找默认的 release profile
    home = os.path.expanduser("~")
    profiles_root = os.path.join(
        home, "Library", "Application Support", "Firefox", "Profiles"
    )
    candidates = sorted(
        glob_mod.glob(os.path.join(profiles_root, "*.default-release")),
        key=os.path.getmtime,
        reverse=True,
    )
    if not candidates:
        candidates = sorted(
            glob_mod.glob(os.path.join(profiles_root, "*.default*")),
            key=os.path.getmtime,
            reverse=True,
        )
    if not candidates:
        raise FileNotFoundError(f"未找到 Firefox Profile: {profiles_root}")

    profile = candidates[0]
    # 验证必要文件
    required = ["logins.json", "key4.db", "cert9.db"]
    missing = [f for f in required if not os.path.exists(os.path.join(profile, f))]
    if missing:
        raise FileNotFoundError(f"Profile {profile} 缺少文件: {missing}")

    return profile


def init_nss(lib_dir: str = FIREFOX_LIB_DIR) -> ctypes.CDLL:
    """加载 NSS 动态库并设置函数签名
    
    关键：必须先加载 libmozglue，再加载 libnss3，否则符号解析失败。
    """
    # 设置 DYLD_LIBRARY_PATH 以便 NSS 能找到其依赖库
    if lib_dir not in os.environ.get("DYLD_LIBRARY_PATH", "").split(":"):
        existing = os.environ.get("DYLD_LIBRARY_PATH", "")
        os.environ["DYLD_LIBRARY_PATH"] = f"{lib_dir}:{existing}" if existing else lib_dir

    # 按依赖顺序加载：mozglue → nss3 → freebl3 → softokn3
    try:
        ctypes.CDLL(os.path.join(lib_dir, "libmozglue.dylib"), ctypes.RTLD_GLOBAL)
        nss3 = ctypes.CDLL(os.path.join(lib_dir, "libnss3.dylib"), ctypes.RTLD_GLOBAL)
        ctypes.CDLL(os.path.join(lib_dir, "libfreebl3.dylib"), ctypes.RTLD_GLOBAL)
        ctypes.CDLL(os.path.join(lib_dir, "libsoftokn3.dylib"), ctypes.RTLD_GLOBAL)
    except OSError as e:
        raise RuntimeError(f"加载 NSS 库失败: {e}") from e

    SECItemPtr = POINTER(SECItem)

    # ── 函数签名定义 ──
    nss3.NSS_Init.argtypes = [c_char_p]
    nss3.NSS_Init.restype = c_int

    nss3.NSS_Shutdown.argtypes = []
    nss3.NSS_Shutdown.restype = c_int

    nss3.PK11_GetInternalKeySlot.argtypes = []
    nss3.PK11_GetInternalKeySlot.restype = c_void_p

    nss3.PK11_CheckUserPassword.argtypes = [c_void_p, c_char_p]
    nss3.PK11_CheckUserPassword.restype = c_int

    nss3.PK11_FreeSlot.argtypes = [c_void_p]
    nss3.PK11_FreeSlot.restype = None

    # SECStatus PK11SDR_Decrypt(SECItem *data, SECItem *result, void *pwdata)
    # result=NULL 会尝试原地解密，但在 ARM64 ctypes 下不稳定
    # 我们始终传入一个空的 result SECItem
    nss3.PK11SDR_Decrypt.argtypes = [SECItemPtr, SECItemPtr, c_void_p]
    nss3.PK11SDR_Decrypt.restype = c_int

    # void SECITEM_ZfreeItem(SECItem *item, PRBool freeit)
    # freeit=0: 只释放 item 结构体；freeit=1: 同时释放 item->data
    nss3.SECITEM_ZfreeItem.argtypes = [SECItemPtr, c_int]
    nss3.SECITEM_ZfreeItem.restype = None

    return nss3


def decrypt_sdr(enc_b64: str, nss3: ctypes.CDLL) -> Optional[str]:
    """使用 NSS SDR 解密单个 base64 编码的加密条目
    
    流程:
    1. Python base64 解码 → 原始密文字节
    2. 构建 input SECItem（type=siBuffer, 指向密文字节）
    3. 构建 output SECItem（空，由 PK11SDR_Decrypt 填充）
    4. PK11SDR_Decrypt(&input, &output, NULL) → 解密
    5. 从 output 读取明文密码字符串
    6. SECITEM_ZfreeItem 释放 output
    """
    encoded = enc_b64.encode("utf-8")

    # 解码 base64
    try:
        raw = base64.b64decode(encoded)
    except Exception:
        return None

    if not raw:
        return None

    # 构建输入 SECItem
    input_item = SECItem()
    input_item.type = 0  # siBuffer
    input_buf = ctypes.create_string_buffer(raw, len(raw))
    input_item.data = ctypes.cast(input_buf, c_char_p)
    input_item.len = len(raw)

    # 构建输出 SECItem（将由 PK11SDR_Decrypt 填充）
    output_item = SECItem()

    status = nss3.PK11SDR_Decrypt(byref(input_item), byref(output_item), None)
    if status != 0:
        return None

    try:
        if output_item.data and output_item.len > 0:
            return ctypes.string_at(
                output_item.data, output_item.len
            ).decode("utf-8", errors="replace")
        return ""
    finally:
        nss3.SECITEM_ZfreeItem(byref(output_item), 0)


def main():
    """主入口：读取 logins.json，解密所有密码，输出 JSON"""
    profile_path = None
    # 支持 --profile 参数指定 profile 路径
    for i, arg in enumerate(sys.argv[1:]):
        if arg == "--profile" and i + 1 < len(sys.argv) - 1:
            profile_path = sys.argv[i + 2]

    try:
        profile = find_firefox_profile(profile_path)
    except FileNotFoundError as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(1)

    # 加载 NSS 并初始化
    nss3 = init_nss()

    # NSS_Init 必须使用 "sql:" 前缀（cert9.db/key4.db 是 SQLite 格式）
    init_status = nss3.NSS_Init(f"sql:{profile}".encode())
    if init_status != 0:
        print(json.dumps({"error": f"NSS_Init 失败，返回码 {init_status}"}), file=sys.stderr)
        sys.exit(1)

    # 认证 internal key slot（空密码 = 无 Master Password）
    slot = nss3.PK11_GetInternalKeySlot()
    auth_status = nss3.PK11_CheckUserPassword(slot, b"")
    if auth_status != 0:
        nss3.PK11_FreeSlot(slot)
        nss3.NSS_Shutdown()
        print(
            json.dumps({"error": "需要 Master Password，当前不支持", "code": auth_status}),
            file=sys.stderr,
        )
        sys.exit(1)
    nss3.PK11_FreeSlot(slot)

    # 读取 logins.json
    logins_path = os.path.join(profile, "logins.json")
    with open(logins_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    logins = data.get("logins", [])

    # 解密每条记录
    results = []
    for idx, login in enumerate(logins):
        url = login.get("hostname", "")
        enc_user = login.get("encryptedUsername", "")
        enc_pass = login.get("encryptedPassword", "")

        username = decrypt_sdr(enc_user, nss3) if enc_user else ""
        password = decrypt_sdr(enc_pass, nss3) if enc_pass else ""

        results.append({
            "id": idx,
            "url": url,
            "username": username or "",
            "password": password or "",
            "browser": "firefox",
        })

    nss3.NSS_Shutdown()

    # 输出 JSON
    json.dump(results, sys.stdout, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
