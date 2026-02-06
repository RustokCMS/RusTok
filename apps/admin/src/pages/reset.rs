use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::api::{rest_post, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::{translate, use_locale};

#[derive(Serialize)]
struct ResetRequestParams {
    email: String,
}

#[derive(Deserialize)]
struct ResetRequestResponse {
    reset_token: Option<String>,
}

#[derive(Serialize)]
struct ResetConfirmParams {
    token: String,
    password: String,
}

#[derive(Deserialize)]
struct GenericStatus {}

#[component]
pub fn ResetPassword() -> impl IntoView {
    let auth = crate::providers::auth::use_auth();
    let locale = use_locale();

    let initial_tenant = auth.tenant_slug.get().unwrap_or_default();
    let (tenant, set_tenant) = signal(initial_tenant);
    let (email, set_email) = signal(String::new());
    let (token, set_token) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (status, set_status) = signal(Option::<String>::None);
    let (token_expired, set_token_expired) = signal(false);

    let on_request = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "reset.errorRequired").to_string(),
            ));
            set_status.set(None);
            set_token_expired.set(false);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();
        let set_error = set_error;
        let set_status = set_status;
        let set_token = set_token;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<ResetRequestParams, ResetRequestResponse>(
                "/api/auth/reset/request",
                &ResetRequestParams { email: email_value },
                None,
                Some(tenant_value.clone()),
            )
            .await;

            match result {
                Ok(response) => {
                    set_error.set(None);
                    if let Some(reset_token) = response.reset_token {
                        set_token.set(reset_token);
                    }
                    set_status.set(Some(
                        translate(locale_signal.get(), "reset.requestSent").to_string(),
                    ));
                    set_token_expired.set(false);
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => {
                            translate(locale_signal.get(), "errors.auth.unauthorized").to_string()
                        }
                        ApiError::Http(_) => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.http").to_string()
                        }
                        ApiError::Network => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.network").to_string()
                        }
                        ApiError::Graphql(_) => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.unknown").to_string()
                        }
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                    set_token_expired.set(false);
                }
            }
        });
    };

    let on_reset = move |_| {
        if token.get().is_empty() || new_password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "reset.tokenRequired").to_string(),
            ));
            set_status.set(None);
            set_token_expired.set(false);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let token_value = token.get();
        let password_value = new_password.get();
        let set_error = set_error;
        let set_status = set_status;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<ResetConfirmParams, GenericStatus>(
                "/api/auth/reset/confirm",
                &ResetConfirmParams {
                    token: token_value,
                    password: password_value,
                },
                None,
                Some(tenant_value.clone()),
            )
            .await;

            match result {
                Ok(_) => {
                    set_error.set(None);
                    set_status.set(Some(
                        translate(locale_signal.get(), "reset.updated").to_string(),
                    ));
                    set_token_expired.set(false);
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => {
                            set_token_expired.set(true);
                            translate(locale_signal.get(), "reset.tokenExpired").to_string()
                        }
                        ApiError::Http(_) => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.http").to_string()
                        }
                        ApiError::Network => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.network").to_string()
                        }
                        ApiError::Graphql(_) => {
                            set_token_expired.set(false);
                            translate(locale_signal.get(), "errors.unknown").to_string()
                        }
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                }
            }
        });
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">{move || translate(locale.locale.get(), "reset.badge")}</span>
                <h1>{move || translate(locale.locale.get(), "reset.heroTitle")}</h1>
                <p>{move || translate(locale.locale.get(), "reset.heroSubtitle")}</p>
                <div class="auth-note">
                    <p><strong>{move || translate(locale.locale.get(), "reset.heroListTitle")}</strong></p>
                    <p>{move || translate(locale.locale.get(), "reset.heroListSubtitle")}</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>{move || translate(locale.locale.get(), "reset.title")}</h2>
                        <p>{move || translate(locale.locale.get(), "reset.subtitle")}</p>
                    </div>
                    <div class="auth-locale">
                        <span>{move || translate(locale.locale.get(), "reset.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || token_expired.get()>
                        <div class="alert warning">{move || translate(locale.locale.get(), "reset.requestNewLink")}</div>
                    </Show>
                    <Input value=tenant set_value=set_tenant placeholder="demo" label=move || translate(locale.locale.get(), "reset.tenantLabel") />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label=move || translate(locale.locale.get(), "reset.emailLabel") />
                    <Button on_click=on_request class="w-full">{move || translate(locale.locale.get(), "reset.requestSubmit")}</Button>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "reset.tokenTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "reset.tokenSubtitle")}</p>
                    </div>
                    <Input value=token set_value=set_token placeholder="RESET-2024-0001" label=move || translate(locale.locale.get(), "reset.tokenLabel") />
                    <Input value=new_password set_value=set_new_password placeholder="••••••••" type_="password" label=move || translate(locale.locale.get(), "reset.newPasswordLabel") />
                    <p class="form-hint">{move || translate(locale.locale.get(), "reset.tokenHint")}</p>
                    <Button on_click=on_reset class="w-full ghost-button">{move || translate(locale.locale.get(), "reset.tokenSubmit")}</Button>
                    <div class="auth-links">
                        <a class="secondary-link" href="/login">{move || translate(locale.locale.get(), "reset.loginLink")}</a>
                        <a class="secondary-link" href="/register">{move || translate(locale.locale.get(), "reset.registerLink")}</a>
                    </div>
                </div>
            </div>
        </section>
    }
}
