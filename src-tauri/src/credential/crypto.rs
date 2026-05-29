use ring::aead::{Aad, AES_256_GCM, Nonce, UnboundKey, LessSafeKey};
use ring::error::Unspecified;
use ring::hkdf::{HKDF_SHA256, Salt as HkdfSalt, KeyType};
use ring::pbkdf2::PBKDF2_HMAC_SHA256;

const DEK_LEN: usize = 32;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const HKDF_INFO: &[u8] = b"oasis-credential-key";

/// Key type for HKDF output length.
struct DekKeyType;

impl KeyType for DekKeyType {
    fn len(&self) -> usize {
        DEK_LEN
    }
}

/// Generate a random 32-byte salt.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::fill(&mut salt);
    salt
}

/// Generate a random 12-byte nonce.
pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    rand::fill(&mut nonce);
    nonce
}

/// Derive master key hash via PBKDF2-SHA256 with 600,000 rounds.
pub fn derive_master_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let rounds = std::num::NonZeroU32::new(600_000).expect("rounds must be non-zero");
    ring::pbkdf2::derive(PBKDF2_HMAC_SHA256, rounds, salt, password.as_bytes(), &mut hash);
    hash
}

/// Derive DEK from master password via PBKDF2 → HKDF.
pub fn derive_dek(password: &str, dek_salt: &[u8]) -> [u8; 32] {
    // First: PBKDF2 to get intermediate key material
    let mut intermediate = [0u8; 32];
    let rounds = std::num::NonZeroU32::new(600_000).expect("rounds must be non-zero");
    ring::pbkdf2::derive(PBKDF2_HMAC_SHA256, rounds, dek_salt, password.as_bytes(), &mut intermediate);

    // Second: HKDF to derive final DEK
    let salt = HkdfSalt::new(HKDF_SHA256, dek_salt);
    let prk = salt.extract(&intermediate);
    let okm = prk.expand(&[HKDF_INFO], DekKeyType).expect("HKDF expand failed");
    let mut dek = [0u8; DEK_LEN];
    okm.fill(&mut dek).expect("HKDF fill failed");
    dek
}

/// AES-256-GCM encrypt. Returns (ciphertext_with_tag, nonce).
pub fn encrypt(dek: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_LEN]), Unspecified> {
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let unbound_key = UnboundKey::new(&AES_256_GCM, dek)?;
    let key = LessSafeKey::new(unbound_key);

    let mut in_out = plaintext.to_vec();
    let tag = key.seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)?;

    // Append tag to ciphertext
    let mut result = in_out;
    result.extend_from_slice(tag.as_ref());
    Ok((result, nonce_bytes))
}

/// AES-256-GCM decrypt.
pub fn decrypt(dek: &[u8; 32], ciphertext: &[u8], nonce: &[u8; NONCE_LEN]) -> Result<Vec<u8>, Unspecified> {
    let nonce = Nonce::assume_unique_for_key(*nonce);

    let unbound_key = UnboundKey::new(&AES_256_GCM, dek)?;
    let key = LessSafeKey::new(unbound_key);

    let mut in_out = ciphertext.to_vec();
    let plaintext = key.open_in_place(nonce, Aad::empty(), &mut in_out)?;
    Ok(plaintext.to_vec())
}
