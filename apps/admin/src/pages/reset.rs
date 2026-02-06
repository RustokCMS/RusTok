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
        <section class="grid min-h-screen grid-cols-1 lg:grid-cols-[1.2fr_1fr]">
            <aside class="flex flex-col justify-center gap-6 bg-[radial-gradient(circle_at_top_left,#1e3a8a,#0f172a)] p-12 text-white lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-white/10 px-3 py-1 text-xs font-semibold text-white/80">
                    {move || translate(locale.locale.get(), "reset.badge")}
                </span>
                <h1 class="text-4xl font-semibold">{move || translate(locale.locale.get(), "reset.heroTitle")}</h1>
                <p class="text-lg text-white/80">{move || translate(locale.locale.get(), "reset.heroSubtitle")}</p>
                <div class="grid gap-2">
                    <p class="text-sm font-semibold">
                        {move || translate(locale.locale.get(), "reset.heroListTitle")}
                    </p>
                    <p class="text-sm text-white/75">
                        {move || translate(locale.locale.get(), "reset.heroListSubtitle")}
                    </p>
                </div>
            </aside>
            <div class="flex flex-col justify-center gap-7 bg-slate-50 p-12 lg:p-20">
                <div class="flex flex-col gap-5 rounded-3xl bg-white p-8 shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <div>
                        <h2 class="text-2xl font-semibold">
                            {move || translate(locale.locale.get(), "reset.title")}
                        </h2>
                        <p class="text-slate-500">
                            {move || translate(locale.locale.get(), "reset.subtitle")}
                        </p>
                    </div>
                    <div class="flex items-center justify-between gap-3 text-sm text-slate-600">
                        <span>{move || translate(locale.locale.get(), "reset.languageLabel")}</span>
                        <LanguageToggle />
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
                    <Show when=move || token_expired.get()>
                        <div class="alert warning">{move || translate(locale.locale.get(), "reset.requestNewLink")}</div>
                    </Show>
                    <Input value=tenant set_value=set_tenant placeholder="demo" label=move || translate(locale.locale.get(), "reset.tenantLabel") />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label=move || translate(locale.locale.get(), "reset.emailLabel") />
                    <Button on_click=on_request class="w-full">{move || translate(locale.locale.get(), "reset.requestSubmit")}</Button>
                </div>

                <div class="flex flex-col gap-5 rounded-3xl bg-white p-8 shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <div>
                        <h3 class="text-lg font-semibold">
                            {move || translate(locale.locale.get(), "reset.tokenTitle")}
                        </h3>
                        <p class="text-slate-500">
                            {move || translate(locale.locale.get(), "reset.tokenSubtitle")}
                        </p>
                    </div>
                    <Input
                        value=token
                        set_value=set_token
                        placeholder="RESET-2024-0001"
                        label=move || translate(locale.locale.get(), "reset.tokenLabel")
                    />
                    <Input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "reset.newPasswordLabel")
                    />
                    <p class="text-sm text-slate-500">
                        {move || translate(locale.locale.get(), "reset.tokenHint")}
                    </p>
                    <Button
                        on_click=on_reset
                        class="w-full border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate(locale.locale.get(), "reset.tokenSubmit")}
                    </Button>
                    <div class="flex justify-between gap-3 text-sm">
                        <a class="text-blue-600 hover:underline" href="/login">
                            {move || translate(locale.locale.get(), "reset.loginLink")}
                        </a>
                        <a class="text-blue-600 hover:underline" href="/register">
                            {move || translate(locale.locale.get(), "reset.registerLink")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
