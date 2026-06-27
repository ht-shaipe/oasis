use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Manager;

use crate::crypto;
use crate::db;
use crate::models::{
    Category, CredentialDetail, CredentialView, NewCredential, SensitiveData, UpdateCredential,
    Site, SiteDetail, NewSite, UpdateSite, SiteAccount,
};
use chrono::Utc;

fn get_conn(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    db::init_db(&data_dir).map_err(|e| e.to_string())
}

// ── Master Key Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn is_master_key_set(app: AppHandle) -> Result<bool, String> {
    let conn = get_conn(&app)?;
    db::is_master_key_set(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setup_master_key(app: AppHandle, password: String) -> Result<String, String> {
    let conn = get_conn(&app)?;

    let salt = crypto::generate_salt();
    let dek_salt = crypto::generate_salt();
    let key_hash = crypto::derive_master_key(&password, &salt);

    db::set_master_key(&conn, &key_hash, &salt, &dek_salt).map_err(|e| e.to_string())?;

    // Derive DEK and return as base64
    let dek = crypto::derive_dek(&password, &dek_salt);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &dek,
    ))
}

#[tauri::command]
pub fn verify_master_key(app: AppHandle, password: String) -> Result<String, String> {
    let conn = get_conn(&app)?;

    let (salt, dek_salt) = db::get_master_key_salts(&conn).map_err(|e| e.to_string())?;
    let key_hash = crypto::derive_master_key(&password, &salt);

    if !db::verify_master_key(&conn, &key_hash).map_err(|e| e.to_string())? {
        return Err("Invalid master password".to_string());
    }

    let dek = crypto::derive_dek(&password, &dek_salt);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &dek,
    ))
}

#[tauri::command]
pub fn change_master_key(
    app: AppHandle,
    old_password: String,
    new_password: String,
) -> Result<String, String> {
    let conn = get_conn(&app)?;

    // Verify old password
    let (salt, _old_dek_salt) = db::get_master_key_salts(&conn).map_err(|e| e.to_string())?;
    let old_key_hash = crypto::derive_master_key(&old_password, &salt);

    if !db::verify_master_key(&conn, &old_key_hash).map_err(|e| e.to_string())? {
        return Err("Invalid old password".to_string());
    }

    // Generate new salts and keys
    let new_salt = crypto::generate_salt();
    let new_dek_salt = crypto::generate_salt();
    let new_key_hash = crypto::derive_master_key(&new_password, &new_salt);

    db::set_master_key(&conn, &new_key_hash, &new_salt, &new_dek_salt)
        .map_err(|e| e.to_string())?;

    let dek = crypto::derive_dek(&new_password, &new_dek_salt);
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &dek,
    ))
}

