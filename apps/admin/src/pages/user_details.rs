use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use serde::{Deserialize, Serialize};

use crate::api::{request, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::use_auth;
use crate::providers::locale::{translate, use_locale};

#[derive(Params, PartialEq)]
struct UserParams {
    id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlUserResponse {
    user: Option<GraphqlUser>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlUser {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Clone, Debug, Serialize)]
struct UserVariables {
    id: String,
}

#[component]
pub fn UserDetails() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();
    let navigate = use_navigate();
    let params = use_params::<UserParams>();
    let (tenant_slug, set_tenant_slug) = signal(String::new());

    let user_resource = Resource::new(
        move || {
            (
                params.with(|params| params.as_ref().ok().and_then(|params| params.id.clone())),
                tenant_slug.get(),
            )
        },
        move |_| {
            let token = auth.token.get().unwrap_or_default();
            let tenant = tenant_slug.get().trim().to_string();
            let user_id = params.with(|params| {
                params
                    .as_ref()
                    .ok()
                    .and_then(|params| params.id.clone())
                    .unwrap_or_default()
            });

            async move {
                request::<UserVariables, GraphqlUserResponse>(
                    "query User($id: ID!) { user(id: $id) { id email name role status createdAt } }",
                    UserVariables { id: user_id },
                    if token.is_empty() { None } else { Some(token) },
                    if tenant.is_empty() { None } else { Some(tenant) },
                )
                .await
            }
        },
    );

    let go_back = move |_| {
        navigate("/users", Default::default());
    };

    view! {
        <section class="px-10 py-8">
            <header class="mb-8 flex flex-wrap items-center justify-between gap-4">
                <div>
                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                        {move || translate(locale.locale.get(), "app.nav.users")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold">
                        {move || translate(locale.locale.get(), "users.detail.title")}
                    </h1>
                    <p class="mt-2 text-sm text-slate-500">
                        {move || translate(locale.locale.get(), "users.detail.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <LanguageToggle />
                    <Button
                        on_click=go_back
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate(locale.locale.get(), "users.detail.back")}
                    </Button>
                </div>
            </header>

            <div class="mb-6 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h4 class="mb-4 text-lg font-semibold">
                    {move || translate(locale.locale.get(), "users.access.title")}
                </h4>
                <div class="grid gap-4 md:grid-cols-3">
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label=move || translate(locale.locale.get(), "users.access.tenant")
                    />
                </div>
                <p class="mt-3 text-sm text-slate-500">
                    {move || translate(locale.locale.get(), "users.access.hint")}
                </p>
            </div>

            <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h4 class="mb-4 text-lg font-semibold">
                    {move || translate(locale.locale.get(), "users.detail.section")}
                </h4>
                <Suspense
                    fallback=move || view! {
                        <p class="text-sm text-slate-500">
                            {move || translate(locale.locale.get(), "users.detail.loading")}
                        </p>
                    }
                >
                    {move || match user_resource.get() {
                        None => view! {
                            <p class="text-sm text-slate-500">
                                {move || translate(locale.locale.get(), "users.detail.pending")}
                            </p>
                        }
                        .into_any(),
                        Some(Ok(response)) => {
                            if let Some(user) = response.user {
                                view! {
                                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.email")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.email}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.name")}
                                            </span>
                                            <p class="mt-1 text-sm">
                                                {user.name.unwrap_or_else(|| translate(locale.locale.get(), "users.placeholderDash").to_string())}
                                            </p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.role")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.role}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.status")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.status}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.createdAt")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.created_at}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate(locale.locale.get(), "users.detail.id")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.id}</p>
                                        </div>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                        {move || translate(locale.locale.get(), "users.detail.empty")}
                                    </div>
                                }
                                .into_any()
                            }
                        }
                        Some(Err(err)) => view! {
                            <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                {match err {
                                    ApiError::Unauthorized => translate(locale.locale.get(), "users.graphql.unauthorized").to_string(),
                                    ApiError::Http(code) => format!("{} {}", translate(locale.locale.get(), "users.graphql.error"), code),
                                    ApiError::Network => translate(locale.locale.get(), "users.graphql.network").to_string(),
                                    ApiError::Graphql(message) => format!("{} {}", translate(locale.locale.get(), "users.graphql.error"), message),
                                }}
                            </div>
                        }
                        .into_any(),
                    }}
                </Suspense>
            </div>
        </section>
    }
}
