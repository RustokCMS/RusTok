use super::*;

pub(crate) fn manifest_path() -> PathBuf {
    PathBuf::from("modules.toml")
}

pub(crate) fn load_manifest() -> Result<Manifest> {
    load_manifest_from(&manifest_path())
}

pub(crate) fn load_manifest_from(path: &Path) -> Result<Manifest> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let manifest: Manifest =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

    if manifest.schema != 2 {
        anyhow::bail!("Unsupported manifest schema: {}", manifest.schema);
    }

    Ok(manifest)
}

pub(crate) fn module_package_manifest_path(
    manifest_path: &Path,
    spec: &ModuleSpec,
) -> Option<PathBuf> {
    if spec.source != "path" {
        return None;
    }

    let module_path = spec.path.as_ref()?;
    Some(
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(module_path)
            .join("rustok-module.toml"),
    )
}

pub(crate) fn load_module_package_manifest(path: &Path) -> Result<ModulePackageManifest> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

pub(crate) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("xtask should live under the workspace root")
}

fn workspace_manifest_path() -> PathBuf {
    workspace_root().join("Cargo.toml")
}

pub(crate) fn load_workspace_manifest() -> Result<TomlValue> {
    let path = workspace_manifest_path();
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

pub(crate) fn load_toml_value(path: &Path) -> Result<TomlValue> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

pub(crate) fn resolve_workspace_inherited_string(
    value: Option<&TomlValue>,
    workspace_manifest: &TomlValue,
    field: &str,
) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };

    if let Some(string) = value.as_str() {
        return Ok(Some(string.trim().to_string()));
    }

    if let Some(table) = value.as_table() {
        if table.get("workspace").and_then(TomlValue::as_bool) == Some(true) {
            let inherited = workspace_manifest
                .get("workspace")
                .and_then(TomlValue::as_table)
                .and_then(|workspace| workspace.get("package"))
                .and_then(TomlValue::as_table)
                .and_then(|package| package.get(field))
                .and_then(TomlValue::as_str)
                .map(|value| value.trim().to_string());
            return Ok(inherited);
        }
    }

    anyhow::bail!("Unsupported package.{field} declaration in Cargo manifest")
}

