use leptos::*;
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
}

pub fn provide_auth_context() {
    let (user, set_user) = create_signal(None);
    let (token, set_token) = create_signal(None);

    provide_context(AuthContext {
        user,
        set_user,
        token,
        set_token,
    });
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext not found")
}
