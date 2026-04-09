use super::*;

pub(crate) fn validate_module_server_registry_contract(
    manifest_path: &Path,
    slug: &str,
    spec: &ModuleSpec,
    manifest: &ModulePackageManifest,
    module_root: &Path,
) -> Result<()> {
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let server_modules_path = workspace_root
        .join("apps")
        .join("server")
        .join("src")
        .join("modules")
        .join("mod.rs");
    let server_modules_content = fs::read_to_string(&server_modules_path)
        .with_context(|| format!("Failed to read {}", server_modules_path.display()))?;
    let inferred_module_entry =
        infer_server_registry_module_expr(spec, manifest, module_root, slug)?.with_context(
            || format!("Module '{slug}' must resolve to a runtime ModuleRegistry entry"),
        )?;

    if spec.required {
        if !server_modules_registers_direct_entry(
            &server_modules_content,
            slug,
            &inferred_module_entry,
        ) {
            anyhow::bail!(
                "Required module '{slug}' must be registered directly in apps/server/src/modules/mod.rs"
            );
        }
        return Ok(());
    }

    let server_features = load_server_module_feature_map(workspace_root)?;
    let feature_name = format!("mod-{slug}");
    let feature_entries = server_features.get(&feature_name).with_context(|| {
        format!(
            "Optional module '{slug}' must expose feature '{}' in apps/server/Cargo.toml",
            feature_name
        )
    })?;

    let dependency_feature = format!("dep:{}", spec.crate_name);
    if !feature_entries.contains(&dependency_feature) {
        anyhow::bail!(
            "Optional module '{slug}' must wire feature '{}' to '{}' in apps/server/Cargo.toml",
            feature_name,
            dependency_feature
        );
    }

    let expected_module_features = spec
        .depends_on
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|dependency| format!("mod-{}", dependency.trim()))
        .collect::<HashSet<_>>();
    let actual_module_features = feature_entries
        .iter()
        .filter(|entry| entry.starts_with("mod-"))
        .cloned()
        .collect::<HashSet<_>>();
    if expected_module_features != actual_module_features {
        anyhow::bail!(
            "Optional module '{slug}' has server feature graph drift in apps/server/Cargo.toml: expected mod-dependencies {:?}, got {:?}",
            expected_module_features,
            actual_module_features
        );
    }

    let crate_ident = spec.crate_name.replace('-', "_");
    if !inferred_module_entry.starts_with(&format!("{crate_ident}::")) {
        anyhow::bail!(
            "Optional module '{slug}' resolved unexpected server registry entry '{}'; expected '{}' namespace",
            inferred_module_entry,
            crate_ident
        );
    }

    if server_modules_registers_direct_entry(&server_modules_content, slug, &inferred_module_entry)
    {
        anyhow::bail!(
            "Optional module '{slug}' must not be registered directly in apps/server/src/modules/mod.rs; use feature/codegen wiring instead"
        );
    }

    Ok(())
}

fn server_modules_registers_direct_entry(
    content: &str,
    slug: &str,
    inferred_module_entry: &str,
) -> bool {
    let entry_ident = inferred_module_entry
        .rsplit("::")
        .next()
        .unwrap_or(inferred_module_entry);
    let slug_pascal = format!("{}Module", to_pascal_case(slug));
    let slug_binding = format!("{}_module", slug.replace('-', "_"));

    [
        format!(".register({entry_ident})"),
        format!(".register({inferred_module_entry})"),
        format!(".register({slug_pascal})"),
        format!(".register({slug_binding})"),
    ]
    .into_iter()
    .any(|needle| content.contains(&needle))
}

fn load_server_module_feature_map(
    workspace_root: &Path,
) -> Result<HashMap<String, HashSet<String>>> {
    let server_manifest_path = workspace_root
        .join("apps")
        .join("server")
        .join("Cargo.toml");
    let server_manifest = load_toml_value(&server_manifest_path)?;
    let feature_table = server_manifest
        .get("features")
        .and_then(TomlValue::as_table)
        .with_context(|| format!("{} is missing [features]", server_manifest_path.display()))?;

    let mut features = HashMap::new();
    for (feature_name, value) in feature_table {
        let values = value.as_array().with_context(|| {
            format!(
                "Feature '{}' in {} must be a TOML array",
                feature_name,
                server_manifest_path.display()
            )
        })?;
        let entries = values
            .iter()
            .filter_map(TomlValue::as_str)
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .collect::<HashSet<_>>();
        features.insert(feature_name.to_string(), entries);
    }

    Ok(features)
}

