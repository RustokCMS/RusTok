mod mutation;
mod query;
mod types;

use std::sync::Arc;

use alloy_scripting::{ScriptEngine, ScriptOrchestrator, SeaOrmExecutionLog, SeaOrmStorage};
use async_graphql::{Context, FieldError, Result};
use rustok_core::{Action, Permission};

use crate::context::AuthContext;
use crate::graphql::errors::GraphQLError;

pub use mutation::AlloyMutation;
pub use query::AlloyQuery;
pub use types::*;

#[derive(Clone)]
pub struct AlloyState {
    pub engine: Arc<ScriptEngine>,
    pub storage: Arc<SeaOrmStorage>,
    pub orchestrator: Arc<ScriptOrchestrator<SeaOrmStorage>>,
    pub execution_log: Arc<SeaOrmExecutionLog>,
}

impl AlloyState {
    pub fn new(
        engine: Arc<ScriptEngine>,
        storage: Arc<SeaOrmStorage>,
        orchestrator: Arc<ScriptOrchestrator<SeaOrmStorage>>,
        execution_log: Arc<SeaOrmExecutionLog>,
    ) -> Self {
        Self {
            engine,
            storage,
            orchestrator,
            execution_log,
        }
    }
}

fn has_effective_permission(auth: &AuthContext, required_permission: Permission) -> bool {
    auth.permissions.contains(&required_permission)
        || auth.permissions.contains(&Permission::new(
            required_permission.resource,
            Action::Manage,
        ))
}

pub(crate) async fn require_admin(ctx: &Context<'_>) -> Result<AuthContext> {
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

    if !has_effective_permission(
        auth,
        Permission::new(rustok_core::Resource::Scripts, Action::Manage),
    ) {
        return Err(<FieldError as GraphQLError>::permission_denied("Forbidden"));
    }

    Ok(auth.clone())
}

pub(crate) use alloy_scripting::utils::{dynamic_to_json, json_to_dynamic};
