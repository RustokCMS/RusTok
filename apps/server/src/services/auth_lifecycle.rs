use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

use crate::auth::{
    decode_password_reset_token, encode_access_token, generate_refresh_token, hash_password,
    hash_refresh_token, verify_password, AuthConfig,
};
use crate::models::{sessions, users};

use super::auth::AuthService;

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug)]
pub enum AuthLifecycleError {
    EmailAlreadyExists,
    InvalidCredentials,
    UserInactive,
    InvalidRefreshToken,
    SessionExpired,
    UserNotFound,
    InvalidResetToken,
    Internal(Error),
}

impl From<Error> for AuthLifecycleError {
    fn from(value: Error) -> Self {
        Self::Internal(value)
    }
}

impl From<AuthLifecycleError> for Error {
    fn from(value: AuthLifecycleError) -> Self {
        match value {
            AuthLifecycleError::EmailAlreadyExists => {
                Error::BadRequest("Email already exists".into())
            }
            AuthLifecycleError::InvalidCredentials => {
                Error::Unauthorized("Invalid credentials".into())
            }
            AuthLifecycleError::UserInactive => Error::Unauthorized("User is inactive".into()),
            AuthLifecycleError::InvalidRefreshToken => {
                Error::Unauthorized("Invalid refresh token".into())
            }
            AuthLifecycleError::SessionExpired => Error::Unauthorized("Session expired".into()),
            AuthLifecycleError::UserNotFound => Error::Unauthorized("User not found".into()),
            AuthLifecycleError::InvalidResetToken => {
                Error::Unauthorized("Invalid reset token".into())
            }
            AuthLifecycleError::Internal(err) => err,
        }
    }
}

pub struct AuthLifecycleService;

