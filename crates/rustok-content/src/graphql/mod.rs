mod mutation;
mod query;
mod types;

use async_graphql::{Context, FieldError, Result};
use rustok_api::{graphql::GraphQLError, has_any_effective_permission, AuthContext};
use rustok_core::Permission;

pub use mutation::ContentMutation;
pub use query::ContentQuery;
pub use types::*;

pub(crate) const MODULE_SLUG: &str = "content";

pub(crate) fn require_content_permission(
    ctx: &Context<'_>,
    permissions: &[Permission],
    message: &str,
) -> Result<AuthContext> {
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?
        .clone();

    if !has_any_effective_permission(&auth.permissions, permissions) {
        return Err(<FieldError as GraphQLError>::permission_denied(message));
    }

    Ok(auth)
}
