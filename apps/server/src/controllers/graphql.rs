use std::collections::BTreeSet;
use std::sync::Arc;

use async_graphql::http::{GraphQLPlaygroundConfig, WebSocketProtocols, WsMessage};
use async_graphql::parser::parse_query;
use async_graphql::parser::types::{ExecutableDocument, OperationType, Selection, SelectionSet};
use async_graphql::{Data, FieldError, Pos};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Extension, Json,
};
use futures_util::{SinkExt, StreamExt};
use loco_rs::app::AppContext;
use loco_rs::controller::Routes;
use rustok_core::i18n::Locale;
use rustok_core::Permission;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::common::RequestContext;
use crate::context::{AuthContext, TenantContext};
use crate::extractors::auth::{
    resolve_current_user_from_access_token, CurrentUser, OptionalCurrentUser,
};
use crate::graphql::errors::GraphQLError;
use crate::graphql::persisted::is_cataloged_admin_hash;
use crate::graphql::AppSchema;
use rustok_core::ModuleRegistry;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SensitiveGraphqlField {
    User,
    Users,
    CreateUser,
    UpdateUser,
    DisableUser,
    DeleteUser,
}

impl SensitiveGraphqlField {
    fn from_root_field(operation_type: OperationType, field_name: &str) -> Option<Self> {
        match (operation_type, field_name) {
            (OperationType::Query, "user") => Some(Self::User),
            (OperationType::Query, "users") => Some(Self::Users),
            (OperationType::Mutation, "createUser") => Some(Self::CreateUser),
            (OperationType::Mutation, "updateUser") => Some(Self::UpdateUser),
            (OperationType::Mutation, "disableUser") => Some(Self::DisableUser),
            (OperationType::Mutation, "deleteUser") => Some(Self::DeleteUser),
            _ => None,
        }
    }

    fn field_name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Users => "users",
            Self::CreateUser => "createUser",
            Self::UpdateUser => "updateUser",
            Self::DisableUser => "disableUser",
            Self::DeleteUser => "deleteUser",
        }
    }

    fn permission_hint(self) -> &'static str {
        match self {
            Self::User => "users:read",
            Self::Users => "users:list",
            Self::CreateUser => "users:create",
            Self::UpdateUser => "users:update",
            Self::DisableUser | Self::DeleteUser => "users:manage",
        }
    }

    fn allows(self, permissions: &[Permission]) -> bool {
        match self {
            Self::User => permissions.contains(&Permission::USERS_READ),
            Self::Users => permissions.contains(&Permission::USERS_LIST),
            Self::CreateUser => {
                permissions.contains(&Permission::USERS_CREATE)
                    || permissions.contains(&Permission::USERS_MANAGE)
            }
            Self::UpdateUser => {
                permissions.contains(&Permission::USERS_UPDATE)
                    || permissions.contains(&Permission::USERS_MANAGE)
            }
            Self::DisableUser | Self::DeleteUser => permissions.contains(&Permission::USERS_MANAGE),
        }
    }
}

fn collect_sensitive_fields_from_selection_set(
    operation_type: OperationType,
    selection_set: &SelectionSet,
    document: &ExecutableDocument,
    fields: &mut BTreeSet<SensitiveGraphqlField>,
) {
    for selection in &selection_set.items {
        match &selection.node {
            Selection::Field(field) => {
                if let Some(sensitive_field) = SensitiveGraphqlField::from_root_field(
                    operation_type,
                    field.node.name.node.as_str(),
                ) {
                    fields.insert(sensitive_field);
                }
            }
            Selection::FragmentSpread(fragment) => {
                if let Some(definition) = document.fragments.get(&fragment.node.fragment_name.node)
                {
                    collect_sensitive_fields_from_selection_set(
                        operation_type,
                        &definition.node.selection_set.node,
                        document,
                        fields,
                    );
                }
            }
            Selection::InlineFragment(fragment) => collect_sensitive_fields_from_selection_set(
                operation_type,
                &fragment.node.selection_set.node,
                document,
                fields,
            ),
        }
    }
}

