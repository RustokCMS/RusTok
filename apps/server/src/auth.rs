use argon2::{PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use loco_rs::{app::AppContext, Error, Result};
use password_hash::rand_core::{OsRng, RngCore};
use rustok_core::UserRole;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const DEFAULT_REFRESH_EXPIRATION_SECS: u64 = 60 * 60 * 24 * 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_id: Uuid,
    pub role: UserRole,
    pub session_id: Uuid,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,

    // OAuth2 fields (backward-compatible via Option/Default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<Uuid>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default = "default_grant_type")]
    pub grant_type: String,
}

fn default_grant_type() -> String {
    "direct".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteClaims {
    pub sub: String,
    pub tenant_id: Uuid,
    pub role: UserRole,
    pub purpose: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetClaims {
    pub sub: String,
    pub tenant_id: Uuid,
    pub purpose: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationClaims {
    pub sub: String,
    pub tenant_id: Uuid,
    pub purpose: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub access_expiration: u64,
    pub refresh_expiration: u64,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Deserialize)]
struct AppSettings {
    #[serde(default)]
    auth: Option<AuthSettings>,
}

#[derive(Debug, Deserialize)]
struct AuthSettings {
    refresh_expiration: Option<u64>,
    issuer: Option<String>,
    audience: Option<String>,
}

impl AuthConfig {
    pub fn from_ctx(ctx: &AppContext) -> Result<Self> {
        let auth = ctx
            .config
            .auth
            .as_ref()
            .and_then(|auth| auth.jwt.as_ref())
            .ok_or_else(|| Error::InternalServerError)?;

        let app_settings = ctx
            .config
            .settings
            .as_ref()
            .and_then(|value| serde_json::from_value::<AppSettings>(value.clone()).ok());

        let auth_settings = app_settings.and_then(|s| s.auth);

        let refresh_expiration = auth_settings
            .as_ref()
            .and_then(|a| a.refresh_expiration)
            .unwrap_or(DEFAULT_REFRESH_EXPIRATION_SECS);

        let issuer = auth_settings
            .as_ref()
            .and_then(|a| a.issuer.clone())
            .unwrap_or_else(|| "rustok".to_string());

        let audience = auth_settings
            .and_then(|a| a.audience)
            .unwrap_or_else(|| "rustok-admin".to_string());

        Ok(Self {
            secret: auth.secret.clone(),
            access_expiration: auth.expiration,
            refresh_expiration,
            issuer,
            audience,
        })
    }
}

pub fn encode_access_token(
    config: &AuthConfig,
    user_id: Uuid,
    tenant_id: Uuid,
    role: UserRole,
    session_id: Uuid,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(config.access_expiration as i64);

    let claims = Claims {
        sub: user_id,
        tenant_id,
        role,
        session_id,
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        client_id: None,
        scopes: Vec::new(),
        grant_type: "direct".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| Error::InternalServerError)
}

/// Encode an OAuth2 access token for an app (client_credentials flow).
/// `sub` is the app ID, `role` is Service, `session_id` is nil.
pub fn encode_oauth_access_token(
    config: &AuthConfig,
    app_id: Uuid,
    tenant_id: Uuid,
    client_id: Uuid,
    scopes: &[String],
    grant_type: &str,
    expires_in_secs: u64,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_secs as i64);

    let claims = Claims {
        sub: app_id,
        tenant_id,
        role: UserRole::Customer, // Service-level: minimal default role
        session_id: Uuid::nil(),  // No user session
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        client_id: Some(client_id),
        scopes: scopes.to_vec(),
        grant_type: grant_type.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| Error::InternalServerError)
}

pub fn decode_access_token(config: &AuthConfig, token: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[config.issuer.as_str()]);
    validation.set_audience(&[config.audience.as_str()]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| Error::Unauthorized("Invalid access token".to_string()))
}

pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = password_hash::SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| Error::InternalServerError)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed = password_hash::PasswordHash::new(password_hash)
        .map_err(|_| Error::Unauthorized("Invalid credentials".to_string()))?;
    Ok(argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn encode_password_reset_token(
    config: &AuthConfig,
    tenant_id: Uuid,
    email: &str,
    ttl_seconds: u64,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds as i64);

    let claims = PasswordResetClaims {
        sub: email.to_lowercase(),
        tenant_id,
        purpose: "password_reset".to_string(),
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| Error::InternalServerError)
}

pub fn decode_password_reset_token(
    config: &AuthConfig,
    token: &str,
) -> Result<PasswordResetClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[config.issuer.as_str()]);
    validation.set_audience(&[config.audience.as_str()]);

    let claims = decode::<PasswordResetClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| Error::Unauthorized("Invalid reset token".to_string()))?;

    if claims.purpose != "password_reset" {
        return Err(Error::Unauthorized("Invalid reset token".to_string()));
    }

    Ok(claims)
}

pub fn encode_email_verification_token(
    config: &AuthConfig,
    tenant_id: Uuid,
    email: &str,
    ttl_seconds: u64,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_seconds as i64);

    let claims = EmailVerificationClaims {
        sub: email.to_lowercase(),
        tenant_id,
        purpose: "email_verification".to_string(),
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|_| Error::InternalServerError)
}

