use async_graphql::{Context, FieldError, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::context::AuthContext;
use crate::graphql::errors::GraphQLError;
use crate::services::mcp_management::{McpAuditFilters, McpManagementService};

use super::{
    ensure_mcp_manage, ensure_mcp_read,
    types::{
        serialize_access_context, McpAuditEventGql, McpClientDetailsGql, McpClientGql,
        McpModuleScaffoldDraftGql,
    },
};

#[derive(Default)]
pub struct McpQuery;

fn require_auth_context<'a>(ctx: &'a Context<'a>) -> Result<&'a AuthContext> {
    ctx.data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())
}

#[Object]
impl McpQuery {
    async fn mcp_clients(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
    ) -> Result<Vec<McpClientGql>> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_read(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let clients = McpManagementService::list_clients(
            db,
            auth.tenant_id,
            limit.map(|value| value.max(0) as u64),
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(clients.into_iter().map(Into::into).collect())
    }

    async fn mcp_client(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<McpClientDetailsGql>> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_read(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let details = McpManagementService::get_client_details(db, auth.tenant_id, id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        details
            .map(|details| {
                Ok(McpClientDetailsGql {
                    client: details.client.into(),
                    policy: details.policy.map(Into::into),
                    tokens: details.tokens.into_iter().map(Into::into).collect(),
                    effective_access_context: serialize_access_context(
                        details.effective_access_context.as_ref(),
                    )?,
                })
            })
            .transpose()
    }

    async fn mcp_audit_events(
        &self,
        ctx: &Context<'_>,
        client_id: Option<Uuid>,
        outcome: Option<String>,
        limit: Option<i32>,
    ) -> Result<Vec<McpAuditEventGql>> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_read(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let events = McpManagementService::list_audit_events(
            db,
            auth.tenant_id,
            McpAuditFilters {
                client_id,
                outcome,
                limit: limit.map(|value| value.max(0) as u64),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(events.into_iter().map(Into::into).collect())
    }

    async fn mcp_module_scaffold_drafts(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
    ) -> Result<Vec<McpModuleScaffoldDraftGql>> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let drafts = McpManagementService::list_scaffold_drafts(
            db,
            auth.tenant_id,
            limit.map(|value| value.max(0) as u64),
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        drafts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()
    }

    async fn mcp_module_scaffold_draft(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> Result<Option<McpModuleScaffoldDraftGql>> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let draft = McpManagementService::get_scaffold_draft(db, auth.tenant_id, id)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        draft.map(TryInto::try_into).transpose()
    }
}
