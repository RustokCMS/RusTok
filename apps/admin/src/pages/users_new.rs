// New Users List Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_graphql::use_query;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant, Button, ButtonVariant, Input};

use crate::providers::auth::use_auth;
use crate::api::{
    API_URL, UsersResponse, UserEdge, USERS_QUERY,
    UsersFilter, PaginationInput,
};

const PAGE_SIZE: i64 = 10;

#[component]
pub fn UsersNew() -> impl IntoView {
    let auth = use_auth();

    // Get token and tenant from auth context
    let token = move || auth.token.get();
    let tenant_slug = move || auth.tenant_slug.get();

    // Search and filter signals
    let (search_query, set_search_query) = signal(String::new());
    let (role_filter, set_role_filter) = signal(String::new());
    let (status_filter, set_status_filter) = signal(String::new());
    let (after_cursor, set_after_cursor) = signal(None::<String>);
    let (before_cursor, set_before_cursor) = signal(None::<String>);
    let (page_direction, set_page_direction) = signal("next".to_string());

    // Build filter from selection
    let filter = move || {
        let mut f = UsersFilter::default();
        if !role_filter.get().is_empty() {
            f.role = Some(role_filter.get().to_lowercase());
        }
        if !status_filter.get().is_empty() {
            f.status = Some(status_filter.get().to_lowercase());
        }
        f
    };

    // Build pagination from cursors
    let pagination = move || {
        let mut p = PaginationInput {
            first: Some(PAGE_SIZE),
            after: None,
        };

        if page_direction.get() == "next" && after_cursor.get().is_some() {
            p.after = after_cursor.get();
        } else if page_direction.get() == "prev" && before_cursor.get().is_some() {
            // For simplicity, we're using first/after - in production you'd use last/before
            p.after = before_cursor.get();
        }
        p
    };

    // Fetch users using GraphQL
    let users_query = use_query(
        API_URL.to_string(),
        USERS_QUERY.to_string(),
        Some(move || serde_json::json!({
            "pagination": pagination(),
            "filter": filter(),
            "search": search_query.get().trim().to_string(),
        })),
        token(),
        tenant_slug(),
    );

    // Get current page users
    let users_data = move || {
        users_query.data.get().map(|r: UsersResponse| r.users)
    };

    // Pagination handlers
    let handle_next_page = move |_| {
        if let Some(ref data) = users_data() {
            if data.page_info.has_next_page {
                if let Some(cursor) = data.page_info.end_cursor {
                    set_after_cursor.set(Some(cursor));
                    set_page_direction.set("next".to_string());
                }
            }
        }
    };

    let handle_prev_page = move |_| {
        if let Some(ref data) = users_data() {
            if data.page_info.has_previous_page {
                if let Some(cursor) = data.page_info.start_cursor {
                    set_before_cursor.set(Some(cursor));
                    set_page_direction.set("prev".to_string());
                }
            }
        }
    };

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
                            <option value="">"All Roles"</option>
                            <option value="admin">"Admin"</option>
                            <option value="manager">"Manager"</option>
                            <option value="customer">"Customer"</option>
                        </select>
                        <select
                            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500"
                            on:change=move |ev| {
                                let value = leptos::ev::event_target_value(&ev);
                                set_status_filter.set(value);
                            }
                        >
                            <option value="">"All Status"</option>
                            <option value="active">"Active"</option>
                            <option value="inactive">"Inactive"</option>
                            <option value="banned">"Banned"</option>
                        </select>
                    </div>
                </CardContent>
            </Card>

            // Users Table
            <Card>
                {move || {
                    if users_query.loading.get() {
                        view! { <UsersTableSkeleton /> }.into_any()
                    } else if let Some(ref data) = users_data() {
                        let edges = data.edges.clone();
                        let page_info = data.page_info.clone();

                        view! {
                            <>
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
                                            {edges.into_iter().map(|edge| {
                                                view! { <UserRow edge=edge /> }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>

                                // Pagination
                                <div class="border-t border-gray-200 bg-gray-50 px-6 py-4">
                                    <div class="flex items-center justify-between">
                                        <div class="text-sm text-gray-700">
                                            "Showing "
                                            <span class="font-medium">{
                                                if page_info.total_count == 0 {
                                                    "0".to_string()
                                                } else {
                                                    format!("{}", edges.len())
                                                }
                                            }</span>
                                            " of "
                                            <span class="font-medium">{page_info.total_count}</span>
                                            " results"
                                        </div>
                                        <div class="flex gap-2">
                                            <Button
                                                variant=ButtonVariant::Outline
                                                disabled=!page_info.has_previous_page
                                                on_click=handle_prev_page
                                            >
                                                "Previous"
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Outline
                                                disabled=!page_info.has_next_page
                                                on_click=handle_next_page
                                            >
                                                "Next"
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            </>
                        }.into_any()
                    } else if let Some(error) = users_query.error.get() {
                        view! {
                            <div class="p-8 text-center">
                                <div class="text-red-600">
                                    "Error loading users: " {error.to_string()}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="p-8 text-center text-gray-500">
                                "No users found"
                            </div>
                        }.into_any()
                    }
                }}
            </Card>
        </div>
    }
}

// ============================================================================
// UsersTableSkeleton Component
// ============================================================================

#[component]
fn UsersTableSkeleton() -> impl IntoView {
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
                    {(0..5).map(|_| {
                        view! {
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="flex items-center">
                                        <div class="h-10 w-10 rounded-full bg-gray-200 animate-pulse"></div>
                                        <div class="ml-4 space-y-2">
                                            <div class="h-4 w-32 bg-gray-200 rounded animate-pulse"></div>
                                            <div class="h-3 w-24 bg-gray-200 rounded animate-pulse"></div>
                                        </div>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="h-6 w-16 bg-gray-200 rounded animate-pulse"></div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="h-6 w-16 bg-gray-200 rounded animate-pulse"></div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="h-4 w-24 bg-gray-200 rounded animate-pulse"></div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-right">
                                    <div class="flex items-center justify-end gap-2">
                                        <div class="h-4 w-12 bg-gray-200 rounded animate-pulse"></div>
                                        <div class="h-4 w-12 bg-gray-200 rounded animate-pulse"></div>
                                        <div class="h-4 w-12 bg-gray-200 rounded animate-pulse"></div>
                                    </div>
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>

        // Pagination skeleton
        <div class="border-t border-gray-200 bg-gray-50 px-6 py-4">
            <div class="flex items-center justify-between">
                <div class="h-4 w-32 bg-gray-200 rounded animate-pulse"></div>
                <div class="flex gap-2">
                    <div class="h-9 w-24 bg-gray-200 rounded animate-pulse"></div>
                    <div class="h-9 w-24 bg-gray-200 rounded animate-pulse"></div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// UserRow Component
// ============================================================================

#[component]
fn UserRow(edge: UserEdge) -> impl IntoView {
    let user = edge.node;

    let role_badge = match user.role.as_str() {
        "admin" | "super_admin" => BadgeVariant::Primary,
        "manager" => BadgeVariant::Warning,
        _ => BadgeVariant::Default,
    };

    let status_badge = match user.status.as_str() {
        "active" => BadgeVariant::Success,
        "inactive" => BadgeVariant::Warning,
        "banned" => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
    };

    let display_name = user.name.clone().unwrap_or_else(|| user.email.clone());
    let initial = display_name.chars().next().unwrap_or('U');

    // Format date
    let created_date = user.created_at.split('T').next().unwrap_or(&user.created_at);

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
                {created_date.to_string()}
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
