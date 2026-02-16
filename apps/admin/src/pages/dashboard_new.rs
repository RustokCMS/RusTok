// New Dashboard Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::providers::auth::use_auth;
use crate::providers::locale::translate;

// GraphQL Queries
const DASHBOARD_STATS_QUERY: &str = r#"
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

const RECENT_ACTIVITY_QUERY: &str = r#"
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DashboardStatsData {
    dashboard_stats: DashboardStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DashboardStats {
    total_users: i64,
    total_posts: i64,
    total_orders: i64,
    total_revenue: i64,
    users_change: f64,
    posts_change: f64,
    orders_change: f64,
    revenue_change: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RecentActivityData {
    recent_activity: Vec<ActivityItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ActivityItem {
    id: String,
    #[serde(rename = "type")]
    activity_type: String,
    description: String,
    timestamp: String,
    user: Option<ActivityUser>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ActivityUser {
    id: String,
    name: String,
}

#[component]
pub fn DashboardNew() -> impl IntoView {
    let auth = use_auth();

    // Query dashboard stats
    let stats_result = leptos_graphql::use_query::<serde_json::Value, DashboardStatsData>(
        leptos_graphql::GRAPHQL_ENDPOINT.to_string(),
        DASHBOARD_STATS_QUERY.to_string(),
        None,
        auth.token.get(),
        auth.tenant_slug.get(),
    );

    // Query recent activity
    let activity_result = leptos_graphql::use_query::<serde_json::Value, RecentActivityData>(
        leptos_graphql::GRAPHQL_ENDPOINT.to_string(),
        RECENT_ACTIVITY_QUERY.to_string(),
        Some(serde_json::json!({ "limit": 10 })),
        auth.token.get(),
        auth.tenant_slug.get(),
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
                    let stats_data = stats_result.data.get();
                    match stats_data {
                        Some(data) => {
                            let stats = &data.dashboard_stats;
                            vec![
                                StatData {
                                    title: "Total Users",
                                    value: stats.total_users.to_string(),
                                    change: format!("{:+.1}%", stats.users_change),
                                    change_positive: stats.users_change >= 0.0,
                                    icon: "👥",
                                },
                                StatData {
                                    title: "Total Posts",
                                    value: stats.total_posts.to_string(),
                                    change: format!("{:+.1}%", stats.posts_change),
                                    change_positive: stats.posts_change >= 0.0,
                                    icon: "📝",
                                },
                                StatData {
                                    title: "Total Orders",
                                    value: stats.total_orders.to_string(),
                                    change: format!("{:+.1}%", stats.orders_change),
                                    change_positive: stats.orders_change >= 0.0,
                                    icon: "📦",
                                },
                                StatData {
                                    title: "Revenue",
                                    value: format!("${}", stats.total_revenue),
                                    change: format!("{:+.1}%", stats.revenue_change),
                                    change_positive: stats.revenue_change >= 0.0,
                                    icon: "💰",
                                },
                            ]
                        }
                        None => vec![]
                    }
                }.into_iter().map(|stat| {
                    view! { <StatCard stat=stat /> }
                }).collect_view()}
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
                            {move || {
                                if activity_result.loading.get() {
                                    view! {
                                        <div class="flex items-center justify-center py-8">
                                            <div class="text-gray-500">"Loading activity..."</div>
                                        </div>
                                    }.into_any()
                                } else if let Some(error) = activity_result.error.get() {
                                    view! {
                                        <div class="flex items-center justify-center py-8">
                                            <div class="text-red-500">{format!("Error: {}", error)}</div>
                                        </div>
                                    }.into_any()
                                } else if let Some(data) = activity_result.data.get() {
                                    let activities = data.recent_activity.clone();
                                    view! {
                                        <div class="space-y-4">
                                            {activities.into_iter().map(|activity| {
                                                view! { <ActivityItemGraphQL activity=activity /> }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="flex items-center justify-center py-8">
                                            <div class="text-gray-500">"No activity yet"</div>
                                        </div>
                                    }.into_any()
                                }
                            }}
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

#[derive(Clone)]
struct StatData {
    title: &'static str,
    value: &'static str,
    change: &'static str,
    change_positive: bool,
    icon: &'static str,
}

#[component]
fn StatCard(stat: StatData) -> impl IntoView {
    let change_color = if stat.change_positive {
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
                            {stat.title}
                        </p>
                        <p class="mt-2 text-3xl font-bold text-gray-900">
                            {stat.value}
                        </p>
                        <p class=format!("mt-2 text-sm font-medium {}", change_color)>
                            {stat.change}
                            " from last month"
                        </p>
                    </div>
                    <div class="ml-4 text-4xl">
                        {stat.icon}
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}

// ============================================================================
// ActivityItemGraphQL Component (GraphQL-based)
// ============================================================================

#[component]
fn ActivityItemGraphQL(activity: ActivityItem) -> impl IntoView {
    let (icon, user_name, description) = match activity.activity_type.as_str() {
        "user.created" => ("👤", activity.user.as_ref().map(|u| u.name.as_str()), Some(activity.description.clone())),
        "system.started" => ("🚀", None, Some(activity.description.clone())),
        "tenant.checked" => ("🔍", None, Some(activity.description.clone())),
        _ => ("📌", activity.user.as_ref().map(|u| u.name.as_str()), Some(activity.description.clone())),
    };

    // Parse timestamp for relative time
    let time_ago = match DateTime::parse_from_rfc3339(&activity.timestamp) {
        Ok(dt) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&Utc));
            if duration.num_minutes() < 60 {
                format!("{} minutes ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!("{} hours ago", duration.num_hours())
            } else {
                format!("{} days ago", duration.num_days())
            }
        }
        Err(_) => "Recently".to_string(),
    };

    view! {
        <div class="flex items-start gap-4">
            <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-100 text-xl">
                {icon}
            </div>
            <div class="flex-1">
                <p class="text-sm text-gray-900">
                    {match user_name {
                        Some(name) => view! { <span class="font-semibold">{name}</span> }.into_any(),
                        None => view! {}.into_any(),
                    }}
                    " "
                    {description.unwrap_or_else(|| "Unknown activity".to_string())}
                </p>
                <p class="mt-1 text-xs text-gray-500">
                    {time_ago}
                </p>
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
