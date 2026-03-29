use axum::{extract::State, routing::get, Json};
use loco_rs::app::AppContext;
use loco_rs::controller::Routes;

use crate::error::Error;
use crate::modules::{CatalogManifestModule, ManifestManager, ModulesManifest};
use crate::services::marketplace_catalog::{
    legacy_registry_catalog_path, registry_catalog_from_modules, registry_catalog_path,
    RegistryCatalogResponse,
};

/// GET /v1/catalog - Reference read-only marketplace registry catalog
#[utoipa::path(
    get,
    path = "/v1/catalog",
    tag = "marketplace",
    responses(
        (
            status = 200,
            description = "Schema-versioned reference catalog of first-party modules",
            body = RegistryCatalogResponse
        )
    )
)]
async fn catalog(State(_ctx): State<AppContext>) -> Result<Json<RegistryCatalogResponse>, Error> {
    let manifest = ManifestManager::load().unwrap_or_else(|error| {
        tracing::warn!(
            error = %error,
            "Failed to load modules manifest for registry catalog; falling back to builtin catalog"
        );
        ModulesManifest::default()
    });
    let modules = catalog_modules_with_builtin_fallback(&manifest)
        .map_err(|error| Error::Message(format!("Failed to build marketplace catalog: {error}")))?;
    let first_party_modules = modules
        .into_iter()
        .filter(|module| module.ownership == "first_party")
        .collect();

    Ok(Json(registry_catalog_from_modules(first_party_modules)))
}

pub fn routes() -> Routes {
    Routes::new()
        .add(registry_catalog_path(), get(catalog))
        .add(legacy_registry_catalog_path(), get(catalog))
}

fn catalog_modules_with_builtin_fallback(
    manifest: &ModulesManifest,
) -> Result<Vec<CatalogManifestModule>, crate::modules::ManifestError> {
    match ManifestManager::catalog_modules(manifest) {
        Ok(modules) => Ok(modules),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "Registry catalog generation fell back to builtin first-party module catalog"
            );
            ManifestManager::catalog_modules(&ModulesManifest::default())
        }
    }
}
