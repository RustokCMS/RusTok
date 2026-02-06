use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::api::{rest_get, rest_post, ApiError};
use crate::components::ui::{Button, Input};
use crate::providers::auth::use_auth;
use crate::providers::locale::{translate, use_locale};

#[derive(Clone, Deserialize)]
struct SessionItem {
    user_agent: Option<String>,
    ip_address: Option<String>,
    created_at: String,
    current: bool,
}

#[derive(Deserialize)]
struct SessionsResponse {
    sessions: Vec<SessionItem>,
}

#[derive(Serialize)]
struct ChangePasswordParams {
    current_password: String,
    new_password: String,
}

#[derive(Deserialize)]
struct GenericStatus {}

#[component]
pub fn Security() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();

    let (current_password, set_current_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (status, set_status) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);
    let (sessions, set_sessions) = signal(Vec::<SessionItem>::new());
    let (history, set_history) = signal(Vec::<SessionItem>::new());

    let load_sessions = move || {
        let token = auth.token.get();
        let tenant_slug = auth.tenant_slug.get();
        let set_sessions = set_sessions;
        let set_error = set_error;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result =
                rest_get::<SessionsResponse>("/api/auth/sessions", token, tenant_slug).await;
            match result {
                Ok(response) => {
                    set_error.set(None);
                    set_sessions.set(response.sessions);
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
                }
            }
        });
    };

    let load_history = move || {
        let token = auth.token.get();
        let tenant_slug = auth.tenant_slug.get();
        let set_history = set_history;
        let set_error = set_error;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result =
                rest_get::<SessionsResponse>("/api/auth/history", token, tenant_slug).await;
            match result {
                Ok(response) => {
                    set_error.set(None);
                    set_history.set(response.sessions);
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
                }
            }
        });
    };

    let on_change_password = move |_| {
        if current_password.get().is_empty() || new_password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "security.passwordRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        let token = auth.token.get();
        let tenant_slug = auth.tenant_slug.get();
        if token.is_none() {
            set_error.set(Some(
                translate(locale.locale.get(), "errors.auth.unauthorized").to_string(),
            ));
            set_status.set(None);
            return;
        }

        let current_password_value = current_password.get();
        let new_password_value = new_password.get();
        let set_error = set_error;
        let set_status = set_status;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<ChangePasswordParams, GenericStatus>(
                "/api/auth/change-password",
                &ChangePasswordParams {
                    current_password: current_password_value,
                    new_password: new_password_value,
                },
                token,
                tenant_slug,
            )
            .await;

            match result {
                Ok(_) => {
                    set_error.set(None);
                    set_status.set(Some(
                        translate(locale_signal.get(), "security.signOutAll").to_string(),
                    ));
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

    let on_sign_out_all = move |_| {
        let token = auth.token.get();
        let tenant_slug = auth.tenant_slug.get();
        let set_error = set_error;
        let set_status = set_status;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<serde_json::Value, GenericStatus>(
                "/api/auth/sessions/revoke-all",
                &serde_json::json!({}),
                token,
                tenant_slug,
            )
            .await;

            match result {
                Ok(_) => {
                    set_error.set(None);
                    set_status.set(Some(
                        translate(locale_signal.get(), "security.passwordUpdated").to_string(),
                    ));
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
                }
            }
        });
    };

    Effect::new(move |_| {
        load_sessions();
        load_history();
    });

    view! {
        <section class="settings-page">
            <header class="settings-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "security.badge")}</span>
                    <h1>{move || translate(locale.locale.get(), "security.title")}</h1>
                    <p>{move || translate(locale.locale.get(), "security.subtitle")}</p>
                </div>
                <div class="settings-actions">
                    <Button on_click=on_sign_out_all class="ghost-button">{move || translate(locale.locale.get(), "security.signOutAll")}</Button>
                </div>
            </header>

            <div class="settings-grid">
                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.passwordTitle")}</h3>
                    <p class="section-subtitle">{move || translate(locale.locale.get(), "security.passwordSubtitle")}</p>
                    <Input value=current_password set_value=set_current_password placeholder="••••••••" type_="password" label=move || translate(locale.locale.get(), "security.currentPasswordLabel") />
                    <Input value=new_password set_value=set_new_password placeholder="••••••••" type_="password" label=move || translate(locale.locale.get(), "security.newPasswordLabel") />
                    <p class="form-hint">{move || translate(locale.locale.get(), "security.passwordHint")}</p>
                    <Button on_click=on_change_password class="w-full">{move || translate(locale.locale.get(), "security.passwordSubmit")}</Button>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                </div>

                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.sessionsTitle")}</h3>
                    <p class="section-subtitle">{move || translate(locale.locale.get(), "security.sessionsSubtitle")}</p>
                    <div class="session-list">
                        {move || {
                            sessions
                                .get()
                                .into_iter()
                                .map(|session| {
                                    let label = session
                                        .user_agent
                                        .clone()
                                        .unwrap_or_else(|| "Unknown device".to_string());
                                    let ip = session
                                        .ip_address
                                        .clone()
                                        .unwrap_or_else(|| "Unknown IP".to_string());
                                    let status_label = if session.current {
                                        "Current"
                                    } else {
                                        "Other"
                                    };
                                    view! {
                                        <div class="session-item">
                                            <div>
                                                <strong>{label}</strong>
                                                <p class="form-hint">{move || translate(locale.locale.get(), "security.sessionIp")} ": " {ip}</p>
                                            </div>
                                            <div class="session-meta">
                                                <span class="status-pill">{status_label}</span>
                                                <span class="meta-text">{session.created_at}</span>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>

                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.historyTitle")}</h3>
                    <p class="section-subtitle">{move || translate(locale.locale.get(), "security.historySubtitle")}</p>
                    <div class="session-list">
                        {move || {
                            history
                                .get()
                                .into_iter()
                                .map(|event| {
                                    let label = event
                                        .user_agent
                                        .clone()
                                        .unwrap_or_else(|| "Unknown device".to_string());
                                    let ip = event
                                        .ip_address
                                        .clone()
                                        .unwrap_or_else(|| "Unknown IP".to_string());
                                    view! {
                                        <div class="session-item">
                                            <div>
                                                <strong>{label}</strong>
                                                <p class="form-hint">{move || translate(locale.locale.get(), "security.sessionIp")} ": " {ip}</p>
                                            </div>
                                            <span class="status-pill">{event.created_at}</span>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
