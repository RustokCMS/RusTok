use leptos_graphql::{execute as execute_graphql, GraphqlHttpError, GraphqlRequest};
use serde::{Deserialize, Serialize};

use crate::model::{CurrentTenant, NodeDetail, NodeDraft, NodeList};

pub type ApiError = GraphqlHttpError;

const CURRENT_TENANT_QUERY: &str = "query ContentCurrentTenant { currentTenant { id slug name } }";
const NODES_QUERY: &str = "query ContentNodes($tenantId: UUID!, $filter: NodesFilter) { nodes(tenantId: $tenantId, filter: $filter) { total items { id kind status effectiveLocale title slug excerpt createdAt publishedAt } } }";
const NODE_QUERY: &str = "query ContentNode($tenantId: UUID!, $id: UUID!, $locale: String) { node(tenantId: $tenantId, id: $id, locale: $locale) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const CREATE_NODE_MUTATION: &str = "mutation CreateContentNode($tenantId: UUID!, $input: CreateNodeInput!) { createNode(tenantId: $tenantId, input: $input) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const UPDATE_NODE_MUTATION: &str = "mutation UpdateContentNode($tenantId: UUID!, $id: UUID!, $input: UpdateNodeInput!) { updateNode(tenantId: $tenantId, id: $id, input: $input) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const PUBLISH_NODE_MUTATION: &str = "mutation PublishContentNode($tenantId: UUID!, $id: UUID!) { publishNode(tenantId: $tenantId, id: $id) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const UNPUBLISH_NODE_MUTATION: &str = "mutation UnpublishContentNode($tenantId: UUID!, $id: UUID!) { unpublishNode(tenantId: $tenantId, id: $id) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const ARCHIVE_NODE_MUTATION: &str = "mutation ArchiveContentNode($tenantId: UUID!, $id: UUID!) { archiveNode(tenantId: $tenantId, id: $id) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const RESTORE_NODE_MUTATION: &str = "mutation RestoreContentNode($tenantId: UUID!, $id: UUID!) { restoreNode(tenantId: $tenantId, id: $id) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } updatedAt publishedAt } }";
const DELETE_NODE_MUTATION: &str =
    "mutation DeleteContentNode($tenantId: UUID!, $id: UUID!) { deleteNode(tenantId: $tenantId, id: $id) }";

#[derive(Debug, Deserialize)]
struct CurrentTenantResponse {
    #[serde(rename = "currentTenant")]
    current_tenant: CurrentTenant,
}

#[derive(Debug, Deserialize)]
struct NodesResponse {
    nodes: NodeList,
}

#[derive(Debug, Deserialize)]
struct NodeResponse {
    node: Option<NodeDetail>,
}

#[derive(Debug, Deserialize)]
struct CreateNodeResponse {
    #[serde(rename = "createNode")]
    create_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct UpdateNodeResponse {
    #[serde(rename = "updateNode")]
    update_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct PublishNodeResponse {
    #[serde(rename = "publishNode")]
    publish_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct UnpublishNodeResponse {
    #[serde(rename = "unpublishNode")]
    unpublish_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct ArchiveNodeResponse {
    #[serde(rename = "archiveNode")]
    archive_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct RestoreNodeResponse {
    #[serde(rename = "restoreNode")]
    restore_node: NodeDetail,
}

#[derive(Debug, Deserialize)]
struct DeleteNodeResponse {
    #[serde(rename = "deleteNode")]
    delete_node: bool,
}

#[derive(Debug, Serialize)]
struct TenantScopedVariables<T> {
    #[serde(rename = "tenantId")]
    tenant_id: String,
    #[serde(flatten)]
    extra: T,
}

#[derive(Debug, Serialize)]
struct NodesVariables {
    filter: NodesFilter,
}

#[derive(Debug, Serialize)]
struct NodeVariables {
    id: String,
    locale: Option<String>,
}

#[derive(Debug, Serialize)]
struct NodeIdVariables {
    id: String,
}

#[derive(Debug, Serialize)]
struct CreateNodeVariables {
    input: CreateNodeInput,
}

#[derive(Debug, Serialize)]
struct UpdateNodeVariables {
    id: String,
    input: UpdateNodeInput,
}

#[derive(Debug, Serialize)]
struct NodesFilter {
    kind: Option<String>,
    status: Option<String>,
    locale: Option<String>,
    page: Option<u64>,
    #[serde(rename = "perPage")]
    per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
struct CreateNodeInput {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    translations: Vec<NodeTranslationInput>,
    bodies: Vec<BodyInput>,
}

#[derive(Debug, Serialize)]
struct UpdateNodeInput {
    translations: Option<Vec<NodeTranslationInput>>,
    bodies: Option<Vec<BodyInput>>,
}

#[derive(Debug, Serialize)]
struct NodeTranslationInput {
    locale: String,
    title: Option<String>,
    slug: Option<String>,
    excerpt: Option<String>,
}

#[derive(Debug, Serialize)]
struct BodyInput {
    locale: String,
    body: Option<String>,
    format: Option<String>,
}

fn graphql_url() -> String {
    if let Some(url) = option_env!("RUSTOK_GRAPHQL_URL") {
        return url.to_string();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let origin = web_sys::window()
            .and_then(|window| window.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string());
        format!("{origin}/api/graphql")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base =
            std::env::var("RUSTOK_API_URL").unwrap_or_else(|_| "http://localhost:5150".to_string());
        format!("{base}/api/graphql")
    }
}

async fn request<V, T>(
    query: &str,
    variables: Option<V>,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &graphql_url(),
        GraphqlRequest::new(query, variables),
        token,
        tenant_slug,
        None,
    )
    .await
}

async fn fetch_current_tenant(
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<CurrentTenant, ApiError> {
    let response: CurrentTenantResponse = request::<serde_json::Value, CurrentTenantResponse>(
        CURRENT_TENANT_QUERY,
        None,
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.current_tenant)
}

fn tenant_scoped<T>(tenant_id: String, extra: T) -> TenantScopedVariables<T> {
    TenantScopedVariables { tenant_id, extra }
}

fn build_translation_input(draft: &NodeDraft) -> NodeTranslationInput {
    NodeTranslationInput {
        locale: draft.locale.clone(),
        title: Some(draft.title.clone()),
        slug: Some(draft.slug.clone()),
        excerpt: (!draft.excerpt.is_empty()).then_some(draft.excerpt.clone()),
    }
}

fn build_body_input(draft: &NodeDraft) -> BodyInput {
    BodyInput {
        locale: draft.locale.clone(),
        body: Some(draft.body.clone()),
        format: Some(draft.body_format.clone()),
    }
}

pub async fn fetch_nodes(
    token: Option<String>,
    tenant_slug: Option<String>,
    locale: Option<String>,
    kind: Option<String>,
) -> Result<NodeList, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: NodesResponse = request(
        NODES_QUERY,
        Some(tenant_scoped(
            tenant.id,
            NodesVariables {
                filter: NodesFilter {
                    kind,
                    status: None,
                    locale,
                    page: Some(1),
                    per_page: Some(24),
                },
            },
        )),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.nodes)
}

pub async fn fetch_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
    locale: Option<String>,
) -> Result<Option<NodeDetail>, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: NodeResponse = request(
        NODE_QUERY,
        Some(tenant_scoped(tenant.id, NodeVariables { id, locale })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.node)
}

pub async fn create_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    draft: NodeDraft,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: CreateNodeResponse = request(
        CREATE_NODE_MUTATION,
        Some(tenant_scoped(
            tenant.id,
            CreateNodeVariables {
                input: CreateNodeInput {
                    kind: draft.kind.clone(),
                    status: None,
                    translations: vec![build_translation_input(&draft)],
                    bodies: vec![build_body_input(&draft)],
                },
            },
        )),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.create_node)
}

pub async fn update_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
    draft: NodeDraft,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: UpdateNodeResponse = request(
        UPDATE_NODE_MUTATION,
        Some(tenant_scoped(
            tenant.id,
            UpdateNodeVariables {
                id,
                input: UpdateNodeInput {
                    translations: Some(vec![build_translation_input(&draft)]),
                    bodies: Some(vec![build_body_input(&draft)]),
                },
            },
        )),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.update_node)
}

pub async fn publish_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: PublishNodeResponse = request(
        PUBLISH_NODE_MUTATION,
        Some(tenant_scoped(tenant.id, NodeIdVariables { id })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.publish_node)
}

pub async fn unpublish_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: UnpublishNodeResponse = request(
        UNPUBLISH_NODE_MUTATION,
        Some(tenant_scoped(tenant.id, NodeIdVariables { id })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.unpublish_node)
}

pub async fn archive_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: ArchiveNodeResponse = request(
        ARCHIVE_NODE_MUTATION,
        Some(tenant_scoped(tenant.id, NodeIdVariables { id })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.archive_node)
}

pub async fn restore_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
) -> Result<NodeDetail, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: RestoreNodeResponse = request(
        RESTORE_NODE_MUTATION,
        Some(tenant_scoped(tenant.id, NodeIdVariables { id })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.restore_node)
}

pub async fn delete_node(
    token: Option<String>,
    tenant_slug: Option<String>,
    id: String,
) -> Result<bool, ApiError> {
    let tenant = fetch_current_tenant(token.clone(), tenant_slug.clone()).await?;
    let response: DeleteNodeResponse = request(
        DELETE_NODE_MUTATION,
        Some(tenant_scoped(tenant.id, NodeIdVariables { id })),
        token,
        tenant_slug,
    )
    .await?;
    Ok(response.delete_node)
}
