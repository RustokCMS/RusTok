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
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlUserResponse {
    user: Option<GraphqlUser>,
}

#[derive(Clone, Debug, Deserialize)]
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
    let (tenant_slug, set_tenant_slug) = create_signal(String::new());

    let user_resource = create_resource(
        move || (params.get().ok().map(|p| p.id), tenant_slug.get()),
        move |_| {
            let token = auth.token.get().unwrap_or_default();
            let tenant = tenant_slug.get().trim().to_string();
            let user_id = params.get().ok().map(|p| p.id).unwrap_or_default();

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
        <section class="users-page">
            <header class="dashboard-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "app.nav.users")}</span>
                    <h1>{move || translate(locale.locale.get(), "users.detail.title")}</h1>
                    <p style="margin:8px 0 0; color:#64748b;">
                        {move || translate(locale.locale.get(), "users.detail.subtitle")}
                    </p>
                </div>
                <div class="dashboard-actions">
                    <LanguageToggle />
                    <Button on_click=go_back class="ghost-button">
                        {move || translate(locale.locale.get(), "users.detail.back")}
                    </Button>
                </div>
            </header>

            <div class="panel users-panel">
                <h4>{move || translate(locale.locale.get(), "users.access.title")}</h4>
                <div class="form-grid">
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label=move || translate(locale.locale.get(), "users.access.tenant").to_string()
                    />
                </div>
                <p class="form-hint">
                    {move || translate(locale.locale.get(), "users.access.hint")}
                </p>
            </div>

            <div class="panel">
                <h4>{move || translate(locale.locale.get(), "users.detail.section")}</h4>
                <Suspense fallback=move || view! { <p>{move || translate(locale.locale.get(), "users.detail.loading")}</p> }>
                    {move || match user_resource.get() {
                        None => view! { <p>{move || translate(locale.locale.get(), "users.detail.pending")}</p> }.into_view(),
                        Some(Ok(response)) => {
                            if let Some(user) = response.user {
                                view! {
                                    <div class="details-grid">
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.email")}</span>
                                            <p>{user.email}</p>
                                        </div>
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.name")}</span>
                                            <p>{user.name.unwrap_or_else(|| translate(locale.locale.get(), "users.placeholderDash").to_string())}</p>
                                        </div>
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.role")}</span>
                                            <p>{user.role}</p>
                                        </div>
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.status")}</span>
                                            <p>{user.status}</p>
                                        </div>
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.createdAt")}</span>
                                            <p>{user.created_at}</p>
                                        </div>
                                        <div>
                                            <span class="meta-text">{move || translate(locale.locale.get(), "users.detail.id")}</span>
                                            <p>{user.id}</p>
                                        </div>
                                    </div>
                                }
                                .into_view()
                            } else {
                                view! {
                                    <div class="alert">
                                        {move || translate(locale.locale.get(), "users.detail.empty")}
                                    </div>
                                }
                                .into_view()
                            }
                        }
                        Some(Err(err)) => view! {
                            <div class="alert">
                                {match err {
                                    ApiError::Unauthorized => translate(locale.locale.get(), "users.graphql.unauthorized").to_string(),
                                    ApiError::Http(code) => format!("{} {}", translate(locale.locale.get(), "users.graphql.error"), code),
                                    ApiError::Network => translate(locale.locale.get(), "users.graphql.network").to_string(),
                                    ApiError::Graphql(message) => format!("{} {}", translate(locale.locale.get(), "users.graphql.error"), message),
                                }}
                            </div>
                        }
                        .into_view(),
                    }}
                </Suspense>
            </div>
        </section>
    }
}
