use crate::entities::oauth_app::model::{AppType, OAuthApp};
use crate::shared::api::{request, ApiError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const OAUTH_APPS_QUERY: &str = r#"
query OAuthApps($limit: Int) {
  oauthApps(limit: $limit) {
    id
    name
    slug
    description
    iconUrl
    appType
    clientId
    redirectUris
    scopes
    grantTypes
    manifestRef
    autoCreated
    managedByManifest
    isActive
    canEdit
    canRotateSecret
    canRevoke
    activeTokenCount
    lastUsedAt
    createdAt
  }
}
"#;

pub const CREATE_OAUTH_APP_MUTATION: &str = r#"
mutation CreateOAuthApp($input: CreateOAuthAppInput!) {
  createOAuthApp(input: $input) {
    app {
      id
      name
      slug
      description
      iconUrl
      appType
      clientId
      redirectUris
      scopes
      grantTypes
      manifestRef
      autoCreated
      managedByManifest
      isActive
      canEdit
      canRotateSecret
      canRevoke
      activeTokenCount
      lastUsedAt
      createdAt
    }
    clientSecret
  }
}
"#;

pub const UPDATE_OAUTH_APP_MUTATION: &str = r#"
mutation UpdateOAuthApp($id: UUID!, $input: UpdateOAuthAppInput!) {
  updateOAuthApp(id: $id, input: $input) {
    id
    name
    slug
    description
    iconUrl
    appType
    clientId
    redirectUris
    scopes
    grantTypes
    manifestRef
    autoCreated
    managedByManifest
    isActive
    canEdit
    canRotateSecret
    canRevoke
    activeTokenCount
    lastUsedAt
    createdAt
  }
}
"#;

pub const ROTATE_OAUTH_APP_SECRET_MUTATION: &str = r#"
mutation RotateOAuthAppSecret($id: UUID!) {
  rotateOAuthAppSecret(id: $id) {
    app {
      id
      name
      slug
      description
      iconUrl
      appType
      clientId
      redirectUris
      scopes
      grantTypes
      manifestRef
      autoCreated
      managedByManifest
      isActive
      canEdit
      canRotateSecret
      canRevoke
      activeTokenCount
      lastUsedAt
      createdAt
    }
    clientSecret
  }
}
"#;

pub const REVOKE_OAUTH_APP_MUTATION: &str = r#"
mutation RevokeOAuthApp($id: UUID!) {
  revokeOAuthApp(id: $id) {
    id
  }
}
"#;

#[derive(Clone, Debug, Default, Serialize)]
pub struct OAuthAppsVariables {
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthAppsResponse {
    #[serde(rename = "oauthApps")]
    pub oauth_apps: Vec<OAuthApp>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOAuthAppVariables {
    pub input: CreateOAuthAppInput,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthAppInput {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub app_type: AppType,
    pub redirect_uris: Option<Vec<String>>,
    pub scopes: Vec<String>,
    pub grant_types: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateOAuthAppResponse {
    #[serde(rename = "createOAuthApp")]
    pub create_oauth_app: CreateOAuthAppResult,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthAppResult {
    pub app: OAuthApp,
    pub client_secret: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateOAuthAppVariables {
    pub id: Uuid,
    pub input: UpdateOAuthAppInput,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOAuthAppInput {
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub grant_types: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateOAuthAppResponse {
    #[serde(rename = "updateOAuthApp")]
    pub update_oauth_app: OAuthApp,
}

#[derive(Clone, Debug, Serialize)]
pub struct OAuthAppIdVariables {
    pub id: Uuid,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RotateOAuthAppSecretResponse {
    #[serde(rename = "rotateOAuthAppSecret")]
    pub rotate_oauth_app_secret: CreateOAuthAppResult,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RevokeOAuthAppResponse {
    #[serde(rename = "revokeOAuthApp")]
    pub _revoke_oauth_app: RevokeOAuthAppPayload,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RevokeOAuthAppPayload {
    pub id: Uuid,
}

pub async fn list_oauth_apps(
    token: Option<String>,
    tenant: Option<String>,
) -> Result<Vec<OAuthApp>, ApiError> {
    let response = request::<OAuthAppsVariables, OAuthAppsResponse>(
        OAUTH_APPS_QUERY,
        OAuthAppsVariables { limit: Some(100) },
        token,
        tenant,
    )
    .await?;

    Ok(response.oauth_apps)
}
