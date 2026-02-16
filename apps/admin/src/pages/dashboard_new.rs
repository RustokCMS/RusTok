// New Dashboard Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_graphql::use_query;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant};
use chrono_humanize::HumanTime;
use chrono::DateTime;

use crate::providers::auth::use_auth;
use crate::providers::locale::translate;
use crate::api::{
    API_URL, DashboardStatsResponse, RecentActivityResponse,
    DASHBOARD_STATS_QUERY, RECENT_ACTIVITY_QUERY,
};

#[component]
pub fn DashboardNew() -> impl IntoView {
    let auth = use_auth();

    // Get token and tenant from auth context
    let token = move || auth.token.get();
    let tenant_slug = move || auth.tenant_slug.get();

    // Fetch dashboard stats using GraphQL
    let stats_query = use_query(
        API_URL.to_string(),
        DASHBOARD_STATS_QUERY.to_string(),
        None::<serde_json::Value>,
        token(),
        tenant_slug(),
    );

    // Fetch recent activity using GraphQL (limit: 10)
    let activity_query = use_query(
        API_URL.to_string(),
        RECENT_ACTIVITY_QUERY.to_string(),
        Some(serde_json::json!({ "limit": 10 })),
        token(),
        tenant_slug(),
    );

    view! {
        <div class="space-y-6">
            // Welcome Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900">
                    {move || {
                        let name = auth.user
                            .get()
                            .and_then(|u| u.name.clone())
                            .unwrap_or_else(|| "User".to_string());
                        format!("Welcome back, {}!", name)
                    }}
                </h1>
                <p class="mt-2 text-gray-600">
                    "Here's what's happening with your platform today."
                </p>
            </div>

            // Stats Grid
            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
                {move || {
                    if stats_query.loading.get() {
                        view! {
                            <>
                                <StatCardSkeleton />
                                <StatCardSkeleton />
                                <StatCardSkeleton />
                                <StatCardSkeleton />
                            </>
                        }.into_any()
                    } else if let Some(data) = stats_query.data.get() {
                        let stats = data.dashboard_stats;
                        view! {
                            <>
                                <StatCard
                                    title="Total Users"
                                    value=format_number(stats.total_users)
                                    change=format_change(stats.users_change)
                                    change_positive=stats.users_change >= 0.0
                                    icon="👥"
                                />
                                <StatCard
                                    title="Total Posts"
                                    value=format_number(stats.total_posts)
                                    change=format_change(stats.posts_change)
                                    change_positive=stats.posts_change >= 0.0
                                    icon="📝"
                                />
                                <StatCard
                                    title="Total Orders"
                                    value=format_number(stats.total_orders)
                                    change=format_change(stats.orders_change)
                                    change_positive=stats.orders_change >= 0.0
                                    icon="📦"
                                />
                                <StatCard
                                    title="Revenue"
                                    value=format!("${}", format_number(stats.total_revenue))
                                    change=format_change(stats.revenue_change)
                                    change_positive=stats.revenue_change >= 0.0
                                    icon="💰"
                                />
                            </>
                        }.into_any()
                    } else if let Some(error) = stats_query.error.get() {
                        view! {
                            <div class="col-span-4 p-4 text-red-600 bg-red-50 rounded-lg">
                                "Error loading stats: " {error.to_string()}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div class="col-span-4">"No data available"</div> }.into_any()
                    }
                }}
            </div>

            // Main Content Grid
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                // Recent Activity (2 columns)
                <div class="lg:col-span-2">
                    <Card>
                        <CardHeader class="border-b border-gray-200">
                            <h3 class="text-lg font-semibold text-gray-900">
                                "Recent Activity"
                            </h3>
                        </CardHeader>
                        <CardContent>
                            <div class="space-y-4">
                                {move || {
                                    if activity_query.loading.get() {
                                        view! {
                                            <>
                                                <ActivityItemSkeleton />
                                                <ActivityItemSkeleton />
                                                <ActivityItemSkeleton />
                                                <ActivityItemSkeleton />
                                            </>
                                        }.into_any()
                                    } else if let Some(data) = activity_query.data.get() {
                                        let activities = data.recent_activity;
                                        if activities.is_empty() {
                                            view! {
                                                <div class="text-gray-500 text-center py-8">
                                                    "No recent activity"
                                                </div>
                                            }.into_any()
                                        } else {
                                            activities.into_iter().map(|activity| {
                                                view! { <ActivityItem activity=activity /> }
                                            }).collect_view().into_any()
                                        }
                                    } else if let Some(error) = activity_query.error.get() {
                                        view! {
                                            <div class="p-4 text-red-600 bg-red-50 rounded-lg">
                                                "Error loading activity: " {error.to_string()}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div>"No activity available"</div> }.into_any()
                                    }
                                }}
                            </div>
                        </CardContent>
                    </Card>
                </div>

                // Quick Actions (1 column)
                <div>
                    <Card>
                        <CardHeader class="border-b border-gray-200">
                            <h3 class="text-lg font-semibold text-gray-900">
                                "Quick Actions"
                            </h3>
                        </CardHeader>
                        <CardContent>
                            <div class="space-y-3">
                                <QuickActionLink href="/users" icon="👥">
                                    "Manage Users"
                                </QuickActionLink>
                                <QuickActionLink href="/posts" icon="📝">
                                    "Create Post"
                                </QuickActionLink>
                                <QuickActionLink href="/products" icon="🛍️">
                                    "Add Product"
                                </QuickActionLink>
                                <QuickActionLink href="/settings" icon="⚙️">
                                    "System Settings"
                                </QuickActionLink>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// StatCard Component
// ============================================================================

#[component]
fn StatCard(
    title: &'static str,
    value: String,
    change: String,
    change_positive: bool,
    icon: &'static str,
) -> impl IntoView {
    let change_color = if change_positive {
        "text-green-600"
    } else {
        "text-red-600"
    };

    view! {
        <Card class="hover:shadow-lg transition-shadow">
            <CardContent class="p-6">
                <div class="flex items-center justify-between">
                    <div class="flex-1">
                        <p class="text-sm font-medium text-gray-600">
                            {title}
                        </p>
                        <p class="mt-2 text-3xl font-bold text-gray-900">
                            {value}
                        </p>
                        <p class=format!("mt-2 text-sm font-medium {}", change_color)>
                            {change}
                            " from last month"
                        </p>
                    </div>
                    <div class="ml-4 text-4xl">
                        {icon}
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}

/// Skeleton loader for stat card
#[component]
fn StatCardSkeleton() -> impl IntoView {
    view! {
        <Card class="hover:shadow-lg transition-shadow">
            <CardContent class="p-6">
                <div class="flex items-center justify-between">
                    <div class="flex-1">
                        <div class="h-4 w-24 bg-gray-200 rounded animate-pulse"></div>
                        <div class="mt-2 h-8 w-20 bg-gray-200 rounded animate-pulse"></div>
                        <div class="mt-2 h-4 w-32 bg-gray-200 rounded animate-pulse"></div>
                    </div>
                    <div class="ml-4 h-12 w-12 bg-gray-200 rounded-full animate-pulse"></div>
                </div>
            </CardContent>
        </Card>
    }
}

// ============================================================================
// ActivityItem Component
// ============================================================================

#[component]
fn ActivityItem(activity: crate::api::ActivityItem) -> impl IntoView {
    let icon = get_activity_icon(&activity.activity_type);
    let time_ago = format_timestamp(&activity.timestamp);

    view! {
        <div class="flex items-start gap-4">
            <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-100 text-xl">
                {icon}
            </div>
            <div class="flex-1">
                <p class="text-sm text-gray-900">
                    {match &activity.user {
                        Some(user) => view! {
                            <span class="font-semibold">{&user.name}</span>
                        }.into_any(),
                        None => view! { <span class="font-semibold">"System"</span> }.into_any(),
                    }}
                    " "
                    {&activity.description}
                </p>
                <p class="mt-1 text-xs text-gray-500">
                    {time_ago}
                </p>
            </div>
        </div>
    }
}

/// Skeleton loader for activity item
#[component]
fn ActivityItemSkeleton() -> impl IntoView {
    view! {
        <div class="flex items-start gap-4">
            <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-200 animate-pulse"></div>
            <div class="flex-1">
                <div class="h-4 w-48 bg-gray-200 rounded animate-pulse"></div>
                <div class="mt-1 h-3 w-24 bg-gray-200 rounded animate-pulse"></div>
            </div>
        </div>
    }
}

// ============================================================================
// QuickActionLink Component
// ============================================================================

#[component]
fn QuickActionLink(
    href: &'static str,
    icon: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center gap-3 rounded-lg border border-gray-200 px-4 py-3 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:border-gray-300"
        >
            <span class="text-xl">{icon}</span>
            <span>{children()}</span>
        </a>
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format a number with commas as thousand separators
fn format_number(num: i64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let mut count = 0;

    for ch in num_str.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
        count += 1;
    }

    result.chars().rev().collect()
}

/// Format a change percentage for display
fn format_change(change: f64) -> String {
    if change >= 0.0 {
        format!("+{:.0}%", change)
    } else {
        format!("{:.0}%", change)
    }
}

/// Get an icon emoji based on activity type
fn get_activity_icon(activity_type: &str) -> &'static str {
    match activity_type {
        "user.created" => "👤",
        "user.updated" => "✏️",
        "user.deleted" => "🗑️",
        "post.created" => "📝",
        "post.published" => "📰",
        "order.created" => "📦",
        "order.completed" => "✅",
        "system.started" => "🚀",
        "tenant.checked" => "🔍",
        _ => "📌",
    }
}

/// Format a timestamp to a human-readable "time ago" format
fn format_timestamp(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(|dt| HumanTime::from(dt).to_string())
        .unwrap_or_else(|| timestamp.to_string())
}