fn sensitive_graphql_fields(
    query: &str,
    operation_name: Option<&str>,
) -> Result<Vec<SensitiveGraphqlField>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let document = parse_query(query).map_err(|error| error.to_string())?;
    let mut fields = BTreeSet::new();

    for (name, operation) in document.operations.iter() {
        if let Some(target_name) = operation_name {
            if name.map(|candidate| candidate.as_str()) != Some(target_name) {
                continue;
            }
        }

        collect_sensitive_fields_from_selection_set(
            operation.node.ty,
            &operation.node.selection_set.node,
            &document,
            &mut fields,
        );
    }

    Ok(fields.into_iter().collect())
}

fn unauthorized_sensitive_fields(
    fields: &[SensitiveGraphqlField],
    current_user: Option<&CurrentUser>,
) -> Vec<SensitiveGraphqlField> {
    let Some(current_user) = current_user else {
        return fields.to_vec();
    };

    fields
        .iter()
        .copied()
        .filter(|field| !field.allows(&current_user.permissions))
        .collect()
}

fn forbidden_sensitive_response(fields: &[SensitiveGraphqlField]) -> async_graphql::Response {
    let required_permissions = fields
        .iter()
        .map(|field| format!("{} -> {}", field.field_name(), field.permission_hint()))
        .collect::<Vec<_>>()
        .join(", ");
    async_graphql::Response::from_errors(vec![<FieldError as GraphQLError>::permission_denied(
        &format!("Forbidden admin GraphQL operation. Required permissions: {required_permissions}"),
    )
    .into_server_error(Pos::default())])
}

async fn graphql_handler(
    State(ctx): State<AppContext>,
    Extension(registry): Extension<ModuleRegistry>,
    Extension(schema): Extension<Arc<AppSchema>>,
    tenant_ctx: TenantContext,
    request_context: RequestContext,
    OptionalCurrentUser(current_user): OptionalCurrentUser,
    headers: HeaderMap,
    Json(req): Json<async_graphql::Request>,
) -> Json<async_graphql::Response> {
    let locale = Locale::parse(&request_context.locale).unwrap_or_default();
    if let Some(hash) = persisted_query_hash(&req) {
        tracing::debug!(
            persisted_query_hash = hash,
            cataloged_admin_hash = is_cataloged_admin_hash(hash),
            "Observed persisted query hash for GraphQL telemetry"
        );
    }

    match sensitive_graphql_fields(req.query.as_str(), req.operation_name.as_deref()) {
        Ok(sensitive_fields) if !sensitive_fields.is_empty() => {
            let denied_fields =
                unauthorized_sensitive_fields(&sensitive_fields, current_user.as_ref());
            if !denied_fields.is_empty() {
                tracing::warn!(
                    denied_fields = ?denied_fields.iter().map(|field| field.field_name()).collect::<Vec<_>>(),
                    operation_name = ?req.operation_name,
                    "Rejected sensitive GraphQL document before resolver execution"
                );
                return Json(forbidden_sensitive_response(&denied_fields));
            }
        }
        Ok(_) => {}
        Err(error) => {
            tracing::debug!(%error, "Skipping sensitive GraphQL field classification for unparsable document");
        }
    }

    let mut request = req
        .data(ctx)
        .data(tenant_ctx)
        .data(request_context)
        .data(headers)
        .data(registry)
        .data(locale);

    if let Some(current_user) = current_user {
        let auth_ctx = AuthContext {
            user_id: current_user.user.id,
            session_id: current_user.session_id,
            tenant_id: current_user.user.tenant_id,
            permissions: current_user.permissions,
            client_id: current_user.client_id,
            scopes: current_user.scopes.clone(),
            grant_type: current_user.grant_type.clone(),
        };
        request = request.data(auth_ctx);
    }

    Json(schema.execute(request).await)
}

fn persisted_query_hash(req: &async_graphql::Request) -> Option<&str> {
    use async_graphql::Value;

    let value = req.extensions.get("persistedQuery")?;
    let Value::Object(obj) = value else {
        return None;
    };
    let Value::String(hash) = obj.get("sha256Hash")? else {
        return None;
    };
    Some(hash.as_ref())
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        GraphQLPlaygroundConfig::new("/api/graphql").subscription_endpoint("/api/graphql/ws"),
    ))
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
struct GraphqlWsInitPayload {
    token: Option<String>,
    #[serde(rename = "tenantSlug", alias = "tenant_slug")]
    tenant_slug: Option<String>,
    locale: Option<String>,
}

