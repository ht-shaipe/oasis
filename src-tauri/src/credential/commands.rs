use tauri::AppHandle;
use tauri::Manager;

use crate::credential::crypto;
use crate::credential::db;
use crate::credential::models::{
    Category, CredentialDetail, CredentialView, NewCredential, SensitiveData, UpdateCredential,
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
    cred: &crate::credential::models::Credential,
    category_name: Option<String>,
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
    }
}

#[tauri::command]
pub fn list_credentials(
    app: AppHandle,
    category_id: Option<i64>,
) -> Result<Vec<CredentialView>, String> {
    let conn = get_conn(&app)?;
    db::list_credentials(&conn, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_credential(
    app: AppHandle,
    id: i64,
    dek_base64: String,
) -> Result<CredentialDetail, String> {
    let conn = get_conn(&app)?;
    let cred = db::get_credential(&conn, id).map_err(|e| e.to_string())?;

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
    let (encrypted, nonce_bytes) =
        crypto::encrypt(&dek, plaintext).map_err(|e| format!("Encryption failed: {:?}", e))?;

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

    Ok(credential_to_view(&created, cat_name))
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

    Ok(credential_to_view(&result, cat_name))
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
