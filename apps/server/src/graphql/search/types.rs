use async_graphql::{InputObject, SimpleObject};
use rustok_search::{
    LaggingSearchDocument, SearchConnectorDescriptor, SearchDiagnosticsSnapshot, SearchResult,
    SearchResultItem, SearchSettingsRecord,
};

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchEngineDescriptor {
    pub kind: String,
    pub label: String,
    pub provided_by: String,
    pub enabled: bool,
    pub default_engine: bool,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchSettingsPayload {
    pub tenant_id: Option<String>,
    pub active_engine: String,
    pub fallback_engine: String,
    pub config: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct UpdateSearchSettingsInput {
    pub tenant_id: Option<String>,
    pub active_engine: String,
    pub fallback_engine: Option<String>,
    pub config: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateSearchSettingsPayload {
    pub success: bool,
    pub settings: SearchSettingsPayload,
}

#[derive(Debug, Clone, InputObject)]
pub struct TriggerSearchRebuildInput {
    pub tenant_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TriggerSearchRebuildPayload {
    pub success: bool,
    pub queued: bool,
    pub tenant_id: String,
    pub target_type: String,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, InputObject)]
pub struct SearchPreviewInput {
    pub query: String,
    pub locale: Option<String>,
    pub tenant_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub entity_types: Option<Vec<String>>,
    pub source_modules: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchFacetBucketPayload {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchFacetGroupPayload {
    pub name: String,
    pub buckets: Vec<SearchFacetBucketPayload>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchPreviewResultItem {
    pub id: String,
    pub entity_type: String,
    pub source_module: String,
    pub title: String,
    pub snippet: Option<String>,
    pub score: f64,
    pub locale: Option<String>,
    pub payload: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchPreviewPayload {
    pub items: Vec<SearchPreviewResultItem>,
    pub total: u64,
    pub took_ms: u64,
    pub engine: String,
    pub facets: Vec<SearchFacetGroupPayload>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SearchDiagnosticsPayload {
    pub tenant_id: String,
    pub total_documents: u64,
    pub public_documents: u64,
    pub content_documents: u64,
    pub product_documents: u64,
    pub stale_documents: u64,
    pub newest_indexed_at: Option<String>,
    pub oldest_indexed_at: Option<String>,
    pub max_lag_seconds: u64,
    pub state: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct LaggingSearchDocumentPayload {
    pub document_key: String,
    pub document_id: String,
    pub source_module: String,
    pub entity_type: String,
    pub locale: String,
    pub status: String,
    pub is_public: bool,
    pub title: String,
    pub updated_at: String,
    pub indexed_at: String,
    pub lag_seconds: u64,
}

impl From<SearchConnectorDescriptor> for SearchEngineDescriptor {
    fn from(value: SearchConnectorDescriptor) -> Self {
        Self {
            kind: value.kind.as_str().to_string(),
            label: value.label,
            provided_by: value.provided_by,
            enabled: value.enabled,
            default_engine: value.default_engine,
        }
    }
}

impl From<SearchSettingsRecord> for SearchSettingsPayload {
    fn from(value: SearchSettingsRecord) -> Self {
        Self {
            tenant_id: value.tenant_id.map(|tenant_id| tenant_id.to_string()),
            active_engine: value.active_engine.as_str().to_string(),
            fallback_engine: value.fallback_engine.as_str().to_string(),
            config: value.config.to_string(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

impl From<SearchResultItem> for SearchPreviewResultItem {
    fn from(value: SearchResultItem) -> Self {
        Self {
            id: value.id.to_string(),
            entity_type: value.entity_type,
            source_module: value.source_module,
            title: value.title,
            snippet: value.snippet,
            score: value.score,
            locale: value.locale,
            payload: value.payload.to_string(),
        }
    }
}

impl From<SearchResult> for SearchPreviewPayload {
    fn from(value: SearchResult) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            total: value.total,
            took_ms: value.took_ms,
            engine: value.engine.as_str().to_string(),
            facets: value
                .facets
                .into_iter()
                .map(|facet| SearchFacetGroupPayload {
                    name: facet.name,
                    buckets: facet
                        .buckets
                        .into_iter()
                        .map(|bucket| SearchFacetBucketPayload {
                            value: bucket.value,
                            count: bucket.count,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<SearchDiagnosticsSnapshot> for SearchDiagnosticsPayload {
    fn from(value: SearchDiagnosticsSnapshot) -> Self {
        Self {
            tenant_id: value.tenant_id.to_string(),
            total_documents: value.total_documents,
            public_documents: value.public_documents,
            content_documents: value.content_documents,
            product_documents: value.product_documents,
            stale_documents: value.stale_documents,
            newest_indexed_at: value.newest_indexed_at.map(|value| value.to_rfc3339()),
            oldest_indexed_at: value.oldest_indexed_at.map(|value| value.to_rfc3339()),
            max_lag_seconds: value.max_lag_seconds,
            state: value.state,
        }
    }
}

impl From<LaggingSearchDocument> for LaggingSearchDocumentPayload {
    fn from(value: LaggingSearchDocument) -> Self {
        Self {
            document_key: value.document_key,
            document_id: value.document_id.to_string(),
            source_module: value.source_module,
            entity_type: value.entity_type,
            locale: value.locale,
            status: value.status,
            is_public: value.is_public,
            title: value.title,
            updated_at: value.updated_at.to_rfc3339(),
            indexed_at: value.indexed_at.to_rfc3339(),
            lag_seconds: value.lag_seconds,
        }
    }
}
