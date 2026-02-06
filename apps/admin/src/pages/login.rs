use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

use crate::api::{rest_post, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::{use_auth, User};
use crate::providers::locale::{translate, use_locale};

#[derive(Serialize)]
struct LoginParams {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    #[serde(rename = "access_token")]
    access_token: String,
    user: AuthUser,
}

#[derive(Deserialize)]
struct AuthUser {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
}

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();
    let navigate = use_navigate();

    let (tenant, set_tenant) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);

    let navigate_effect = navigate.clone();
    Effect::new(move |_| {
        if auth.token.get().is_some() {
            navigate_effect("/dashboard", Default::default());
        }
    });

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "auth.errorRequired").to_string(),
            ));
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();
        let password_value = password.get();
        let set_token = auth.set_token;
        let set_user = auth.set_user;
        let set_tenant_slug = auth.set_tenant_slug;
        let locale = locale.locale;
        let navigate = navigate.clone();

        set_error.set(None);
        set_is_loading.set(true);

        spawn_local(async move {
            let result = rest_post::<LoginParams, AuthResponse>(
                "/api/auth/login",
                &LoginParams {
                    email: email_value,
                    password: password_value,
                },
                None,
                Some(tenant_value.clone()),
            )
            .await;

            match result {
                Ok(response) => {
                    set_token.set(Some(response.access_token));
                    set_tenant_slug.set(Some(tenant_value));
                    set_user.set(Some(User {
                        id: response.user.id,
                        email: response.user.email,
                        name: response.user.name,
                        role: response.user.role,
                    }));
                    navigate("/dashboard", Default::default());
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => {
                            translate(locale.get(), "errors.auth.invalid_credentials").to_string()
                        }
                        ApiError::Http(_) => translate(locale.get(), "errors.http").to_string(),
                        ApiError::Network => translate(locale.get(), "errors.network").to_string(),
                        ApiError::Graphql(_) => {
                            translate(locale.get(), "errors.unknown").to_string()
                        }
                    };
                    set_error.set(Some(message));
                }
            }

            set_is_loading.set(false);
        });
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">{move || translate(locale.locale.get(), "auth.badge")}</span>
                <h1>{move || translate(locale.locale.get(), "auth.heroTitle")}</h1>
                <p>
                    {move || translate(locale.locale.get(), "auth.heroSubtitle")}
                </p>
                <div>
                    <p>
                        <strong>{move || translate(locale.locale.get(), "auth.heroListTitle")}</strong>
                    </p>
                    <p>{move || translate(locale.locale.get(), "auth.heroListSubtitle")}</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>{move || translate(locale.locale.get(), "auth.title")}</h2>
                        <p>{move || translate(locale.locale.get(), "auth.subtitle")}</p>
                    </div>
                    <div class="auth-locale">
                        <span>{move || translate(locale.locale.get(), "auth.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Input
                        value=tenant
                        set_value=set_tenant
                        placeholder="demo"
                        label=move || translate(locale.locale.get(), "auth.tenantLabel")
                    />
                    <Input
                        value=email
                        set_value=set_email
                        placeholder="admin@rustok.io"
                        label=move || translate(locale.locale.get(), "auth.emailLabel")
                    />
                    <Input
                        value=password
                        set_value=set_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "auth.passwordLabel")
                    />
                    <Button
                        on_click=on_submit
                        class="w-full"
                        disabled=Signal::derive(move || is_loading.get())
                    >
                        {move || translate(locale.locale.get(), "auth.submit")}
                    </Button>
                    <div class="auth-links">
                        <a class="secondary-link" href="/register">
                            {move || translate(locale.locale.get(), "auth.registerLink")}
                        </a>
                        <a class="secondary-link" href="/reset">
                            {move || translate(locale.locale.get(), "auth.resetLink")}
                        </a>
                    </div>
                </div>
                <p style="margin:0; color:#64748b;">
                    {move || translate(locale.locale.get(), "auth.footer")}
                </p>
            </div>
        </section>
    }
}
