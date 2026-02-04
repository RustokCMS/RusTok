use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use alloy_scripting::storage::ScriptQuery;

use super::{AlloyState, GqlEventType, GqlScript, GqlScriptStatus};

#[derive(Default)]
pub struct AlloyQuery;

#[Object]
impl AlloyQuery {
    async fn scripts(
        &self,
        ctx: &Context<'_>,
        status: Option<GqlScriptStatus>,
    ) -> Result<Vec<GqlScript>> {
        let state = ctx.data::<AlloyState>()?;
        let query = match status {
            Some(status) => ScriptQuery::ByStatus(status.into()),
            None => ScriptQuery::ByStatus(alloy_scripting::model::ScriptStatus::Active),
        };

        let scripts = state
            .storage
            .find(query)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(scripts.into_iter().map(GqlScript::from).collect())
    }

    async fn script(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<GqlScript>> {
        let state = ctx.data::<AlloyState>()?;
        match state.storage.get(id).await {
            Ok(script) => Ok(Some(script.into())),
            Err(_) => Ok(None),
        }
    }

    async fn script_by_name(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<Option<GqlScript>> {
        let state = ctx.data::<AlloyState>()?;
        match state.storage.get_by_name(&name).await {
            Ok(script) => Ok(Some(script.into())),
            Err(_) => Ok(None),
        }
    }

    async fn scripts_for_event(
        &self,
        ctx: &Context<'_>,
        entity_type: String,
        event: GqlEventType,
    ) -> Result<Vec<GqlScript>> {
        let state = ctx.data::<AlloyState>()?;
        let scripts = state
            .storage
            .find(ScriptQuery::ByEvent {
                entity_type,
                event: event.into(),
            })
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(scripts.into_iter().map(GqlScript::from).collect())
    }
}
