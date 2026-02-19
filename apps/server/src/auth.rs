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