pub fn decode_email_verification_token(
    config: &AuthConfig,
    token: &str,
) -> Result<EmailVerificationClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[config.issuer.as_str()]);
    validation.set_audience(&[config.audience.as_str()]);

    let claims = decode::<EmailVerificationClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| Error::Unauthorized("Invalid verification token".to_string()))?;

    if claims.purpose != "email_verification" {
        return Err(Error::Unauthorized(
            "Invalid verification token".to_string(),
        ));
    }

    Ok(claims)
}

pub fn decode_invite_token(config: &AuthConfig, token: &str) -> Result<InviteClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[config.issuer.as_str()]);
    validation.set_audience(&[config.audience.as_str()]);

    let claims = decode::<InviteClaims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| Error::Unauthorized("Invalid invite token".to_string()))?;

    if claims.purpose != "invite" {
        return Err(Error::Unauthorized("Invalid invite token".to_string()));
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AuthConfig {
        AuthConfig {
            secret: "test-secret-key-for-unit-tests-only-32bytes!".to_string(),
            access_expiration: 900,
            refresh_expiration: 2_592_000,
            issuer: "rustok".to_string(),
            audience: "rustok-admin".to_string(),
        }
    }

    // ===================================================================
    // RFC 7519 — JWT Claims compliance
    // ===================================================================

    #[test]
    fn rfc7519_jwt_required_claims_present() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let claims = decode_access_token(&config, &token).unwrap();
        assert_ne!(claims.sub, Uuid::nil());
        assert_eq!(claims.iss, "rustok");
        assert_eq!(claims.aud, "rustok-admin");
        assert!(claims.exp > claims.iat);
        assert!(claims.iat > 0);
    }

    #[test]
    fn rfc7519_jwt_expiration_enforced() {
        let config = AuthConfig {
            access_expiration: 0,
            ..test_config()
        };
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let result = decode_access_token(&config, &token);
        assert!(result.is_err(), "Expired JWT MUST be rejected");
    }

    #[test]
    fn rfc7519_jwt_issuer_validated() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let wrong_config = AuthConfig {
            issuer: "wrong-issuer".to_string(),
            ..test_config()
        };
        assert!(
            decode_access_token(&wrong_config, &token).is_err(),
            "Wrong issuer MUST be rejected"
        );
    }

    #[test]
    fn rfc7519_jwt_audience_validated() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let wrong_config = AuthConfig {
            audience: "wrong-audience".to_string(),
            ..test_config()
        };
        assert!(
            decode_access_token(&wrong_config, &token).is_err(),
            "Wrong audience MUST be rejected"
        );
    }

    #[test]
    fn rfc7519_jwt_signature_validated() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let wrong_config = AuthConfig {
            secret: "completely-different-secret-key-32bytes!!".to_string(),
            ..test_config()
        };
        assert!(
            decode_access_token(&wrong_config, &token).is_err(),
            "Wrong signature MUST be rejected"
        );
    }

    // ===================================================================
    // OAuth2 JWT extension claims
    // ===================================================================

    #[test]
    fn oauth2_direct_login_no_client_id() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Admin,
            Uuid::new_v4(),
        )
        .unwrap();

        let claims = decode_access_token(&config, &token).unwrap();
        assert!(claims.client_id.is_none());
        assert!(claims.scopes.is_empty());
        assert_eq!(claims.grant_type, "direct");
    }

    #[test]
    fn oauth2_client_credentials_token_claims() {
        let config = test_config();
        let client_id = Uuid::new_v4();
        let scopes = vec!["catalog:read".to_string(), "orders:write".to_string()];

        let token = encode_oauth_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            client_id,
            &scopes,
            "client_credentials",
            3600,
        )
        .unwrap();

        let claims = decode_access_token(&config, &token).unwrap();
        assert_eq!(claims.client_id, Some(client_id));
        assert_eq!(claims.scopes, scopes);
        assert_eq!(claims.grant_type, "client_credentials");
    }

    #[test]
    fn oauth2_token_ttl_matches_requested() {
        let config = test_config();
        let token = encode_oauth_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            &["catalog:read".to_string()],
            "client_credentials",
            3600,
        )
        .unwrap();

        let claims = decode_access_token(&config, &token).unwrap();
        assert_eq!(claims.exp - claims.iat, 3600);
    }

    #[test]
    fn oauth2_claims_backward_compatible() {
        let config = test_config();
        let token = encode_access_token(
            &config,
            Uuid::new_v4(),
            Uuid::new_v4(),
            UserRole::Customer,
            Uuid::new_v4(),
        )
        .unwrap();

        let claims = decode_access_token(&config, &token).unwrap();
        assert!(claims.client_id.is_none());
        assert!(claims.scopes.is_empty());
        assert_eq!(claims.grant_type, "direct");
    }

    // ===================================================================
    // Credential generation security
    // ===================================================================

    #[test]
    fn refresh_token_256_bit_entropy() {
        let token = generate_refresh_token();
        assert_eq!(
            token.len(),
            64,
            "Refresh token must be 64 hex chars (256 bits)"
        );
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn refresh_token_unique() {
        let t1 = generate_refresh_token();
        let t2 = generate_refresh_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn refresh_token_hash_sha256() {
        let token = generate_refresh_token();
        let hash = hash_refresh_token(&token);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn password_hash_argon2() {
        let hash = hash_password("test_password_123!").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn password_verify_roundtrip() {
        let password = "SecureP@ssw0rd!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn password_hash_unique_salt() {
        let h1 = hash_password("same_password").unwrap();
        let h2 = hash_password("same_password").unwrap();
        assert_ne!(
            h1, h2,
            "Same password must produce different hashes (unique salt)"
        );
    }
}
