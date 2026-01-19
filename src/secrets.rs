/// Secrets Management with Encryption
/// 
/// Provides secure storage and retrieval of sensitive data.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::{rand_core::RngCore, SaltString};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::fs;
use std::path::Path;
use crate::error::{BotError, BotResult};

const NONCE_SIZE: usize = 12;

/// Secrets manager for encrypted storage
pub struct SecretsManager {
    cipher: Aes256Gcm,
}

impl SecretsManager {
    /// Create a new secrets manager with a master key
    pub fn new(master_password: &str) -> BotResult<Self> {
        let key = Self::derive_key(master_password)?;
        let cipher = Aes256Gcm::new(&key);
        
        Ok(Self { cipher })
    }

    /// Derive encryption key from password using Argon2
    fn derive_key(password: &str) -> BotResult<Key<Aes256Gcm>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| BotError::ConfigError(format!("Key derivation failed: {}", e)))?;

        let hash_bytes = password_hash.hash.ok_or_else(|| {
            BotError::ConfigError("Hash generation failed".to_string())
        })?;

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        
        Ok(Key::<Aes256Gcm>::from(key_bytes))
    }

    /// Encrypt a secret value
    pub fn encrypt(&self, plaintext: &str) -> BotResult<String> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| BotError::ConfigError(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt a secret value
    pub fn decrypt(&self, encrypted: &str) -> BotResult<String> {
        let data = BASE64
            .decode(encrypted)
            .map_err(|e| BotError::ConfigError(format!("Base64 decode failed: {}", e)))?;

        if data.len() < NONCE_SIZE {
            return Err(BotError::ConfigError("Invalid encrypted data".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| BotError::ConfigError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| BotError::ConfigError(format!("UTF-8 decode failed: {}", e)))
    }

    /// Save encrypted secret to file
    pub fn save_secret(&self, key: &str, value: &str, path: &Path) -> BotResult<()> {
        let encrypted = self.encrypt(value)?;
        let content = format!("{}={}", key, encrypted);
        
        fs::write(path, content)
            .map_err(|e| BotError::ConfigError(format!("Failed to write secret: {}", e)))?;

        Ok(())
    }

    /// Load encrypted secret from file
    pub fn load_secret(&self, key: &str, path: &Path) -> BotResult<String> {
        let content = fs::read_to_string(path)
            .map_err(|e| BotError::ConfigError(format!("Failed to read secret: {}", e)))?;

        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key {
                    return self.decrypt(v.trim());
                }
            }
        }

        Err(BotError::ConfigError(format!("Secret key '{}' not found", key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt() {
        let manager = SecretsManager::new("test-password").unwrap();
        
        let plaintext = "my-secret-key";
        let encrypted = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_save_load_secret() {
        let manager = SecretsManager::new("test-password").unwrap();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        manager.save_secret("wallet_key", "my-private-key", temp_file.path()).unwrap();
        let loaded = manager.load_secret("wallet_key", temp_file.path()).unwrap();
        
        assert_eq!("my-private-key", loaded);
    }
}
