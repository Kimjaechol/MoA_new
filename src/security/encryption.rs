//! Enhanced encryption utilities for ZeroClaw.
//!
//! Provides AES-256-GCM encryption for data at rest and in transit,
//! complementing the existing ChaCha20-Poly1305 secret store.
//!
//! ## Usage
//! - Field-level encryption for sensitive config values
//! - Database column encryption for PII in memory entries
//! - File encryption for exported snapshots and backups

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use std::path::Path;

/// Nonce size for AES-256-GCM (12 bytes / 96 bits).
const AES_GCM_NONCE_SIZE: usize = 12;

/// Prefix for AES-256-GCM encrypted values (to distinguish from ChaCha20).
const AES_GCM_PREFIX: &str = "aes256:";

/// AES-256-GCM encryption engine.
pub struct AesEncryptor {
    key: [u8; 32],
}

impl AesEncryptor {
    /// Create a new encryptor with a 256-bit key.
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Create an encryptor by loading the key from a file.
    pub fn from_key_file(path: &Path) -> anyhow::Result<Self> {
        let key_bytes = std::fs::read(path)?;
        if key_bytes.len() != 32 {
            anyhow::bail!("AES key must be exactly 32 bytes, got {}", key_bytes.len());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(Self { key })
    }

    /// Generate a new random AES-256 key and save it to a file.
    pub fn generate_key_file(path: &Path) -> anyhow::Result<Self> {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        std::fs::write(path, &key)?;
        Ok(Self { key })
    }

    /// Encrypt plaintext, returning a prefixed base64 string.
    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("AES cipher init failed: {e}"))?;

        let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("AES encryption failed: {e}"))?;

        // Format: aes256:<base64(nonce + ciphertext)>
        let mut combined = Vec::with_capacity(AES_GCM_NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
        Ok(format!("{AES_GCM_PREFIX}{encoded}"))
    }

    /// Decrypt a prefixed base64 string back to plaintext.
    pub fn decrypt(&self, encrypted: &str) -> anyhow::Result<String> {
        let encoded = encrypted
            .strip_prefix(AES_GCM_PREFIX)
            .ok_or_else(|| anyhow::anyhow!("Missing AES-256-GCM prefix"))?;

        use base64::Engine;
        let combined = base64::engine::general_purpose::STANDARD.decode(encoded)?;

        if combined.len() < AES_GCM_NONCE_SIZE {
            anyhow::bail!("Ciphertext too short");
        }

        let (nonce_bytes, ciphertext) = combined.split_at(AES_GCM_NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("AES cipher init failed: {e}"))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES decryption failed: {e}"))?;

        String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8 in plaintext: {e}"))
    }

    /// Check if a string is AES-256-GCM encrypted (has the prefix).
    pub fn is_encrypted(value: &str) -> bool {
        value.starts_with(AES_GCM_PREFIX)
    }
}

/// Encrypt a file in-place using AES-256-GCM.
pub fn encrypt_file(key: &[u8; 32], file_path: &Path) -> anyhow::Result<()> {
    let plaintext = std::fs::read(file_path)?;
    let encryptor = AesEncryptor::new(*key);
    let plaintext_str = String::from_utf8(plaintext)
        .map_err(|e| anyhow::anyhow!("File is not valid UTF-8: {e}"))?;
    let encrypted = encryptor.encrypt(&plaintext_str)?;
    std::fs::write(file_path, encrypted.as_bytes())?;
    Ok(())
}

/// Decrypt a file in-place using AES-256-GCM.
pub fn decrypt_file(key: &[u8; 32], file_path: &Path) -> anyhow::Result<()> {
    let ciphertext = std::fs::read_to_string(file_path)?;
    let encryptor = AesEncryptor::new(*key);
    let plaintext = encryptor.decrypt(&ciphertext)?;
    std::fs::write(file_path, plaintext.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let encryptor = AesEncryptor::new(key);
        let plaintext = "Hello, ZeroClaw!";

        let encrypted = encryptor.encrypt(plaintext).unwrap();
        assert!(encrypted.starts_with(AES_GCM_PREFIX));
        assert_ne!(encrypted, plaintext);

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_empty_string() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let encryptor = AesEncryptor::new(key);
        let encrypted = encryptor.encrypt("").unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn encrypt_decrypt_unicode() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let encryptor = AesEncryptor::new(key);
        let plaintext = "안녕하세요 ZeroClaw 🦀";

        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let mut key1 = [0u8; 32];
        let mut key2 = [0u8; 32];
        OsRng.fill_bytes(&mut key1);
        OsRng.fill_bytes(&mut key2);

        let enc1 = AesEncryptor::new(key1);
        let enc2 = AesEncryptor::new(key2);

        let encrypted = enc1.encrypt("secret").unwrap();
        assert!(enc2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn is_encrypted_detects_prefix() {
        assert!(AesEncryptor::is_encrypted("aes256:AAAA"));
        assert!(!AesEncryptor::is_encrypted("enc2:AAAA"));
        assert!(!AesEncryptor::is_encrypted("plain text"));
    }

    #[test]
    fn key_file_generate_and_load() {
        let tmp = TempDir::new().unwrap();
        let key_path = tmp.path().join("test.key");

        let enc1 = AesEncryptor::generate_key_file(&key_path).unwrap();
        let enc2 = AesEncryptor::from_key_file(&key_path).unwrap();

        let encrypted = enc1.encrypt("test").unwrap();
        let decrypted = enc2.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "test");
    }

    #[test]
    fn file_encrypt_decrypt_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("data.txt");
        let original = "Sensitive data for ZeroClaw";
        std::fs::write(&file_path, original).unwrap();

        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        encrypt_file(&key, &file_path).unwrap();
        let encrypted_content = std::fs::read_to_string(&file_path).unwrap();
        assert_ne!(encrypted_content, original);

        decrypt_file(&key, &file_path).unwrap();
        let decrypted_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(decrypted_content, original);
    }

    #[test]
    fn decrypt_invalid_prefix_fails() {
        let key = [0u8; 32];
        let encryptor = AesEncryptor::new(key);
        assert!(encryptor.decrypt("invalid_prefix:data").is_err());
    }

    #[test]
    fn decrypt_truncated_ciphertext_fails() {
        let key = [0u8; 32];
        let encryptor = AesEncryptor::new(key);
        // Too short — less than nonce size
        assert!(encryptor.decrypt("aes256:AQID").is_err());
    }
}