async fn graphql_ws_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<AppContext>,
    Extension(registry): Extension<ModuleRegistry>,
    Extension(schema): Extension<Arc<AppSchema>>,
) -> impl IntoResponse {
    let ws = ws.protocols(async_graphql::http::ALL_WEBSOCKET_PROTOCOLS);
    let protocol = ws
        .selected_protocol()
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<WebSocketProtocols>().ok())
        .unwrap_or(WebSocketProtocols::GraphQLWS);

    ws.on_upgrade(move |socket| handle_graphql_ws(socket, schema, ctx, registry, protocol))
}

async fn handle_graphql_ws(
    socket: WebSocket,
    schema: Arc<AppSchema>,
    app_ctx: AppContext,
    registry: ModuleRegistry,
    protocol: WebSocketProtocols,
) {
    let (mut sink, mut source) = socket.split();
    let (incoming_tx, incoming_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let schema_for_stream = schema.as_ref().clone();
    let app_ctx_for_init = app_ctx.clone();
    let registry_for_init = registry.clone();
    let mut graphql_stream = async_graphql::http::WebSocket::new(
        schema_for_stream,
        UnboundedReceiverStream::new(incoming_rx),
        protocol,
    )
    .on_connection_init(move |payload| {
        build_ws_connection_data(app_ctx_for_init.clone(), registry_for_init.clone(), payload)
    });

    let forward_incoming = tokio::spawn(async move {
        while let Some(message) = source.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if incoming_tx.send(text.to_string()).is_err() {
                        break;
                    }
                }
                Ok(Message::Binary(bytes)) => {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        if incoming_tx.send(text).is_err() {
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Err(_) => break,
            }
        }
    });

    while let Some(message) = graphql_stream.next().await {
        let result = match message {
            WsMessage::Text(text) => sink.send(Message::Text(text.into())).await,
            WsMessage::Close(code, reason) => {
                sink.send(Message::Close(Some(CloseFrame {
                    code: code.into(),
                    reason: reason.into(),
                })))
                .await
            }
        };

        if result.is_err() {
            break;
        }
    }

    forward_incoming.abort();
}

async fn build_ws_connection_data(
    app_ctx: AppContext,
    registry: ModuleRegistry,
    payload: serde_json::Value,
) -> async_graphql::Result<Data> {
    let payload: GraphqlWsInitPayload = serde_json::from_value(payload)
        .map_err(|_| async_graphql::Error::new("Invalid connection_init payload"))?;
    let tenant_slug = payload
        .tenant_slug
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| async_graphql::Error::new("connection_init.tenantSlug is required"))?;
    let token = payload
        .token
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| async_graphql::Error::new("connection_init.token is required"))?;

    let tenant = crate::models::tenants::Entity::find_by_slug(&app_ctx.db, &tenant_slug)
        .await
        .map_err(|_| async_graphql::Error::new("Failed to resolve tenant"))?
        .ok_or_else(|| async_graphql::Error::new("Tenant not found"))?;

    if !tenant.is_enabled() {
        return Err(async_graphql::Error::new("Tenant is disabled"));
    }

    let access_token = token
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| token.trim().strip_prefix("bearer "))
        .unwrap_or(token.trim());
    let current_user = resolve_current_user_from_access_token(&app_ctx, tenant.id, access_token)
        .await
        .map_err(|(_, message)| async_graphql::Error::new(message))?;

    let locale = payload
        .locale
        .as_deref()
        .and_then(Locale::parse)
        .or_else(|| Locale::parse(&tenant.default_locale))
        .unwrap_or_default();
    let tenant_ctx = TenantContext {
        id: tenant.id,
        name: tenant.name,
        slug: tenant.slug,
        domain: tenant.domain,
        settings: tenant.settings,
        default_locale: tenant.default_locale,
        is_active: tenant.is_active,
    };
    let auth_ctx = AuthContext {
        user_id: current_user.user.id,
        session_id: current_user.session_id,
        tenant_id: current_user.user.tenant_id,
        permissions: current_user.permissions,
        client_id: current_user.client_id,
        scopes: current_user.scopes,
        grant_type: current_user.grant_type,
    };

    let mut data = Data::default();
    data.insert(app_ctx);
    data.insert(registry);
    data.insert(locale);
    data.insert(tenant_ctx);
    data.insert(auth_ctx);
    Ok(data)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/graphql")
        .add("/", get(graphql_playground).post(graphql_handler))
        .add("/ws", get(graphql_ws_handler))
}

