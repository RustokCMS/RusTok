use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use serde::{Deserialize, Serialize};

use crate::api::{request_with_persisted, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::use_auth;
use crate::providers::locale::translate;

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
    #[serde(rename = "tenantName")]
    tenant_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct UserVariables {
    id: String,
}

#[component]
pub fn UserDetails() -> impl IntoView {
    let auth = use_auth();
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
                request_with_persisted::<UserVariables, GraphqlUserResponse>(
                    "query User($id: UUID!) { user(id: $id) { id email name role status createdAt tenantName } }",
                    UserVariables { id: user_id },
                    "85f7f7ba212ab47e951fcf7dbb30bb918e66b88710574a576b0088877653f3b7",
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
                        {move || translate("app.nav.users")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold">
                        {move || translate("users.detail.title")}
                    </h1>
                    <p class="mt-2 text-sm text-slate-500">
                        {move || translate("users.detail.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <LanguageToggle />
                    <Button
                        on_click=go_back
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate("users.detail.back")}
                    </Button>
                </div>
            </header>

            <div class="mb-6 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h4 class="mb-4 text-lg font-semibold">
                    {move || translate("users.access.title")}
                </h4>
                <div class="grid gap-4 md:grid-cols-3">
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label=move || translate("users.access.tenant")
                    />
                </div>
                <p class="mt-3 text-sm text-slate-500">
                    {move || translate("users.access.hint")}
                </p>
            </div>

            <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h4 class="mb-4 text-lg font-semibold">
                    {move || translate("users.detail.section")}
                </h4>
                <Suspense
                    fallback=move || view! {
                        <p class="text-sm text-slate-500">
                            {move || translate("users.detail.loading")}
                        </p>
                    }
                >
                    {move || match user_resource.get() {
                        None => view! {
                            <p class="text-sm text-slate-500">
                                {move || translate("users.detail.pending")}
                            </p>
                        }
                        .into_any(),
                        Some(Ok(response)) => {
                            if let Some(user) = response.user {
                                view! {
                                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.email")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.email}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.name")}
                                            </span>
                                            <p class="mt-1 text-sm">
                                                {user.name.unwrap_or_else(|| translate("users.placeholderDash").to_string())}
                                            </p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.role")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.role}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.status")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.status}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                Tenant
                                            </span>
                                            <p class="mt-1 text-sm">{user.tenant_name.unwrap_or_else(|| "—".to_string())}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.createdAt")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.created_at}</p>
                                        </div>
                                        <div>
                                            <span class="text-xs text-slate-400">
                                                {move || translate("users.detail.id")}
                                            </span>
                                            <p class="mt-1 text-sm">{user.id}</p>
                                        </div>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                        {move || translate("users.detail.empty")}
                                    </div>
                                }
                                .into_any()
                            }
                        }
                        Some(Err(err)) => view! {
                            <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                {match err {
                                    ApiError::Unauthorized => translate("users.graphql.unauthorized").to_string(),
                                    ApiError::Http(code) => format!("{} {}", translate("users.graphql.error"), code),
                                    ApiError::Network => translate("users.graphql.network").to_string(),
                                    ApiError::Graphql(message) => format!("{} {}", translate("users.graphql.error"), message),
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
