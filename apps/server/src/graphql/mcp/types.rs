use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{mcp_audit_logs, mcp_clients, mcp_policies, mcp_scaffold_drafts, mcp_tokens};
use rustok_mcp::{McpAccessContext, McpActorType, ModuleScaffoldDraftStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum McpActorTypeGql {
    HumanUser,
    ServiceClient,
    ModelAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum McpScaffoldDraftStatusGql {
    Staged,
    Applied,
}

impl From<ModuleScaffoldDraftStatus> for McpScaffoldDraftStatusGql {
    fn from(value: ModuleScaffoldDraftStatus) -> Self {
        match value {
            ModuleScaffoldDraftStatus::Staged => Self::Staged,
            ModuleScaffoldDraftStatus::Applied => Self::Applied,
        }
    }
}

impl McpActorTypeGql {
    pub fn to_runtime(self) -> McpActorType {
        match self {
            Self::HumanUser => McpActorType::HumanUser,
            Self::ServiceClient => McpActorType::ServiceClient,
            Self::ModelAgent => McpActorType::ModelAgent,
        }
    }
}

impl From<McpActorType> for McpActorTypeGql {
    fn from(value: McpActorType) -> Self {
        match value {
            McpActorType::HumanUser => Self::HumanUser,
            McpActorType::ServiceClient => Self::ServiceClient,
            McpActorType::ModelAgent => Self::ModelAgent,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpClientGql {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub client_key: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub actor_type: McpActorTypeGql,
    pub delegated_user_id: Option<Uuid>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<mcp_clients::Model> for McpClientGql {
    fn from(value: mcp_clients::Model) -> Self {
        let actor_type = value.actor_type();
        let is_active = value.is_active();
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            client_key: value.client_key,
            slug: value.slug,
            display_name: value.display_name,
            description: value.description,
            actor_type: actor_type.into(),
            delegated_user_id: value.delegated_user_id,
            is_active,
            revoked_at: value.revoked_at.map(Into::into),
            last_used_at: value.last_used_at.map(Into::into),
            metadata: value.metadata.to_string(),
            created_by: value.created_by,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpPolicyGql {
    pub id: Uuid,
    pub client_id: Uuid,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub granted_permissions: Vec<String>,
    pub granted_scopes: Vec<String>,
    pub metadata: String,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<mcp_policies::Model> for McpPolicyGql {
    fn from(value: mcp_policies::Model) -> Self {
        Self {
            id: value.id,
            client_id: value.client_id,
            allowed_tools: value.allowed_tools_list(),
            denied_tools: value.denied_tools_list(),
            granted_permissions: value.granted_permissions_list(),
            granted_scopes: value.granted_scopes_list(),
            metadata: value.metadata.to_string(),
            updated_by: value.updated_by,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpTokenGql {
    pub id: Uuid,
    pub client_id: Uuid,
    pub token_name: String,
    pub token_preview: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub metadata: String,
    pub created_at: DateTime<Utc>,
}

impl From<mcp_tokens::Model> for McpTokenGql {
    fn from(value: mcp_tokens::Model) -> Self {
        let is_active = value.is_active();
        Self {
            id: value.id,
            client_id: value.client_id,
            token_name: value.token_name,
            token_preview: value.token_preview,
            is_active,
            last_used_at: value.last_used_at.map(Into::into),
            expires_at: value.expires_at.map(Into::into),
            revoked_at: value.revoked_at.map(Into::into),
            metadata: value.metadata.to_string(),
            created_at: value.created_at.into(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpAuditEventGql {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub action: String,
    pub outcome: String,
    pub tool_name: Option<String>,
    pub reason: Option<String>,
    pub correlation_id: Option<String>,
    pub metadata: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<mcp_audit_logs::Model> for McpAuditEventGql {
    fn from(value: mcp_audit_logs::Model) -> Self {
        Self {
            id: value.id,
            client_id: value.client_id,
            token_id: value.token_id,
            actor_id: value.actor_id,
            actor_type: value.actor_type,
            action: value.action,
            outcome: value.outcome,
            tool_name: value.tool_name,
            reason: value.reason,
            correlation_id: value.correlation_id,
            metadata: value.metadata.to_string(),
            created_by: value.created_by,
            created_at: value.created_at.into(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpClientDetailsGql {
    pub client: McpClientGql,
    pub policy: Option<McpPolicyGql>,
    pub tokens: Vec<McpTokenGql>,
    pub effective_access_context: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct McpModuleScaffoldDraftGql {
    pub id: Uuid,
    pub client_id: Option<Uuid>,
    pub slug: String,
    pub crate_name: String,
    pub status: McpScaffoldDraftStatusGql,
    pub request_json: String,
    pub preview_json: String,
    pub workspace_root: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<mcp_scaffold_drafts::Model> for McpModuleScaffoldDraftGql {
    type Error = async_graphql::Error;

    fn try_from(value: mcp_scaffold_drafts::Model) -> Result<Self, Self::Error> {
        let status = value.status_value();
        Ok(Self {
            id: value.id,
            client_id: value.client_id,
            slug: value.slug,
            crate_name: value.crate_name,
            status: status.into(),
            request_json: serde_json::to_string(&value.request_payload)
                .map_err(|err| async_graphql::Error::new(err.to_string()))?,
            preview_json: serde_json::to_string(&value.preview_payload)
                .map_err(|err| async_graphql::Error::new(err.to_string()))?,
            workspace_root: value.workspace_root,
            applied_at: value.applied_at.map(Into::into),
            created_by: value.created_by,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        })
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateMcpClientResultGql {
    pub client: McpClientGql,
    pub policy: McpPolicyGql,
    pub token: McpTokenGql,
    pub plaintext_token: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RotateMcpTokenResultGql {
    pub client: McpClientGql,
    pub token: McpTokenGql,
    pub plaintext_token: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct CreateMcpClientInput {
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub actor_type: McpActorTypeGql,
    pub delegated_user_id: Option<Uuid>,
    pub token_name: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    #[graphql(default)]
    pub allowed_tools: Vec<String>,
    #[graphql(default)]
    pub denied_tools: Vec<String>,
    #[graphql(default)]
    pub granted_permissions: Vec<String>,
    #[graphql(default)]
    pub granted_scopes: Vec<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct RotateMcpTokenInput {
    pub token_name: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoke_existing_tokens: Option<bool>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpdateMcpPolicyInput {
    #[graphql(default)]
    pub allowed_tools: Vec<String>,
    #[graphql(default)]
    pub denied_tools: Vec<String>,
    #[graphql(default)]
    pub granted_permissions: Vec<String>,
    #[graphql(default)]
    pub granted_scopes: Vec<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct StageMcpModuleScaffoldDraftInput {
    pub client_id: Option<Uuid>,
    pub slug: String,
    pub name: String,
    pub description: String,
    #[graphql(default)]
    pub dependencies: Vec<String>,
    pub with_graphql: Option<bool>,
    pub with_rest: Option<bool>,
}

#[derive(Debug, Clone, InputObject)]
pub struct ApplyMcpModuleScaffoldDraftInput {
    pub workspace_root: String,
    pub confirm: bool,
}

pub fn parse_metadata(metadata: Option<String>) -> async_graphql::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(&value)
            .map_err(|err| async_graphql::Error::new(format!("Invalid metadata JSON: {err}"))),
        None => Ok(serde_json::json!({})),
    }
}

pub fn serialize_access_context(
    value: Option<&McpAccessContext>,
) -> async_graphql::Result<Option<String>> {
    value
        .map(|ctx| {
            serde_json::to_string(ctx).map_err(|err| {
                async_graphql::Error::new(format!("Failed to serialize access context: {err}"))
            })
        })
        .transpose()
}
