use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use alloy_scripting::{integration::ScriptableEntity, model::EventType, runner::HookOutcome};

use crate::auth::error::AuthError;
use crate::auth::jwt::{encode_token, JwtConfig};
use crate::auth::password::{hash_password, verify_password};
use crate::auth::repository::UserRepository;
use crate::auth::user::Model as User;
use crate::scripting::ScriptingContext;
use crate::types::{UserRole, UserStatus};

#[derive(Debug)]
pub struct AuthTokens {
    pub access_token: String,
}

#[derive(Debug)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone)]
pub struct AuthService {
    repo: UserRepository,
    jwt_config: JwtConfig,
    scripting: Option<Arc<ScriptingContext>>,
}

impl AuthService {
    pub fn new(repo: UserRepository, jwt_config: JwtConfig) -> Self {
        Self {
            repo,
            jwt_config,
            scripting: None,
        }
    }

    pub fn with_scripting(mut self, scripting: Arc<ScriptingContext>) -> Self {
        self.scripting = Some(scripting);
        self
    }

    pub async fn register(&self, input: RegisterInput) -> Result<User, AuthError> {
        if self.repo.find_by_email(&input.email).await?.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let password_hash = hash_password(&input.password)
            .map_err(|err| AuthError::PasswordHashing(err.to_string()))?;

        let now = chrono::DateTime::<chrono::FixedOffset>::from(Utc::now());
        let mut user = User {
            id: Uuid::new_v4(),
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

        if let Some(scripting) = &self.scripting {
            let proxy = user.to_entity_proxy();
            match scripting
                .orchestrator
                .run_before(user.entity_type(), EventType::BeforeCreate, proxy, None)
                .await
            {
                HookOutcome::Continue { changes } => {
                    if !changes.is_empty() {
                        user.apply_changes(changes);
                    }
                }
                HookOutcome::Rejected { reason } => {
                    return Err(AuthError::ValidationFailed(reason));
                }
                HookOutcome::Error { error } => {
                    return Err(AuthError::ScriptError(error.to_string()));
                }
            }
        }

        let saved = self.repo.create(user.clone()).await?;

        if let Some(scripting) = &self.scripting {
            let scripting = scripting.clone();
            let after_user = saved.clone();
            tokio::spawn(async move {
                let proxy = after_user.to_entity_proxy();
                let _ = scripting
                    .orchestrator
                    .run_after(
                        after_user.entity_type(),
                        EventType::AfterCreate,
                        proxy,
                        None,
                        None,
                    )
                    .await;

                let commit_proxy = after_user.to_entity_proxy();
                let _ = scripting
                    .orchestrator
                    .run_on_commit(after_user.entity_type(), commit_proxy, None)
                    .await;
            });
        }

        Ok(saved)
    }

    pub async fn login(
        &self,
        tenant_id: Uuid,
        email: &str,
        password: &str,
    ) -> Result<AuthTokens, AuthError> {
        let user = self
            .repo
            .find_by_email(email)
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

        Ok(AuthTokens {
            access_token: token,
        })
    }
}
