use leptos_graphql::{
    execute as execute_graphql, persisted_query_extension, GraphqlHttpError, GraphqlRequest,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const API_URL: &str = "http://localhost:5150/api/graphql";
pub const REST_API_URL: &str = "http://localhost:5150";

pub type ApiError = GraphqlHttpError;

// ============================================================================
// Dashboard GraphQL Types
// ============================================================================

/// Response from dashboardStats query
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DashboardStatsResponse {
    pub dashboard_stats: DashboardStats,
}

/// Dashboard statistics data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

/// Response from recentActivity query
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RecentActivityResponse {
    pub recent_activity: Vec<ActivityItem>,
}

/// Activity item in the recent activity feed
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActivityItem {
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub description: String,
    pub timestamp: String,
    pub user: Option<ActivityUser>,
}

/// User information in activity item
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActivityUser {
    pub id: String,
    pub name: String,
}

// ============================================================================
// Dashboard GraphQL Queries
// ============================================================================

/// GraphQL query for dashboard statistics
pub const DASHBOARD_STATS_QUERY: &str = r#"
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

/// GraphQL query for recent activity
pub const RECENT_ACTIVITY_QUERY: &str = r#"
query RecentActivity($limit: Int = 10) {
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

// ============================================================================
// Users GraphQL Types
// ============================================================================

/// Response from users query
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UsersResponse {
    pub users: UserConnection,
}

/// User connection with pagination
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserConnection {
    pub edges: Vec<UserEdge>,
    pub page_info: PageInfo,
}

/// User edge for pagination
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserEdge {
    pub node: User,
    pub cursor: String,
}

/// User data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
}

/// Page info for pagination
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub total_count: i64,
}

/// Users filter input
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UsersFilter {
    pub role: Option<String>,
    pub status: Option<String>,
}

/// Pagination input
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaginationInput {
    pub first: Option<i64>,
    pub after: Option<String>,
}

// ============================================================================
// Users GraphQL Queries
// ============================================================================

/// GraphQL query for users list with pagination
pub const USERS_QUERY: &str = r#"
query Users($pagination: PaginationInput!, $filter: UsersFilter, $search: String) {
  users(pagination: $pagination, filter: $filter, search: $search) {
    edges {
      node {
        id
        email
        name
        role
        status
        createdAt
      }
      cursor
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
      totalCount
    }
  }
}
"#;

pub async fn request<V, T>(
    query: &str,
    variables: V,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        API_URL,
        GraphqlRequest::new(query, Some(variables)),
        token,
        tenant_slug,
    )
    .await
}

pub async fn request_with_persisted<V, T>(
    query: &str,
    variables: V,
    sha256_hash: &str,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        API_URL,
        GraphqlRequest::new(query, Some(variables))
            .with_extensions(persisted_query_extension(sha256_hash)),
        token,
        tenant_slug,
    )
    .await
}

pub async fn rest_get<T>(
    path: &str,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}{}", REST_API_URL, path));

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    if let Some(slug) = tenant_slug {
        req = req.header("X-Tenant-Slug", slug);
    }

    let res = req.send().await.map_err(|_| ApiError::Network)?;

    if res.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !res.status().is_success() {
        return Err(ApiError::Http(res.status().to_string()));
    }

    res.json().await.map_err(|_| ApiError::Network)
}

pub async fn rest_post<B, T>(
    path: &str,
    body: &B,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    B: Serialize,
    T: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}{}", REST_API_URL, path))
        .json(body)
        .header("Idempotency-Key", Uuid::new_v4().to_string());

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    if let Some(slug) = tenant_slug {
        req = req.header("X-Tenant-Slug", slug);
    }

    let res = req.send().await.map_err(|_| ApiError::Network)?;

    if res.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !res.status().is_success() {
        return Err(ApiError::Http(res.status().to_string()));
    }

    res.json().await.map_err(|_| ApiError::Network)
}
