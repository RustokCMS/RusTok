use rustok_core::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRankingProfile {
    Balanced,
    Exact,
    Fresh,
    Catalog,
    Content,
}

impl SearchRankingProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Exact => "exact",
            Self::Fresh => "fresh",
            Self::Catalog => "catalog",
            Self::Content => "content",
        }
    }

    pub fn try_from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "balanced" => Some(Self::Balanced),
            "exact" => Some(Self::Exact),
            "fresh" => Some(Self::Fresh),
            "catalog" => Some(Self::Catalog),
            "content" => Some(Self::Content),
            _ => None,
        }
    }

    pub fn resolve(
        config: &serde_json::Value,
        surface: &str,
        requested: Option<&str>,
        preset_profile: Option<SearchRankingProfile>,
    ) -> Result<Self> {
        if let Some(requested) = requested.map(str::trim).filter(|value| !value.is_empty()) {
            return Self::try_from_str(requested).ok_or_else(|| {
                Error::Validation(format!(
                    "Unsupported ranking profile '{}'. Expected one of: balanced, exact, fresh, catalog, content",
                    requested
                ))
            });
        }

        if let Some(preset_profile) = preset_profile {
            return Ok(preset_profile);
        }

        Ok(lookup_config_profile(config, surface)
            .unwrap_or_else(|| Self::default_for_surface(surface)))
    }

    pub fn default_for_surface(surface: &str) -> Self {
        match surface {
            "admin_global_search" => Self::Exact,
            "storefront_search" => Self::Balanced,
            "search_preview" => Self::Balanced,
            _ => Self::Balanced,
        }
    }

    pub fn fts_score_sql(&self) -> &'static str {
        match self {
            Self::Balanced => {
                r#"
                ts_rank_cd(sd.search_vector, q.ts_query)
                + CASE
                    WHEN lower(sd.title) = $3 THEN 2.2
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 1.1
                    WHEN lower(COALESCE(sd.slug, '')) = replace($3, ' ', '-') THEN 0.9
                    ELSE 0.0
                  END
                + CASE
                    WHEN sd.is_public THEN 0.05
                    ELSE 0.0
                  END
                "#
            }
            Self::Exact => {
                r#"
                ts_rank_cd(sd.search_vector, q.ts_query)
                + CASE
                    WHEN lower(sd.title) = $3 THEN 4.0
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 2.4
                    WHEN lower(COALESCE(sd.slug, '')) = replace($3, ' ', '-') THEN 1.8
                    WHEN lower(COALESCE(sd.handle, '')) = replace($3, ' ', '-') THEN 1.8
                    ELSE 0.0
                  END
                "#
            }
            Self::Fresh => {
                r#"
                ts_rank_cd(sd.search_vector, q.ts_query)
                + LEAST(
                    1.5,
                    GREATEST(
                        0.0,
                        2592000.0 - EXTRACT(EPOCH FROM (NOW() - COALESCE(sd.published_at, sd.updated_at)))
                    ) / 2592000.0
                  )
                + CASE
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 0.6
                    ELSE 0.0
                  END
                "#
            }
            Self::Catalog => {
                r#"
                ts_rank_cd(sd.search_vector, q.ts_query)
                + CASE
                    WHEN sd.entity_type = 'product' THEN 1.8
                    ELSE 0.0
                  END
                + CASE
                    WHEN lower(sd.title) = $3 THEN 2.0
                    WHEN lower(COALESCE(sd.handle, '')) = replace($3, ' ', '-') THEN 1.5
                    WHEN lower(COALESCE(sd.handle, '')) LIKE ($3 || '%') THEN 1.0
                    ELSE 0.0
                  END
                "#
            }
            Self::Content => {
                r#"
                ts_rank_cd(sd.search_vector, q.ts_query)
                + CASE
                    WHEN sd.entity_type = 'node' THEN 1.6
                    ELSE 0.0
                  END
                + CASE
                    WHEN lower(sd.title) = $3 THEN 1.8
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 1.0
                    ELSE 0.0
                  END
                + LEAST(
                    1.0,
                    GREATEST(
                        0.0,
                        5184000.0 - EXTRACT(EPOCH FROM (NOW() - COALESCE(sd.published_at, sd.updated_at)))
                    ) / 5184000.0
                  )
                "#
            }
        }
    }

    pub fn typo_score_sql(&self) -> &'static str {
        match self {
            Self::Balanced => {
                r#"
                GREATEST(
                    similarity(lower(sd.title), $3),
                    similarity(lower(COALESCE(sd.slug, '')), $3),
                    similarity(lower(COALESCE(sd.handle, '')), $3),
                    similarity(lower(COALESCE(sd.keywords_text, '')), $3)
                )
                + CASE
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 0.18
                    ELSE 0.0
                  END
                "#
            }
            Self::Exact => {
                r#"
                GREATEST(
                    similarity(lower(sd.title), $3),
                    similarity(lower(COALESCE(sd.slug, '')), $3),
                    similarity(lower(COALESCE(sd.handle, '')), $3)
                )
                + CASE
                    WHEN lower(sd.title) LIKE ($3 || '%') THEN 0.28
                    WHEN lower(COALESCE(sd.slug, '')) LIKE ($3 || '%') THEN 0.24
                    WHEN lower(COALESCE(sd.handle, '')) LIKE ($3 || '%') THEN 0.24
                    ELSE 0.0
                  END
                "#
            }
            Self::Fresh => {
                r#"
                GREATEST(
                    similarity(lower(sd.title), $3),
                    similarity(lower(COALESCE(sd.slug, '')), $3),
                    similarity(lower(COALESCE(sd.handle, '')), $3),
                    similarity(lower(COALESCE(sd.keywords_text, '')), $3)
                )
                + LEAST(
                    0.35,
                    GREATEST(
                        0.0,
                        2592000.0 - EXTRACT(EPOCH FROM (NOW() - COALESCE(sd.published_at, sd.updated_at)))
                    ) / 2592000.0 * 0.35
                  )
                "#
            }
            Self::Catalog => {
                r#"
                GREATEST(
                    similarity(lower(sd.title), $3),
                    similarity(lower(COALESCE(sd.handle, '')), $3),
                    similarity(lower(COALESCE(sd.slug, '')), $3),
                    similarity(lower(COALESCE(sd.keywords_text, '')), $3)
                )
                + CASE
                    WHEN sd.entity_type = 'product' THEN 0.25
                    ELSE 0.0
                  END
                "#
            }
            Self::Content => {
                r#"
                GREATEST(
                    similarity(lower(sd.title), $3),
                    similarity(lower(COALESCE(sd.slug, '')), $3),
                    similarity(lower(COALESCE(sd.keywords_text, '')), $3)
                )
                + CASE
                    WHEN sd.entity_type = 'node' THEN 0.22
                    ELSE 0.0
                  END
                "#
            }
        }
    }
}

