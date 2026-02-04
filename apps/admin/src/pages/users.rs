use leptos::*;
use leptos_router::Link;
use serde::{Deserialize, Serialize};

use crate::api::{request, rest_get, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::auth::use_auth;
use crate::providers::locale::{translate, use_locale};

#[derive(Clone, Debug, Deserialize)]
struct RestUser {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlUsersResponse {
    users: GraphqlUsersConnection,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlUsersConnection {
    edges: Vec<GraphqlUserEdge>,
    #[serde(rename = "pageInfo")]
    page_info: GraphqlPageInfo,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlUserEdge {
    node: GraphqlUser,
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

#[derive(Clone, Debug, Deserialize)]
struct GraphqlPageInfo {
    #[serde(rename = "totalCount")]
    total_count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct UsersVariables {
    pagination: PaginationInput,
}

#[derive(Clone, Debug, Serialize)]
struct PaginationInput {
    offset: i64,
    limit: i64,
}

#[component]
pub fn Users() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();
    let (api_token, set_api_token) = create_signal(auth.token.get().unwrap_or_default());
    let (tenant_slug, set_tenant_slug) = create_signal(String::new());
    let (refresh_counter, set_refresh_counter) = create_signal(0u32);
    let (page, set_page) = create_signal(1i64);
    let (limit, set_limit) = create_signal(12i64);
    let (limit_input, set_limit_input) = create_signal("12".to_string());
    let (search_query, set_search_query) = create_signal(String::new());
    let (role_filter, set_role_filter) = create_signal(String::new());
    let (status_filter, set_status_filter) = create_signal(String::new());

    let rest_resource = create_resource(
        move || refresh_counter.get(),
        move |_| {
            let token = api_token.get().trim().to_string();
            let tenant = tenant_slug.get().trim().to_string();
            async move {
                rest_get::<RestUser>(
                    "/api/auth/me",
                    if token.is_empty() { None } else { Some(token) },
                    if tenant.is_empty() {
                        None
                    } else {
                        Some(tenant)
                    },
                )
                .await
            }
        },
    );

    let graphql_resource = create_resource(
        move || (refresh_counter.get(), page.get(), limit.get()),
        move |_| {
            let token = api_token.get().trim().to_string();
            let tenant = tenant_slug.get().trim().to_string();
            let offset = (page.get().saturating_sub(1)) * limit.get();
            async move {
                request::<UsersVariables, GraphqlUsersResponse>(
                    "query Users($pagination: PaginationInput) { users(pagination: $pagination) { edges { node { id email name role status createdAt } } pageInfo { totalCount } } }",
                    UsersVariables {
                        pagination: PaginationInput {
                            offset,
                            limit: limit.get(),
                        },
                    },
                    if token.is_empty() { None } else { Some(token) },
                    if tenant.is_empty() {
                        None
                    } else {
                        Some(tenant)
                    },
                )
                .await
            }
        },
    );

    let refresh = move |_| set_refresh_counter.update(|value| *value += 1);
    let next_page = move |_| set_page.update(|value| *value += 1);
    let previous_page = move |_| set_page.update(|value| *value = (*value - 1).max(1));
    let reset_pagination = move || set_page.set(1);

    view! {
        <section class="users-page">
            <header class="dashboard-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "app.users")}</span>
                    <h1>{move || translate(locale.locale.get(), "users.title")}</h1>
                    <p style="margin:8px 0 0; color:#64748b;">
                        {move || translate(locale.locale.get(), "users.subtitle")}
                    </p>
                </div>
                <div class="dashboard-actions">
                    <LanguageToggle />
                    <Button on_click=refresh class="ghost-button">
                        {move || translate(locale.locale.get(), "users.refresh")}
                    </Button>
                </div>
            </header>

            <div class="panel users-panel">
                <h4>{move || translate(locale.locale.get(), "users.access.title")}</h4>
                <div class="form-grid">
                    <Input
                        value=api_token
                        set_value=set_api_token
                        placeholder="Bearer token"
                        label=move || translate(locale.locale.get(), "users.access.token").to_string()
                    />
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label=move || translate(locale.locale.get(), "users.access.tenant").to_string()
                    />
                    <Input
                        value=limit_input
                        set_value=move |value| {
                            set_limit_input.set(value.clone());
                            if let Ok(parsed) = value.parse::<i64>() {
                                set_limit.set(parsed.max(1));
                                reset_pagination();
                            }
                        }
                        placeholder="12"
                        label=move || translate(locale.locale.get(), "users.access.limit").to_string()
                    />
                </div>
                <p class="form-hint">
                    {move || translate(locale.locale.get(), "users.access.hint")}
                </p>
            </div>

            <div class="users-grid">
                <div class="panel">
                    <h4>{move || translate(locale.locale.get(), "users.rest.title")}</h4>
                    <Suspense fallback=move || view! { <p>{move || translate(locale.locale.get(), "users.rest.loading")}</p> }>
                        {move || match rest_resource.get() {
                            None => view! { <p>{move || translate(locale.locale.get(), "users.rest.pending")}</p> }.into_view(),
                            Some(Ok(user)) => view! {
                                <div class="user-card">
                                    <strong>{user.email}</strong>
                                    <span class="badge">{user.role}</span>
                                    <p style="margin:8px 0 0; color:#64748b;">
                                        {user.name.unwrap_or_else(|| translate(locale.locale.get(), "users.noName").to_string())}
                                    </p>
                                    <p class="meta-text">{user.id}</p>
                                </div>
                            }
                            .into_view(),
                            Some(Err(err)) => view! {
                                <div class="alert">
                                    {match err {
                                        ApiError::Unauthorized => translate(locale.locale.get(), "users.rest.unauthorized").to_string(),
                                        ApiError::Http(code) => format!("{} {}", translate(locale.locale.get(), "users.rest.error"), code),
                                        ApiError::Network => translate(locale.locale.get(), "users.rest.error").to_string(),
                                        ApiError::Graphql(message) => format!("Ошибка: {}", message),
                                    }}
                                </div>
                            }
                            .into_view(),
                        }}
                    </Suspense>
                </div>

                <div class="panel">
                    <h4>{move || translate(locale.locale.get(), "users.graphql.title")}</h4>
                    <Suspense fallback=move || view! { <p>{move || translate(locale.locale.get(), "users.rest.loading")}</p> }>
                        {move || match graphql_resource.get() {
                            None => view! { <p>{move || translate(locale.locale.get(), "users.rest.pending")}</p> }.into_view(),
                            Some(Ok(response)) => view! {
                                <p class="meta-text">
                                    {move || translate(locale.locale.get(), "users.graphql.total")} " " {response.users.page_info.total_count}
                                </p>
                                <div class="table-filters">
                                    <Input
                                        value=search_query
                                        set_value=set_search_query
                                        placeholder=move || translate(locale.locale.get(), "users.filters.searchPlaceholder").to_string()
                                        label=move || translate(locale.locale.get(), "users.filters.search").to_string()
                                    />
                                    <Input
                                        value=role_filter
                                        set_value=set_role_filter
                                        placeholder=move || translate(locale.locale.get(), "users.filters.rolePlaceholder").to_string()
                                        label=move || translate(locale.locale.get(), "users.filters.role").to_string()
                                    />
                                    <Input
                                        value=status_filter
                                        set_value=set_status_filter
                                        placeholder=move || translate(locale.locale.get(), "users.filters.statusPlaceholder").to_string()
                                        label=move || translate(locale.locale.get(), "users.filters.status").to_string()
                                    />
                                </div>
                                <div class="table-wrap">
                                    <table class="data-table">
                                        <thead>
                                            <tr>
                                                <th>{move || translate(locale.locale.get(), "users.graphql.email")}</th>
                                                <th>{move || translate(locale.locale.get(), "users.graphql.name")}</th>
                                                <th>{move || translate(locale.locale.get(), "users.graphql.role")}</th>
                                                <th>{move || translate(locale.locale.get(), "users.graphql.status")}</th>
                                                <th>{move || translate(locale.locale.get(), "users.graphql.createdAt")}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {({
                                                let query = search_query.get().to_lowercase();
                                                let role = role_filter.get().to_lowercase();
                                                let status = status_filter.get().to_lowercase();

                                                response
                                                    .users
                                                    .edges
                                                    .iter()
                                                    .filter(|edge| {
                                                        let user = &edge.node;
                                                        let name = user.name.clone().unwrap_or_default().to_lowercase();
                                                        let email = user.email.to_lowercase();
                                                        let role_value = user.role.to_lowercase();
                                                        let status_value = user.status.to_lowercase();

                                                        let matches_query = query.is_empty()
                                                            || email.contains(&query)
                                                            || name.contains(&query);
                                                        let matches_role = role.is_empty()
                                                            || role_value.contains(&role);
                                                        let matches_status = status.is_empty()
                                                            || status_value.contains(&status);

                                                        matches_query && matches_role && matches_status
                                                    })
                                                    .map(|edge| {
                                                        let user = &edge.node;
                                                        view! {
                                                            <tr>
                                                                <td>
                                                                    <Link href=format!("/users/{}", user.id.clone())>
                                                                        {user.email.clone()}
                                                                    </Link>
                                                                </td>
                                                                <td>{user.name.clone().unwrap_or_else(|| translate(locale.locale.get(), "users.placeholderDash").to_string())}</td>
                                                                <td>{user.role.clone()}</td>
                                                                <td>
                                                                    <span class="status-pill">{user.status.clone()}</span>
                                                                </td>
                                                                <td>{user.created_at.clone()}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                            })}
                                        </tbody>
                                    </table>
                                </div>
                                <div class="table-actions">
                                    <Button
                                        on_click=previous_page
                                        class="ghost-button"
                                        disabled=move || page.get() <= 1
                                    >
                                        {move || translate(locale.locale.get(), "users.pagination.prev")}
                                    </Button>
                                    <span class="meta-text">
                                        {move || translate(locale.locale.get(), "users.pagination.page")} " " {page.get()}
                                    </span>
                                    <Button
                                        on_click=next_page
                                        class="ghost-button"
                                        disabled=move || {
                                            let total = response.users.page_info.total_count;
                                            page.get() * limit.get() >= total
                                        }
                                    >
                                        {move || translate(locale.locale.get(), "users.pagination.next")}
                                    </Button>
                                </div>
                            }
                            .into_view(),
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
            </div>
        </section>
    }
}