// ── Category Commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_categories(app: AppHandle) -> Result<Vec<Category>, String> {
    let conn = get_conn(&app)?;
    db::list_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_category(
    app: AppHandle,
    name: String,
    icon: Option<String>,
    parent_id: Option<i64>,
) -> Result<Category, String> {
    let conn = get_conn(&app)?;
    db::create_category(&conn, &name, icon.as_deref(), parent_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_category(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_category(&conn, id).map_err(|e| e.to_string())
}

// ── Credential Commands ────────────────────────────────────────────────────────────

fn credential_to_view(
    cred: &crate::models::Credential,
    category_name: Option<String>,
    accounts_count: Option<i64>,
) -> CredentialView {
    CredentialView {
        id: cred.id,
        category_id: cred.category_id,
        title: cred.title.clone(),
        username: cred.username.clone(),
        url: cred.url.clone(),
        tags: cred.tags.clone(),
        notes: cred.notes.clone(),
        created_at: cred.created_at.clone(),
        updated_at: cred.updated_at.clone(),
        category_name,
        accounts_count,
    }
}

#[tauri::command]
pub fn list_credentials(
    app: AppHandle,
    category_id: Option<i64>,
    dek_base64: Option<String>,
) -> Result<Vec<CredentialView>, String> {
    let conn = get_conn(&app)?;
    let raw_creds = db::list_raw_credentials(&conn).map_err(|e| e.to_string())?;

    let dek_opt: Option<[u8; 32]> = if let Some(ref dek_b64) = dek_base64 {
        let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, dek_b64)
            .map_err(|_| "Invalid DEK base64".to_string())?;
        Some(dek_bytes.as_slice().try_into().map_err(|_| "DEK must be 32 bytes".to_string())?)
    } else {
        None
    };

    let mut views = Vec::new();
    for cred in &raw_creds {
        if category_id.is_some() && cred.category_id != category_id.unwrap() {
            continue;
        }

        let cat_name: Option<String> = conn
            .query_row(
                "SELECT name FROM categories WHERE id = ?1",
                rusqlite::params![cred.category_id],
                |row| row.get(0),
            )
            .ok();

        let accounts_count = if let Some(dek) = dek_opt {
            count_accounts(&cred, dek)
        } else {
            None
        };

        views.push(credential_to_view(cred, cat_name, accounts_count));
    }

    Ok(views)
}

fn count_accounts(cred: &crate::models::Credential, dek: [u8; 32]) -> Option<i64> {
    let nonce_arr: [u8; 12] = cred.nonce.as_slice().try_into().ok()?;
    let plaintext = crypto::decrypt(&dek, &cred.encrypted_data, &nonce_arr).ok()?;
    let sensitive: SensitiveData = serde_json::from_slice(&plaintext).ok()?;
    let set_count = sensitive.sensitive_sets.as_ref().map_or(0, |s| s.len()) as i64;
    let account_count = sensitive.account_sets.as_ref().map_or(0, |s| s.len()) as i64;
    let standalone = if sensitive.password.is_some() || sensitive.api_key.is_some() {
        1
    } else {
        0
    };
    let total = set_count.max(account_count).max(standalone);
    if total > 0 { Some(total) } else { None }
}

#[tauri::command]
pub fn get_credential(
    app: AppHandle,
    id: i64,
    dek_base64: String,
) -> Result<CredentialDetail, String> {
    let conn = get_conn(&app)?;
    let cred = db::get_credential(&conn, id).map_err(|e| e.to_string())?;

    eprintln!("DEBUG: Retrieved credential id={}, cipher_len={}, nonce_len={}",
        id, cred.encrypted_data.len(), cred.nonce.len());
    eprintln!("DEBUG: Encrypted data (hex): {:02x?}", cred.encrypted_data);
    eprintln!("DEBUG: Nonce (hex): {:02x?}", cred.nonce);

    // Decode DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Decrypt sensitive data — try a few fallbacks and record which cipher/nonce succeeded.
    let try_decrypt = |cipher: &[u8], nonce_arr: [u8; 12]| -> Result<Vec<u8>, String> {
        crypto::decrypt(&dek, cipher, &nonce_arr).map_err(|e| format!("Decryption failed: {:?}", e))
    };

    // Build nonce candidate: direct 12-byte blob, or base64-decode if stored as text
    let mut nonce_candidate: Option<[u8; 12]> = None;
    if let Ok(arr) = cred.nonce.as_slice().try_into() {
        nonce_candidate = Some(arr);
    } else if let Ok(s) = std::str::from_utf8(&cred.nonce) {
        if let Ok(v) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s) {
            if v.len() >= 12 {
                let mut arr = [0u8; 12];
                arr.copy_from_slice(&v[..12]);
                nonce_candidate = Some(arr);
            }
        }
    }

    // Try decrypting and capture used values
    let mut result_opt: Option<(Vec<u8>, Vec<u8>, [u8; 12])> = None;
    if let Some(nonce_arr) = nonce_candidate {
        if let Ok(p) = try_decrypt(&cred.encrypted_data, nonce_arr) {
            result_opt = Some((p, cred.encrypted_data.clone(), nonce_arr));
        }
    }

    // Fallback: encrypted_data may be base64 text
    if result_opt.is_none() {
        if let Ok(s) = std::str::from_utf8(&cred.encrypted_data) {
            if let Ok(decoded) =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
            {
                if let Some(nonce_arr) = nonce_candidate {
                    if let Ok(p) = try_decrypt(&decoded, nonce_arr) {
                        result_opt = Some((p, decoded, nonce_arr));
                    }
                } else if let Ok(nonce_s) = std::str::from_utf8(&cred.nonce) {
                    if let Ok(nv) =
                        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nonce_s)
                    {
                        if nv.len() >= 12 {
                            let mut arr = [0u8; 12];
                            arr.copy_from_slice(&nv[..12]);
                            if let Ok(p) = try_decrypt(&decoded, arr) {
                                result_opt = Some((p, decoded, arr));
                            }
                        }
                    }
                }
            }
        }
    }

    let (plaintext, used_cipher, used_nonce) = if let Some(t) = result_opt {
        t
    } else {
        return Err(format!(
            "Decryption failed. dek_len={}, cipher_len={}, nonce_len={}",
            dek.len(),
            cred.encrypted_data.len(),
            cred.nonce.len()
        ));
    };

    // Normalize DB if we decoded base64 blobs
    let need_update = used_cipher.as_slice() != cred.encrypted_data.as_slice()
        || used_nonce.as_slice() != cred.nonce.as_slice();
    if need_update {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let _ = conn.execute(
            "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![used_cipher, used_nonce.as_ref(), now, cred.id],
        );
    }

    let sensitive: SensitiveData = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("Failed to parse sensitive data: {}", e))?;

    // Get category name
    let cat_name: Option<String> = conn
        .query_row(
            "SELECT name FROM categories WHERE id = ?1",
            rusqlite::params![cred.category_id],
            |row| row.get(0),
        )
        .ok();

    Ok(CredentialDetail {
        id: cred.id,
        category_id: cred.category_id,
        title: cred.title,
        username: cred.username,
        url: cred.url,
        sensitive_data: sensitive,
        tags: cred.tags,
        notes: cred.notes,
        created_at: cred.created_at,
        updated_at: cred.updated_at,
        category_name: cat_name,
    })
}

#[tauri::command]
pub fn create_credential(
    app: AppHandle,
    credential: NewCredential,
) -> Result<CredentialView, String> {
    let conn = get_conn(&app)?;

    // Decode DEK
    let dek_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &credential.dek_base64,
    )
    .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Encrypt sensitive_data_json
    let plaintext = credential.sensitive_data_json.as_bytes();
    eprintln!("DEBUG: Encrypting plaintext (len={}): {}", plaintext.len(), credential.sensitive_data_json);
    let (encrypted, nonce_bytes) =
        crypto::encrypt(&dek, plaintext).map_err(|e| format!("Encryption failed: {:?}", e))?;
    eprintln!("DEBUG: Encrypted result (len={}, nonce_len={})", encrypted.len(), nonce_bytes.len());

    // Create a modified NewCredential with encrypted data
    let cred_with_enc = NewCredential {
        category_id: credential.category_id,
        title: credential.title.clone(),
        username: credential.username.clone(),
        url: credential.url.clone(),
        sensitive_data_json: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &encrypted,
        ),
        dek_base64: credential.dek_base64.clone(),
        nonce_base64: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &nonce_bytes,
        ),
        tags: credential.tags.clone(),
        notes: credential.notes.clone(),
    };

    let created = db::create_credential(&conn, &cred_with_enc).map_err(|e| e.to_string())?;

    // Get category name
    let cat_name: Option<String> = conn
        .query_row(
            "SELECT name FROM categories WHERE id = ?1",
            rusqlite::params![created.category_id],
            |row| row.get(0),
        )
        .ok();

    Ok(credential_to_view(&created, cat_name, None))
}

