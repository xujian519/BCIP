//! Frame protocol for network transport — length-prefixed JSON encoding.
//!
//! Frame format: `[4 bytes BE length][1 byte version][N bytes JSON payload]`

use crate::agent_bus::AgentBusMessage;

/// Current frame protocol version.
pub const FRAME_VERSION: u8 = 1;

/// Maximum frame size (4 MB).
pub const MAX_FRAME_SIZE: usize = 4 * 1024 * 1024;

/// Frame header size: 4 bytes length + 1 byte version.
pub const FRAME_HEADER_SIZE: usize = 5;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame too large: {size} bytes (max {max})")]
    TooLarge { size: usize, max: usize },
    #[error("invalid frame version: {0}")]
    InvalidVersion(u8),
    #[error("frame too short: need {needed}, got {available}")]
    Truncated { needed: usize, available: usize },
    #[error("payload decode failed: {0}")]
    DecodeFailed(#[from] serde_json::Error),
}

pub struct FrameEncoder;

impl FrameEncoder {
    pub fn encode(msg: &AgentBusMessage) -> Result<Vec<u8>, FrameError> {
        let payload = serde_json::to_vec(msg)?;
        let total_len = FRAME_HEADER_SIZE + payload.len();
        if total_len > MAX_FRAME_SIZE {
            return Err(FrameError::TooLarge {
                size: total_len,
                max: MAX_FRAME_SIZE,
            });
        }

        let mut buf = Vec::with_capacity(total_len);
        // 4 bytes big-endian length of payload (excluding header)
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        // 1 byte version
        buf.push(FRAME_VERSION);
        // payload
        buf.extend_from_slice(&payload);
        Ok(buf)
    }
}

pub struct FrameDecoder;

impl FrameDecoder {
    /// Decode a single frame from a byte buffer.
    /// Returns the decoded message and the number of bytes consumed.
    pub fn decode(data: &[u8]) -> Result<(AgentBusMessage, usize), FrameError> {
        if data.len() < FRAME_HEADER_SIZE {
            return Err(FrameError::Truncated {
                needed: FRAME_HEADER_SIZE,
                available: data.len(),
            });
        }

        let payload_len = u32::from_be_bytes(
            data[..4]
                .try_into()
                .expect("checked len >= FRAME_HEADER_SIZE above"),
        ) as usize;
        let version = data[4];

        if version != FRAME_VERSION {
            return Err(FrameError::InvalidVersion(version));
        }

        let total_len = FRAME_HEADER_SIZE + payload_len;
        if total_len > MAX_FRAME_SIZE {
            return Err(FrameError::TooLarge {
                size: total_len,
                max: MAX_FRAME_SIZE,
            });
        }

        if data.len() < total_len {
            return Err(FrameError::Truncated {
                needed: total_len,
                available: data.len(),
            });
        }

        let msg: AgentBusMessage = serde_json::from_slice(&data[FRAME_HEADER_SIZE..total_len])?;
        Ok((msg, total_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentPath;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> AgentPath {
        AgentPath::try_from(format!("/root/{name}")).unwrap()
    }

    #[test]
    fn encode_decode_roundtrip() {
        let msg = AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!({"key": "value"}),
        );
        let encoded = FrameEncoder::encode(&msg).unwrap();
        let (decoded, consumed) = FrameDecoder::decode(&encoded).unwrap();

        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.id, msg.id);
        assert_eq!(decoded.from, msg.from);
        assert_eq!(decoded.payload["key"], "value");
    }

    #[test]
    fn frame_starts_with_header() {
        let msg =
            AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!("test"));
        let encoded = FrameEncoder::encode(&msg).unwrap();
        assert!(encoded.len() >= FRAME_HEADER_SIZE);
        assert_eq!(encoded[4], FRAME_VERSION);
    }

    #[test]
    fn truncated_frame_returns_error() {
        let result = FrameDecoder::decode(&[0, 0, 0, 5, 1, 1, 2]);
        assert!(matches!(result, Err(FrameError::Truncated { .. })));
    }

    #[test]
    fn invalid_version_returns_error() {
        let data = vec![0, 0, 0, 2, 99, b'{', b'}'];
        let result = FrameDecoder::decode(&data);
        assert!(matches!(result, Err(FrameError::InvalidVersion(99))));
    }

    #[test]
    fn decode_consumes_correct_bytes() {
        let msg = AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!("x"));
        let mut encoded = FrameEncoder::encode(&msg).unwrap();
        // Append garbage after the frame
        encoded.extend_from_slice(&[0xFF, 0xFF, 0xFF]);

        let (_, consumed) = FrameDecoder::decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len() - 3);
    }
}
