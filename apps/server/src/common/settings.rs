use rustok_iggy::IggyConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(1);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RustokSettings {
    #[serde(default)]
    pub tenant: TenantSettings,
    #[serde(default)]
    pub search: SearchSettings,
    #[serde(default)]
    pub features: FeatureSettings,
    #[serde(default)]
    pub rate_limit: RateLimitSettings,
    #[serde(default)]
    pub events: EventSettings,
    #[serde(default)]
    pub email: EmailSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EmailSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub smtp: SmtpSettings,
    #[serde(default = "default_email_from")]
    pub from: String,
    #[serde(default = "default_reset_base_url")]
    pub reset_base_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpSettings {
    #[serde(default = "default_smtp_host")]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl Default for SmtpSettings {
    fn default() -> Self {
        Self {
            host: default_smtp_host(),
            port: default_smtp_port(),
            username: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EventSettings {
    #[serde(default)]
    pub transport: EventTransportKind,
    #[serde(default)]
    pub relay_target: RelayTargetKind,
    #[serde(default = "default_relay_interval_ms")]
    pub relay_interval_ms: u64,
    #[serde(default)]
    pub relay_retry_policy: RelayRetryPolicy,
    #[serde(default)]
    pub dlq: DlqSettings,
    #[serde(default)]
    pub iggy: IggyConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RelayRetryPolicy {
    #[serde(default = "default_relay_max_attempts")]
    pub max_attempts: i32,
    #[serde(default = "default_relay_backoff_base_ms")]
    pub base_backoff_ms: u64,
    #[serde(default = "default_relay_backoff_max_ms")]
    pub max_backoff_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DlqSettings {
    #[serde(default = "default_dlq_enabled")]
    pub enabled: bool,
    #[serde(default = "default_dlq_max_attempts")]
    pub max_attempts: i32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RelayTargetKind {
    #[default]
    Memory,
    Iggy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventTransportKind {
    #[default]
    Memory,
    Outbox,
    Iggy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TenantSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_header_name")]
    pub header_name: String,
    #[serde(default = "default_tenant_id")]
    pub default_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureSettings {
    #[serde(default = "default_true")]
    pub registration_enabled: bool,
    #[serde(default)]
    pub email_verification: bool,
    #[serde(default = "default_true")]
    pub multi_tenant: bool,
    #[serde(default = "default_true")]
    pub search_indexing: bool,
    #[serde(default)]
    pub oauth_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_search_driver")]
    pub driver: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_index_prefix")]
    pub index_prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    #[serde(default = "default_burst")]
    pub burst: u32,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            resolution: default_resolution(),
            header_name: default_header_name(),
            default_id: default_tenant_id(),
        }
    }
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self {
            registration_enabled: true,
            email_verification: false,
            multi_tenant: true,
            search_indexing: true,
            oauth_enabled: false,
        }
    }
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            driver: default_search_driver(),
            url: String::new(),
            api_key: None,
            index_prefix: default_index_prefix(),
        }
    }
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: default_requests_per_minute(),
            burst: default_burst(),
        }
    }
}

impl RustokSettings {
    pub fn from_settings(settings: &Option<serde_json::Value>) -> Result<Self, serde_json::Error> {
        let root = settings.clone().unwrap_or_else(|| serde_json::json!({}));
        let rustok = root
            .get("rustok")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let mut parsed: Self = serde_json::from_value(rustok)?;

        if let Ok(raw_transport) = std::env::var("RUSTOK_EVENT_TRANSPORT") {
            parsed.events.transport = parse_event_transport(&raw_transport)?;
        }

        if parsed.events.relay_retry_policy.max_attempts <= 0 {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "rustok.events.relay_retry_policy.max_attempts must be > 0",
            )));
        }

        if parsed.events.dlq.max_attempts <= 0 {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "rustok.events.dlq.max_attempts must be > 0",
            )));
        }

        Ok(parsed)
    }
}

fn parse_event_transport(value: &str) -> Result<EventTransportKind, serde_json::Error> {
    match value.trim().to_ascii_lowercase().as_str() {
        "memory" => Ok(EventTransportKind::Memory),
        "outbox" => Ok(EventTransportKind::Outbox),
        "iggy" => Ok(EventTransportKind::Iggy),
        _ => Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Invalid RUSTOK_EVENT_TRANSPORT='{value}'. Expected one of: memory, outbox, iggy"
            ),
        ))),
    }
}

