// New Users List Page with GraphQL integration
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant, Button, ButtonVariant, Input};
use serde::{Deserialize, Serialize};
use leptos_auth::use_token;
use leptos_auth::use_tenant;
use leptos_graphql::use_query;

use crate::providers::locale::translate;

// ============================================================================
// GraphQL Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total: i64,
    pub page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEdge {
    pub node: UserNode,
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserNode {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnection {
    pub edges: Vec<UserEdge>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    pub users: UserConnection,
}

// ============================================================================
// UsersList Component
// ============================================================================

#[component]
pub fn UsersNew() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();

    let (search_query, set_search_query) = signal(String::new());
    let (role_filter, set_role_filter) = signal(String::from("all"));
    let (status_filter, set_status_filter) = signal(String::from("all"));
    let (page, set_page) = signal(0i64);

    // Users Query
    let users_query = r#"
        query Users($search: String, $role: String, $status: String, $page: Int, $limit: Int) {
            users(search: $search, role: $role, status: $status, page: $page, limit: $limit) {
                edges {
                    node {
                        id
                        name
                        email
                        role
                        status
                        createdAt
                    }
                    cursor
                }
                pageInfo {
                    total
                    page
                    totalPages
                }
            }
        }
    "#;

    let users_result = use_query::<serde_json::Value, UsersResponse>(
        "/api/graphql".to_string(),
        users_query.to_string(),
        Signal::derive(move || {
            let variables = serde_json::json!({
                "search": if search_query.get().is_empty() { None } else { Some(search_query.get()) },
                "role": if role_filter.get() == "all" { None } else { Some(role_filter.get()) },
                "status": if status_filter.get() == "all" { None } else { Some(status_filter.get()) },
                "page": page.get(),
                "limit": 10,
            });
            Some(variables)
        }),
        Signal::derive(move || token.get()),
        Signal::derive(move || tenant.get()),
    );

    // Refetch when filters or page changes
    Effect::new(move |_| {
        let _ = search_query.get();
        let _ = role_filter.get();
        let _ = status_filter.get();
        set_page.set(0); // Reset to first page when filters change
    });

    view! {
        <div class="space-y-6">
            // Page Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Users"</h1>
                    <p class="mt-1 text-sm text-gray-600">
                        "Manage your platform users"
                    </p>
                </div>
                <Button variant=ButtonVariant::Primary>
                    "➕ Add User"
                </Button>
            </div>

            // Filters & Search
            <Card>
                <CardContent class="p-4">
                    <div class="flex items-center gap-4">
                        <div class="flex-1">
                            <Input
                                placeholder=Some("Search users...")
                                value=Some(search_query.read_only())
                                on_input=Some(Box::new(move |ev| {
                                    let value = leptos::ev::event_target_value(&ev);
                                    set_search_query.set(value);
                                }))
                            />
                        </div>
                        <select 
                            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = leptos::ev::event_target_value(&ev);
                                set_role_filter.set(value);
                            }
                        >
                            <option value="all">"All Roles"</option>
                            <option value="SUPER_ADMIN">"Super Admin"</option>
                            <option value="ADMIN">"Admin"</option>
                            <option value="MANAGER">"Manager"</option>
                            <option value="CUSTOMER">"Customer"</option>
                        </select>
                        <select 
                            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = leptos::ev::event_target_value(&ev);
                                set_status_filter.set(value);
                            }
                        >
                            <option value="all">"All Status"</option>
                            <option value="ACTIVE">"Active"</option>
                            <option value="INACTIVE">"Inactive"</option>
                            <option value="BANNED">"Banned"</option>
                        </select>
                    </div>
                </CardContent>
            </Card>

            // Users Table
            <Show
                when=move || users_result.loading.get()
                fallback=move || {
                    users_result.data.get().map(|data| {
                        let users = data.users.clone();
                        view! {
                            <Card>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                                    "User"
                                                </th>
                                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                                    "Role"
                                                </th>
                                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                                    "Status"
                                                </th>
                                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                                    "Created"
                                                </th>
                                                <th class="px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">
                                                    "Actions"
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-200 bg-white">
                                            {users.edges.iter().map(|edge| {
                                                view! { <UserRow user=edge.node.clone() /> }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>

                                // Pagination
                                <div class="border-t border-gray-200 bg-gray-50 px-6 py-4">
                                    <div class="flex items-center justify-between">
                                        <div class="text-sm text-gray-700">
                                            "Showing "
                                            <span class="font-medium">
                                                {(users.page_info.page * 10 + 1).to_string()}
                                            </span>
                                            " to "
                                            <span class="font-medium">
                                                {std::cmp::min((users.page_info.page + 1) * 10, users.page_info.total).to_string()}
                                            </span>
                                            " of "
                                            <span class="font-medium">{users.page_info.total.to_string()}</span>
                                            " results"
                                        </div>
                                        <div class="flex gap-2">
                                            <Button
                                                variant=ButtonVariant::Outline
                                                disabled=users.page_info.page == 0
                                                on:click=move |_| {
                                                    set_page.update(|p| *p -= 1);
                                                    users_result.refetch();
                                                }
                                            >
                                                "Previous"
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Outline
                                                disabled=users.page_info.page >= users.page_info.total_pages - 1
                                                on:click=move |_| {
                                                    set_page.update(|p| *p += 1);
                                                    users_result.refetch();
                                                }
                                            >
                                                "Next"
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            </Card>
                        }
                    })
                }
            >
                <Card>
                    <div class="p-6">
                        <UsersTableSkeleton />
                    </div>
                </Card>
            </Show>
        </div>
    }
}

// ============================================================================
// UserRow Component
// ============================================================================

#[component]
fn UserRow(user: UserNode) -> impl IntoView {
    let role_badge = match user.role.as_str() {
        "SUPER_ADMIN" => BadgeVariant::Primary,
        "ADMIN" => BadgeVariant::Primary,
        "MANAGER" => BadgeVariant::Warning,
        "CUSTOMER" => BadgeVariant::Default,
        _ => BadgeVariant::Default,
    };

    let status_badge = match user.status.as_str() {
        "ACTIVE" => BadgeVariant::Success,
        "INACTIVE" => BadgeVariant::Danger,
        "BANNED" => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
    };

    let initials = user.name
        .as_ref()
        .and_then(|n| n.chars().next())
        .unwrap_or_else(|| user.email.chars().next().unwrap_or('U'))
        .to_uppercase();

    view! {
        <tr class="hover:bg-gray-50">
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center">
                    <div class="h-10 w-10 flex-shrink-0">
                        <div class="h-10 w-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                            <span class="text-white text-sm font-semibold">
                                {initials}
                            </span>
                        </div>
                    </div>
                    <div class="ml-4">
                        <div class="text-sm font-medium text-gray-900">
                            {user.name.clone().unwrap_or_else(|| user.email.clone())}
                        </div>
                        <div class="text-sm text-gray-500">
                            {user.email}
                        </div>
                    </div>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=role_badge>
                    {format_role(&user.role)}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=status_badge>
                    {format_status(&user.status)}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {format_date(&user.created_at)}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div class="flex items-center justify-end gap-2">
                    <A
                        href=format!("/users/{}", user.id)
                        class="text-blue-600 hover:text-blue-900"
                    >
                        "View"
                    </A>
                    <button class="text-gray-600 hover:text-gray-900">
                        "Edit"
                    </button>
                    <button class="text-red-600 hover:text-red-900">
                        "Delete"
                    </button>
                </div>
            </td>
        </tr>
    }
}

// ============================================================================
// Skeleton Components
// ============================================================================

#[component]
fn UsersTableSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="animate-pulse rounded h-12 bg-gray-200"></div>
            <div class="animate-pulse rounded h-12 bg-gray-200"></div>
            <div class="animate-pulse rounded h-12 bg-gray-200"></div>
            <div class="animate-pulse rounded h-12 bg-gray-200"></div>
        </div>
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_role(role: &str) -> String {
    match role {
        "SUPER_ADMIN" => "Super Admin".to_string(),
        "ADMIN" => "Admin".to_string(),
        "MANAGER" => "Manager".to_string(),
        "CUSTOMER" => "Customer".to_string(),
        _ => role.to_string(),
    }
}

fn format_status(status: &str) -> String {
    match status {
        "ACTIVE" => "Active".to_string(),
        "INACTIVE" => "Inactive".to_string(),
        "BANNED" => "Banned".to_string(),
        _ => status.to_string(),
    }
}

fn format_date(timestamp: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d").to_string()
    } else {
        "Unknown".to_string()
    }
}
