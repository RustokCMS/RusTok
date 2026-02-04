use async_graphql::{Context, Object, Result};
use chrono::Utc;
use uuid::Uuid;

use alloy_scripting::model::Script;
use alloy_scripting::runner::ExecutionOutcome;

use super::types::{
    CreateScriptInput, GqlExecutionResult, GqlScript, RunScriptInput, UpdateScriptInput,
};
use super::{dynamic_to_json, json_to_dynamic, require_admin, AlloyState};

#[derive(Default)]
pub struct AlloyMutation;

#[Object]
impl AlloyMutation {
    async fn create_script(
        &self,
        ctx: &Context<'_>,
        input: CreateScriptInput,
    ) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        state
            .engine
            .compile(&input.name, &input.code)
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        let mut script = Script::new(input.name, input.code, input.trigger.into());
        script.description = input.description;
        script.run_as_system = input.run_as_system;
        script.permissions = input.permissions;
        script.author_id = input.author_id;
        if let Some(status) = input.status {
            script.status = status.into();
        }

        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn update_script(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: UpdateScriptInput,
    ) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        if let Some(name) = input.name {
            state.engine.invalidate(&script.name);
            script.name = name;
        }
        if let Some(description) = input.description {
            script.description = Some(description);
        }
        if let Some(code) = input.code {
            state.engine.invalidate(&script.name);
            state
                .engine
                .compile(&script.name, &code)
                .map_err(|err| async_graphql::Error::new(err.to_string()))?;
            script.code = code;
        }
        if let Some(trigger) = input.trigger {
            script.trigger = trigger.into();
        }
        if let Some(status) = input.status {
            script.status = status.into();
        }
        if let Some(run_as_system) = input.run_as_system {
            script.run_as_system = run_as_system;
        }
        if let Some(permissions) = input.permissions {
            script.permissions = permissions;
        }
        if input.clear_author_id {
            script.author_id = None;
        } else if let Some(author_id) = input.author_id {
            script.author_id = Some(author_id);
        }

        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn delete_script(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        state
            .storage
            .delete(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(true)
    }

    async fn run_script(
        &self,
        ctx: &Context<'_>,
        input: RunScriptInput,
    ) -> Result<GqlExecutionResult> {
        let auth = require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let user_id = Some(auth.user_id.to_string());

        let params = input
            .params
            .map(|params| {
                params
                    .0
                    .as_object()
                    .ok_or_else(|| async_graphql::Error::new("params must be a JSON object"))?
                    .iter()
                    .map(|(key, value)| (key.clone(), json_to_dynamic(value.clone())))
                    .collect()
            })
            .transpose()?
            .unwrap_or_default();

        let result = state
            .orchestrator
            .run_manual(&input.script_name, params, user_id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        let (success, error, return_value, changes) = match result.outcome {
            ExecutionOutcome::Success {
                return_value,
                entity_changes,
            } => (
                true,
                None,
                return_value.map(dynamic_to_json),
                Some(serde_json::Value::Object(
                    entity_changes
                        .into_iter()
                        .map(|(key, value)| (key, dynamic_to_json(value)))
                        .collect(),
                )),
            ),
            ExecutionOutcome::Aborted { reason } => (false, Some(reason), None, None),
            ExecutionOutcome::Failed { error } => (false, Some(error.to_string()), None, None),
        };

        Ok(GqlExecutionResult {
            execution_id: result.execution_id,
            success,
            duration_ms: result.duration_ms(),
            error,
            return_value: return_value.map(async_graphql::Json),
            changes: changes.map(async_graphql::Json),
        })
    }

    async fn activate_script(&self, ctx: &Context<'_>, id: Uuid) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        script.activate();
        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn pause_script(&self, ctx: &Context<'_>, id: Uuid) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        script.status = alloy_scripting::model::ScriptStatus::Paused;
        script.updated_at = Utc::now();

        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn disable_script(&self, ctx: &Context<'_>, id: Uuid) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        script.disable();
        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn archive_script(&self, ctx: &Context<'_>, id: Uuid) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        script.archive();
        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }

    async fn reset_script_errors(&self, ctx: &Context<'_>, id: Uuid) -> Result<GqlScript> {
        require_admin(ctx)?;
        let state = ctx.data::<AlloyState>()?;
        let mut script = state
            .storage
            .get(id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        script.reset_errors();
        script.updated_at = Utc::now();

        let saved = state
            .storage
            .save(script)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(saved.into())
    }
}