fn default_tenant_id() -> Uuid {
    DEFAULT_TENANT_ID
}

fn default_resolution() -> String {
    "header".to_string()
}

fn default_header_name() -> String {
    "X-Tenant-ID".to_string()
}

fn default_true() -> bool {
    true
}

fn default_search_driver() -> String {
    "meilisearch".to_string()
}

fn default_index_prefix() -> String {
    "rustok_".to_string()
}

fn default_requests_per_minute() -> u32 {
    60
}

fn default_burst() -> u32 {
    10
}

fn default_relay_interval_ms() -> u64 {
    1_000
}

fn default_relay_max_attempts() -> i32 {
    5
}

fn default_relay_backoff_base_ms() -> u64 {
    1_000
}

fn default_relay_backoff_max_ms() -> u64 {
    60_000
}

fn default_dlq_enabled() -> bool {
    true
}

fn default_dlq_max_attempts() -> i32 {
    10
}

fn default_email_from() -> String {
    "no-reply@rustok.local".to_string()
}

fn default_reset_base_url() -> String {
    "http://localhost:3000/reset-password".to_string()
}

fn default_smtp_host() -> String {
    "localhost".to_string()
}

fn default_smtp_port() -> u16 {
    1025
}

#[cfg(test)]
mod tests {
    use super::{EventTransportKind, RelayTargetKind, RustokSettings};
    use std::sync::{Mutex, OnceLock};

    const EVENT_TRANSPORT_ENV: &str = "RUSTOK_EVENT_TRANSPORT";

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn clear(key: &'static str) -> Self {
            let original = std::env::var(key).ok();
            unsafe {
                std::env::remove_var(key);
            }
            Self { key, original }
        }

        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.original {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    #[test]
    fn reads_transport_from_config() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let _env_guard = EnvVarGuard::clear(EVENT_TRANSPORT_ENV);

        let raw = serde_json::json!({
            "rustok": {
                "events": {
                    "transport": "outbox"
                }
            }
        });

        let settings = RustokSettings::from_settings(&Some(raw)).expect("settings parsed");
        assert_eq!(settings.events.transport, EventTransportKind::Outbox);
    }

    #[test]
    fn rejects_invalid_env_transport() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let _env_guard = EnvVarGuard::set(EVENT_TRANSPORT_ENV, "broken");

        let err = RustokSettings::from_settings(&Some(serde_json::json!({ "rustok": {} })))
            .expect_err("transport should fail");
        assert!(err
            .to_string()
            .contains("Invalid RUSTOK_EVENT_TRANSPORT='broken'"));
    }

    #[test]
    fn reads_relay_defaults_from_config() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let _env_guard = EnvVarGuard::clear(EVENT_TRANSPORT_ENV);

        let raw = serde_json::json!({
            "rustok": {
                "events": {
                    "transport": "outbox",
                    "relay_target": "iggy"
                }
            }
        });

        let settings = RustokSettings::from_settings(&Some(raw)).expect("settings parsed");
        assert_eq!(settings.events.transport, EventTransportKind::Outbox);
        assert_eq!(settings.events.relay_target, RelayTargetKind::Iggy);
        assert_eq!(settings.events.relay_retry_policy.max_attempts, 5);
        assert_eq!(settings.events.relay_retry_policy.base_backoff_ms, 1_000);
        assert_eq!(settings.events.relay_retry_policy.max_backoff_ms, 60_000);
        assert!(settings.events.dlq.enabled);
        assert_eq!(settings.events.dlq.max_attempts, 10);
    }

    #[test]
    fn rejects_non_positive_retry_and_dlq_attempts() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let _env_guard = EnvVarGuard::clear(EVENT_TRANSPORT_ENV);

        let bad_retry = serde_json::json!({
            "rustok": {
                "events": {
                    "relay_retry_policy": { "max_attempts": 0 }
                }
            }
        });

        let err =
            RustokSettings::from_settings(&Some(bad_retry)).expect_err("retry validation expected");
        assert!(err
            .to_string()
            .contains("relay_retry_policy.max_attempts must be > 0"));

        let bad_dlq = serde_json::json!({
            "rustok": {
                "events": {
                    "dlq": { "max_attempts": 0 }
                }
            }
        });

        let err =
            RustokSettings::from_settings(&Some(bad_dlq)).expect_err("dlq validation expected");
        assert!(err.to_string().contains("dlq.max_attempts must be > 0"));
    }
}
