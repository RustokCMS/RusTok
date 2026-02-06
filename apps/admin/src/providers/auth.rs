use leptos::prelude::*;
use leptos::web_sys;
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

    Effect::new(move |_| {
        if let Some(storage) = local_storage() {
            match token.get() {
                Some(value) => {
                    let _ = storage.set_item("rustok-admin-token", &value);
                }
                None => {
                    let _ = storage.remove_item("rustok-admin-token");
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(storage) = local_storage() {
            match tenant_slug.get() {
                Some(value) => {
                    let _ = storage.set_item("rustok-admin-tenant", &value);
                }
                None => {
                    let _ = storage.remove_item("rustok-admin-tenant");
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(storage) = local_storage() {
            match user.get() {
                Some(ref value) => {
                    if let Ok(json) = serde_json::to_string(value) {
                        let _ = storage.set_item("rustok-admin-user", &json);
                    }
                }
                None => {
                    let _ = storage.remove_item("rustok-admin-user");
                }
            }
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

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

fn load_token_from_storage() -> Option<String> {
    let storage = local_storage()?;
    storage.get_item("rustok-admin-token").ok().flatten()
}

fn load_user_from_storage() -> Option<User> {
    let storage = local_storage()?;
    let raw = storage.get_item("rustok-admin-user").ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

fn load_tenant_slug_from_storage() -> Option<String> {
    let storage = local_storage()?;
    storage.get_item("rustok-admin-tenant").ok().flatten()
}