impl AuthLifecycleService {
    pub async fn register(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        email: &str,
        password: &str,
        name: Option<String>,
    ) -> std::result::Result<(users::Model, AuthTokens), AuthLifecycleError> {
        let config = AuthConfig::from_ctx(ctx).map_err(AuthLifecycleError::from)?;

        if users::Entity::find_by_email(&ctx.db, tenant_id, email)
            .await
            .map_err(AuthLifecycleError::from)?
            .is_some()
        {
            return Err(AuthLifecycleError::EmailAlreadyExists);
        }

        let password_hash = hash_password(password).map_err(AuthLifecycleError::from)?;
        let mut user = users::ActiveModel::new(tenant_id, email, &password_hash);
        user.name = Set(name);
        let user = user
            .insert(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?;

        let user_role = user.role.clone();
        AuthService::assign_role_permissions(&ctx.db, &user.id, &tenant_id, user_role)
            .await
            .map_err(AuthLifecycleError::from)?;

        let tokens =
            Self::create_session_and_tokens(ctx, tenant_id, &user, None, None, &config).await?;

        Ok((user, tokens))
    }

    pub async fn login(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> std::result::Result<(users::Model, AuthTokens), AuthLifecycleError> {
        let config = AuthConfig::from_ctx(ctx).map_err(AuthLifecycleError::from)?;

        let user = users::Entity::find_by_email(&ctx.db, tenant_id, email)
            .await
            .map_err(AuthLifecycleError::from)?
            .ok_or(AuthLifecycleError::InvalidCredentials)?;

        if !verify_password(password, &user.password_hash).map_err(AuthLifecycleError::from)? {
            return Err(AuthLifecycleError::InvalidCredentials);
        }

        if !user.is_active() {
            return Err(AuthLifecycleError::UserInactive);
        }

        let now = Utc::now();
        let mut user_active: users::ActiveModel = user.clone().into();
        user_active.last_login_at = Set(Some(now.into()));
        let user = user_active
            .update(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?;

        let tokens =
            Self::create_session_and_tokens(ctx, tenant_id, &user, ip_address, user_agent, &config)
                .await?;

        Ok((user, tokens))
    }

    pub async fn refresh(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        refresh_token: &str,
    ) -> std::result::Result<(users::Model, AuthTokens), AuthLifecycleError> {
        let config = AuthConfig::from_ctx(ctx).map_err(AuthLifecycleError::from)?;
        let token_hash = hash_refresh_token(refresh_token);

        let session = sessions::Entity::find_by_token_hash(&ctx.db, tenant_id, &token_hash)
            .await
            .map_err(AuthLifecycleError::from)?
            .ok_or(AuthLifecycleError::InvalidRefreshToken)?;

        if !session.is_active() {
            return Err(AuthLifecycleError::SessionExpired);
        }

        let user = users::Entity::find_by_id(session.user_id)
            .one(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?
            .ok_or(AuthLifecycleError::UserNotFound)?;

        if !user.is_active() {
            return Err(AuthLifecycleError::UserInactive);
        }

        let now = Utc::now();
        let new_refresh_token = generate_refresh_token();
        let new_token_hash = hash_refresh_token(&new_refresh_token);
        let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

        let session_id = session.id;
        let mut session_model: sessions::ActiveModel = session.into();
        session_model.token_hash = Set(new_token_hash);
        session_model.expires_at = Set(expires_at.into());
        session_model.last_used_at = Set(Some(now.into()));
        session_model
            .update(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?;

        let access_token =
            encode_access_token(&config, user.id, tenant_id, user.role.clone(), session_id)
                .map_err(AuthLifecycleError::from)?;

        Ok((
            user,
            AuthTokens {
                access_token,
                refresh_token: new_refresh_token,
                expires_in: config.access_expiration,
            },
        ))
    }

    pub async fn confirm_password_reset(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        token: &str,
        password: &str,
    ) -> std::result::Result<(), AuthLifecycleError> {
        let config = AuthConfig::from_ctx(ctx).map_err(AuthLifecycleError::from)?;
        let claims = decode_password_reset_token(&config, token)
            .map_err(|_| AuthLifecycleError::InvalidResetToken)?;

        if claims.tenant_id != tenant_id {
            return Err(AuthLifecycleError::InvalidResetToken);
        }

        let user = users::Entity::find_by_email(&ctx.db, tenant_id, &claims.sub)
            .await
            .map_err(AuthLifecycleError::from)?
            .ok_or(AuthLifecycleError::InvalidResetToken)?;

        Self::reset_password_and_revoke_sessions(&ctx.db, tenant_id, user, password, None).await?;

        Ok(())
    }

    pub async fn change_password(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        user_id: uuid::Uuid,
        current_session_id: uuid::Uuid,
        current_password: &str,
        new_password: &str,
    ) -> std::result::Result<(), AuthLifecycleError> {
        let user = users::Entity::find_by_id(user_id)
            .filter(users::Column::TenantId.eq(tenant_id))
            .one(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?
            .ok_or(AuthLifecycleError::InvalidCredentials)?;

        if !verify_password(current_password, &user.password_hash)
            .map_err(AuthLifecycleError::from)?
        {
            return Err(AuthLifecycleError::InvalidCredentials);
        }

        let mut user_active: users::ActiveModel = user.into();
        user_active.password_hash =
            Set(hash_password(new_password).map_err(AuthLifecycleError::from)?);
        user_active
            .update(&ctx.db)
            .await
            .map_err(AuthLifecycleError::from)?;

        Self::revoke_user_sessions(ctx, tenant_id, user_id, Some(current_session_id)).await?;

        Ok(())
    }

    async fn reset_password_and_revoke_sessions(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
        user: users::Model,
        new_password: &str,
        except_session_id: Option<uuid::Uuid>,
    ) -> std::result::Result<(), AuthLifecycleError> {
        let user_id = user.id;
        let mut user_active: users::ActiveModel = user.into();
        user_active.password_hash =
            Set(hash_password(new_password).map_err(AuthLifecycleError::from)?);
        user_active
            .update(db)
            .await
            .map_err(AuthLifecycleError::from)?;

        Self::revoke_user_sessions_db(db, tenant_id, user_id, except_session_id).await?;

        Ok(())
    }

    async fn create_session_and_tokens(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        user: &users::Model,
        ip_address: Option<String>,
        user_agent: Option<String>,
        config: &AuthConfig,
    ) -> std::result::Result<AuthTokens, AuthLifecycleError> {
        let now = Utc::now();
        let refresh_token = generate_refresh_token();
        let token_hash = hash_refresh_token(&refresh_token);
        let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

        let session = sessions::ActiveModel::new(
            tenant_id, user.id, token_hash, expires_at, ip_address, user_agent,
        )
        .insert(&ctx.db)
        .await
        .map_err(AuthLifecycleError::from)?;

        let access_token =
            encode_access_token(config, user.id, tenant_id, user.role.clone(), session.id)
                .map_err(AuthLifecycleError::from)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            expires_in: config.access_expiration,
        })
    }

    async fn revoke_user_sessions(
        ctx: &AppContext,
        tenant_id: uuid::Uuid,
        user_id: uuid::Uuid,
        except_session_id: Option<uuid::Uuid>,
    ) -> std::result::Result<(), AuthLifecycleError> {
        Self::revoke_user_sessions_db(&ctx.db, tenant_id, user_id, except_session_id).await
    }

    async fn revoke_user_sessions_db(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
        user_id: uuid::Uuid,
        except_session_id: Option<uuid::Uuid>,
    ) -> std::result::Result<(), AuthLifecycleError> {
        let mut query = sessions::Entity::update_many()
            .col_expr(sessions::Column::RevokedAt, Expr::value(Utc::now()))
            .filter(sessions::Column::TenantId.eq(tenant_id))
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::RevokedAt.is_null());

        if let Some(session_id) = except_session_id {
            query = query.filter(sessions::Column::Id.ne(session_id));
        }

        query.exec(db).await.map_err(AuthLifecycleError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthLifecycleError, AuthLifecycleService, Error};
    use crate::models::{sessions, tenants, users};
    use chrono::{Duration, Utc};
    use migration::Migrator;
    use rustok_test_utils::db::setup_test_db_with_migrations;
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

    #[test]
    fn maps_email_exists_to_bad_request() {
        let err: Error = AuthLifecycleError::EmailAlreadyExists.into();
        match err {
            Error::BadRequest(msg) => assert_eq!(msg, "Email already exists"),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn maps_invalid_credentials_to_unauthorized() {
        let err: Error = AuthLifecycleError::InvalidCredentials.into();
        match err {
            Error::Unauthorized(msg) => assert_eq!(msg, "Invalid credentials"),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn keeps_internal_error_as_is() {
        let err: Error = AuthLifecycleError::Internal(Error::Unauthorized("inner".into())).into();
        match err {
            Error::Unauthorized(msg) => assert_eq!(msg, "inner"),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn reset_password_revoke_sessions_marks_all_sessions_revoked() {
        let db = setup_test_db_with_migrations::<Migrator>().await;
        let tenant = tenants::ActiveModel::new("Test tenant", "test-tenant")
            .insert(&db)
            .await
            .expect("failed to create tenant");

        let user = users::ActiveModel::new(tenant.id, "user@example.com", "old-hash")
            .insert(&db)
            .await
            .expect("failed to create user");

        let now = Utc::now();
        sessions::ActiveModel::new(
            tenant.id,
            user.id,
            "token-1".to_string(),
            now + Duration::hours(1),
            None,
            None,
        )
        .insert(&db)
        .await
        .expect("failed to create session 1");
        sessions::ActiveModel::new(
            tenant.id,
            user.id,
            "token-2".to_string(),
            now + Duration::hours(2),
            None,
            None,
        )
        .insert(&db)
        .await
        .expect("failed to create session 2");

        AuthLifecycleService::reset_password_and_revoke_sessions(
            &db,
            tenant.id,
            user,
            "new-password",
            None,
        )
        .await
        .expect("password reset should succeed");

        let active_sessions = sessions::Entity::find()
            .filter(sessions::Column::TenantId.eq(tenant.id))
            .filter(sessions::Column::UserId.eq(user.id))
            .filter(sessions::Column::RevokedAt.is_null())
            .all(&db)
            .await
            .expect("failed to query active sessions");

        assert!(active_sessions.is_empty());
    }

    #[tokio::test]
    async fn reset_password_revoke_sessions_can_keep_current_session() {
        let db = setup_test_db_with_migrations::<Migrator>().await;
        let tenant = tenants::ActiveModel::new("Test tenant", "test-tenant-2")
            .insert(&db)
            .await
            .expect("failed to create tenant");

        let user = users::ActiveModel::new(tenant.id, "user2@example.com", "old-hash")
            .insert(&db)
            .await
            .expect("failed to create user");

        let now = Utc::now();
        let current_session = sessions::ActiveModel::new(
            tenant.id,
            user.id,
            "current-token".to_string(),
            now + Duration::hours(1),
            None,
            None,
        )
        .insert(&db)
        .await
        .expect("failed to create current session");
        sessions::ActiveModel::new(
            tenant.id,
            user.id,
            "other-token".to_string(),
            now + Duration::hours(2),
            None,
            None,
        )
        .insert(&db)
        .await
        .expect("failed to create secondary session");

        AuthLifecycleService::reset_password_and_revoke_sessions(
            &db,
            tenant.id,
            user,
            "new-password",
            Some(current_session.id),
        )
        .await
        .expect("password reset should succeed");

        let still_active = sessions::Entity::find_by_id(current_session.id)
            .one(&db)
            .await
            .expect("failed to fetch current session")
            .expect("current session should exist");
        assert!(still_active.revoked_at.is_none());

        let revoked_count = sessions::Entity::find()
            .filter(sessions::Column::TenantId.eq(tenant.id))
            .filter(sessions::Column::UserId.eq(user.id))
            .filter(sessions::Column::RevokedAt.is_not_null())
            .count(&db)
            .await
            .expect("failed to query revoked sessions");

        assert_eq!(revoked_count, 1);
    }
}
