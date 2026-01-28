use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};

/// Что лежит внутри нашего токена
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // User ID
    pub tenant: String, // Tenant ID (защита от подмены контекста)
    pub exp: usize,     // Expiration timestamp
    pub iat: usize,     // Issued at
    pub role: String,   // User Role (cached for quick checks)
}

pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64, // seconds
}

impl JwtConfig {
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }
}

/// Генерация токена
pub fn encode_token(
    user_id: &Uuid,
    tenant_id: &Uuid,
    role: &str,
    config: &JwtConfig,
) -> Result<String> {
    let now = Utc::now();
    let expire = now + Duration::seconds(config.expiration);

    let claims = Claims {
        sub: user_id.to_string(),
        tenant: tenant_id.to_string(),
        iat: now.timestamp() as usize,
        exp: expire.timestamp() as usize,
        role: role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| Error::Auth(e.to_string()))
}

/// Проверка токена
pub fn decode_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let validation = Validation::default();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| Error::Auth(e.to_string()))?;

    Ok(token_data.claims)
}
