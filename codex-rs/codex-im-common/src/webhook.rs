//! Webhook support for IM adapters.
//!
//! Provides a common trait for handling inbound webhook requests from IM platforms,
//! plus HMAC-SHA256 signature verification.

use std::collections::HashMap;

use codex_im_protocol::ClientMessage;
use hmac::Hmac;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Decode a hex string to bytes.
fn decode_hex(hex_str: &str) -> Option<Vec<u8>> {
    if hex_str.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for chunk in hex_str.as_bytes().chunks(2) {
        let high = char::from(chunk[0]).to_digit(16)?;
        let low = char::from(chunk[1]).to_digit(16)?;
        bytes.push((high << 4 | low) as u8);
    }
    Some(bytes)
}

/// Encode bytes as hex string.
#[cfg(test)]
fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Result of processing a webhook request.
#[derive(Debug)]
pub enum WebhookResult {
    /// Successfully parsed — contains the client message to forward.
    Accepted(ClientMessage),
    /// Signature or format invalid — drop the request.
    Invalid,
    /// Platform URL verification challenge — respond with the given string.
    ChallengeResponded(String),
}

/// HMAC-SHA256 webhook signature verifier.
#[derive(Clone)]
pub struct WebhookVerifier {
    secret: String,
}

impl WebhookVerifier {
    /// Create a new verifier with the given secret key.
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Verify that the provided signature matches the HMAC-SHA256 of the payload.
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        use hmac::Mac;
        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(payload);
        let expected = mac.finalize().into_bytes();

        if let Some(expected_bytes) = decode_hex(signature) {
            return expected_bytes.len() == expected.len()
                && constant_time_eq(&expected_bytes, &expected);
        }

        false
    }
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

pub trait WebhookReceiver: Send + Sync {
    fn handle_request(&self, headers: &HashMap<String, String>, body: &[u8]) -> WebhookResult;

    fn challenge_response(&self, _body: &[u8]) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_valid_signature() {
        let verifier = WebhookVerifier::new("test-secret".to_string());
        use hmac::Mac;
        let mut mac = HmacSha256::new_from_slice(b"test-secret").unwrap();
        mac.update(b"hello world");
        let sig = encode_hex(&mac.finalize().into_bytes());

        assert!(verifier.verify_signature(b"hello world", &sig));
    }

    #[test]
    fn reject_invalid_signature() {
        let verifier = WebhookVerifier::new("test-secret".to_string());
        assert!(!verifier.verify_signature(b"hello world", "0000000000000000"));
    }

    #[test]
    fn reject_wrong_secret() {
        let verifier = WebhookVerifier::new("correct-secret".to_string());
        use hmac::Mac;
        let mut mac = HmacSha256::new_from_slice(b"wrong-secret").unwrap();
        mac.update(b"hello world");
        let sig = encode_hex(&mac.finalize().into_bytes());

        assert!(!verifier.verify_signature(b"hello world", &sig));
    }

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