pub(crate) fn load_resolved_cargo_package(
    path: &Path,
    workspace_manifest: &TomlValue,
) -> Result<ResolvedCargoPackage> {
    let manifest = load_toml_value(path)?;
    let package = manifest
        .get("package")
        .and_then(TomlValue::as_table)
        .with_context(|| format!("{} is missing [package]", path.display()))?;

    let name = package
        .get("name")
        .and_then(TomlValue::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .with_context(|| format!("{} is missing package.name", path.display()))?;
    let version =
        resolve_workspace_inherited_string(package.get("version"), workspace_manifest, "version")?
            .filter(|value| !value.is_empty())
            .with_context(|| format!("{} is missing package.version", path.display()))?;
    let license =
        resolve_workspace_inherited_string(package.get("license"), workspace_manifest, "license")?
            .filter(|value| !value.is_empty());

    Ok(ResolvedCargoPackage {
        name,
        version,
        license,
        manifest_path: path.to_path_buf(),
    })
}

pub(crate) fn build_module_publish_preview(
    manifest_path: &Path,
    slug: &str,
    spec: &ModuleSpec,
    workspace_manifest: &TomlValue,
) -> Result<ModulePublishDryRunPreview> {
    if spec.source != "path" {
        anyhow::bail!(
            "Module '{slug}' uses source='{}'; publish dry-run currently supports only local path modules",
            spec.source
        );
    }

    let package_manifest_path = module_package_manifest_path(manifest_path, spec)
        .with_context(|| format!("Module '{slug}' has source='path' but no path specified"))?;
    if !package_manifest_path.exists() {
        anyhow::bail!(
            "Module '{slug}' requires rustok-module.toml at {}",
            package_manifest_path.display()
        );
    }

    let package_manifest = load_module_package_manifest(&package_manifest_path)?;
    validate_module_package_metadata(slug, &package_manifest.module)?;
    validate_module_publish_contract(slug, &package_manifest)?;

    let declared_slug = package_manifest.module.slug.trim();
    if declared_slug != slug {
        anyhow::bail!(
            "Module '{slug}' declares slug '{}' in rustok-module.toml",
            declared_slug
        );
    }

    let module_root = package_manifest_path
        .parent()
        .map(PathBuf::from)
        .with_context(|| {
            format!(
                "Failed to resolve module root for '{}'",
                package_manifest_path.display()
            )
        })?;
    validate_module_local_docs_contract(slug, &module_root)?;
    validate_module_entry_type_contract(slug, &package_manifest, &module_root)?;
    validate_module_runtime_metadata_contract(slug, &package_manifest, &module_root)?;
    validate_module_semantics_contract(slug, spec, &package_manifest.module)?;
    validate_module_kind_contract(slug, spec, &module_root)?;
    validate_module_transport_surface_contract(slug, &package_manifest, &module_root)?;
    validate_module_server_http_surface_contract(manifest_path, slug, &package_manifest)?;
    validate_module_ui_classification_contract(slug, &package_manifest)?;
    validate_module_ui_metadata_contract(slug, &package_manifest)?;
    validate_module_admin_surface_contract(slug, &package_manifest)?;
    validate_module_dependency_contract(slug, spec, &package_manifest, &module_root)?;
    validate_module_permission_contract(slug, &module_root)?;
    validate_module_event_listener_contract(slug, &module_root)?;
    validate_module_event_ingress_contract(manifest_path, slug, &module_root)?;
    validate_module_index_search_boundary_contract(slug, &module_root)?;
    validate_module_search_operator_surface_contract(slug, &module_root)?;
    validate_module_server_registry_contract(
        manifest_path,
        slug,
        spec,
        &package_manifest,
        &module_root,
    )?;
    let crate_manifest_path = module_root.join("Cargo.toml");
    let crate_package = load_resolved_cargo_package(&crate_manifest_path, workspace_manifest)?;

    if crate_package.name != spec.crate_name {
        anyhow::bail!(
            "Module '{slug}' points to crate '{}' in modules.toml, but Cargo.toml declares '{}'",
            spec.crate_name,
            crate_package.name
        );
    }

    if crate_package.version != package_manifest.module.version {
        anyhow::bail!(
            "Module '{slug}' version mismatch: rustok-module.toml has '{}', Cargo.toml resolves to '{}'",
            package_manifest.module.version,
            crate_package.version
        );
    }

    let license = crate_package.license.clone().with_context(|| {
        format!(
            "Module '{slug}' must resolve package.license via {}",
            crate_package.manifest_path.display()
        )
    })?;

    validate_module_ui_surface_contract(
        slug,
        &module_root,
        "admin",
        package_manifest
            .provides
            .admin_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
    )?;
    validate_module_ui_surface_contract(
        slug,
        &module_root,
        "storefront",
        package_manifest
            .provides
            .storefront_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
    )?;

    let admin_preview = validate_module_ui_package(
        slug,
        &module_root,
        "admin",
        package_manifest
            .provides
            .admin_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
        &package_manifest.module.version,
        workspace_manifest,
    )?;
    let storefront_preview = validate_module_ui_package(
        slug,
        &module_root,
        "storefront",
        package_manifest
            .provides
            .storefront_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
        &package_manifest.module.version,
        workspace_manifest,
    )?;
    validate_module_host_ui_contract(
        manifest_path,
        slug,
        spec,
        package_manifest
            .provides
            .admin_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
        package_manifest
            .provides
            .storefront_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
    )?;
    validate_module_docs_navigation_contract(
        manifest_path,
        slug,
        spec,
        package_manifest
            .provides
            .admin_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
        package_manifest
            .provides
            .storefront_ui
            .as_ref()
            .and_then(|ui| ui.leptos_crate.as_deref()),
        &package_manifest.module.showcase_admin_surfaces,
    )?;

    Ok(ModulePublishDryRunPreview {
        slug: slug.to_string(),
        version: package_manifest.module.version.clone(),
        crate_name: crate_package.name,
        module_name: package_manifest.module.name.clone(),
        module_description: package_manifest.module.description.clone(),
        ownership: package_manifest.module.ownership.clone(),
        trust_level: package_manifest.module.trust_level.clone(),
        license,
        manifest_path: manifest_path.display().to_string(),
        package_manifest_path: package_manifest_path.display().to_string(),
        module_entry_type: package_manifest.crate_contract.entry_type.clone(),
        marketplace: ModuleMarketplacePreview {
            category: package_manifest.marketplace.category.clone(),
            tags: package_manifest.marketplace.tags.clone(),
        },
        ui_packages: ModuleUiPackagesPreview {
            admin: admin_preview,
            storefront: storefront_preview,
        },
    })
}

pub(crate) fn validate_module_publish_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
) -> Result<()> {
    let module_slug = manifest.module.slug.trim();
    if module_slug.is_empty() {
        anyhow::bail!("Module '{slug}' is missing module.slug in rustok-module.toml");
    }

    if manifest.module.name.trim().is_empty() {
        anyhow::bail!("Module '{slug}' is missing module.name in rustok-module.toml");
    }

    let version = manifest.module.version.trim();
    if version.is_empty() {
        anyhow::bail!("Module '{slug}' is missing module.version in rustok-module.toml");
    }
    Version::parse(version)
        .with_context(|| format!("Module '{slug}' has non-semver module.version '{version}'"))?;

    if manifest.module.description.trim().len() < 20 {
        anyhow::bail!(
            "Module '{slug}' description must be at least 20 characters for publish readiness"
        );
    }

    Ok(())
}
