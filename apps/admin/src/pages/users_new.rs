// New Users List Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant, Button, ButtonVariant, Input};
use serde::{Deserialize, Serialize};

use crate::providers::auth::use_auth;

// GraphQL Query
const USERS_QUERY: &str = r#"
    query Users($search: String, $role: String, $status: String, $page: Int, $limit: Int) {
        users(search: $search, role: $role, status: $status, page: $page, limit: $limit) {
            items {
                id
                name
                email
                role
                status
                createdAt
            }
            total
            page
            totalPages
        }
    }
"#;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsersData {
    users: UsersConnection,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsersConnection {
    items: Vec<UserItem>,
    total: i64,
    page: i64,
    total_pages: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserItem {
    id: String,
    name: Option<String>,
    email: String,
    role: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[component]
pub fn UsersNew() -> impl IntoView {
    let auth = use_auth();

    let (search_query, set_search_query) = signal(String::new());
    let (role_filter, set_role_filter) = signal(String::new());
    let (status_filter, set_status_filter) = signal(String::new());
    let (page, set_page) = signal(1);
    let (limit, _) = signal(10);

    // Query users with reactive variables
    let users_result = leptos_graphql::use_query::<serde_json::Value, UsersData>(
        leptos_graphql::GRAPHQL_ENDPOINT.to_string(),
        USERS_QUERY.to_string(),
        Some(move || {
            let variables = serde_json::json!({
                "search": if search_query.get().is_empty() { None } else { Some(search_query.get()) },
                "role": if role_filter.get().is_empty() { None } else { Some(role_filter.get()) },
                "status": if status_filter.get().is_empty() { None } else { Some(status_filter.get()) },
                "page": page.get(),
                "limit": limit.get(),
            });
            variables
        }),
        auth.token.get(),
        auth.tenant_slug.get(),
    );

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
                                    set_page.set(1); // Reset to page 1 on search
                                }))
                            />
                        </div>
                        <select
                            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = leptos::ev::event_target_value(&ev);
                                set_role_filter.set(value);
                                set_page.set(1);
                            }
                        >
                            <option value="">"All Roles"</option>
                            <option value="admin">"Admin"</option>
                            <option value="editor">"Editor"</option>
                            <option value="user">"User"</option>
                        </select>
                        <select
                            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = leptos::ev::event_target_value(&ev);
                                set_status_filter.set(value);
                                set_page.set(1);
                            }
                        >
                            <option value="">"All Status"</option>
                            <option value="active">"Active"</option>
                            <option value="inactive">"Inactive"</option>
                        </select>
                    </div>
                </CardContent>
            </Card>

            // Users Table
            <Card>
                {move || {
                    if users_result.loading.get() {
                        view! {
                            <div class="flex items-center justify-center py-8">
                                <div class="text-gray-500">"Loading users..."</div>
                            </div>
                        }.into_any()
                    } else if let Some(error) = users_result.error.get() {
                        view! {
                            <div class="flex items-center justify-center py-8">
                                <div class="text-red-500">{format!("Error: {}", error)}</div>
                            </div>
                        }.into_any()
                    } else if let Some(data) = users_result.data.get() {
                        let users = data.users.clone();
                        view! {
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
                                        {users.items.into_iter().map(|user| {
                                            view! { <UserRowGraphQL user=user /> }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>

                            // Pagination
                            <div class="border-t border-gray-200 bg-gray-50 px-6 py-4">
                                <div class="flex items-center justify-between">
                                    <div class="text-sm text-gray-700">
                                        "Showing "
                                        <span class="font-medium">{((users.page - 1) * limit.get() + 1).to_string()}</span>
                                        " to "
                                        <span class="font-medium">{(users.page * limit.get()).min(users.total).to_string()}</span>
                                        " of "
                                        <span class="font-medium">{users.total.to_string()}</span>
                                        " results"
                                    </div>
                                    <div class="flex gap-2">
                                        <Button
                                            variant=ButtonVariant::Outline
                                            disabled=users.page <= 1
                                            on:click=move |_| {
                                                set_page.update(|p| *p -= 1);
                                            }
                                        >
                                            "Previous"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Outline
                                            disabled=users.page >= users.total_pages
                                            on:click=move |_| {
                                                set_page.update(|p| *p += 1);
                                            }
                                        >
                                            "Next"
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center justify-center py-8">
                                <div class="text-gray-500">"No users found"</div>
                            </div>
                        }.into_any()
                    }
                }}
            </Card>
        </div>
    }
}

// ============================================================================
// UserRowGraphQL Component (GraphQL-based)
// ============================================================================

#[component]
fn UserRowGraphQL(user: UserItem) -> impl IntoView {
    let role_badge = match user.role.as_str() {
        "admin" => BadgeVariant::Primary,
        "editor" => BadgeVariant::Warning,
        _ => BadgeVariant::Default,
    };

    let status_badge = match user.status.as_str() {
        "active" => BadgeVariant::Success,
        "inactive" => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
    };

    let display_name = user.name.as_deref().unwrap_or("Unknown");
    let initial = display_name.chars().next().unwrap_or('U');

    // Format the date
    let formatted_date = match chrono::DateTime::parse_from_rfc3339(&user.created_at) {
        Ok(dt) => dt.format("%Y-%m-%d").to_string(),
        Err(_) => user.created_at.clone(),
    };

    view! {
        <tr class="hover:bg-gray-50">
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center">
                    <div class="h-10 w-10 flex-shrink-0">
                        <div class="h-10 w-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                            <span class="text-white text-sm font-semibold">
                                {initial.to_string()}
                            </span>
                        </div>
                    </div>
                    <div class="ml-4">
                        <div class="text-sm font-medium text-gray-900">
                            {display_name}
                        </div>
                        <div class="text-sm text-gray-500">
                            {user.email}
                        </div>
                    </div>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=role_badge>
                    {user.role}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=status_badge>
                    {user.status}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatted_date}
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
