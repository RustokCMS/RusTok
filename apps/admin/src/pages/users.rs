use leptos::*;
use serde::{Deserialize, Serialize};

use crate::api::{request, rest_get, ApiError};
use crate::components::ui::{Button, Input};
use crate::providers::auth::use_auth;

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
    let (api_token, set_api_token) = create_signal(auth.token.get().unwrap_or_default());
    let (tenant_slug, set_tenant_slug) = create_signal(String::new());
    let (refresh_counter, set_refresh_counter) = create_signal(0u32);

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
        move || refresh_counter.get(),
        move |_| {
            let token = api_token.get().trim().to_string();
            let tenant = tenant_slug.get().trim().to_string();
            async move {
                request::<UsersVariables, GraphqlUsersResponse>(
                    "query Users($pagination: PaginationInput) { users(pagination: $pagination) { edges { node { id email name role status createdAt } } pageInfo { totalCount } } }",
                    UsersVariables {
                        pagination: PaginationInput { offset: 0, limit: 12 },
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

    view! {
        <section class="users-page">
            <header class="dashboard-header">
                <div>
                    <span class="badge">"Users"</span>
                    <h1>"Пользователи"</h1>
                    <p style="margin:8px 0 0; color:#64748b;">
                        "Демонстрация работы с REST и GraphQL API. "
                        "Введите токен администратора и tenant slug для доступа."
                    </p>
                </div>
                <div class="dashboard-actions">
                    <Button on_click=refresh class="ghost-button">
                        "Обновить"
                    </Button>
                </div>
            </header>

            <div class="panel users-panel">
                <h4>"Параметры доступа"</h4>
                <div class="form-grid">
                    <Input
                        value=api_token
                        set_value=set_api_token
                        placeholder="Bearer token"
                        label="Bearer token"
                    />
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label="Tenant slug"
                    />
                </div>
                <p class="form-hint">
                    "REST эндпоинт /api/auth/me требует Bearer-токен. "
                    "GraphQL users требует permissions users:list."
                </p>
            </div>

            <div class="users-grid">
                <div class="panel">
                    <h4>"REST: /api/auth/me"</h4>
                    <Suspense fallback=move || view! { <p>"Загрузка..."</p> }>
                        {move || match rest_resource.get() {
                            None => view! { <p>"Ожидание ответа..."</p> }.into_view(),
                            Some(Ok(user)) => view! {
                                <div class="user-card">
                                    <strong>{user.email}</strong>
                                    <span class="badge">{user.role}</span>
                                    <p style="margin:8px 0 0; color:#64748b;">
                                        {user.name.unwrap_or_else(|| "Без имени".to_string())}
                                    </p>
                                    <p class="meta-text">{user.id}</p>
                                </div>
                            }
                            .into_view(),
                            Some(Err(err)) => view! {
                                <div class="alert">
                                    {match err {
                                        ApiError::Unauthorized => "Нет доступа: проверьте токен.".to_string(),
                                        ApiError::Http(code) => format!("Ошибка REST: {}", code),
                                        ApiError::Network => "Сетевая ошибка.".to_string(),
                                        ApiError::Graphql(message) => format!("Ошибка: {}", message),
                                    }}
                                </div>
                            }
                            .into_view(),
                        }}
                    </Suspense>
                </div>

                <div class="panel">
                    <h4>"GraphQL: users"</h4>
                    <Suspense fallback=move || view! { <p>"Загрузка..."</p> }>
                        {move || match graphql_resource.get() {
                            None => view! { <p>"Ожидание ответа..."</p> }.into_view(),
                            Some(Ok(response)) => view! {
                                <p class="meta-text">
                                    "Всего пользователей: " {response.users.page_info.total_count}
                                </p>
                                <div class="table-wrap">
                                    <table class="data-table">
                                        <thead>
                                            <tr>
                                                <th>"Email"</th>
                                                <th>"Имя"</th>
                                                <th>"Роль"</th>
                                                <th>"Статус"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {response
                                                .users
                                                .edges
                                                .iter()
                                                .map(|edge| {
                                                    let user = &edge.node;
                                                    view! {
                                                        <tr>
                                                            <td>{user.email.clone()}</td>
                                                            <td>{user.name.clone().unwrap_or_else(|| "—".to_string())}</td>
                                                            <td>{user.role.clone()}</td>
                                                            <td>
                                                                <span class="status-pill">{user.status.clone()}</span>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }
                            .into_view(),
                            Some(Err(err)) => view! {
                                <div class="alert">
                                    {match err {
                                        ApiError::Unauthorized => "Нет доступа: проверьте токен.".to_string(),
                                        ApiError::Http(code) => format!("Ошибка GraphQL: {}", code),
                                        ApiError::Network => "Сетевая ошибка.".to_string(),
                                        ApiError::Graphql(message) => format!("Ошибка GraphQL: {}", message),
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