#[cfg(test)]
mod tests {
    use super::{
        sensitive_graphql_fields, unauthorized_sensitive_fields, CurrentUser, SensitiveGraphqlField,
    };
    use crate::models::users;
    use chrono::Utc;
    use rustok_core::{Permission, UserRole, UserStatus};
    use serde_json::json;
    use uuid::Uuid;

    fn current_user_with_permissions(permissions: Vec<Permission>) -> CurrentUser {
        CurrentUser {
            user: users::Model {
                id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                email: "admin@example.com".to_string(),
                password_hash: "hash".to_string(),
                name: Some("Admin".to_string()),
                status: UserStatus::Active,
                email_verified_at: Some(Utc::now().into()),
                last_login_at: None,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
                metadata: json!({}),
            },
            session_id: Uuid::new_v4(),
            permissions,
            inferred_role: UserRole::Admin,
            client_id: None,
            scopes: Vec::new(),
            grant_type: "password".to_string(),
        }
    }

    #[test]
    fn classifies_sensitive_fields_independently_of_operation_name() {
        let query = r#"
            query TotallyDifferentName {
                users {
                    edges {
                        node {
                            id
                        }
                    }
                }
            }
        "#;

        let fields = sensitive_graphql_fields(query, Some("TotallyDifferentName"))
            .expect("query should parse");

        assert_eq!(fields, vec![SensitiveGraphqlField::Users]);
    }

    #[test]
    fn classifies_sensitive_fields_inside_root_fragments() {
        let query = r#"
            query UserAdmin {
                ...RootFields
            }

            fragment RootFields on Query {
                user(id: "00000000-0000-0000-0000-000000000000") {
                    id
                }
            }
        "#;

        let fields =
            sensitive_graphql_fields(query, Some("UserAdmin")).expect("query should parse");

        assert_eq!(fields, vec![SensitiveGraphqlField::User]);
    }

    #[test]
    fn rejects_sensitive_fields_without_matching_permissions() {
        let query = r#"
            mutation RenameUser {
                updateUser(
                    id: "00000000-0000-0000-0000-000000000000"
                    input: { name: "Updated" }
                ) {
                    id
                }
            }
        "#;
        let fields =
            sensitive_graphql_fields(query, Some("RenameUser")).expect("query should parse");
        let current_user = current_user_with_permissions(vec![Permission::USERS_READ]);

        let denied = unauthorized_sensitive_fields(&fields, Some(&current_user));

        assert_eq!(denied, vec![SensitiveGraphqlField::UpdateUser]);
    }

    #[test]
    fn allows_sensitive_fields_with_matching_permissions() {
        let query = r#"
            mutation CreateAccount {
                createUser(
                    input: {
                        email: "user@example.com"
                        password: "password"
                    }
                ) {
                    id
                }
            }
        "#;
        let fields =
            sensitive_graphql_fields(query, Some("CreateAccount")).expect("query should parse");
        let current_user = current_user_with_permissions(vec![Permission::USERS_MANAGE]);

        let denied = unauthorized_sensitive_fields(&fields, Some(&current_user));

        assert!(denied.is_empty());
    }

    #[test]
    fn rejects_sensitive_fields_when_request_is_unauthenticated() {
        let query = r#"
            query ListUsers {
                users {
                    edges {
                        node {
                            id
                        }
                    }
                }
            }
        "#;
        let fields =
            sensitive_graphql_fields(query, Some("ListUsers")).expect("query should parse");

        let denied = unauthorized_sensitive_fields(&fields, None);

        assert_eq!(denied, vec![SensitiveGraphqlField::Users]);
    }
}
