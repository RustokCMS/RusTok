use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

use crate::api::{rest_post, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::{use_auth, User};
use crate::providers::locale::{translate, use_locale};

#[derive(Serialize)]
struct RegisterParams {
    email: String,
    password: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct AuthResponse {
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

#[derive(Serialize)]
struct InviteAcceptParams {
    token: String,
}

#[derive(Deserialize)]
struct InviteAcceptResponse {
    email: String,
    role: String,
}

#[component]
pub fn Register() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();
    let navigate = use_navigate();

    let (tenant, set_tenant) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (invite_token, set_invite_token) = signal(String::new());
    let (verification_email, set_verification_email) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (status, set_status) = signal(Option::<String>::None);

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.errorRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();
        let password_value = password.get();
        let name_value = name.get().trim().to_string();
        let set_error = set_error;
        let set_status = set_status;
        let set_token = auth.set_token;
        let set_user = auth.set_user;
        let set_tenant_slug = auth.set_tenant_slug;
        let locale_signal = locale.locale;
        let navigate = navigate.clone();

        spawn_local(async move {
            let result = rest_post::<RegisterParams, AuthResponse>(
                "/api/auth/register",
                &RegisterParams {
                    email: email_value,
                    password: password_value,
                    name: if name_value.is_empty() {
                        None
                    } else {
                        Some(name_value)
                    },
                },
                None,
                Some(tenant_value.clone()),
            )
            .await;

            match result {
                Ok(response) => {
                    set_error.set(None);
                    set_status.set(Some(
                        translate(locale_signal.get(), "register.success").to_string(),
                    ));
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
                            translate(locale_signal.get(), "errors.auth.unauthorized").to_string()
                        }
                        ApiError::Http(_) => {
                            translate(locale_signal.get(), "errors.http").to_string()
                        }
                        ApiError::Network => {
                            translate(locale_signal.get(), "errors.network").to_string()
                        }
                        ApiError::Graphql(_) => {
                            translate(locale_signal.get(), "errors.unknown").to_string()
                        }
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                }
            }
        });
    };

    let on_accept_invite = move |_| {
        if tenant.get().is_empty() || invite_token.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.inviteRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let invite_value = invite_token.get().trim().to_string();
        let set_error = set_error;
        let set_status = set_status;
        let set_email = set_email;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<InviteAcceptParams, InviteAcceptResponse>(
                "/api/auth/invite/accept",
                &InviteAcceptParams {
                    token: invite_value,
                },
                None,
                Some(tenant_value),
            )
            .await;

            match result {
                Ok(response) => {
                    set_error.set(None);
                    set_email.set(response.email);
                    set_status.set(Some(format!(
                        "{} ({})",
                        translate(locale_signal.get(), "register.inviteAccepted"),
                        response.role
                    )));
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => {
                            translate(locale_signal.get(), "register.inviteExpired").to_string()
                        }
                        ApiError::Http(_) => {
                            translate(locale_signal.get(), "errors.http").to_string()
                        }
                        ApiError::Network => {
                            translate(locale_signal.get(), "errors.network").to_string()
                        }
                        ApiError::Graphql(_) => {
                            translate(locale_signal.get(), "errors.unknown").to_string()
                        }
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                }
            }
        });
    };

    let on_resend_verification = move |_| {
        if verification_email.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.verifyRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "register.verifySent").to_string(),
        ));
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">{move || translate(locale.locale.get(), "register.badge")}</span>
                <h1>{move || translate(locale.locale.get(), "register.heroTitle")}</h1>
                <p>{move || translate(locale.locale.get(), "register.heroSubtitle")}</p>
                <div class="auth-note">
                    <p>
                        <strong>{move || translate(locale.locale.get(), "register.heroListTitle")}</strong>
                    </p>
                    <p>{move || translate(locale.locale.get(), "register.heroListSubtitle")}</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>{move || translate(locale.locale.get(), "register.title")}</h2>
                        <p>{move || translate(locale.locale.get(), "register.subtitle")}</p>
                    </div>
                    <div class="auth-locale">
                        <span>{move || translate(locale.locale.get(), "register.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                    <Input value=tenant set_value=set_tenant placeholder="demo" label=move || translate(locale.locale.get(), "register.tenantLabel") />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label=move || translate(locale.locale.get(), "register.emailLabel") />
                    <Input value=name set_value=set_name placeholder="Alex Morgan" label=move || translate(locale.locale.get(), "register.nameLabel") />
                    <Input value=password set_value=set_password placeholder="••••••••" type_="password" label=move || translate(locale.locale.get(), "register.passwordLabel") />
                    <p class="form-hint">{move || translate(locale.locale.get(), "register.passwordHint")}</p>
                    <Button on_click=on_submit class="w-full">
                        {move || translate(locale.locale.get(), "register.submit")}
                    </Button>
                    <div class="auth-links">
                        <a class="secondary-link" href="/login">{move || translate(locale.locale.get(), "register.loginLink")}</a>
                        <a class="secondary-link" href="/reset">{move || translate(locale.locale.get(), "register.resetLink")}</a>
                    </div>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "register.inviteTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "register.inviteSubtitle")}</p>
                    </div>
                    <Input value=invite_token set_value=set_invite_token placeholder="INVITE-2024-ABCDE" label=move || translate(locale.locale.get(), "register.inviteLabel") />
                    <Button on_click=on_accept_invite class="w-full ghost-button">{move || translate(locale.locale.get(), "register.inviteSubmit")}</Button>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "register.verifyTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "register.verifySubtitle")}</p>
                    </div>
                    <Input value=verification_email set_value=set_verification_email placeholder="admin@rustok.io" label=move || translate(locale.locale.get(), "register.verifyLabel") />
                    <Button on_click=on_resend_verification class="w-full ghost-button">{move || translate(locale.locale.get(), "register.verifySubmit")}</Button>
                </div>
            </div>
        </section>
    }
}
