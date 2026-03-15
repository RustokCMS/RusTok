use serde::Deserialize;

const DEFAULT_ACCESS_EXPIRATION_SECS: u64 = 900; // 15 minutes
const DEFAULT_REFRESH_EXPIRATION_SECS: u64 = 60 * 60 * 24 * 30; // 30 days

/// Auth configuration — framework-agnostic.
///
/// The server is responsible for constructing this from whatever config source
/// it uses (Loco YAML, env vars, etc.). `rustok-auth` never reads config files.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub access_expiration: u64,
    pub refresh_expiration: u64,
    pub issuer: String,
    pub audience: String,
}

impl AuthConfig {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_expiration: DEFAULT_ACCESS_EXPIRATION_SECS,
            refresh_expiration: DEFAULT_REFRESH_EXPIRATION_SECS,
            issuer: "rustok".to_string(),
            audience: "rustok-admin".to_string(),
        }
    }

    pub fn with_expiration(mut self, access: u64, refresh: u64) -> Self {
        self.access_expiration = access;
        self.refresh_expiration = refresh;
        self
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = audience.into();
        self
    }
}

/// Helper for loading auth settings from nested YAML `settings.rustok.auth`.
#[derive(Debug, Deserialize, Default)]
pub struct AuthSettingsOverrides {
    pub refresh_expiration: Option<u64>,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

impl AuthSettingsOverrides {
    /// Apply overrides on top of a base `AuthConfig`.
    pub fn apply(self, config: &mut AuthConfig) {
        if let Some(v) = self.refresh_expiration {
            config.refresh_expiration = v;
        }
        if let Some(v) = self.issuer {
            config.issuer = v;
        }
        if let Some(v) = self.audience {
            config.audience = v;
        }
    }
}
