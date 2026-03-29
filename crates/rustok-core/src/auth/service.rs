use chrono::Utc;
use uuid::Uuid;

use crate::id::generate_id;

use crate::auth::error::AuthError;
use crate::auth::jwt::{encode_token, JwtConfig};
use crate::auth::password::{hash_password, verify_password};
use crate::auth::repository::UserRepository;
use crate::auth::user::Model as User;
use crate::types::{UserRole, UserStatus};

#[derive(Debug)]
pub struct IdentityTokens {
    pub access_token: String,
}

#[derive(Debug)]
pub struct RegistrationInput {
    pub tenant_id: Uuid,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone)]
pub struct IdentityService {
    repo: UserRepository,
    jwt_config: JwtConfig,
}

impl IdentityService {
    pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
        Self { repo, jwt_config }
    }

    pub async fn register(&self, input: RegistrationInput) -> Result<User, AuthError> {
        if self
            .repo
            .find_by_email_and_tenant(&input.email, input.tenant_id)
            .await?
            .is_some()
        {
            return Err(AuthError::EmailAlreadyExists);
        }

        let password_hash = hash_password(&input.password)
            .map_err(|err| AuthError::PasswordHashing(err.to_string()))?;

        let now = chrono::DateTime::<chrono::FixedOffset>::from(Utc::now());
        let user = User {
            id: generate_id(),
            tenant_id: input.tenant_id,
            email: input.email,
            password_hash,
            first_name: input.first_name,
            last_name: input.last_name,
            role: UserRole::Customer,
            status: UserStatus::Active,
            email_verified_at: None,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        };

        let saved = self.repo.create(user.clone()).await?;

        Ok(saved)
    }

    pub async fn login(
        &self,
        tenant_id: Uuid,
        email: &str,
        password: &str,
    ) -> Result<IdentityTokens, AuthError> {
        let user = self
            .repo
            .find_by_email_and_tenant(email, tenant_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if user.status != UserStatus::Active {
            return Err(AuthError::UserInactive);
        }

        if !verify_password(password, &user.password_hash) {
            return Err(AuthError::InvalidCredentials);
        }

        self.repo.update_last_login(user.id).await?;

        let token = encode_token(
            &user.id,
            &tenant_id,
            &user.role.to_string(),
            &self.jwt_config,
        )
        .map_err(|err| AuthError::Token(err.to_string()))?;

        Ok(IdentityTokens {
            access_token: token,
        })
    }
}