#[tauri::command]
pub fn update_credential(
    app: AppHandle,
    credential: UpdateCredential,
) -> Result<CredentialView, String> {
    let conn = get_conn(&app)?;

    // If sensitive data needs re-encryption
    let mut updated = credential.clone();
    if let (Some(enc_json), Some(dek_b64)) =
        (&credential.sensitive_data_json, &credential.dek_base64)
    {
        let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, dek_b64)
            .map_err(|_| "Invalid DEK base64".to_string())?;
        let dek: [u8; 32] = dek_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "DEK must be 32 bytes".to_string())?;

        let plaintext = enc_json.as_bytes();
        let (encrypted, nonce_bytes) =
            crypto::encrypt(&dek, plaintext).map_err(|e| format!("Encryption failed: {:?}", e))?;
        updated.sensitive_data_json = Some(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &encrypted,
        ));
        updated.nonce_base64 = Some(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &nonce_bytes,
        ));
    }

    let result =
        db::update_credential(&conn, credential.id, &updated).map_err(|e| e.to_string())?;

    let cat_name: Option<String> = conn
        .query_row(
            "SELECT name FROM categories WHERE id = ?1",
            rusqlite::params![result.category_id],
            |row| row.get(0),
        )
        .ok();

    Ok(credential_to_view(&result, cat_name, None))
}

#[tauri::command]
pub fn delete_credential(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_credential(&conn, id).map_err(|e| e.to_string())
}

// Diagnostic helper: try multiple decryption variants and optionally fix DB (normalize binary fields)
#[tauri::command]
pub fn diagnose_credential(
    app: AppHandle,
    id: i64,
    dek_base64: String,
    fix: bool,
) -> Result<String, String> {
    let conn = get_conn(&app)?;
    let cred = db::get_credential(&conn, id).map_err(|e| e.to_string())?;

    // Decode provided DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Helpers
    let try_decrypt = |cipher: &[u8], nonce_arr: [u8; 12]| -> Result<(), String> {
        crypto::decrypt(&dek, cipher, &nonce_arr)
            .map(|_p| ())
            .map_err(|e| format!("{:?}", e))
    };

    let mut attempts = Vec::new();

    // candidate raw
    let cipher_raw = cred.encrypted_data.clone();
    let nonce_raw = cred.nonce.clone();

    // attempt 1: raw/raw if nonce len>=12
    if nonce_raw.len() >= 12 {
        let mut arr = [0u8; 12];
        arr.copy_from_slice(&nonce_raw[..12]);
        let res = try_decrypt(&cipher_raw, arr);
        let ok = res.is_ok();
        let err = res.as_ref().err().cloned();
        attempts.push(("raw_cipher/raw_nonce".to_string(), ok, err.clone()));
        if ok && fix {
            // write back raw values (they already are raw)
            let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let _ = conn.execute(
                "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![cipher_raw, nonce_raw.as_slice(), now, id],
            );
        }
    }

    // attempt 2: base64-decode cipher text, raw nonce
    if let Ok(s) = std::str::from_utf8(&cipher_raw) {
        if let Ok(decoded) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s) {
            if nonce_raw.len() >= 12 {
                let mut arr = [0u8; 12];
                arr.copy_from_slice(&nonce_raw[..12]);
                let res = try_decrypt(&decoded, arr);
                let ok = res.is_ok();
                let err = res.as_ref().err().cloned();
                attempts.push(("b64_cipher/raw_nonce".to_string(), ok, err.clone()));
                if ok && fix {
                    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                    let _ = conn.execute(
                        "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
                        rusqlite::params![decoded, nonce_raw.as_slice(), now, id],
                    );
                }
            }

            // attempt 3: b64 cipher, b64 nonce
            if let Ok(nonce_s) = std::str::from_utf8(&nonce_raw) {
                if let Ok(nv) =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nonce_s)
                {
                    if nv.len() >= 12 {
                        let mut arr = [0u8; 12];
                        arr.copy_from_slice(&nv[..12]);
                        let res = try_decrypt(&decoded, arr);
                        let ok = res.is_ok();
                        let err = res.as_ref().err().cloned();
                        attempts.push(("b64_cipher/b64_nonce".to_string(), ok, err.clone()));
                        if ok && fix {
                            let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                            let _ = conn.execute(
                                "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
                                rusqlite::params![decoded, nv.as_slice(), now, id],
                            );
                        }
                    }
                }
            }
        }
    }

    // attempt 4: raw cipher, b64 nonce
    if let Ok(nonce_s) = std::str::from_utf8(&nonce_raw) {
        if let Ok(nv) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nonce_s)
        {
            if nv.len() >= 12 {
                let mut arr = [0u8; 12];
                arr.copy_from_slice(&nv[..12]);
                let res = try_decrypt(&cipher_raw, arr);
                let ok = res.is_ok();
                let err = res.as_ref().err().cloned();
                attempts.push(("raw_cipher/b64_nonce".to_string(), ok, err.clone()));
                if ok && fix {
                    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                    let _ = conn.execute(
                        "UPDATE credentials SET encrypted_data = ?1, nonce = ?2, updated_at = ?3 WHERE id = ?4",
                        rusqlite::params![cipher_raw, nv.as_slice(), now, id],
                    );
                }
            }
        }
    }

    // Build report
    let mut report = format!(
        "credential id={} dek_len={} cipher_len={} nonce_len={}\n",
        id,
        dek.len(),
        cred.encrypted_data.len(),
        cred.nonce.len()
    );
    for (name, ok, err) in attempts {
        report.push_str(&format!(
            "attempt {} => {}\n",
            name,
            if ok { "ok" } else { "fail" }
        ));
        if let Some(e) = err {
            report.push_str(&format!("  err: {}\n", e));
        }
    }

    Ok(report)
}

