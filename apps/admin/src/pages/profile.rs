use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_current_user, use_tenant, use_token};
use serde::{Deserialize, Serialize};

use crate::api::{request, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::translate;

// GraphQL mutation for updating profile
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
pub fn Profile() -> impl IntoView {
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
    let (status, set_status) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let on_save = move |_| {
        let token_value = token.get();
        let tenant_value = tenant.get();
        if token_value.is_none() {
            set_error.set(Some(translate("errors.auth.unauthorized").to_string()));
            set_status.set(None);
            return;
        }

        let name_value = name.get().trim().to_string();

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
                    set_error.set(None);
                    set_status.set(Some(translate("profile.saved").to_string()));
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => translate("errors.auth.unauthorized").to_string(),
                        ApiError::Http(_) => translate("errors.http").to_string(),
                        ApiError::Network => translate("errors.network").to_string(),
                        ApiError::Graphql(_) => translate("errors.unknown").to_string(),
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                }
            }
        });
    };

    view! {
        <section class="px-10 py-8">
            <header class="mb-6 flex flex-wrap items-start justify-between gap-4">
                <div>
                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                        {move || translate("profile.badge")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold">
                        {move || translate("profile.title")}
                    </h1>
                    <p class="mt-2 text-sm text-slate-500">
                        {move || translate("profile.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <LanguageToggle />
                    <Button on_click=on_save>{move || translate("profile.save")}</Button>
                </div>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="grid gap-4 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h3 class="text-lg font-semibold">
                        {move || translate("profile.sectionTitle")}
                    </h3>
                    <p class="text-sm text-slate-500">
                        {move || translate("profile.sectionSubtitle")}
                    </p>
                    <Input
                        value=name
                        set_value=set_name
                        placeholder="Alex Morgan"
                        label=move || translate("profile.nameLabel")
                    />
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-slate-600">
                            {move || translate("profile.emailLabel")}
                        </label>
                        <p class="rounded-xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-500">
                            {move || email.get()}
                        </p>
                    </div>
                    <Input
                        value=avatar
                        set_value=set_avatar
                        placeholder="https://cdn.rustok.io/avatar.png"
                        label=move || translate("profile.avatarLabel")
                    />
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-slate-600">
                            {move || translate("profile.timezoneLabel")}
                        </label>
                        <select
                            class="rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| set_timezone.set(event_target_value(&ev))
                            prop:value=timezone
                        >
                            <option value="Europe/Moscow">"Europe/Moscow"</option>
                            <option value="Europe/Berlin">"Europe/Berlin"</option>
                            <option value="America/New_York">"America/New_York"</option>
                            <option value="Asia/Dubai">"Asia/Dubai"</option>
                        </select>
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-sm text-slate-600">
                            {move || translate("profile.userLocaleLabel")}
                        </label>
                        <select
                            class="rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| set_preferred_locale.set(event_target_value(&ev))
                            prop:value=preferred_locale
                        >
                            <option value="ru">{move || translate("profile.localeRu")}</option>
                            <option value="en">{move || translate("profile.localeEn")}</option>
                        </select>
                        <p class="text-sm text-slate-500">
                            {move || translate("profile.localeHint")}
                        </p>
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="rounded-xl bg-emerald-100 px-4 py-2 text-sm text-emerald-700">
                            {move || status.get().unwrap_or_default()}
                        </div>
                    </Show>
                </div>

                <div class="grid gap-4 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h3 class="text-lg font-semibold">
                        {move || translate("profile.preferencesTitle")}
                    </h3>
                    <p class="text-sm text-slate-500">
                        {move || translate("profile.preferencesSubtitle")}
                    </p>
                    <div class="flex items-center justify-between gap-4 border-b border-slate-200 py-3 last:border-b-0">
                        <div>
                            <strong>{move || translate("profile.uiLocaleLabel")}</strong>
                            <p class="text-sm text-slate-500">
                                {move || translate("profile.uiLocaleHint")}
                            </p>
                        </div>
                        <LanguageToggle />
                    </div>
                    <div class="flex items-center justify-between gap-4 border-b border-slate-200 py-3 last:border-b-0">
                        <div>
                            <strong>{move || translate("profile.notificationsTitle")}</strong>
                            <p class="text-sm text-slate-500">
                                {move || translate("profile.notificationsHint")}
                            </p>
                        </div>
                        <span class="inline-flex items-center rounded-full bg-slate-200 px-2.5 py-1 text-xs text-slate-600">
                            {move || translate("profile.notificationsStatus")}
                        </span>
                    </div>
                    <div class="flex items-center justify-between gap-4 border-b border-slate-200 py-3 last:border-b-0">
                        <div>
                            <strong>{move || translate("profile.auditTitle")}</strong>
                            <p class="text-sm text-slate-500">
                                {move || translate("profile.auditHint")}
                            </p>
                        </div>
                        <Button
                            on_click=move |_| {}
                            class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                        >
                            {move || translate("profile.auditAction")}
                        </Button>
                    </div>
                </div>
            </div>
        </section>
    }
}