fn infer_server_registry_module_expr(
    spec: &ModuleSpec,
    manifest: &ModulePackageManifest,
    module_root: &Path,
    slug: &str,
) -> Result<Option<String>> {
    if let Some(entry_type) = manifest
        .crate_contract
        .entry_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(format!(
            "{}::{}",
            spec.crate_name.replace('-', "_"),
            entry_type
        )));
    }

    if let Some(entry_type) = infer_runtime_module_entry_type(module_root)? {
        return Ok(Some(format!(
            "{}::{}",
            spec.crate_name.replace('-', "_"),
            entry_type
        )));
    }

    let has_package_manifest = module_root.join("rustok-module.toml").exists();
    if has_package_manifest {
        return Ok(None);
    }

    Ok(Some(format!(
        "{}::{}Module",
        spec.crate_name.replace('-', "_"),
        to_pascal_case(slug)
    )))
}

pub(crate) fn validate_default_enabled_server_contract(
    manifest_path: &Path,
    manifest: &Manifest,
) -> Result<()> {
    let Some(default_enabled) = manifest
        .settings
        .as_ref()
        .and_then(|settings| settings.default_enabled.as_ref())
    else {
        return Ok(());
    };

    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let server_features = load_server_module_feature_map(workspace_root)?;
    let server_default_features = server_features.get("default").cloned().unwrap_or_default();
    let invalid_required_defaults = default_enabled
        .iter()
        .map(|slug| slug.trim())
        .filter(|slug| !slug.is_empty())
        .filter(|slug| {
            manifest
                .modules
                .get(*slug)
                .map(|spec| spec.required)
                .unwrap_or(false)
        })
        .map(|slug| slug.to_string())
        .collect::<Vec<_>>();

    if !invalid_required_defaults.is_empty() {
        anyhow::bail!(
            "default_enabled must list only optional modules; required modules are always active: {}",
            invalid_required_defaults.join(", ")
        );
    }

    let default_enabled_set = default_enabled
        .iter()
        .map(|slug| slug.trim())
        .filter(|slug| !slug.is_empty())
        .collect::<HashSet<_>>();
    let missing_default_dependencies = default_enabled
        .iter()
        .map(|slug| slug.trim())
        .filter(|slug| !slug.is_empty())
        .filter_map(|slug| manifest.modules.get(slug).map(|spec| (slug, spec)))
        .flat_map(|(slug, spec)| {
            spec.depends_on
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(move |dependency| (slug, dependency.trim()))
        })
        .filter(|(_, dependency)| !dependency.is_empty())
        .filter(|(_, dependency)| {
            manifest
                .modules
                .get(*dependency)
                .map(|spec| !spec.required)
                .unwrap_or(false)
        })
        .filter(|(_, dependency)| !default_enabled_set.contains(*dependency))
        .map(|(slug, dependency)| format!("{slug} -> {dependency}"))
        .collect::<Vec<_>>();

    if !missing_default_dependencies.is_empty() {
        anyhow::bail!(
            "default_enabled must include optional dependency closure: {}",
            missing_default_dependencies.join(", ")
        );
    }

    let missing_server_defaults = default_enabled
        .iter()
        .map(|slug| slug.trim())
        .filter(|slug| !slug.is_empty())
        .filter(|slug| {
            manifest
                .modules
                .get(*slug)
                .map(|spec| !spec.required)
                .unwrap_or(false)
        })
        .filter(|slug| !server_default_features.contains(&format!("mod-{slug}")))
        .map(|slug| slug.to_string())
        .collect::<Vec<_>>();

    if !missing_server_defaults.is_empty() {
        anyhow::bail!(
            "default_enabled modules must be present in apps/server default features: {}",
            missing_server_defaults.join(", ")
        );
    }

    Ok(())
}
