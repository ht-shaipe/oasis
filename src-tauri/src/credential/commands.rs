use tauri::AppHandle;
use tauri::Manager;

use crate::credential::crypto;
use crate::credential::db;
use crate::credential::models::{Category, CredentialView, CredentialDetail, NewCredential, UpdateCredential, SensitiveData};

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

    db::set_master_key(&conn, &key_hash, &salt, &dek_salt)
        .map_err(|e| e.to_string())?;

    // Derive DEK and return as base64
    let dek = crypto::derive_dek(&password, &dek_salt);
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &dek))
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
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &dek))
}

#[tauri::command]
pub fn change_master_key(app: AppHandle, old_password: String, new_password: String) -> Result<String, String> {
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
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &dek))
}

// ── Category Commands ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_categories(app: AppHandle) -> Result<Vec<Category>, String> {
    let conn = get_conn(&app)?;
    db::list_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_category(app: AppHandle, name: String, icon: Option<String>) -> Result<Category, String> {
    let conn = get_conn(&app)?;
    db::create_category(&conn, &name, icon.as_deref()).map_err(|e| e.to_string())
}

// ── Credential Commands ────────────────────────────────────────────────────────────

fn credential_to_view(cred: &crate::credential::models::Credential, category_name: Option<String>) -> CredentialView {
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
pub fn list_credentials(app: AppHandle, category_id: Option<i64>) -> Result<Vec<CredentialView>, String> {
    let conn = get_conn(&app)?;
    db::list_credentials(&conn, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_credential(app: AppHandle, id: i64, dek_base64: String) -> Result<CredentialDetail, String> {
    let conn = get_conn(&app)?;
    let cred = db::get_credential(&conn, id).map_err(|e| e.to_string())?;

    // Decode DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes.as_slice().try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Decrypt sensitive data
    let nonce_bytes: [u8; 12] = cred.nonce.as_slice().try_into()
        .map_err(|_| "Invalid nonce length".to_string())?;
    let plaintext = crypto::decrypt(&dek, &cred.encrypted_data, &nonce_bytes)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    let sensitive: SensitiveData = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("Failed to parse sensitive data: {}", e))?;

    // Get category name
    let cat_name: Option<String> = conn.query_row(
        "SELECT name FROM categories WHERE id = ?1",
        rusqlite::params![cred.category_id],
        |row| row.get(0),
    ).ok();

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
pub fn create_credential(app: AppHandle, credential: NewCredential) -> Result<CredentialView, String> {
    let conn = get_conn(&app)?;

    // Decode DEK
    let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &credential.dek_base64)
        .map_err(|_| "Invalid DEK base64".to_string())?;
    let dek: [u8; 32] = dek_bytes.as_slice().try_into()
        .map_err(|_| "DEK must be 32 bytes".to_string())?;

    // Encrypt sensitive_data_json
    let plaintext = credential.sensitive_data_json.as_bytes();
    let (encrypted, _nonce) = crypto::encrypt(&dek, plaintext)
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    // Create a modified NewCredential with encrypted data
    let cred_with_enc = NewCredential {
        category_id: credential.category_id,
        title: credential.title.clone(),
        username: credential.username.clone(),
        url: credential.url.clone(),
        sensitive_data_json: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted),
        dek_base64: credential.dek_base64.clone(),
        tags: credential.tags.clone(),
        notes: credential.notes.clone(),
    };

    let created = db::create_credential(&conn, &cred_with_enc).map_err(|e| e.to_string())?;

    // Get category name
    let cat_name: Option<String> = conn.query_row(
        "SELECT name FROM categories WHERE id = ?1",
        rusqlite::params![created.category_id],
        |row| row.get(0),
    ).ok();

    Ok(credential_to_view(&created, cat_name))
}

#[tauri::command]
pub fn update_credential(app: AppHandle, credential: UpdateCredential) -> Result<CredentialView, String> {
    let conn = get_conn(&app)?;

    // If sensitive data needs re-encryption
    let mut updated = credential.clone();
    if let (Some(enc_json), Some(dek_b64)) = (&credential.sensitive_data_json, &credential.dek_base64) {
        let dek_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, dek_b64)
            .map_err(|_| "Invalid DEK base64".to_string())?;
        let dek: [u8; 32] = dek_bytes.as_slice().try_into()
            .map_err(|_| "DEK must be 32 bytes".to_string())?;

        let plaintext = enc_json.as_bytes();
        let (encrypted, _nonce) = crypto::encrypt(&dek, plaintext)
            .map_err(|e| format!("Encryption failed: {:?}", e))?;

        updated.sensitive_data_json = Some(
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted)
        );
    }

    let result = db::update_credential(&conn, credential.id, &updated)
        .map_err(|e| e.to_string())?;

    let cat_name: Option<String> = conn.query_row(
        "SELECT name FROM categories WHERE id = ?1",
        rusqlite::params![result.category_id],
        |row| row.get(0),
    ).ok();

    Ok(credential_to_view(&result, cat_name))
}

#[tauri::command]
pub fn delete_credential(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_conn(&app)?;
    db::delete_credential(&conn, id).map_err(|e| e.to_string())
}
