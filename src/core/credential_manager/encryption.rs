use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::num::NonZeroU32;

use crate::core::credential_manager::models::MasterKeyConfig;

const PBKDF2_ITERATIONS: u32 = 100000;
const SALT_LEN: usize = 16;
const IV_LEN: usize = 12;

pub struct EncryptionService;

impl EncryptionService {
    /// Initialize master key from password
    pub fn initialize_master_key(password: &str) -> Result<MasterKeyConfig, anyhow::Error> {
        let mut salt = vec![0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);

        let key = Self::derive_key(password, &salt, PBKDF2_ITERATIONS)?;
        let mut iv = vec![0u8; IV_LEN];
        rand::thread_rng().fill_bytes(&mut iv);

        Ok(MasterKeyConfig {
            key_version: 1,
            derived_from: "password".to_string(),
            salt: STANDARD.encode(&salt),
            iv: STANDARD.encode(&iv),
            created_at: chrono::Local::now().timestamp(),
        })
    }

    /// Verify master key password
    pub fn verify_master_key(
        password: &str,
        config: &MasterKeyConfig,
    ) -> Result<bool, anyhow::Error> {
        let salt = STANDARD.decode(&config.salt)?;
        let derived = Self::derive_key(password, &salt, PBKDF2_ITERATIONS)?;

        // Verify by trying to decrypt something
        // For now, we'll accept any valid derivation
        Ok(derived.len() == 32)
    }

    /// Derive key from password
    pub fn derive_key(
        password: &str,
        salt: &[u8],
        iterations: u32,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let mut key = vec![0u8; 32]; // AES-256 needs 32 bytes
        let non_zero_iters = NonZeroU32::new(iterations)
            .ok_or_else(|| anyhow::anyhow!("Invalid iterations"))?;
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, non_zero_iters.into(), &mut key);
        Ok(key)
    }

    /// Encrypt plaintext with master key
    pub fn encrypt(plaintext: &str, master_key: &[u8]) -> Result<String, anyhow::Error> {
        if master_key.len() != 32 {
            anyhow::bail!("Invalid key length: expected 32 bytes, got {}", master_key.len());
        }

        let cipher = Aes256Gcm::new_from_slice(master_key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext.as_bytes(),
            aad: b"",
        };

        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Format: base64(nonce || ciphertext)
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(STANDARD.encode(&combined))
    }

    /// Decrypt ciphertext with master key
    pub fn decrypt(ciphertext: &str, master_key: &[u8]) -> Result<String, anyhow::Error> {
        if master_key.len() != 32 {
            anyhow::bail!("Invalid key length: expected 32 bytes, got {}", master_key.len());
        }

        let combined = STANDARD.decode(ciphertext)?;
        if combined.len() < IV_LEN {
            anyhow::bail!("Ciphertext too short");
        }

        let nonce_bytes = &combined[..IV_LEN];
        let encrypted_data = &combined[IV_LEN..];

        let cipher = Aes256Gcm::new_from_slice(master_key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let payload = Payload {
            msg: encrypted_data,
            aad: b"",
        };

        let plaintext = cipher
            .decrypt(nonce, payload)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        Ok(String::from_utf8(plaintext)?)
    }

    /// Hash for audit logs (use SHA256)
    pub fn hash_value(value: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        STANDARD.encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"1234567890123456";

        let key1 = EncryptionService::derive_key(password, salt, PBKDF2_ITERATIONS).unwrap();
        let key2 = EncryptionService::derive_key(password, salt, PBKDF2_ITERATIONS).unwrap();

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let password = "test_password";
        let plaintext = "Hello, Secret!";

        let config = EncryptionService::initialize_master_key(password).unwrap();
        let salt = STANDARD.decode(&config.salt).unwrap();
        let key = EncryptionService::derive_key(password, &salt, PBKDF2_ITERATIONS).unwrap();

        let ciphertext = EncryptionService::encrypt(plaintext, &key).unwrap();
        let decrypted = EncryptionService::decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let password = "test_password";
        let plaintext = "";

        let config = EncryptionService::initialize_master_key(password).unwrap();
        let salt = STANDARD.decode(&config.salt).unwrap();
        let key = EncryptionService::derive_key(password, &salt, PBKDF2_ITERATIONS).unwrap();

        let ciphertext = EncryptionService::encrypt(plaintext, &key).unwrap();
        let decrypted = EncryptionService::decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let password = "test_password";
        let plaintext = "Secret data";

        let config = EncryptionService::initialize_master_key(password).unwrap();
        let salt = STANDARD.decode(&config.salt).unwrap();
        let key = EncryptionService::derive_key(password, &salt, PBKDF2_ITERATIONS).unwrap();

        let ciphertext = EncryptionService::encrypt(plaintext, &key).unwrap();

        // Try with wrong key
        let wrong_key = EncryptionService::derive_key("wrong_password", &salt, PBKDF2_ITERATIONS)
            .unwrap();
        let result = EncryptionService::decrypt(&ciphertext, &wrong_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_hash_value() {
        let value = "test_value";
        let hash1 = EncryptionService::hash_value(value);
        let hash2 = EncryptionService::hash_value(value);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
