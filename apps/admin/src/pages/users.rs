use base64::{engine::general_purpose::STANDARD, Engine};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};

use crate::api::{request_with_persisted, rest_get, ApiError};
use crate::components::ui::{Button, Input, LanguageToggle, PageHeader};
use crate::providers::auth::use_auth;
use crate::providers::locale::translate;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RestUser {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlUsersResponse {
    users: GraphqlUsersConnection,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlUsersConnection {
    edges: Vec<GraphqlUserEdge>,
    #[serde(rename = "pageInfo")]
    page_info: GraphqlPageInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlUserEdge {
    node: GraphqlUser,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GraphqlPageInfo {
    #[serde(rename = "totalCount")]
    total_count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct UsersVariables {
    pagination: PaginationInput,
    filter: Option<UsersFilterInput>,
    search: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct PaginationInput {
    first: i64,
    after: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct UsersFilterInput {
    role: Option<String>,
    status: Option<String>,
}

fn cursor_for_page(page: i64, limit: i64) -> String {
    let index = ((page - 1) * limit).saturating_sub(1).max(0);
    STANDARD.encode(index.to_string())
}

#[component]
pub fn Users() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let query = use_query_map();

    // Initialize state from URL params
    let initial_search = query.get_untracked().get("search").unwrap_or_default();
    let initial_role = query.get_untracked().get("role").unwrap_or_default();
    let initial_status = query.get_untracked().get("status").unwrap_or_default();
    let initial_page = query
        .get_untracked()
        .get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1);

    let (api_token, set_api_token) = signal(auth.token.get().unwrap_or_default());
    let (tenant_slug, set_tenant_slug) = signal(String::new());
    let (refresh_counter, set_refresh_counter) = signal(0u32);
    let (page, set_page) = signal(initial_page);
    let (limit, set_limit) = signal(12i64);
    let (limit_input, set_limit_input) = signal("12".to_string());

    // Filter signals
    let (search_query, set_search_query) = signal(initial_search);
    let (role_filter, set_role_filter) = signal(initial_role);
    let (status_filter, set_status_filter) = signal(initial_status);

    // Sync filters to URL
    Effect::new(move |_| {
        let s = search_query.get();
        let r = role_filter.get();
        let st = status_filter.get();
        let p = page.get();

        let mut params: Vec<(&str, String)> = Vec::new();
        if !s.is_empty() {
            params.push(("search", s));
        }
        if !r.is_empty() {
            params.push(("role", r));
        }
        if !st.is_empty() {
            params.push(("status", st));
        }
        if p > 1 {
            params.push(("page", p.to_string()));
        }

        let search_string = serde_urlencoded::to_string(params)
            .ok()
            .filter(|encoded| !encoded.is_empty())
            .map(|encoded| format!("?{}", encoded))
            .unwrap_or_default();

        // Update URL without reloading page
        navigate(&format!("/users{}", search_string), Default::default());
    });

    let rest_resource = Resource::new(
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

    let graphql_resource = Resource::new(
        move || (refresh_counter.get(), page.get(), limit.get()),
        move |_| {
            let token = api_token.get().trim().to_string();
            let tenant = tenant_slug.get().trim().to_string();
            let after = if page.get() > 1 {
                Some(cursor_for_page(page.get(), limit.get()))
            } else {
                None
            };
            async move {
                request_with_persisted::<UsersVariables, GraphqlUsersResponse>(
                    "query Users($pagination: PaginationInput, $filter: UsersFilter, $search: String) { users(pagination: $pagination, filter: $filter, search: $search) { edges { cursor node { id email name role status createdAt tenantName } } pageInfo { totalCount hasNextPage endCursor } } }",
                    UsersVariables {
                        pagination: PaginationInput {
                            first: limit.get(),
                            after,
                        },
                        filter: Some(UsersFilterInput {
                            role: if role_filter.get().is_empty() { None } else { Some(role_filter.get().to_uppercase()) },
                            status: if status_filter.get().is_empty() { None } else { Some(status_filter.get().to_uppercase()) },
                        }),
                        search: if search_query.get().is_empty() { None } else { Some(search_query.get()) },
                    },
                    "ff1e132e28d2e1c804d8d5ade5966307e17685b9f4b39262d70ecaa4d49abb66",
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

    Effect::new(move |_| {
        let value = limit_input.get();
        if let Ok(parsed) = value.parse::<i64>() {
            set_limit.set(parsed.max(1));
            reset_pagination();
        }
    });

    view! {
        <section class="px-10 py-8">
            <PageHeader
                title=translate("users.title")
                subtitle=translate("users.subtitle")
                eyebrow=translate("app.nav.users")
                actions=view! {
                    <LanguageToggle />
                    <Button
                        on_click=refresh
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate("users.refresh")}
                    </Button>
                }
                .into_any()
            />

            <div class="mb-6 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h4 class="mb-4 text-lg font-semibold">
                    {move || translate("users.access.title")}
                </h4>
                <div class="grid gap-4 md:grid-cols-3">
                    <Input
                        value=api_token
                        set_value=set_api_token
                        placeholder="Bearer token"
                        label=move || translate("users.access.token")
                    />
                    <Input
                        value=tenant_slug
                        set_value=set_tenant_slug
                        placeholder="demo"
                        label=move || translate("users.access.tenant")
                    />
                    <Input
                        value=limit_input
                        set_value=set_limit_input
                        placeholder="12"
                        label=move || translate("users.access.limit")
                    />
                </div>
                <p class="mt-3 text-sm text-slate-500">
                    {move || translate("users.access.hint")}
                </p>
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h4 class="mb-4 text-lg font-semibold">
                        {move || translate("users.rest.title")}
                    </h4>
                    <Suspense
                        fallback=move || view! {
                            <p class="text-sm text-slate-500">
                                {move || translate("users.rest.loading")}
                            </p>
                        }
                    >
                        {move || match rest_resource.get() {
                            None => view! {
                                <div>
                                    <p class="text-sm text-slate-500">
                                        {move || translate("users.rest.pending")}
                                    </p>
                                </div>
                            }
                            .into_any(),
                            Some(Ok(user)) => view! {
                                <div class="grid gap-2">
                                    <strong class="text-base">{user.email}</strong>
                                    <span class="inline-flex w-fit items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                                        {user.role}
                                    </span>
                                    <p class="mt-2 text-sm text-slate-500">
                                        {user.name.unwrap_or_else(|| translate("users.noName").to_string())}
                                    </p>
                                    <p class="text-xs text-slate-400">{user.id}</p>
                                </div>
                            }
                            .into_any(),
                            Some(Err(err)) => view! {
                                <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                    {match err {
                                        ApiError::Unauthorized => translate("users.rest.unauthorized").to_string(),
                                        ApiError::Http(code) => format!("{} {}", translate("users.rest.error"), code),
                                        ApiError::Network => translate("users.rest.error").to_string(),
                                        ApiError::Graphql(message) => format!("Ошибка: {}", message),
                                    }}
                                </div>
                            }
                            .into_any(),
                        }}
                    </Suspense>
                </div>

                <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h4 class="mb-4 text-lg font-semibold">
                        {move || translate("users.graphql.title")}
                    </h4>
                    <Suspense
                        fallback=move || view! {
                            <p class="text-sm text-slate-500">
                                {move || translate("users.rest.loading")}
                            </p>
                        }
                    >
                        {move || match graphql_resource.get() {
                            None => view! {
                                <div>
                                    <p class="text-sm text-slate-500">
                                        {move || translate("users.rest.pending")}
                                    </p>
                                </div>
                            }
                            .into_any(),
                            Some(Ok(response)) => {
                                let total_count = response.users.page_info.total_count;
                                let edges = response.users.edges;
                                view! {
                                <div>
                                    <p class="text-xs text-slate-400">
                                        {move || translate("users.graphql.total")} " " {total_count}
                                    </p>
                                    <div class="mb-4 grid gap-3 md:grid-cols-3">
                                        <Input
                                            value=search_query
                                            set_value=set_search_query
                                            placeholder=move || translate("users.filters.searchPlaceholder")
                                            label=move || translate("users.filters.search")
                                        />
                                        <Input
                                            value=role_filter
                                            set_value=set_role_filter
                                            placeholder=move || translate("users.filters.rolePlaceholder")
                                            label=move || translate("users.filters.role")
                                        />
                                        <Input
                                            value=status_filter
                                            set_value=set_status_filter
                                            placeholder=move || translate("users.filters.statusPlaceholder")
                                            label=move || translate("users.filters.status")
                                        />
                                    </div>
                                    <div class="overflow-x-auto">
                                        <table class="w-full border-collapse text-sm">
                                            <thead>
                                                <tr>
                                                    <th class="pb-2 text-left text-xs font-semibold text-slate-500">
                                                        {move || translate("users.graphql.email")}
                                                    </th>
                                                    <th class="pb-2 text-left text-xs font-semibold text-slate-500">
                                                        {move || translate("users.graphql.name")}
                                                    </th>
                                                    <th class="pb-2 text-left text-xs font-semibold text-slate-500">
                                                        {move || translate("users.graphql.role")}
                                                    </th>
                                                    <th class="pb-2 text-left text-xs font-semibold text-slate-500">
                                                        {move || translate("users.graphql.status")}
                                                    </th>
                                                    <th class="pb-2 text-left text-xs font-semibold text-slate-500">
                                                        {move || translate("users.graphql.createdAt")}
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {{
                                                    let query = search_query.get().to_lowercase();
                                                    let role = role_filter.get().to_lowercase();
                                                    let status = status_filter.get().to_lowercase();

                                                    edges
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
                                                            let GraphqlUser {
                                                                id,
                                                                email,
                                                                name,
                                                                role,
                                                                status,
                                                                created_at,
                                                                ..
                                                            } = edge.node.clone();
                                                            view! {
                                                                <tr>
                                                                    <td class="border-b border-slate-200 py-2">
                                                                        <A href=format!("/users/{}", id)>
                                                                            <span class="text-blue-600 hover:underline">
                                                                                {email}
                                                                            </span>
                                                                        </A>
                                                                    </td>
                                                                    <td class="border-b border-slate-200 py-2">
                                                                        {name.unwrap_or_else(|| translate("users.placeholderDash").to_string())}
                                                                    </td>
                                                                    <td class="border-b border-slate-200 py-2">{role}</td>
                                                                    <td class="border-b border-slate-200 py-2">
                                                                        <span class="inline-flex items-center rounded-full bg-slate-200 px-2.5 py-1 text-xs text-slate-600">
                                                                            {status}
                                                                        </span>
                                                                    </td>
                                                                    <td class="border-b border-slate-200 py-2">{created_at}</td>
                                                                </tr>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </tbody>
                                        </table>
                                    </div>
                                    <div class="mt-4 flex flex-wrap items-center gap-3">
                                        <Button
                                            on_click=previous_page
                                            class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                                            disabled=Signal::derive(move || page.get() <= 1)
                                        >
                                            {move || translate("users.pagination.prev")}
                                        </Button>
                                        <span class="text-xs text-slate-400">
                                            {move || translate("users.pagination.page")} " " {page.get()}
                                        </span>
                                        <Button
                                            on_click=next_page
                                            class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                                            disabled=Signal::derive(move || {
                                                let total = total_count;
                                                page.get() * limit.get() >= total
                                            })
                                        >
                                            {move || translate("users.pagination.next")}
                                        </Button>
                                    </div>
                                </div>
                                }
                                .into_any()
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
            </div>
        </section>
    }
}
