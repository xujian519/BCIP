//! IM (Instant Messaging) service initialization.
//!
//! Reads `[im]` section from config.toml and starts the appropriate
//! IM adapters (Telegram, Feishu) connected via the IM Bridge.

use std::path::PathBuf;
use std::sync::Arc;

use codex_config::types::ImConfigToml;
use codex_im_bridge::BridgeConfig;
use codex_im_bridge::ImBridge;
use codex_im_bridge::session::SessionStore;
use tracing::error;
use tracing::info;

/// Handle to the running IM services. Dropping this will cancel the background tasks.
#[allow(dead_code)]
pub struct ImServiceHandle {
    /// The central bridge connecting all IM adapters to the core.
    bridge: Arc<ImBridge>,
    /// Background task handles for adapters.
    adapter_handles: Vec<tokio::task::JoinHandle<()>>,
}

/// Initialize IM services from config.
///
/// Returns `None` if IM is disabled or not configured.
/// Returns `Some(handle)` on success — keep the handle alive to keep the service running.
pub async fn init_im_services(im_config: &ImConfigToml) -> Option<ImServiceHandle> {
    // Check if IM is explicitly enabled
    if im_config.enabled.is_none() || !im_config.enabled.unwrap_or(false) {
        info!("IM integration is disabled in config");
        return None;
    }

    // Build bridge config from TOML config
    let bridge_config = build_bridge_config(im_config);

    // Initialize session store
    let session_store = match init_session_store(&bridge_config).await {
        Ok(store) => Arc::new(store),
        Err(e) => {
            error!(error = %e, "Failed to initialize IM session store");
            return None;
        }
    };

    // Create bridge
    let bridge = Arc::new(ImBridge::new(bridge_config, session_store));

    // Connect bridge to WebSocket server (non-blocking)
    let bridge_clone = Arc::clone(&bridge);
    tokio::spawn(async move {
        match bridge_clone.connect().await {
            Ok(()) => info!("IM Bridge connected successfully"),
            Err(e) => error!(error = %e, "IM Bridge connection failed"),
        }
    });

    let mut adapter_handles = Vec::new();

    // Start Telegram adapter if configured
    if let Some(ref tg_config) = im_config.telegram {
        if let Some(bot_token) = &tg_config.bot_token {
            let telegram_config = codex_im_telegram::TelegramConfig {
                bot_token: bot_token.clone(),
                allowed_users: tg_config.allowed_users.clone(),
            };
            let adapter =
                codex_im_telegram::TelegramAdapter::new(telegram_config, Arc::clone(&bridge));

            let handle = tokio::spawn(async move {
                info!("Telegram adapter starting");
                adapter.run().await;
            });
            adapter_handles.push(handle);
            info!("Telegram adapter initialized");
        } else {
            info!("Telegram config present but missing bot_token, skipping");
        }
    }

    // Start Feishu adapter if configured
    if let Some(ref fs_config) = im_config.feishu {
        if let (Some(app_id), Some(app_secret)) = (&fs_config.app_id, &fs_config.app_secret) {
            let feishu_config = codex_im_feishu::FeishuConfig {
                app_id: app_id.clone(),
                app_secret: app_secret.clone(),
                allowed_users: fs_config.allowed_users.clone(),
            };
            let adapter = codex_im_feishu::FeishuAdapter::new(feishu_config, Arc::clone(&bridge));

            let handle = tokio::spawn(async move {
                info!("Feishu adapter starting");
                adapter.run().await;
            });
            adapter_handles.push(handle);
            info!("Feishu adapter initialized");
        } else {
            info!("Feishu config present but missing app_id or app_secret, skipping");
        }
    }

    if adapter_handles.is_empty() {
        info!("No IM adapters configured — bridge running in standalone mode");
    }

    Some(ImServiceHandle {
        bridge,
        adapter_handles,
    })
}

/// Build `BridgeConfig` from `ImConfigToml`, applying defaults where needed.
fn build_bridge_config(im_config: &ImConfigToml) -> BridgeConfig {
    let mut config = BridgeConfig::default();

    if let Some(ref bridge) = im_config.bridge {
        if let Some(ref url) = bridge.server_url {
            config.server_url = url.clone();
        }
        if let Some(max) = bridge.max_reconnect {
            config.max_reconnect = max;
        }
        if let Some(secs) = bridge.heartbeat_interval_secs {
            config.heartbeat_interval_secs = secs;
        }
        if let Some(ref path) = bridge.session_db_path {
            config.session_db_path = path.clone();
        }
    }

    config
}

/// Initialize the session store for IM adapters.
async fn init_session_store(config: &BridgeConfig) -> Result<SessionStore, String> {
    let db_path = PathBuf::from(&config.session_db_path);

    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            let parent = parent.to_path_buf();
            tokio::task::spawn_blocking(move || {
                std::fs::create_dir_all(&parent)
                    .map_err(|e| format!("failed to create session db directory: {e}"))
            })
            .await
            .map_err(|e| format!("task error creating directory: {e}"))??;
        }
    }

    SessionStore::new(&db_path).map_err(|e| format!("failed to open session store: {e}"))
}
