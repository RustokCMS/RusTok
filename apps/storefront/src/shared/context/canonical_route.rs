use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::shared::api::{configured_tenant_slug, request, ApiError};

const RESOLVE_CANONICAL_ROUTE_QUERY: &str = r#"
    query ResolveCanonicalRoute($route: String!, $locale: String!) {
        resolveCanonicalRoute(route: $route, locale: $locale) {
            targetKind
            targetId
            locale
            matchedUrl
            canonicalUrl
            redirectRequired
        }
    }
"#;

#[derive(Debug, Clone, Serialize)]
struct ResolveCanonicalRouteVariables {
    route: String,
    locale: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ResolveCanonicalRouteResponse {
    #[serde(rename = "resolveCanonicalRoute")]
    resolved: Option<ResolvedCanonicalRoute>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolvedCanonicalRoute {
    #[serde(rename = "targetKind")]
    pub target_kind: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
    pub locale: String,
    #[serde(rename = "matchedUrl")]
    pub matched_url: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    #[serde(rename = "redirectRequired")]
    pub redirect_required: bool,
}

pub async fn fetch_canonical_route(
    locale: &str,
    route_segment: &str,
    query_params: &HashMap<String, String>,
) -> Result<Option<ResolvedCanonicalRoute>, ApiError> {
    let Some(tenant_slug) = configured_tenant_slug() else {
        return Ok(None);
    };

    let route = build_module_route(route_segment, query_params);
    let response: ResolveCanonicalRouteResponse = request(
        RESOLVE_CANONICAL_ROUTE_QUERY,
        ResolveCanonicalRouteVariables {
            route,
            locale: locale.to_string(),
        },
        None,
        Some(tenant_slug),
    )
    .await?;

    Ok(response.resolved)
}

pub fn build_redirect_location(
    resolved: &ResolvedCanonicalRoute,
    query_params: &HashMap<String, String>,
) -> String {
    let Some(lang) = query_params.get("lang") else {
        return resolved.canonical_url.clone();
    };

    let normalized_lang = lang.trim().to_lowercase();
    if normalized_lang.is_empty() {
        return resolved.canonical_url.clone();
    }

    let lang_suffix = serde_urlencoded::to_string([("lang", normalized_lang.as_str())])
        .expect("serializing lang query for redirect should not fail");

    if resolved.canonical_url.contains('?') {
        format!("{}&{}", resolved.canonical_url, lang_suffix)
    } else {
        format!("{}?{}", resolved.canonical_url, lang_suffix)
    }
}

fn build_module_route(route_segment: &str, query_params: &HashMap<String, String>) -> String {
    let base = format!("/modules/{route_segment}");
    let filtered = query_params
        .iter()
        .filter(|(key, _)| key.as_str() != "lang")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();

    if filtered.is_empty() {
        return base;
    }

    let query = serde_urlencoded::to_string(filtered)
        .expect("serializing module route query should not fail");
    format!("{base}?{query}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_route_excludes_lang_and_sorts_query_keys() {
        let query_params = HashMap::from([
            ("lang".to_string(), "ru".to_string()),
            ("topic".to_string(), "42".to_string()),
            ("view".to_string(), "full".to_string()),
        ]);

        let route = build_module_route("forum", &query_params);
        assert_eq!(route, "/modules/forum?topic=42&view=full");
    }

    #[test]
    fn redirect_location_preserves_explicit_lang_query() {
        let resolved = ResolvedCanonicalRoute {
            target_kind: "blog_post".to_string(),
            target_id: "123".to_string(),
            locale: "ru".to_string(),
            matched_url: "/modules/forum?topic=1".to_string(),
            canonical_url: "/modules/blog?slug=release".to_string(),
            redirect_required: true,
        };
        let query_params = HashMap::from([("lang".to_string(), "RU".to_string())]);

        let redirect = build_redirect_location(&resolved, &query_params);
        assert_eq!(redirect, "/modules/blog?slug=release&lang=ru");
    }
}