// ── Site Commands ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_sites(
    app: AppHandle,
    category_id: Option<i64>,
) -> Result<Vec<Site>, String> {
    let conn = get_conn(&app)?;
    db::list_sites(&conn, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_site(app: AppHandle, id: i64, dek_base64: String) -> Result<SiteDetail, String> {
    let conn = get_conn(&app)?;

    // Decode DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Get site info
    let site = db::get_site(&conn, id).map_err(|e| e.to_string())?;

    // Get encrypted accounts
    let encrypted_accounts = db::get_encrypted_site_accounts(&conn, id).map_err(|e| e.to_string())?;

    // Decrypt each account
    let mut accounts = Vec::new();
    for (username, pwd_enc, pwd_nonce, api_key_enc, api_key_nonce, secret_enc, secret_nonce) in encrypted_accounts {
        // Decrypt password
        let pwd_nonce_arr: [u8; 12] = pwd_nonce.as_slice()
            .try_into()
            .map_err(|_| "Invalid password nonce".to_string())?;
        let pwd_plain = crypto::decrypt(&dek, &pwd_enc, &pwd_nonce_arr)
            .map_err(|e| format!("Failed to decrypt password: {:?}", e))?;
        let password = String::from_utf8(pwd_plain)
            .map_err(|e| format!("Invalid password UTF-8: {}", e))?;

        // Build account
        let mut account = SiteAccount {
            username,
            password,
            api_key: None,
            secret_key: None,
        };

        // Decrypt api_key if present
        if let (Some(akey_enc), Some(akey_nonce)) = (api_key_enc, api_key_nonce) {
            let akey_nonce_arr: [u8; 12] = akey_nonce.as_slice()
                .try_into()
                .map_err(|_| "Invalid API key nonce".to_string())?;
            let akey_plain = crypto::decrypt(&dek, &akey_enc, &akey_nonce_arr)
                .map_err(|e| format!("Failed to decrypt API key: {:?}", e))?;
            account.api_key = Some(String::from_utf8(akey_plain)
                .map_err(|e| format!("Invalid API key UTF-8: {}", e))?);
        }

        // Decrypt secret_key if present
        if let (Some(skey_enc), Some(skey_nonce)) = (secret_enc, secret_nonce) {
            let skey_nonce_arr: [u8; 12] = skey_nonce.as_slice()
                .try_into()
                .map_err(|_| "Invalid secret key nonce".to_string())?;
            let skey_plain = crypto::decrypt(&dek, &skey_enc, &skey_nonce_arr)
                .map_err(|e| format!("Failed to decrypt secret key: {:?}", e))?;
            account.secret_key = Some(String::from_utf8(skey_plain)
                .map_err(|e| format!("Invalid secret key UTF-8: {}", e))?);
        }

        accounts.push(account);
    }

    Ok(SiteDetail {
        id: site.id,
        name: site.name,
        url: site.url,
        category_id: site.category_id,
        tags: site.tags,
        notes: site.notes,
        created_at: site.created_at,
        updated_at: site.updated_at,
        category_name: site.category_name,
        accounts,
    })
}

#[tauri::command]
pub fn create_site(app: AppHandle, site: NewSite) -> Result<Site, String> {
    let conn = get_conn(&app)?;

    // Decode DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &site.dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Create site record
    let site_id = db::create_site(
        &conn,
        &site.name,
        site.url.as_deref(),
        site.category_id,
        site.tags.as_deref(),
        site.notes.as_deref(),
        "", // placeholder for accounts_json
    ).map_err(|e| e.to_string())?;

    // Encrypt and create each account
    for account in &site.accounts {
        // Encrypt password
        let (pwd_enc, pwd_nonce) = crypto::encrypt(&dek, account.password.as_bytes())
            .map_err(|e| format!("Failed to encrypt password: {:?}", e))?;

        // Encrypt api_key if present
        let (api_key_enc, api_key_nonce) = if let Some(ref key) = account.api_key {
            let (enc, nonce) = crypto::encrypt(&dek, key.as_bytes())
                .map_err(|e| format!("Failed to encrypt API key: {:?}", e))?;
            (Some(enc), Some(nonce))
        } else {
            (None, None)
        };

        // Encrypt secret_key if present
        let (secret_key_enc, secret_key_nonce) = if let Some(ref key) = account.secret_key {
            let (enc, nonce) = crypto::encrypt(&dek, key.as_bytes())
                .map_err(|e| format!("Failed to encrypt secret key: {:?}", e))?;
            (Some(enc), Some(nonce))
        } else {
            (None, None)
        };

        // Create account record
        db::create_site_account(
            &conn,
            site_id,
            &account.username,
            &pwd_enc,
            &pwd_nonce,
            api_key_enc.as_deref(),
            api_key_nonce.as_ref().map(|v| v.as_slice()),
            secret_key_enc.as_deref(),
            secret_key_nonce.as_ref().map(|v| v.as_slice()),
        ).map_err(|e| e.to_string())?;
    }

    // Get category name
    let cat_name: Option<String> = conn
        .query_row(
            "SELECT name FROM categories WHERE id = ?1",
            rusqlite::params![site.category_id],
            |row| row.get(0),
        )
        .ok();

    // Get site with timestamp
    let site_data: Site = conn.query_row(
        "SELECT id, name, url, category_id, tags, notes, created_at, updated_at, NULL, 0
         FROM sites WHERE id = ?1",
        rusqlite::params![site_id],
        |row| {
            Ok(Site {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                category_id: row.get(3)?,
                tags: row.get(4)?,
                notes: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                category_name: row.get(8)?,
                accounts_count: row.get(9)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    // Get accounts count
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM site_accounts WHERE site_id = ?1",
            rusqlite::params![site_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(Site {
        id: site_id,
        name: site_data.name,
        url: site_data.url,
        category_id: site_data.category_id,
        tags: site_data.tags,
        notes: site_data.notes,
        created_at: site_data.created_at,
        updated_at: site_data.updated_at,
        category_name: cat_name,
        accounts_count: Some(count),
    })
}

#[tauri::command]
pub fn update_site(app: AppHandle, site: UpdateSite) -> Result<Site, String> {
    let conn = get_conn(&app)?;

    // Update site fields
    db::update_site(
        &conn,
        site.id,
        site.name.as_deref(),
        site.url.as_deref(),
        site.category_id,
        site.tags.as_deref(),
        site.notes.as_deref(),
    ).map_err(|e| e.to_string())?;

    // Update accounts if provided
    if let (Some(accounts), Some(dek_base64)) = (site.accounts, &site.dek_base64) {
        // Decode DEK
        let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, dek_base64)
            .map_err(|_| "Invalid DEK base64".to_string())?;
        let dek: [u8; 32] = dek_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "DEK must be 32 bytes".to_string())?;

        // Delete existing accounts
        db::delete_site_accounts(&conn, site.id).map_err(|e| e.to_string())?;

        // Create new accounts
        for account in &accounts {
            // Encrypt password
            let (pwd_enc, pwd_nonce) = crypto::encrypt(&dek, account.password.as_bytes())
                .map_err(|e| format!("Failed to encrypt password: {:?}", e))?;

            // Encrypt api_key if present
            let (api_key_enc, api_key_nonce) = if let Some(ref key) = account.api_key {
                let (enc, nonce) = crypto::encrypt(&dek, key.as_bytes())
                    .map_err(|e| format!("Failed to encrypt API key: {:?}", e))?;
                (Some(enc), Some(nonce))
            } else {
                (None, None)
            };

            // Encrypt secret_key if present
            let (secret_key_enc, secret_key_nonce) = if let Some(ref key) = account.secret_key {
                let (enc, nonce) = crypto::encrypt(&dek, key.as_bytes())
                    .map_err(|e| format!("Failed to encrypt secret key: {:?}", e))?;
                (Some(enc), Some(nonce))
            } else {
                (None, None)
            };

            // Create account record
            db::create_site_account(
                &conn,
                site.id,
                &account.username,
                &pwd_enc,
                &pwd_nonce,
                api_key_enc.as_deref(),
                api_key_nonce.as_ref().map(|v| v.as_slice()),
                secret_key_enc.as_deref(),
                secret_key_nonce.as_ref().map(|v| v.as_slice()),
            ).map_err(|e| e.to_string())?;
        }
    }

    // Get updated site
    let sites = db::list_sites(&conn, None).map_err(|e| e.to_string())?;
    sites.into_iter()
        .find(|s| s.id == site.id)
        .ok_or_else(|| "Site not found after update".to_string())
}

#[tauri::command]
pub fn delete_site(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_site(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_sites(app: AppHandle, query: String) -> Result<Vec<Site>, String> {
    let conn = get_conn(&app)?;
    db::search_sites(&conn, &query).map_err(|e| e.to_string())
}

// ── Merge / Tidy Credentials ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergeGroup {
    pub hostname: String,
    pub credential_count: i64,
    pub usernames: Vec<String>,
    pub is_intranet: bool,
    pub sample_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergePreview {
    pub total_credentials: i64,
    pub duplicates_count: i64,
    pub merge_groups: Vec<MergeGroup>,
    pub intranet_groups_count: i64,
    pub intranet_credential_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergeResult {
    pub duplicates_removed: i64,
    pub sites_created: i64,
    pub accounts_created: i64,
    pub credentials_remaining: i64,
    pub intranet_skipped: i64,
}

/// 预览整理结果：返回分组信息，不做任何修改
#[tauri::command]
pub async fn preview_merge_by_url(app: AppHandle, dek_base64: String) -> Result<MergePreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
    let conn = get_conn(&app)?;

    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    let all_creds = db::list_raw_credentials(&conn).map_err(|e| e.to_string())?;
    let creds_with_url: Vec<_> = all_creds
        .iter()
        .filter(|c| c.url.is_some() && !c.url.as_ref().unwrap().is_empty())
        .collect();

    let normalize_host = |url: &str| -> String {
        let url = url.trim();
        let stripped = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .or_else(|| url.strip_prefix("www."))
            .unwrap_or(url);
        if let Some(slash_pos) = stripped.find('/') {
            stripped[..slash_pos].to_lowercase()
        } else {
            stripped.to_lowercase()
        }
    };

    let decrypt_cred = |cred: &crate::models::Credential| -> Option<(String, String)> {
        let nonce_arr: [u8; 12] = cred.nonce.as_slice().try_into().ok()?;
        let plaintext = crypto::decrypt(&dek, &cred.encrypted_data, &nonce_arr).ok()?;
        let sensitive: SensitiveData = serde_json::from_slice(&plaintext).ok()?;
        if let Some(sets) = &sensitive.sensitive_sets {
            if let Some(first) = sets.first() {
                let username = first.username.clone()
                    .or_else(|| cred.username.clone())
                    .unwrap_or_default();
                let password = first.password.clone().unwrap_or_default();
                return Some((username, password));
            }
        }
        if let Some(sets) = &sensitive.account_sets {
            if let Some(first) = sets.first() {
                return Some((first.username.clone(), first.password.clone().unwrap_or_default()));
            }
        }
        let username = cred.username.clone().unwrap_or_default();
        let password = sensitive.password.unwrap_or_default();
        Some((username, password))
    };

    // Count duplicates
    let mut seen: std::collections::HashMap<(String, String, String), i64> = std::collections::HashMap::new();
    let mut dedup_count: i64 = 0;

    for cred in &creds_with_url {
        let url = cred.url.as_deref().unwrap();
        if let Some((username, password)) = decrypt_cred(cred) {
            let key = (url.to_lowercase(), username, password);
            if seen.contains_key(&key) {
                dedup_count += 1;
            } else {
                seen.insert(key, cred.id);
            }
        }
    }

    // Group by hostname
    let mut host_groups: std::collections::HashMap<String, Vec<&crate::models::Credential>> =
        std::collections::HashMap::new();
    for cred in &creds_with_url {
        let host = normalize_host(cred.url.as_deref().unwrap());
        host_groups.entry(host).or_default().push(cred);
    }

    let mut merge_groups: Vec<MergeGroup> = Vec::new();
    let mut intranet_groups_count: i64 = 0;
    let mut intranet_credential_count: i64 = 0;

    for (host, creds) in &host_groups {
        let is_intranet = is_intranet_url(
            creds.first().and_then(|c| c.url.as_deref()).unwrap_or(""),
        );
        if is_intranet {
            intranet_groups_count += 1;
            intranet_credential_count += creds.len() as i64;
        }

        let mut usernames = std::collections::HashSet::new();
        for c in creds {
            if let Some((u, _)) = decrypt_cred(c) {
                if !u.is_empty() {
                    usernames.insert(u);
                }
            }
        }

        if creds.len() >= 2 && usernames.len() >= 2 {
            merge_groups.push(MergeGroup {
                hostname: host.clone(),
                credential_count: creds.len() as i64,
                usernames: usernames.into_iter().collect(),
                is_intranet,
                sample_url: creds.first().and_then(|c| c.url.clone()).unwrap_or_default(),
            });
        }
    }

    merge_groups.sort_by(|a, b| {
        b.credential_count.cmp(&a.credential_count)
            .then(a.hostname.cmp(&b.hostname))
    });

    Ok(MergePreview {
        total_credentials: all_creds.len() as i64,
        duplicates_count: dedup_count,
        merge_groups,
        intranet_groups_count,
        intranet_credential_count,
    })
    }).await.map_err(|e| format!("task join error: {}", e))?
}

/// 整理凭证：真正的去重 + 同域名多账号合并
///
/// 逻辑：
/// 1. **去重**：URL + 用户名 + 密码完全相同的凭证，只保留最早创建的一条，删除其余
/// 2. **合并**：同域名下有多个不同账号的凭证，合并为一个 Site + 多个 SiteAccount
///    - 仅当同一域名下有 ≥2 条凭证且用户名不同时才合并
///    - 合并后原 credential 全部删除
/// 3. **不动**：域名下只有 1 条凭证的，保持原样
#[tauri::command]
pub async fn merge_credentials_by_url(app: AppHandle, dek_base64: String, filter_intranet: bool) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
    let conn = get_conn(&app)?;

    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    conn.execute_batch("BEGIN TRANSACTION").map_err(|e| format!("Failed to begin transaction: {}", e))?;

    let result = (|| -> Result<MergeResult, String> {
    let all_creds = db::list_raw_credentials(&conn).map_err(|e| format!("Failed to list credentials: {}", e))?;
    let creds_with_url: Vec<_> = all_creds
        .iter()
        .filter(|c| c.url.is_some() && !c.url.as_ref().unwrap().is_empty())
        .collect();

    let normalize_host = |url: &str| -> String {
        let url = url.trim();
        let stripped = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .or_else(|| url.strip_prefix("www."))
            .unwrap_or(url);
        if let Some(slash_pos) = stripped.find('/') {
            stripped[..slash_pos].to_lowercase()
        } else {
            stripped.to_lowercase()
        }
    };

    let decrypt_cred = |cred: &crate::models::Credential| -> Option<(String, String)> {
        let nonce_arr: [u8; 12] = cred.nonce.as_slice().try_into().ok()?;
        let plaintext = crypto::decrypt(&dek, &cred.encrypted_data, &nonce_arr).ok()?;
        let sensitive: SensitiveData = serde_json::from_slice(&plaintext).ok()?;
        if let Some(sets) = &sensitive.sensitive_sets {
            if let Some(first) = sets.first() {
                let username = first.username.clone()
                    .or_else(|| cred.username.clone())
                    .unwrap_or_default();
                let password = first.password.clone().unwrap_or_default();
                return Some((username, password));
            }
        }
        if let Some(sets) = &sensitive.account_sets {
            if let Some(first) = sets.first() {
                return Some((first.username.clone(), first.password.clone().unwrap_or_default()));
            }
        }
        let username = cred.username.clone().unwrap_or_default();
        let password = sensitive.password.unwrap_or_default();
        Some((username, password))
    };

    // ── Phase 1: Deduplicate — same URL + username + password, keep earliest ──

    let mut dedup_ids_to_delete: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut seen: std::collections::HashMap<(String, String, String), i64> = std::collections::HashMap::new();

    for cred in &creds_with_url {
        let url = cred.url.as_deref().unwrap();
        if let Some((username, password)) = decrypt_cred(cred) {
            let key = (url.to_lowercase(), username, password);
            if let Some(&existing_id) = seen.get(&key) {
                if cred.id > existing_id {
                    dedup_ids_to_delete.insert(cred.id);
                } else {
                    dedup_ids_to_delete.insert(existing_id);
                    seen.insert(key, cred.id);
                }
            } else {
                seen.insert(key, cred.id);
            }
        }
    }

    for id in &dedup_ids_to_delete {
        db::delete_credential(&conn, *id).map_err(|e| format!("Failed to delete duplicate credential {}: {}", id, e))?;
    }

    // ── Phase 2: Merge — same hostname, different usernames → Site + SiteAccounts ──

    let remaining_creds: Vec<_> = creds_with_url
        .iter()
        .filter(|c| !dedup_ids_to_delete.contains(&c.id))
        .collect();

    let mut host_groups: std::collections::HashMap<String, Vec<&crate::models::Credential>> =
        std::collections::HashMap::new();
    for cred in &remaining_creds {
        let host = normalize_host(cred.url.as_deref().unwrap());
        host_groups.entry(host).or_default().push(cred);
    }

    // Collect valid category IDs to avoid foreign key violations
    let valid_category_ids: std::collections::HashSet<i64> = db::list_categories(&conn)
        .map_err(|e| format!("Failed to list categories: {}", e))?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let fallback_category_id = *valid_category_ids.iter().next().ok_or_else(|| "No categories found".to_string())?;

    let multi_account_groups: Vec<_> = host_groups
        .into_iter()
        .filter(|(_, creds)| {
            if creds.len() < 2 {
                return false;
            }
            let mut usernames = std::collections::HashSet::new();
            for c in creds {
                if let Some((u, _)) = decrypt_cred(c) {
                    if !u.is_empty() {
                        usernames.insert(u);
                    }
                }
            }
            usernames.len() >= 2
        })
        .collect();

    let mut intranet_skipped: i64 = 0;

    let mut result = MergeResult {
        duplicates_removed: dedup_ids_to_delete.len() as i64,
        sites_created: 0,
        accounts_created: 0,
        credentials_remaining: 0,
        intranet_skipped: 0,
    };

    for (host, creds) in multi_account_groups {
        if filter_intranet {
            let sample_url = creds.first().and_then(|c| c.url.as_deref()).unwrap_or("");
            if is_intranet_url(sample_url) {
                intranet_skipped += creds.len() as i64;
                continue;
            }
        }
        let mut seen_pairs: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        let mut unique_accounts: Vec<(String, String)> = Vec::new();
        let mut cred_ids_to_delete: Vec<i64> = Vec::new();

        for cred in &creds {
            cred_ids_to_delete.push(cred.id);
            if let Some((username, password)) = decrypt_cred(cred) {
                if username.is_empty() && password.is_empty() {
                    continue;
                }
                let pair = (username.clone(), password.clone());
                if seen_pairs.contains(&pair) {
                    continue;
                }
                seen_pairs.insert(pair);
                unique_accounts.push((username, password));
            }
        }

        if unique_accounts.len() < 2 {
            continue;
        }

        let site_url = creds
            .first()
            .and_then(|c| c.url.clone())
            .unwrap_or_default();

        let site_name = if host.len() < creds.first().map(|c| c.title.len()).unwrap_or(0) {
            creds.first().map(|c| c.title.clone()).unwrap_or_else(|| host.clone())
        } else {
            host.clone()
        };

        let raw_category_id = creds.first().map(|c| c.category_id).unwrap_or(fallback_category_id);
        let category_id = if valid_category_ids.contains(&raw_category_id) {
            raw_category_id
        } else {
            fallback_category_id
        };

        let all_tags: Vec<String> = creds
            .iter()
            .filter_map(|c| c.tags.clone())
            .flat_map(|t: String| t.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
            .filter(|t| !t.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let tags = if all_tags.is_empty() { None } else { Some(all_tags.join(",")) };

        let site_id = db::create_site(
            &conn,
            &site_name,
            Some(&site_url),
            category_id,
            tags.as_deref(),
            None,
            "",
        )
        .map_err(|e| format!("Failed to create site '{}': {}", site_name, e))?;

        for (username, password) in &unique_accounts {
            let (pwd_enc, pwd_nonce) = crypto::encrypt(&dek, password.as_bytes())
                .map_err(|e| format!("Failed to encrypt password for '{}': {:?}", username, e))?;

            db::create_site_account(
                &conn,
                site_id,
                username,
                &pwd_enc,
                &pwd_nonce,
                None, None, None, None,
            )
            .map_err(|e| format!("Failed to create site account for '{}': {}", username, e))?;

            result.accounts_created += 1;
        }

        result.sites_created += 1;

        for id in &cred_ids_to_delete {
            db::delete_credential(&conn, *id).map_err(|e| format!("Failed to delete credential {}: {}", id, e))?;
        }
    }

    let total_remaining = db::list_raw_credentials(&conn).map_err(|e| format!("Failed to count remaining credentials: {}", e))?.len();
    result.credentials_remaining = total_remaining as i64;
    result.intranet_skipped = intranet_skipped;

    Ok(result)
    })();

    match result {
        Ok(r) => {
            conn.execute_batch("COMMIT").map_err(|e| {
                let _ = conn.execute_batch("ROLLBACK");
                format!("Failed to commit merge transaction: {}", e)
            })?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
    }).await.map_err(|e| format!("task join error: {}", e))?
}

use crate::browser_import::{self, BrowserCredential};

#[tauri::command]
pub fn import_csv_passwords(
    csv_path: String,
) -> Result<Vec<BrowserCredential>, String> {
    let mut creds = browser_import::parse_csv_passwords(&csv_path)?;

    creds.sort_by_key(|c| if c.password.is_empty() { 1 } else { 0 });

    eprintln!(
        "[import_csv_passwords] total={} with_pw={}",
        creds.len(),
        creds.iter().filter(|c| !c.password.is_empty()).count()
    );

    Ok(creds)
}

// ── Batch Import ────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct BatchImportItem {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchImportRequest {
    pub items: Vec<BatchImportItem>,
    pub category_id: i64,
    pub dek_base64: String,
    pub credential_type: String,
    pub filter_intranet: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchImportResult {
    pub imported: i64,
    pub skipped_intranet: i64,
    pub skipped_empty: i64,
    pub failed: i64,
}

fn is_intranet_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    let stripped = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);

    let host = if let Some(slash) = stripped.find('/') {
        &stripped[..slash]
    } else {
        stripped
    };

    let host = host.trim_start_matches("www.");

    if host.starts_with("localhost")
        || host.starts_with("127.")
        || host.starts_with("0.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.contains("::1")
        || !host.contains('.')
    {
        return true;
    }

    if let Some(ip_part) = host.split(':').next() {
        if ip_part.starts_with("10.")
            || ip_part.starts_with("192.168.")
            || ip_part.starts_with("127.")
            || ip_part.starts_with("0.")
        {
            return true;
        }
        if ip_part.starts_with("172.") {
            if let Some(second) = ip_part.strip_prefix("172.").and_then(|s| s.split('.').next()) {
                if let Ok(n) = second.parse::<u8>() {
                    if (16..=31).contains(&n) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[tauri::command]
pub async fn batch_import_credentials(
    app: AppHandle,
    request: BatchImportRequest,
) -> Result<BatchImportResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = get_conn(&app)?;

        let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &request.dek_base64)
            .map_err(|_| "Invalid DEK base64".to_string())?;
        let dek: [u8; 32] = dek_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "DEK must be 32 bytes".to_string())?;

        let mut result = BatchImportResult {
            imported: 0,
            skipped_intranet: 0,
            skipped_empty: 0,
            failed: 0,
        };

        conn.execute_batch("BEGIN TRANSACTION").map_err(|e| e.to_string())?;

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut stmt = conn.prepare(
            "INSERT INTO credentials (category_id, title, username, url, encrypted_data, nonce, tags, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)"
        ).map_err(|e| e.to_string())?;

        for item in &request.items {
            if request.filter_intranet && is_intranet_url(&item.url) {
                result.skipped_intranet += 1;
                continue;
            }

            if item.password.is_empty() && item.username.is_empty() {
                result.skipped_empty += 1;
                continue;
            }

            let title = if !item.url.is_empty() {
                item.url.clone()
            } else if !item.username.is_empty() {
                item.username.clone()
            } else {
                "未命名".to_string()
            };

            let sensitive_data = SensitiveData {
                credential_type: Some(request.credential_type.clone()),
                password: if item.password.is_empty() { None } else { Some(item.password.clone()) },
                ..Default::default()
            };

            let plaintext = serde_json::to_vec(&sensitive_data)
                .map_err(|e| format!("serialize failed: {}", e))?;

            let (encrypted, nonce_bytes) = crypto::encrypt(&dek, &plaintext)
                .map_err(|e| format!("Encryption failed: {:?}", e))?;

            let username_opt = if item.username.is_empty() { None } else { Some(item.username.as_str()) };
            let url_opt = if item.url.is_empty() { None } else { Some(item.url.as_str()) };

            match stmt.execute(params![
                request.category_id,
                title,
                username_opt,
                url_opt,
                encrypted,
                nonce_bytes.as_ref(),
                None as Option<&str>,
                None as Option<&str>,
                now,
            ]) {
                Ok(_) => result.imported += 1,
                Err(e) => {
                    eprintln!("[batch_import] insert failed for '{}': {}", title, e);
                    result.failed += 1;
                }
            }
        }

        drop(stmt);

        conn.execute_batch("COMMIT").map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK");
            format!("commit failed: {}", e)
        })?;

        eprintln!(
            "[batch_import] done: imported={} intranet_skipped={} empty_skipped={} failed={}",
            result.imported, result.skipped_intranet, result.skipped_empty, result.failed
        );

        Ok(result)
    }).await.map_err(|e| format!("task join error: {}", e))?
}
