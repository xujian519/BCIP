use aes_gcm::aead::Aead;
use aes_gcm::Aes256Gcm;
use aes_gcm::KeyInit;
use aes_gcm::Nonce;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

#[derive(Clone)]
pub struct EncryptionKey(pub [u8; 32]);

impl EncryptionKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(*bytes)
    }

    pub fn from_base64(b64: &str) -> Result<Self, CryptoError> {
        let bytes = BASE64
            .decode(b64)
            .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKey(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self(key))
    }

    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        use rand::RngCore;
        rand::rng().fill_bytes(&mut key);
        Self(key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("加密失败: {0}")]
    Encryption(String),
    #[error("解密失败: {0}")]
    Decryption(String),
    #[error("无效的密钥: {0}")]
    InvalidKey(String),
}

pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(&key.0)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    use rand::RngCore;
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(key: &EncryptionKey, ciphertext_b64: &str) -> Result<Vec<u8>, CryptoError> {
    let combined = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;

    if combined.len() < 12 {
        return Err(CryptoError::Decryption("ciphertext too short".to_string()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key.0)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::Decryption(e.to_string()))
}

pub fn encrypt_json(key: &EncryptionKey, value: &serde_json::Value) -> Result<String, CryptoError> {
    let json_bytes = serde_json::to_vec(value)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    encrypt(key, &json_bytes)
}

pub fn decrypt_json(key: &EncryptionKey, ciphertext_b64: &str) -> Result<serde_json::Value, CryptoError> {
    let json_bytes = decrypt(key, ciphertext_b64)?;
    serde_json::from_slice(&json_bytes).map_err(|e| CryptoError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = EncryptionKey::generate();
        let plaintext = b"hello world, this is a secret message!";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_json_roundtrip() {
        let key = EncryptionKey::generate();
        let value = serde_json::json!({"key": "value", "nested": {"a": 1}});
        let encrypted = encrypt_json(&key, &value).unwrap();
        let decrypted = decrypt_json(&key, &encrypted).unwrap();
        assert_eq!(value, decrypted);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        let encrypted = encrypt(&key1, b"secret data").unwrap();
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn invalid_base64_fails() {
        let key = EncryptionKey::generate();
        assert!(decrypt(&key, "not-valid-base64!!!").is_err());
    }

    #[test]
    fn from_base64_roundtrip() {
        let key = EncryptionKey::generate();
        let b64 = BASE64.encode(key.0);
        let restored = EncryptionKey::from_base64(&b64).unwrap();
        assert_eq!(key.0, restored.0);
    }
}
