use rustok_core::{Error, Result};

use crate::ranking::SearchRankingProfile;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SearchFilterPreset {
    pub key: String,
    pub label: String,
    pub entity_types: Vec<String>,
    pub source_modules: Vec<String>,
    pub statuses: Vec<String>,
    pub ranking_profile: Option<SearchRankingProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSearchFilterPreset {
    pub preset: Option<SearchFilterPreset>,
    pub entity_types: Vec<String>,
    pub source_modules: Vec<String>,
    pub statuses: Vec<String>,
    pub ranking_profile: Option<SearchRankingProfile>,
}

pub struct SearchFilterPresetService;

impl SearchFilterPresetService {
    pub fn list(config: &serde_json::Value, surface: &str) -> Vec<SearchFilterPreset> {
        config
            .get("filter_presets")
            .and_then(|value| value.get(surface).or_else(|| value.get("default")))
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| parse_preset(item).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn resolve(
        config: &serde_json::Value,
        surface: &str,
        preset_key: Option<&str>,
        entity_types: Vec<String>,
        source_modules: Vec<String>,
        statuses: Vec<String>,
    ) -> Result<ResolvedSearchFilterPreset> {
        let presets = Self::list(config, surface);
        let requested_key = preset_key
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase());

        let preset = match requested_key {
            Some(ref key) => presets
                .into_iter()
                .find(|preset| preset.key == *key)
                .ok_or_else(|| Error::Validation(format!("Unknown filter preset '{}'", key)))?
                .into(),
            None => None,
        };

        let resolved = match preset.clone() {
            Some(preset) => ResolvedSearchFilterPreset {
                ranking_profile: preset.ranking_profile,
                entity_types: if entity_types.is_empty() {
                    preset.entity_types.clone()
                } else {
                    entity_types
                },
                source_modules: if source_modules.is_empty() {
                    preset.source_modules.clone()
                } else {
                    source_modules
                },
                statuses: if statuses.is_empty() {
                    preset.statuses.clone()
                } else {
                    statuses
                },
                preset: Some(preset),
            },
            None => ResolvedSearchFilterPreset {
                preset: None,
                entity_types,
                source_modules,
                statuses,
                ranking_profile: None,
            },
        };

        Ok(resolved)
    }
}

fn parse_preset(value: &serde_json::Value) -> Result<SearchFilterPreset> {
    let key = value
        .get("key")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::Validation("filter preset is missing key".to_string()))?
        .to_ascii_lowercase();
    let label = value
        .get("label")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&key)
        .to_string();

    Ok(SearchFilterPreset {
        key,
        label,
        entity_types: parse_string_array(value.get("entity_types")),
        source_modules: parse_string_array(value.get("source_modules")),
        statuses: parse_string_array(value.get("statuses")),
        ranking_profile: value
            .get("ranking_profile")
            .and_then(serde_json::Value::as_str)
            .and_then(SearchRankingProfile::try_from_str),
    })
}

fn parse_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::SearchFilterPresetService;
    use crate::SearchRankingProfile;

    #[test]
    fn resolve_uses_preset_defaults_when_explicit_filters_are_empty() {
        let config = serde_json::json!({
            "filter_presets": {
                "storefront_search": [
                    {
                        "key": "products",
                        "label": "Products",
                        "entity_types": ["product"],
                        "source_modules": ["commerce"],
                        "ranking_profile": "catalog"
                    }
                ]
            }
        });

        let resolved = SearchFilterPresetService::resolve(
            &config,
            "storefront_search",
            Some("products"),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("preset should resolve");

        assert_eq!(resolved.entity_types, vec!["product".to_string()]);
        assert_eq!(resolved.source_modules, vec!["commerce".to_string()]);
        assert_eq!(
            resolved.ranking_profile,
            Some(SearchRankingProfile::Catalog)
        );
    }

    #[test]
    fn resolve_keeps_explicit_filters_over_preset_values() {
        let config = serde_json::json!({
            "filter_presets": {
                "search_preview": [
                    {
                        "key": "content",
                        "label": "Content",
                        "entity_types": ["node"]
                    }
                ]
            }
        });

        let resolved = SearchFilterPresetService::resolve(
            &config,
            "search_preview",
            Some("content"),
            vec!["product".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .expect("preset should resolve");

        assert_eq!(resolved.entity_types, vec!["product".to_string()]);
    }
}
