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
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EventSettings {
    #[serde(default)]
    pub transport: EventTransportKind,
    #[serde(default = "default_relay_interval_ms")]
    pub relay_interval_ms: u64,
    #[serde(default)]
    pub iggy: IggyConfig,
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

#[cfg(test)]
mod tests {
    use super::{EventTransportKind, RustokSettings};
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
}
