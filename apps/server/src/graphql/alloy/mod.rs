mod mutation;
mod query;
mod types;

use std::sync::Arc;

use alloy_scripting::{ScriptEngine, ScriptOrchestrator, SeaOrmExecutionLog, SeaOrmStorage};
use async_graphql::{Context, FieldError, Result};

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

pub(crate) fn require_admin(ctx: &Context<'_>) -> Result<AuthContext> {
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

    if !matches!(
        auth.role,
        rustok_core::UserRole::Admin | rustok_core::UserRole::SuperAdmin
    ) {
        return Err(<FieldError as GraphQLError>::permission_denied("Forbidden"));
    }

    Ok(auth.clone())
}

pub(crate) use alloy_scripting::utils::{dynamic_to_json, json_to_dynamic};
