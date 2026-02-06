use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::api::{rest_post, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::use_auth;
use crate::providers::locale::{translate, use_locale};

#[derive(Serialize)]
struct UpdateProfileParams {
    name: Option<String>,
}

#[derive(Deserialize)]
struct UserResponse {
    name: Option<String>,
}

#[component]
pub fn Profile() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();

    let initial_name = auth
        .user
        .get()
        .and_then(|user| user.name)
        .unwrap_or_default();
    let initial_email = auth.user.get().map(|user| user.email).unwrap_or_default();

    let (name, set_name) = signal(initial_name);
    let (email, set_email) = signal(initial_email);
    let (avatar, set_avatar) = signal(String::new());
    let (timezone, set_timezone) = signal(String::from("Europe/Moscow"));
    let (preferred_locale, set_preferred_locale) = signal(String::from("ru"));
    let (status, set_status) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let on_save = move |_| {
        let token = auth.token.get();
        let tenant_slug = auth.tenant_slug.get();
        if token.is_none() {
            set_error.set(Some(
                translate(locale.locale.get(), "errors.auth.unauthorized").to_string(),
            ));
            set_status.set(None);
            return;
        }

        let name_value = name.get().trim().to_string();
        let set_status = set_status;
        let set_error = set_error;
        let set_name = set_name;
        let locale_signal = locale.locale;

        spawn_local(async move {
            let result = rest_post::<UpdateProfileParams, UserResponse>(
                "/api/auth/profile",
                &UpdateProfileParams {
                    name: if name_value.is_empty() {
                        None
                    } else {
                        Some(name_value)
                    },
                },
                token,
                tenant_slug,
            )
            .await;

            match result {
                Ok(user) => {
                    if let Some(new_name) = user.name {
                        set_name.set(new_name);
                    }
                    set_error.set(None);
                    set_status.set(Some(
                        translate(locale_signal.get(), "profile.saved").to_string(),
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

    view! {
        <section class="settings-page">
            <header class="settings-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "profile.badge")}</span>
                    <h1>{move || translate(locale.locale.get(), "profile.title")}</h1>
                    <p>{move || translate(locale.locale.get(), "profile.subtitle")}</p>
                </div>
                <div class="settings-actions">
                    <LanguageToggle />
                    <Button on_click=on_save>{move || translate(locale.locale.get(), "profile.save")}</Button>
                </div>
            </header>

            <div class="settings-grid">
                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "profile.sectionTitle")}</h3>
                    <p class="section-subtitle">{move || translate(locale.locale.get(), "profile.sectionSubtitle")}</p>
                    <Input value=name set_value=set_name placeholder="Alex Morgan" label=move || translate(locale.locale.get(), "profile.nameLabel") />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label=move || translate(locale.locale.get(), "profile.emailLabel") />
                    <Input value=avatar set_value=set_avatar placeholder="https://cdn.rustok.io/avatar.png" label=move || translate(locale.locale.get(), "profile.avatarLabel") />
                    <div class="input-group">
                        <label>{move || translate(locale.locale.get(), "profile.timezoneLabel")}</label>
                        <select class="input-select" on:change=move |ev| set_timezone.set(event_target_value(&ev)) prop:value=timezone>
                            <option value="Europe/Moscow">"Europe/Moscow"</option>
                            <option value="Europe/Berlin">"Europe/Berlin"</option>
                            <option value="America/New_York">"America/New_York"</option>
                            <option value="Asia/Dubai">"Asia/Dubai"</option>
                        </select>
                    </div>
                    <div class="input-group">
                        <label>{move || translate(locale.locale.get(), "profile.userLocaleLabel")}</label>
                        <select class="input-select" on:change=move |ev| set_preferred_locale.set(event_target_value(&ev)) prop:value=preferred_locale>
                            <option value="ru">{move || translate(locale.locale.get(), "profile.localeRu")}</option>
                            <option value="en">{move || translate(locale.locale.get(), "profile.localeEn")}</option>
                        </select>
                        <p class="form-hint">{move || translate(locale.locale.get(), "profile.localeHint")}</p>
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                </div>

                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "profile.preferencesTitle")}</h3>
                    <p class="section-subtitle">{move || translate(locale.locale.get(), "profile.preferencesSubtitle")}</p>
                    <div class="preference-row">
                        <div>
                            <strong>{move || translate(locale.locale.get(), "profile.uiLocaleLabel")}</strong>
                            <p class="form-hint">{move || translate(locale.locale.get(), "profile.uiLocaleHint")}</p>
                        </div>
                        <LanguageToggle />
                    </div>
                    <div class="preference-row">
                        <div>
                            <strong>{move || translate(locale.locale.get(), "profile.notificationsTitle")}</strong>
                            <p class="form-hint">{move || translate(locale.locale.get(), "profile.notificationsHint")}</p>
                        </div>
                        <span class="status-pill">{move || translate(locale.locale.get(), "profile.notificationsStatus")}</span>
                    </div>
                    <div class="preference-row">
                        <div>
                            <strong>{move || translate(locale.locale.get(), "profile.auditTitle")}</strong>
                            <p class="form-hint">{move || translate(locale.locale.get(), "profile.auditHint")}</p>
                        </div>
                        <Button on_click=move |_| {} class="ghost-button">{move || translate(locale.locale.get(), "profile.auditAction")}</Button>
                    </div>
                </div>
            </div>
        </section>
    }
}
