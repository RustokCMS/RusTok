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
        <section class="grid min-h-screen grid-cols-1 lg:grid-cols-[1.2fr_1fr]">
            <aside class="flex flex-col justify-center gap-6 bg-[radial-gradient(circle_at_top_left,#1e3a8a,#0f172a)] p-12 text-white lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-white/10 px-3 py-1 text-xs font-semibold text-white/80">
                    {move || translate(locale.locale.get(), "auth.badge")}
                </span>
                <h1 class="text-4xl font-semibold">{move || translate(locale.locale.get(), "auth.heroTitle")}</h1>
                <p class="text-lg text-white/80">
                    {move || translate(locale.locale.get(), "auth.heroSubtitle")}
                </p>
                <div>
                    <p class="text-sm font-semibold">
                        {move || translate(locale.locale.get(), "auth.heroListTitle")}
                    </p>
                    <p class="text-sm text-white/75">
                        {move || translate(locale.locale.get(), "auth.heroListSubtitle")}
                    </p>
                </div>
            </aside>
            <div class="flex flex-col justify-center gap-7 bg-slate-50 p-12 lg:p-20">
                <div class="flex flex-col gap-5 rounded-3xl bg-white p-8 shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <div>
                        <h2 class="text-2xl font-semibold">
                            {move || translate(locale.locale.get(), "auth.title")}
                        </h2>
                        <p class="text-slate-500">
                            {move || translate(locale.locale.get(), "auth.subtitle")}
                        </p>
                    </div>
                    <div class="flex items-center justify-between gap-3 text-sm text-slate-600">
                        <span>{move || translate(locale.locale.get(), "auth.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                            {move || error.get().unwrap_or_default()}
                        </div>
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
                    <div class="flex justify-between gap-3 text-sm">
                        <a class="text-blue-600 hover:underline" href="/register">
                            {move || translate(locale.locale.get(), "auth.registerLink")}
                        </a>
                        <a class="text-blue-600 hover:underline" href="/reset">
                            {move || translate(locale.locale.get(), "auth.resetLink")}
                        </a>
                    </div>
                </div>
                <p class="text-sm text-slate-500">
                    {move || translate(locale.locale.get(), "auth.footer")}
                </p>
            </div>
        </section>
    }
}