fn lookup_config_profile(
    config: &serde_json::Value,
    surface: &str,
) -> Option<SearchRankingProfile> {
    let ranking_profiles = config.get("ranking_profiles")?;

    ranking_profiles
        .get(surface)
        .and_then(serde_json::Value::as_str)
        .and_then(SearchRankingProfile::try_from_str)
        .or_else(|| {
            ranking_profiles
                .get("default")
                .and_then(serde_json::Value::as_str)
                .and_then(SearchRankingProfile::try_from_str)
        })
}

#[cfg(test)]
mod tests {
    use super::SearchRankingProfile;

    #[test]
    fn resolve_prefers_requested_profile() {
        let config = serde_json::json!({
            "ranking_profiles": {
                "default": "fresh",
                "storefront_search": "catalog"
            }
        });

        let profile =
            SearchRankingProfile::resolve(&config, "storefront_search", Some("exact"), None)
                .expect("requested profile should parse");
        assert_eq!(profile, SearchRankingProfile::Exact);
    }

    #[test]
    fn resolve_falls_back_to_surface_or_default_profile() {
        let config = serde_json::json!({
            "ranking_profiles": {
                "storefront_search": "catalog"
            }
        });

        assert_eq!(
            SearchRankingProfile::resolve(&config, "storefront_search", None, None).unwrap(),
            SearchRankingProfile::Catalog
        );
        assert_eq!(
            SearchRankingProfile::resolve(
                &serde_json::json!({}),
                "admin_global_search",
                None,
                None
            )
            .unwrap(),
            SearchRankingProfile::Exact
        );
    }

    #[test]
    fn resolve_uses_preset_profile_when_requested_profile_is_missing() {
        let profile = SearchRankingProfile::resolve(
            &serde_json::json!({}),
            "storefront_search",
            None,
            Some(SearchRankingProfile::Catalog),
        )
        .unwrap();

        assert_eq!(profile, SearchRankingProfile::Catalog);
    }
}
