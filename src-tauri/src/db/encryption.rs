/// Database encryption and field-level security
/// Supports both database-level (WAL mode) and field-level encryption

use sha2::{Sha256, Digest};

/// Derive encryption key from password using SHA256
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    key
}

/// Encrypt sensitive device notes
pub fn encrypt_notes(notes: &str, password: &str) -> Result<String, String> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use rand::Rng;

    let salt = generate_salt();
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, notes.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Format: base64(salt || nonce || ciphertext)
    let mut encrypted = Vec::new();
    encrypted.extend_from_slice(&salt);
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);

    Ok(base64_encode(&encrypted))
}

/// Decrypt sensitive device notes
pub fn decrypt_notes(encrypted: &str, password: &str) -> Result<String, String> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};

    let encrypted_bytes = base64_decode(encrypted)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    if encrypted_bytes.len() < 32 + 12 {
        return Err("Invalid encrypted data".to_string());
    }

    // Parse: first 32 bytes = salt, next 12 = nonce, rest = ciphertext
    let salt = &encrypted_bytes[..32];
    let nonce_bytes = &encrypted_bytes[32..44];
    let ciphertext = &encrypted_bytes[44..];

    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Generate random salt (32 bytes)
fn generate_salt() -> [u8; 32] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt);
    salt
}

/// Generate random nonce (12 bytes for AES-GCM)
fn generate_nonce() -> [u8; 12] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce);
    nonce
}

/// Base64 encode
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

/// Base64 decode
fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Base64 decode failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let key1 = derive_key("password", b"salt");
        let key2 = derive_key("password", b"salt");
        assert_eq!(key1, key2);

        let key3 = derive_key("different", b"salt");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "This is sensitive data";
        let password = "mypassword";

        let encrypted = encrypt_notes(plaintext, password).unwrap();
        let decrypted = decrypt_notes(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let plaintext = "Secret";
        let encrypted = encrypt_notes(plaintext, "password1").unwrap();

        let result = decrypt_notes(&encrypted, "password2");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ciphertext() {
        let result = decrypt_notes("not-valid-base64!", "password");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_notes() {
        let plaintext = "";
        let encrypted = encrypt_notes(plaintext, "password").unwrap();
        let decrypted = decrypt_notes(&encrypted, "password").unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
