use super::*;

pub(crate) fn validate_module_ui_classification_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
) -> Result<()> {
    let explicit = manifest.module.ui_classification.trim();
    if explicit.is_empty() {
        anyhow::bail!("Module '{slug}' is missing module.ui_classification in rustok-module.toml");
    }

    let normalized = normalize_module_ui_classification(explicit).with_context(|| {
        format!("Module '{slug}' has invalid module.ui_classification '{explicit}'")
    })?;
    let has_admin_ui = manifest.provides.admin_ui.is_some();
    let has_storefront_ui = manifest.provides.storefront_ui.is_some();
    let derived = catalog_module_ui_classification(has_admin_ui, has_storefront_ui);
    let matches_surface_contract = match normalized.as_str() {
        "dual_surface" => has_admin_ui && has_storefront_ui,
        "admin_only" => has_admin_ui && !has_storefront_ui,
        "storefront_only" => !has_admin_ui && has_storefront_ui,
        "no_ui" | "capability_only" | "future_ui" => !has_admin_ui && !has_storefront_ui,
        _ => false,
    };

    if !matches_surface_contract {
        anyhow::bail!(
            "Module '{slug}' has module.ui_classification='{}' but manifest UI surfaces resolve to '{}'",
            explicit,
            derived
        );
    }

    Ok(())
}

pub(crate) fn validate_module_ui_metadata_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
) -> Result<()> {
    if let Some(admin_ui) = manifest.provides.admin_ui.as_ref() {
        validate_ui_surface_metadata_field(
            slug,
            "provides.admin_ui.route_segment",
            admin_ui.route_segment.as_deref(),
        )?;
        validate_ui_surface_metadata_field(
            slug,
            "provides.admin_ui.nav_label",
            admin_ui.nav_label.as_deref(),
        )?;
        validate_ui_i18n_contract(slug, "provides.admin_ui.i18n", admin_ui.i18n.as_ref())?;
    }

    if let Some(storefront_ui) = manifest.provides.storefront_ui.as_ref() {
        validate_ui_surface_metadata_field(
            slug,
            "provides.storefront_ui.route_segment",
            storefront_ui.route_segment.as_deref(),
        )?;
        validate_ui_surface_metadata_field(
            slug,
            "provides.storefront_ui.slot",
            storefront_ui.slot.as_deref(),
        )?;
        validate_ui_surface_metadata_field(
            slug,
            "provides.storefront_ui.page_title",
            storefront_ui.page_title.as_deref(),
        )?;
        validate_ui_i18n_contract(
            slug,
            "provides.storefront_ui.i18n",
            storefront_ui.i18n.as_ref(),
        )?;
    }

    Ok(())
}

pub(crate) fn normalize_module_ui_classification(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "dual_surface" | "admin_only" | "storefront_only" | "no_ui" | "capability_only"
        | "future_ui" => Ok(normalized),
        _ => anyhow::bail!("unsupported value"),
    }
}

fn validate_ui_surface_metadata_field(
    slug: &str,
    field_name: &str,
    value: Option<&str>,
) -> Result<()> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        anyhow::bail!("Module '{slug}' must declare non-empty {field_name}");
    };
    if value.contains('\\') {
        anyhow::bail!("Module '{slug}' declares invalid {field_name}='{value}'");
    }
    Ok(())
}

fn validate_ui_i18n_contract(
    slug: &str,
    field_prefix: &str,
    i18n: Option<&ModuleUiI18nProvides>,
) -> Result<()> {
    let Some(i18n) = i18n else {
        anyhow::bail!("Module '{slug}' must declare [{field_prefix}]");
    };

    let default_locale = i18n
        .default_locale
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .with_context(|| {
            format!("Module '{slug}' must declare non-empty {field_prefix}.default_locale")
        })?;
    if !i18n
        .supported_locales
        .iter()
        .map(|locale| locale.trim())
        .any(|locale| locale == default_locale)
    {
        anyhow::bail!(
            "Module '{slug}' must include {field_prefix}.default_locale='{default_locale}' in {field_prefix}.supported_locales"
        );
    }

    let locales_path = i18n
        .leptos_locales_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .with_context(|| {
            format!("Module '{slug}' must declare non-empty {field_prefix}.leptos_locales_path")
        })?;
    if locales_path.contains('\\') {
        anyhow::bail!(
            "Module '{slug}' declares invalid {field_prefix}.leptos_locales_path='{locales_path}'"
        );
    }

    Ok(())
}

fn catalog_module_ui_classification(has_admin_ui: bool, has_storefront_ui: bool) -> &'static str {
    match (has_admin_ui, has_storefront_ui) {
        (true, true) => "dual_surface",
        (true, false) => "admin_only",
        (false, true) => "storefront_only",
        (false, false) => "no_ui",
    }
}
