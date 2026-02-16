// New Dashboard Page with GraphQL integration
use leptos::prelude::*;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant};
use serde::{Deserialize, Serialize};
use leptos_auth::use_token;
use leptos_auth::use_tenant;
use leptos_graphql::use_query;

use crate::providers::auth::use_auth;
use crate::providers::locale::translate;

// ============================================================================
// GraphQL Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_posts: i64,
    pub total_orders: i64,
    pub total_revenue: i64,
    pub users_change: f64,
    pub posts_change: f64,
    pub orders_change: f64,
    pub revenue_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityUser {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub r#type: String,
    pub description: String,
    pub timestamp: String,
    pub user: Option<ActivityUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatsResponse {
    pub dashboard_stats: DashboardStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivityResponse {
    pub recent_activity: Vec<ActivityItem>,
}

// ============================================================================
// Dashboard Component
// ============================================================================

#[component]
pub fn DashboardNew() -> impl IntoView {
    let auth = use_auth();
    let token = use_token();
    let tenant = use_tenant();

    // Dashboard Stats Query
    let stats_query = r#"
        query DashboardStats {
            dashboardStats {
                totalUsers
                totalPosts
                totalOrders
                totalRevenue
                usersChange
                postsChange
                ordersChange
                revenueChange
            }
        }
    "#;

    let stats_result = use_query::<serde_json::Value, DashboardStatsResponse>(
        "/api/graphql".to_string(),
        stats_query.to_string(),
        None::<serde_json::Value>,
        Signal::derive(move || token.get()),
        Signal::derive(move || tenant.get()),
    );

    // Recent Activity Query
    let activity_query = r#"
        query RecentActivity($limit: Int) {
            recentActivity(limit: $limit) {
                id
                type
                description
                timestamp
                user {
                    id
                    name
                }
            }
        }
    "#;

    let activity_result = use_query::<serde_json::Value, RecentActivityResponse>(
        "/api/graphql".to_string(),
        activity_query.to_string(),
        Signal::derive(move || Some(serde_json::json!({ "limit": 10 })),
        Signal::derive(move || token.get()),
        Signal::derive(move || tenant.get()),
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
            <Show
                when=move || stats_result.loading.get()
                fallback=move || {
                    stats_result.data.get().and_then(|data| {
                        let stats = data.dashboard_stats.clone();
                        Some(view! {
                            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
                                <StatCard
                                    title="Total Users"
                                    value=stats.total_users.to_string()
                                    change=format!("{:+.1}%", stats.users_change)
                                    change_positive=stats.users_change >= 0.0
                                    icon="👥"
                                />
                                <StatCard
                                    title="Total Posts"
                                    value=stats.total_posts.to_string()
                                    change=format!("{:+.1}%", stats.posts_change)
                                    change_positive=stats.posts_change >= 0.0
                                    icon="📝"
                                />
                                <StatCard
                                    title="Total Orders"
                                    value=stats.total_orders.to_string()
                                    change=format!("{:+.1}%", stats.orders_change)
                                    change_positive=stats.orders_change >= 0.0
                                    icon="📦"
                                />
                                <StatCard
                                    title="Revenue"
                                    value=format!("${}", stats.total_revenue)
                                    change=format!("{:+.1}%", stats.revenue_change)
                                    change_positive=stats.revenue_change >= 0.0
                                    icon="💰"
                                />
                            </div>
                        })
                    })
                }
            >
                <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                    <StatCardSkeleton />
                </div>
            </Show>

            // Main Content Grid
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                // Recent Activity (2 columns)
                <div class="lg:col-span-2">
                    <Show
                        when=move || activity_result.loading.get()
                        fallback=move || {
                            activity_result.data.get().and_then(|data| {
                                let activities = data.recent_activity.clone();
                                Some(view! {
                                    <Card>
                                        <CardHeader class="border-b border-gray-200">
                                            <h3 class="text-lg font-semibold text-gray-900">
                                                "Recent Activity"
                                            </h3>
                                        </CardHeader>
                                        <CardContent>
                                            <div class="space-y-4">
                                                {activities.into_iter().map(|activity| {
                                                    view! { <ActivityItemComponent activity=activity /> }
                                                }).collect_view()}
                                            </div>
                                        </CardContent>
                                    </Card>
                                })
                            })
                        }
                    >
                        <Card>
                            <CardHeader class="border-b border-gray-200">
                                <h3 class="text-lg font-semibold text-gray-900">
                                    "Recent Activity"
                                </h3>
                            </CardHeader>
                            <CardContent>
                                <div class="space-y-4">
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                    <ActivityItemSkeleton />
                                </div>
                            </CardContent>
                        </Card>
                    </Show>
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
    title: String,
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

#[component]
fn StatCardSkeleton() -> impl IntoView {
    view! {
        <Card>
            <CardContent class="p-6">
                <div class="flex items-center justify-between">
                    <div class="flex-1 space-y-3">
                        <div class="h-4 bg-gray-200 rounded animate-pulse" style="width: 40%"></div>
                        <div class="h-8 bg-gray-200 rounded animate-pulse"></div>
                        <div class="h-4 bg-gray-200 rounded animate-pulse" style="width: 50%"></div>
                    </div>
                    <div class="ml-4 text-4xl opacity-30">📊</div>
                </div>
            </CardContent>
        </Card>
    }
}

// ============================================================================
// ActivityItem Component
// ============================================================================

#[component]
fn ActivityItemComponent(activity: ActivityItem) -> impl IntoView {
    let icon = match activity.r#type.as_str() {
        "user.created" => "👤",
        "system.started" => "🚀",
        "tenant.checked" => "✅",
        "node.created" => "📝",
        "node.updated" => "✏️",
        "node.published" => "📢",
        "order.created" => "🛒",
        "order.paid" => "💳",
        _ => "📌",
    };

    view! {
        <div class="flex items-start gap-4">
            <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-100 text-xl">
                {icon}
            </div>
            <div class="flex-1">
                <p class="text-sm text-gray-900">
                    {activity.user.as_ref().map(|u| view! {
                        <span class="font-semibold">{u.name.as_deref().unwrap_or(&u.id)}</span>
                    })}
                    " "
                    {activity.description}
                </p>
                <p class="mt-1 text-xs text-gray-500">
                    {format_timestamp(&activity.timestamp)}
                </p>
            </div>
        </div>
    }
}

#[component]
fn ActivityItemSkeleton() -> impl IntoView {
    view! {
        <div class="flex items-start gap-4">
            <div class="h-10 w-10 rounded-full bg-gray-200 animate-pulse"></div>
            <div class="flex-1 space-y-2">
                <div class="h-4 bg-gray-200 rounded animate-pulse" style="width: 70%"></div>
                <div class="h-3 bg-gray-200 rounded animate-pulse" style="width: 40%"></div>
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

fn format_timestamp(timestamp: &str) -> String {
    // Try to parse the timestamp and format it nicely
    // For now, return a simple relative time
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
        
        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    } else {
        "Unknown time".to_string()
    }
}
