use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_current_user, use_tenant, use_token};
use leptos_hook_form::FormState;
use leptos_ui::{Select, SelectOption};
use serde::{Deserialize, Serialize};

use crate::app::providers::locale::translate;
use crate::shared::api::{request, ApiError};
use crate::shared::ui::{ui_button, ui_input, ui_language_toggle};

const UPDATE_PROFILE_MUTATION: &str = r#"
mutation UpdateProfile($input: UpdateProfileInput!) {
    updateProfile(input: $input) {
        id
        email
        name
        role
    }
}
"#;

#[derive(Serialize)]
struct UpdateProfileInput {
    input: ProfileData,
}

#[derive(Serialize)]
struct ProfileData {
    name: Option<String>,
}

#[derive(Deserialize)]
struct UpdateProfileResponse {
    #[serde(rename = "updateProfile")]
    update_profile: ProfileUser,
}

#[derive(Deserialize)]
struct ProfileUser {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    email: String,
    name: Option<String>,
    #[allow(dead_code)]
    role: String,
}

#[component]
pub fn profile() -> impl IntoView {
    let current_user = use_current_user();
    let token = use_token();
    let tenant = use_tenant();

    let initial_name = current_user
        .get()
        .and_then(|user| user.name)
        .unwrap_or_default();
    let initial_email = current_user
        .get()
        .map(|user| user.email)
        .unwrap_or_default();

    let (name, set_name) = signal(initial_name);
    let (email, _set_email) = signal(initial_email);
    let (avatar, set_avatar) = signal(String::new());
    let (timezone, set_timezone) = signal(String::from("Europe/Moscow"));
    let (preferred_locale, set_preferred_locale) = signal(String::from("ru"));
    let (form_state, set_form_state) = signal(FormState::idle());
    let (success_message, set_success_message) = signal(Option::<String>::None);

    let on_save = move |_| {
        let token_value = token.get();
        let tenant_value = tenant.get();
        if token_value.is_none() {
            set_form_state.set(FormState::with_form_error(
                translate("errors.auth.unauthorized").to_string(),
            ));
            set_success_message.set(None);
            return;
        }

        let name_value = name.get().trim().to_string();

        set_form_state.set(FormState::submitting());
        set_success_message.set(None);

        spawn_local(async move {
            let result = request::<UpdateProfileInput, UpdateProfileResponse>(
                UPDATE_PROFILE_MUTATION,
                UpdateProfileInput {
                    input: ProfileData {
                        name: if name_value.is_empty() {
                            None
                        } else {
                            Some(name_value)
                        },
                    },
                },
                token_value,
                tenant_value,
            )
            .await;

            match result {
                Ok(response) => {
                    if let Some(new_name) = response.update_profile.name {
                        set_name.set(new_name);
                    }
                    set_form_state.set(FormState::idle());
                    set_success_message.set(Some(translate("profile.saved").to_string()));
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => translate("errors.auth.unauthorized").to_string(),
                        ApiError::Http(_) => translate("errors.http").to_string(),
                        ApiError::Network => translate("errors.network").to_string(),
                        ApiError::Graphql(_) => translate("errors.unknown").to_string(),
                    };
                    set_form_state.set(FormState::with_form_error(message));
                    set_success_message.set(None);
                }
            }
        });
    };

    view! {
        <section class="px-10 py-8">
            <header class="mb-6 flex flex-wrap items-start justify-between gap-4">
                <div>
                    <span class="inline-flex items-center rounded-full bg-secondary px-3 py-1 text-xs font-semibold text-secondary-foreground">
                        {move || translate("profile.badge")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold text-foreground">
                        {move || translate("profile.title")}
                    </h1>
                    <p class="mt-2 text-sm text-muted-foreground">
                        {move || translate("profile.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <ui_language_toggle />
                    <ui_button
                        on_click=on_save
                        disabled=Signal::derive(move || form_state.get().is_submitting)
                    >
                        {move || {
                            if form_state.get().is_submitting {
                                translate("common.saving").to_string()
                            } else {
                                translate("profile.save").to_string()
                            }
                        }}
                    </ui_button>
                </div>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="grid gap-4 rounded-2xl bg-card p-6 shadow border border-border">
                    <h3 class="text-lg font-semibold text-card-foreground">
                        {move || translate("profile.sectionTitle")}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || translate("profile.sectionSubtitle")}
                    </p>
                    <ui_input
                        value=name
                        set_value=set_name
                        placeholder="Alex Morgan"
                        label=move || translate("profile.nameLabel")
                    />
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-muted-foreground">
                            {move || translate("profile.emailLabel")}
                        </label>
                        <p class="rounded-xl border border-input bg-muted px-4 py-3 text-sm text-muted-foreground">
                            {move || email.get()}
                        </p>
                    </div>
                    <ui_input
                        value=avatar
                        set_value=set_avatar
                        placeholder="https://cdn.rustok.io/avatar.png"
                        label=move || translate("profile.avatarLabel")
                    />
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-muted-foreground">
                            {move || translate("profile.timezoneLabel")}
                        </label>
                        <Select
                            options=vec![
                                SelectOption::new("Europe/Moscow", "Europe/Moscow"),
                                SelectOption::new("Europe/Berlin", "Europe/Berlin"),
                                SelectOption::new("America/New_York", "America/New_York"),
                                SelectOption::new("Asia/Dubai", "Asia/Dubai"),
                            ]
                            value=Some(timezone)
                            set_value=Some(set_timezone)
                        />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-muted-foreground">
                            {move || translate("profile.userLocaleLabel")}
                        </label>
                        <Select
                            options=vec![
                                SelectOption::new("ru", translate("profile.localeRu")),
                                SelectOption::new("en", translate("profile.localeEn")),
                            ]
                            value=Some(preferred_locale)
                            set_value=Some(set_preferred_locale)
                        />
                        <p class="text-sm text-muted-foreground">
                            {move || translate("profile.localeHint")}
                        </p>
                    </div>
                    <Show when=move || form_state.get().form_error.is_some()>
                        <div class="rounded-xl bg-destructive/10 border border-destructive/20 px-4 py-2 text-sm text-destructive">
                            {move || form_state.get().form_error.unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || success_message.get().is_some()>
                        <div class="rounded-xl bg-emerald-100 border border-emerald-200 px-4 py-2 text-sm text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400">
                            {move || success_message.get().unwrap_or_default()}
                        </div>
                    </Show>
                </div>

                <div class="grid gap-4 rounded-2xl bg-card p-6 shadow border border-border">
                    <h3 class="text-lg font-semibold text-card-foreground">
                        {move || translate("profile.preferencesTitle")}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || translate("profile.preferencesSubtitle")}
                    </p>
                    <div class="flex items-center justify-between gap-4 border-b border-border py-3 last:border-b-0">
                        <div>
                            <strong class="text-foreground">{move || translate("profile.uiLocaleLabel")}</strong>
                            <p class="text-sm text-muted-foreground">
                                {move || translate("profile.uiLocaleHint")}
                            </p>
                        </div>
                        <ui_language_toggle />
                    </div>
                    <div class="flex items-center justify-between gap-4 border-b border-border py-3 last:border-b-0">
                        <div>
                            <strong class="text-foreground">{move || translate("profile.notificationsTitle")}</strong>
                            <p class="text-sm text-muted-foreground">
                                {move || translate("profile.notificationsHint")}
                            </p>
                        </div>
                        <span class="inline-flex items-center rounded-full bg-secondary px-2.5 py-1 text-xs text-secondary-foreground">
                            {move || translate("profile.notificationsStatus")}
                        </span>
                    </div>
                    <div class="flex items-center justify-between gap-4 border-b border-border py-3 last:border-b-0">
                        <div>
                            <strong class="text-foreground">{move || translate("profile.auditTitle")}</strong>
                            <p class="text-sm text-muted-foreground">
                                {move || translate("profile.auditHint")}
                            </p>
                        </div>
                        <ui_button
                            on_click=move |_| {}
                            class="border border-input bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground"
                        >
                            {move || translate("profile.auditAction")}
                        </ui_button>
                    </div>
                </div>
            </div>
        </section>
    }
}
