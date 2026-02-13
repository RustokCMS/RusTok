// GraphQL API для аутентификации (leptos-auth)
// Использует leptos-graphql как transport layer

use leptos_graphql::{execute, GraphqlRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AuthError, AuthSession, AuthUser};

// ============================================================================
// GraphQL Mutations
// ============================================================================

const SIGN_IN_MUTATION: &str = r#"
mutation SignIn($input: SignInInput!) {
    signIn(input: $input) {
        accessToken
        refreshToken
        tokenType
        expiresIn
        user {
            id
            email
            name
            role
            status
        }
    }
}
"#;

const SIGN_UP_MUTATION: &str = r#"
mutation SignUp($input: SignUpInput!) {
    signUp(input: $input) {
        accessToken
        refreshToken
        tokenType
        expiresIn
        user {
            id
            email
            name
            role
            status
        }
    }
}
"#;

const SIGN_OUT_MUTATION: &str = r#"
mutation SignOut {
    signOut {
        success
    }
}
"#;

const REFRESH_TOKEN_MUTATION: &str = r#"
mutation RefreshToken($input: RefreshTokenInput!) {
    refreshToken(input: $input) {
        accessToken
        refreshToken
        tokenType
        expiresIn
        user {
            id
            email
            name
            role
            status
        }
    }
}
"#;

const FORGOT_PASSWORD_MUTATION: &str = r#"
mutation ForgotPassword($input: ForgotPasswordInput!) {
    forgotPassword(input: $input) {
        success
        message
    }
}
"#;

const CURRENT_USER_QUERY: &str = r#"
query CurrentUser {
    me {
        id
        email
        name
        role
        status
    }
}
"#;

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Deserialize)]
struct SignInResponse {
    #[serde(rename = "signIn")]
    sign_in: AuthPayload,
}

#[derive(Debug, Deserialize)]
struct SignUpResponse {
    #[serde(rename = "signUp")]
    sign_up: AuthPayload,
}

#[derive(Debug, Deserialize)]
struct SignOutResponse {
    #[serde(rename = "signOut")]
    sign_out: SignOutPayload,
}

#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    #[serde(rename = "refreshToken")]
    refresh_token: AuthPayload,
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordResponse {
    #[serde(rename = "forgotPassword")]
    forgot_password: ForgotPasswordPayload,
}

#[derive(Debug, Deserialize)]
struct CurrentUserResponse {
    me: Option<AuthUserGraphQL>,
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "tokenType")]
    token_type: String,
    #[serde(rename = "expiresIn")]
    expires_in: i32,
    user: AuthUserGraphQL,
}

#[derive(Debug, Deserialize)]
struct AuthUserGraphQL {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct SignOutPayload {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct ForgotPasswordPayload {
    success: bool,
    message: String,
}

// ============================================================================
// Helper functions
// ============================================================================

fn get_api_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("RUSTOK_API_URL")
            .unwrap_or_else(|_| "http://localhost:5150".to_string())
    }
}

fn get_graphql_url() -> String {
    format!("{}/api/graphql", get_api_url())
}

// ============================================================================
// Public API
// ============================================================================

/// Sign in with email and password
pub async fn sign_in(
    email: String,
    password: String,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let url = get_graphql_url();
    
    let variables = json!({
        "input": {
            "email": email,
            "password": password,
        }
    });

    let request = GraphqlRequest {
        query: SIGN_IN_MUTATION.to_string(),
        variables,
    };

    let response: SignInResponse = execute(&url, request, None, Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    let payload = response.sign_in;
    
    let user = AuthUser {
        id: payload.user.id,
        email: payload.user.email,
        name: payload.user.name,
    };

    let session = AuthSession {
        token: payload.access_token,
        tenant,
    };

    Ok((user, session))
}

/// Sign up with email, password, and optional name
pub async fn sign_up(
    email: String,
    password: String,
    name: Option<String>,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let url = get_graphql_url();
    
    let variables = json!({
        "input": {
            "email": email,
            "password": password,
            "name": name,
        }
    });

    let request = GraphqlRequest {
        query: SIGN_UP_MUTATION.to_string(),
        variables,
    };

    let response: SignUpResponse = execute(&url, request, None, Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    let payload = response.sign_up;
    
    let user = AuthUser {
        id: payload.user.id,
        email: payload.user.email,
        name: payload.user.name,
    };

    let session = AuthSession {
        token: payload.access_token,
        tenant,
    };

    Ok((user, session))
}

/// Sign out (invalidate current session)
pub async fn sign_out(token: String, tenant: String) -> Result<(), AuthError> {
    let url = get_graphql_url();

    let request = GraphqlRequest {
        query: SIGN_OUT_MUTATION.to_string(),
        variables: serde_json::Value::Null,
    };

    let _response: SignOutResponse = execute(&url, request, Some(&token), Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    Ok(())
}

/// Refresh access token using refresh token
pub async fn refresh_token(refresh_token: String, tenant: String) -> Result<AuthSession, AuthError> {
    let url = get_graphql_url();
    
    let variables = json!({
        "input": {
            "refreshToken": refresh_token,
        }
    });

    let request = GraphqlRequest {
        query: REFRESH_TOKEN_MUTATION.to_string(),
        variables,
    };

    let response: RefreshTokenResponse = execute(&url, request, None, Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    let payload = response.refresh_token;

    let session = AuthSession {
        token: payload.access_token,
        tenant,
    };

    Ok(session)
}

/// Request password reset
pub async fn forgot_password(email: String, tenant: String) -> Result<String, AuthError> {
    let url = get_graphql_url();
    
    let variables = json!({
        "input": {
            "email": email,
        }
    });

    let request = GraphqlRequest {
        query: FORGOT_PASSWORD_MUTATION.to_string(),
        variables,
    };

    let response: ForgotPasswordResponse = execute(&url, request, None, Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    Ok(response.forgot_password.message)
}

/// Get current user
pub async fn fetch_current_user(token: String, tenant: String) -> Result<Option<AuthUser>, AuthError> {
    let url = get_graphql_url();

    let request = GraphqlRequest {
        query: CURRENT_USER_QUERY.to_string(),
        variables: serde_json::Value::Null,
    };

    let response: CurrentUserResponse = execute(&url, request, Some(&token), Some(&tenant))
        .await
        .map_err(|_| AuthError::Network)?;

    Ok(response.me.map(|user| AuthUser {
        id: user.id,
        email: user.email,
        name: user.name,
    }))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_in_mutation() {
        assert!(SIGN_IN_MUTATION.contains("mutation SignIn"));
        assert!(SIGN_IN_MUTATION.contains("signIn"));
        assert!(SIGN_IN_MUTATION.contains("accessToken"));
    }

    #[test]
    fn test_sign_up_mutation() {
        assert!(SIGN_UP_MUTATION.contains("mutation SignUp"));
        assert!(SIGN_UP_MUTATION.contains("signUp"));
        assert!(SIGN_UP_MUTATION.contains("user"));
    }

    #[test]
    fn test_graphql_url() {
        let url = get_graphql_url();
        assert!(url.contains("/api/graphql"));
    }
}
