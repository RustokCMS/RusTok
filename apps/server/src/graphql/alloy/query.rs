use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use alloy_scripting::storage::ScriptQuery;
use alloy_scripting::ScriptRegistry;

use super::{
    require_admin, AlloyState, GqlEventType, GqlScript, GqlScriptConnection, GqlScriptStatus,
};
use crate::graphql::common::PaginationInput;

#[derive(Default)]
pub struct AlloyQuery;

#[Object]
impl AlloyQuery {
    async fn scripts(
        &self,
        ctx: &Context<'_>,
        status: Option<GqlScriptStatus>,
        #[graphql(default)] pagination: PaginationInput,
    ) -> Result<GqlScriptConnection> {
        require_admin(ctx).await?;
        let state = ctx.data::<AlloyState>()?;
        let query = match status {
            Some(status) => ScriptQuery::ByStatus(status.into()),
            None => ScriptQuery::All,
        };

        let (offset, limit) = pagination.normalize()?;
        let page = state
            .storage
            .find_paginated(query, offset as u64, limit as u64)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        let items = page.items.into_iter().map(GqlScript::from).collect();

        Ok(GqlScriptConnection {
            items,
            page_info: crate::graphql::common::PageInfo::new(page.total as i64, offset, limit),
        })
    }

    async fn script(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<GqlScript>> {
        require_admin(ctx).await?;
        let state = ctx.data::<AlloyState>()?;
        match state.storage.get(id).await {
            Ok(script) => Ok(Some(script.into())),
            Err(_) => Ok(None),
        }
    }

    async fn script_by_name(&self, ctx: &Context<'_>, name: String) -> Result<Option<GqlScript>> {
        require_admin(ctx).await?;
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
        require_admin(ctx).await?;
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
