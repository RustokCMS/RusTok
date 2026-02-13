use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user: ReadSignal<Option<User>>,
    pub set_user: WriteSignal<Option<User>>,
    pub token: ReadSignal<Option<String>>,
    pub set_token: WriteSignal<Option<String>>,
    pub tenant_slug: ReadSignal<Option<String>>,
    pub set_tenant_slug: WriteSignal<Option<String>>,
}

pub fn provide_auth_context() {
    let (user, set_user) = signal(load_user_from_storage());
    let (token, set_token) = signal(load_token_from_storage());
    let (tenant_slug, set_tenant_slug) = signal(load_tenant_slug_from_storage());

    Effect::new(move |_| match token.get() {
        Some(value) => {
            let _ = LocalStorage::set("rustok-admin-token", value);
        }
        None => {
            LocalStorage::delete("rustok-admin-token");
        }
    });

    Effect::new(move |_| match tenant_slug.get() {
        Some(value) => {
            let _ = LocalStorage::set("rustok-admin-tenant", value);
        }
        None => {
            LocalStorage::delete("rustok-admin-tenant");
        }
    });

    Effect::new(move |_| match user.get() {
        Some(ref value) => {
            let _ = LocalStorage::set("rustok-admin-user", value);
        }
        None => {
            LocalStorage::delete("rustok-admin-user");
        }
    });

    provide_context(AuthContext {
        user,
        set_user,
        token,
        set_token,
        tenant_slug,
        set_tenant_slug,
    });
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found")
}

fn load_token_from_storage() -> Option<String> {
    LocalStorage::get("rustok-admin-token").ok()
}

fn load_user_from_storage() -> Option<User> {
    LocalStorage::get("rustok-admin-user").ok()
}

fn load_tenant_slug_from_storage() -> Option<String> {
    LocalStorage::get("rustok-admin-tenant").ok()
}
