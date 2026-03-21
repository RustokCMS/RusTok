use async_graphql::{Context, FieldError, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::context::AuthContext;
use crate::graphql::errors::GraphQLError;
use crate::services::mcp_management::{
    ApplyMcpScaffoldDraftInput as ApplyMcpScaffoldDraftServiceInput,
    CreateMcpClientInput as CreateMcpClientServiceInput, McpManagementService,
    RotateMcpTokenInput as RotateMcpTokenServiceInput,
    StageMcpScaffoldDraftInput as StageMcpScaffoldDraftServiceInput,
    UpdateMcpPolicyInput as UpdateMcpPolicyServiceInput,
};
use rustok_mcp::ScaffoldModuleRequest;

use super::{
    ensure_mcp_manage,
    types::{
        parse_metadata, ApplyMcpModuleScaffoldDraftInput, CreateMcpClientInput,
        CreateMcpClientResultGql, McpModuleScaffoldDraftGql, McpPolicyGql, RotateMcpTokenInput,
        RotateMcpTokenResultGql, StageMcpModuleScaffoldDraftInput, UpdateMcpPolicyInput,
    },
};

#[derive(Default)]
pub struct McpMutation;

fn require_auth_context<'a>(ctx: &'a Context<'a>) -> Result<&'a AuthContext> {
    ctx.data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())
}

#[Object]
impl McpMutation {
    async fn create_mcp_client(
        &self,
        ctx: &Context<'_>,
        input: CreateMcpClientInput,
    ) -> Result<CreateMcpClientResultGql> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let metadata = parse_metadata(input.metadata)?;
        let result = McpManagementService::create_client(
            db,
            auth.tenant_id,
            CreateMcpClientServiceInput {
                slug: input.slug,
                display_name: input.display_name,
                description: input.description,
                actor_type: input.actor_type.to_runtime(),
                delegated_user_id: input.delegated_user_id,
                token_name: input.token_name,
                token_expires_at: input.token_expires_at,
                allowed_tools: input.allowed_tools,
                denied_tools: input.denied_tools,
                granted_permissions: input.granted_permissions,
                granted_scopes: input.granted_scopes,
                metadata,
                created_by: Some(auth.user_id),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(CreateMcpClientResultGql {
            client: result.client.into(),
            policy: result.policy.into(),
            token: result.token.into(),
            plaintext_token: result.plaintext_token,
        })
    }

    async fn rotate_mcp_client_token(
        &self,
        ctx: &Context<'_>,
        client_id: Uuid,
        input: RotateMcpTokenInput,
    ) -> Result<RotateMcpTokenResultGql> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let metadata = parse_metadata(input.metadata)?;
        let result = McpManagementService::rotate_token(
            db,
            auth.tenant_id,
            client_id,
            RotateMcpTokenServiceInput {
                token_name: input.token_name,
                expires_at: input.expires_at,
                metadata,
                created_by: Some(auth.user_id),
                revoke_existing_tokens: input.revoke_existing_tokens.unwrap_or(true),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(RotateMcpTokenResultGql {
            client: result.client.into(),
            token: result.token.into(),
            plaintext_token: result.plaintext_token,
        })
    }

    async fn update_mcp_client_policy(
        &self,
        ctx: &Context<'_>,
        client_id: Uuid,
        input: UpdateMcpPolicyInput,
    ) -> Result<McpPolicyGql> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let metadata = parse_metadata(input.metadata)?;
        let policy = McpManagementService::update_policy(
            db,
            auth.tenant_id,
            client_id,
            UpdateMcpPolicyServiceInput {
                allowed_tools: input.allowed_tools,
                denied_tools: input.denied_tools,
                granted_permissions: input.granted_permissions,
                granted_scopes: input.granted_scopes,
                metadata,
                updated_by: Some(auth.user_id),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(policy.into())
    }

    async fn revoke_mcp_token(
        &self,
        ctx: &Context<'_>,
        token_id: Uuid,
        reason: Option<String>,
    ) -> Result<bool> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        McpManagementService::revoke_token(
            db,
            auth.tenant_id,
            token_id,
            Some(auth.user_id),
            reason,
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(true)
    }

    async fn deactivate_mcp_client(
        &self,
        ctx: &Context<'_>,
        client_id: Uuid,
        reason: Option<String>,
    ) -> Result<bool> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        McpManagementService::deactivate_client(
            db,
            auth.tenant_id,
            client_id,
            Some(auth.user_id),
            reason,
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(true)
    }

    async fn stage_mcp_module_scaffold_draft(
        &self,
        ctx: &Context<'_>,
        input: StageMcpModuleScaffoldDraftInput,
    ) -> Result<McpModuleScaffoldDraftGql> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let draft = McpManagementService::stage_scaffold_draft(
            db,
            auth.tenant_id,
            StageMcpScaffoldDraftServiceInput {
                client_id: input.client_id,
                request: ScaffoldModuleRequest {
                    slug: input.slug,
                    name: input.name,
                    description: input.description,
                    dependencies: input.dependencies,
                    with_graphql: input.with_graphql.unwrap_or(true),
                    with_rest: input.with_rest.unwrap_or(true),
                    write_files: false,
                },
                created_by: Some(auth.user_id),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        draft.try_into()
    }

    async fn apply_mcp_module_scaffold_draft(
        &self,
        ctx: &Context<'_>,
        draft_id: Uuid,
        input: ApplyMcpModuleScaffoldDraftInput,
    ) -> Result<McpModuleScaffoldDraftGql> {
        let auth = require_auth_context(ctx)?;
        ensure_mcp_manage(auth)?;
        let db = ctx.data::<DatabaseConnection>()?;

        let (draft, _) = McpManagementService::apply_scaffold_draft(
            db,
            auth.tenant_id,
            draft_id,
            ApplyMcpScaffoldDraftServiceInput {
                workspace_root: input.workspace_root,
                confirm: input.confirm,
                applied_by: Some(auth.user_id),
            },
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        draft.try_into()
    }
}
