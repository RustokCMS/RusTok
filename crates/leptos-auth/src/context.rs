use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::storage;
use crate::{AuthError, AuthSession, AuthUser};

#[derive(Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<AuthUser>>,
    pub session: RwSignal<Option<AuthSession>>,
    pub is_loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let user = RwSignal::new(storage::load_user().ok());
        let session = RwSignal::new(storage::load_session().ok());
        let is_loading = RwSignal::new(false);
        let error = RwSignal::new(None);

        Self {
            user,
            session,
            is_loading,
            error,
        }
    }

    pub async fn sign_in(
        &self,
        email: String,
        password: String,
        tenant: String,
    ) -> Result<(), AuthError> {
        self.is_loading.set(true);
        self.error.set(None);

        let result = api::sign_in(email, password, tenant).await;

        match result {
            Ok((user, session)) => {
                let _ = storage::save_user(&user);
                let _ = storage::save_session(&session);
                self.user.set(Some(user));
                self.session.set(Some(session));
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                self.error.set(Some(format!("{}", e)));
                Err(e)
            }
        }
    }

    pub async fn sign_up(
        &self,
        email: String,
        password: String,
        name: Option<String>,
        tenant: String,
    ) -> Result<(), AuthError> {
        self.is_loading.set(true);
        self.error.set(None);

        let result = api::sign_up(email, password, name, tenant).await;

        match result {
            Ok((user, session)) => {
                let _ = storage::save_user(&user);
                let _ = storage::save_session(&session);
                self.user.set(Some(user));
                self.session.set(Some(session));
                self.is_loading.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_loading.set(false);
                self.error.set(Some(format!("{}", e)));
                Err(e)
            }
        }
    }

    pub async fn sign_out(&self) -> Result<(), AuthError> {
        self.is_loading.set(true);

        if let Some(session) = self.session.get() {
            let _ = api::sign_out(session.token.clone(), session.tenant.clone()).await;
        }

        storage::clear_session();
        self.user.set(None);
        self.session.set(None);
        self.is_loading.set(false);

        Ok(())
    }

    pub async fn refresh_session(&self) -> Result<(), AuthError> {
        if let Some(session) = self.session.get() {
            let new_session =
                api::refresh_token(session.token.clone(), session.tenant.clone()).await?;
            let _ = storage::save_session(&new_session);
            self.session.set(Some(new_session));
            Ok(())
        } else {
            Err(AuthError::Unauthorized)
        }
    }

    pub async fn fetch_current_user(&self) -> Result<(), AuthError> {
        if let Some(session) = self.session.get() {
            let user =
                api::fetch_current_user(session.token.clone(), session.tenant.clone()).await?;
            if let Some(ref u) = user {
                let _ = storage::save_user(u);
            }
            self.user.set(user);
            Ok(())
        } else {
            Err(AuthError::Unauthorized)
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some() && self.session.get().is_some()
    }

    pub fn get_token(&self) -> Option<String> {
        self.session.get().map(|s| s.token)
    }

    pub fn get_tenant(&self) -> Option<String> {
        self.session.get().map(|s| s.tenant)
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let auth_context = AuthContext::new();

    provide_context(auth_context.clone());

    Effect::new(move |_| {
        if auth_context.session.get().is_some() {
            spawn_local(async move {
                let _ = auth_context.fetch_current_user().await;
            });
        }
    });

    children()
}
