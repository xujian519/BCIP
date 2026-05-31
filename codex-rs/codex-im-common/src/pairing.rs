use rand::Rng;

const POOL: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH: usize = 6;
const MAX_ATTEMPTS: u32 = 5;
const ATTEMPT_WINDOW_SECS: i64 = 300;

#[derive(Debug, Clone)]
pub struct PairingCode {
    pub code: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct PairingConfig {
    pub code_ttl_secs: i64,
    pub max_attempts: u32,
    pub attempt_window_secs: i64,
}

impl Default for PairingConfig {
    fn default() -> Self {
        Self {
            code_ttl_secs: 300,
            max_attempts: MAX_ATTEMPTS,
            attempt_window_secs: ATTEMPT_WINDOW_SECS,
        }
    }
}

#[derive(Debug, Default)]
struct AttemptTracker {
    attempts: Vec<chrono::DateTime<chrono::Utc>>,
}

impl AttemptTracker {
    fn record(&mut self, now: chrono::DateTime<chrono::Utc>, window: chrono::TimeDelta) {
        self.attempts.retain(|t| *t > now - window);
        self.attempts.push(now);
    }

    fn is_limited(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        window: chrono::TimeDelta,
        max: u32,
    ) -> bool {
        let active: Vec<_> = self
            .attempts
            .iter()
            .filter(|t| **t > now - window)
            .collect();
        active.len() >= max as usize
    }
}

#[derive(Debug)]
pub struct PairingManager {
    current_code: Option<PairingCode>,
    config: PairingConfig,
    attempts: std::collections::HashMap<String, AttemptTracker>,
}

impl PairingManager {
    pub fn new(config: PairingConfig) -> Self {
        Self {
            current_code: None,
            config,
            attempts: std::collections::HashMap::new(),
        }
    }

    pub fn generate_code(&mut self) -> String {
        let mut rng = rand::rng();
        let code: String = (0..CODE_LENGTH)
            .map(|_| POOL[rng.random_range(0..POOL.len())] as char)
            .collect();

        let now = chrono::Utc::now();
        self.current_code = Some(PairingCode {
            code: code.clone(),
            expires_at: now + chrono::TimeDelta::seconds(self.config.code_ttl_secs),
        });
        code
    }

    pub fn validate(&mut self, user_id: &str, submitted_code: &str) -> Result<bool, PairingError> {
        let now = chrono::Utc::now();
        let window = chrono::TimeDelta::seconds(self.config.attempt_window_secs);

        let tracker = self.attempts.entry(user_id.to_string()).or_default();
        if tracker.is_limited(now, window, self.config.max_attempts) {
            return Err(PairingError::TooManyAttempts);
        }

        tracker.record(now, window);

        let current = self
            .current_code
            .as_ref()
            .ok_or(PairingError::NoActiveCode)?;

        if now > current.expires_at {
            return Err(PairingError::CodeExpired);
        }

        if submitted_code.to_uppercase() == current.code {
            Ok(true)
        } else {
            Err(PairingError::InvalidCode)
        }
    }

    pub fn is_paired(&self, _user_id: &str) -> bool {
        false
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PairingError {
    #[error("配对码已过期，请生成新码")]
    CodeExpired,
    #[error("配对码无效")]
    InvalidCode,
    #[error("尝试次数过多，请等待 5 分钟后再试")]
    TooManyAttempts,
    #[error("没有活动的配对码")]
    NoActiveCode,
}
