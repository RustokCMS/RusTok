use async_graphql::{Context, FieldError, Object, Result};
use chrono::{Duration, Utc};
use loco_rs::prelude::AppContext;
use sea_orm::{ActiveModelTrait, Set};

use crate::auth::{
    encode_access_token, generate_refresh_token, hash_password, hash_refresh_token,
    verify_password, AuthConfig,
};
use crate::context::TenantContext;
use crate::graphql::errors::GraphQLError;
use crate::models::{sessions, users};
use crate::services::auth::AuthService;

use super::types::*;

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Sign in with email and password
    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<AuthPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let config = AuthConfig::from_ctx(app_ctx)
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let user = users::Entity::find_by_email(&app_ctx.db, tenant.id, &input.email)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?
            .ok_or_else(|| FieldError::new("Invalid credentials"))?;

        if !verify_password(&input.password, &user.password_hash)
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?
        {
            return Err(FieldError::new("Invalid credentials"));
        }

        let now = Utc::now();
        let refresh_token = generate_refresh_token();
        let token_hash = hash_refresh_token(&refresh_token);
        let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

        let session = sessions::ActiveModel::new(
            tenant.id, user.id, token_hash, expires_at,
            None, // IP address (not available in GraphQL context)
            None, // User agent (not available in GraphQL context)
        )
        .insert(&app_ctx.db)
        .await
        .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        // Update last login
        let mut user_active: users::ActiveModel = user.clone().into();
        user_active.last_login_at = Set(Some(now.into()));
        let user = user_active
            .update(&app_ctx.db)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let user_role = user.role.clone();
        let access_token =
            encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)
                .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        Ok(AuthPayload {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_expiration as i32,
            user: AuthUser {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user_role.to_string(),
                status: user.status.to_string(),
            },
        })
    }

    /// Sign up with email, password, and optional name
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<AuthPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let config = AuthConfig::from_ctx(app_ctx)
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        // Check if user already exists
        if users::Entity::find_by_email(&app_ctx.db, tenant.id, &input.email)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?
            .is_some()
        {
            return Err(FieldError::new("Email already exists"));
        }

        let password_hash = hash_password(&input.password)
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let mut user_model = users::ActiveModel::new(tenant.id, &input.email, &password_hash);
        user_model.name = Set(input.name);

        let user = user_model
            .insert(&app_ctx.db)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let user_role = user.role.clone();

        // Assign role permissions
        AuthService::assign_role_permissions(&app_ctx.db, &user.id, &tenant.id, user_role.clone())
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let now = Utc::now();
        let refresh_token = generate_refresh_token();
        let token_hash = hash_refresh_token(&refresh_token);
        let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

        let session =
            sessions::ActiveModel::new(tenant.id, user.id, token_hash, expires_at, None, None)
                .insert(&app_ctx.db)
                .await
                .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let access_token =
            encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)
                .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        Ok(AuthPayload {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_expiration as i32,
            user: AuthUser {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user_role.to_string(),
                status: user.status.to_string(),
            },
        })
    }

    /// Sign out (invalidate current session)
    async fn sign_out(&self, ctx: &Context<'_>) -> Result<SignOutPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let auth = ctx.data_opt::<crate::context::AuthContext>();

        if let Some(auth) = auth {
            // Delete session from database
            sessions::Entity::delete_by_id(auth.session_id)
                .exec(&app_ctx.db)
                .await
                .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;
        }

        Ok(SignOutPayload { success: true })
    }

    /// Refresh access token using refresh token
    async fn refresh_token(
        &self,
        ctx: &Context<'_>,
        input: RefreshTokenInput,
    ) -> Result<AuthPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let config = AuthConfig::from_ctx(app_ctx)
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        let token_hash = hash_refresh_token(&input.refresh_token);

        let session = sessions::Entity::find_by_token_hash(&app_ctx.db, tenant.id, &token_hash)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?
            .ok_or_else(|| FieldError::new("Invalid refresh token"))?;

        if !session.is_active() {
            return Err(FieldError::new("Session expired"));
        }

        let user = users::Entity::find_by_id(session.user_id)
            .one(&app_ctx.db)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?
            .ok_or_else(|| FieldError::new("User not found"))?;

        let user_role = user.role.clone();
        let access_token =
            encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)
                .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        // Optionally generate new refresh token
        let new_refresh_token = generate_refresh_token();
        let new_token_hash = hash_refresh_token(&new_refresh_token);
        let now = Utc::now();
        let new_expires_at = now + Duration::seconds(config.refresh_expiration as i64);

        let mut session_active: sessions::ActiveModel = session.into();
        session_active.token_hash = Set(new_token_hash);
        session_active.expires_at = Set(new_expires_at);
        session_active.last_used_at = Set(Some(now));

        session_active
            .update(&app_ctx.db)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        Ok(AuthPayload {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_expiration as i32,
            user: AuthUser {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user_role.to_string(),
                status: user.status.to_string(),
            },
        })
    }

    /// Request password reset email
    async fn forgot_password(
        &self,
        ctx: &Context<'_>,
        input: ForgotPasswordInput,
    ) -> Result<ForgotPasswordPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;

        // Check if user exists
        let user = users::Entity::find_by_email(&app_ctx.db, tenant.id, &input.email)
            .await
            .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        if user.is_none() {
            // Don't reveal if user exists
            return Ok(ForgotPasswordPayload {
                success: true,
                message: "If the email exists, a password reset link has been sent".to_string(),
            });
        }

        // TODO: Generate reset token and send email
        // For now, just return success

        Ok(ForgotPasswordPayload {
            success: true,
            message: "If the email exists, a password reset link has been sent".to_string(),
        })
    }

    /// Reset password using reset token
    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        input: ResetPasswordInput,
    ) -> Result<ResetPasswordPayload> {
        let app_ctx = ctx.data::<AppContext>()?;

        // TODO: Verify reset token and update password
        // For now, just return error

        Err(FieldError::new("Password reset not yet implemented"))
    }
}
