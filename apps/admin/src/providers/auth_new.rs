// New auth provider using leptos-auth library
// This is a compatibility wrapper that can gradually replace the old auth.rs

use leptos::prelude::*;

pub use leptos_auth::{
    use_auth as use_leptos_auth, use_current_user, use_is_authenticated, use_is_loading,
    use_session, use_tenant, use_token, AuthContext, AuthError, AuthProvider, AuthSession,
    AuthUser, GuestRoute, ProtectedRoute as LeptosProtectedRoute, RequireAuth,
};

// Re-export for compatibility with existing code
pub type User = AuthUser;

/// Compatibility wrapper for existing code that uses the old AuthContext
#[derive(Clone, Debug)]
pub struct LegacyAuthContext {
    pub user: Signal<Option<User>>,
    pub token: Signal<Option<String>>,
    pub tenant_slug: Signal<Option<String>>,
    inner: AuthContext,
}

impl LegacyAuthContext {
    pub fn new(inner: AuthContext) -> Self {
        let user = Signal::derive(move || inner.user.get());
        let token = Signal::derive(move || inner.get_token());
        let tenant_slug = Signal::derive(move || inner.get_tenant());

        Self {
            user,
            token,
            tenant_slug,
            inner,
        }
    }

    pub async fn sign_in(
        &self,
        email: String,
        password: String,
        tenant: String,
    ) -> Result<(), AuthError> {
        self.inner.sign_in(email, password, tenant).await
    }

    pub async fn sign_up(
        &self,
        email: String,
        password: String,
        name: Option<String>,
        tenant: String,
    ) -> Result<(), AuthError> {
        self.inner.sign_up(email, password, name, tenant).await
    }

    pub async fn sign_out(&self) -> Result<(), AuthError> {
        self.inner.sign_out().await
    }

    pub fn is_authenticated(&self) -> bool {
        self.inner.is_authenticated()
    }
}

/// Drop-in replacement for provide_auth_context() that uses leptos-auth
pub fn provide_auth_context_new() {
    // The AuthProvider will be added at the app level
    // This function is kept for compatibility but doesn't need to do anything
    // since <AuthProvider> component handles context provision
}

/// Get legacy-compatible auth context
pub fn use_auth_legacy() -> LegacyAuthContext {
    let inner = use_leptos_auth();
    LegacyAuthContext::new(inner)
}
