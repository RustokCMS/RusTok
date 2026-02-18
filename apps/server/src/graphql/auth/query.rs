use async_graphql::Object;

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    /// Health check for auth module
    async fn auth_health(&self) -> &str {
        "Auth module is working!"
    }
}
