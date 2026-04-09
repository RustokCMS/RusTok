use super::*;

pub(crate) fn validate_module_admin_surface_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
) -> Result<()> {
    let has_admin_ui = manifest.provides.admin_ui.is_some();
    let recommended = &manifest.module.recommended_admin_surfaces;
    let showcase = &manifest.module.showcase_admin_surfaces;

    if !has_admin_ui && (!recommended.is_empty() || !showcase.is_empty()) {
        anyhow::bail!(
            "Module '{slug}' declares recommended/showcase admin surfaces but does not declare [provides.admin_ui]"
        );
    }

    let Some(admin_ui) = manifest.provides.admin_ui.as_ref() else {
        return Ok(());
    };

    if recommended.is_empty() {
        anyhow::bail!(
            "Module '{slug}' declares [provides.admin_ui] and must declare at least one recommended_admin_surface"
        );
    }

    if admin_ui
        .leptos_crate
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
        && !recommended
            .iter()
            .any(|surface| surface.trim() == "leptos-admin")
    {
        anyhow::bail!(
            "Module '{slug}' declares admin_ui.leptos_crate and must include 'leptos-admin' in recommended_admin_surfaces"
        );
    }

    Ok(())
}

pub(crate) fn validate_module_host_ui_contract(
    manifest_path: &Path,
    slug: &str,
    spec: &ModuleSpec,
    admin_ui_crate: Option<&str>,
    storefront_ui_crate: Option<&str>,
) -> Result<()> {
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let root_manifest = load_manifest_from(manifest_path)?;

    if let Some(crate_name) = admin_ui_crate
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let admin_manifest_path = workspace_root.join("apps").join("admin").join("Cargo.toml");
        let expected_admin_manifest_path =
            expected_module_ui_manifest_path(manifest_path, spec, "admin")?;
        validate_host_ui_crate_wiring(
            slug,
            crate_name,
            &admin_manifest_path,
            Some(&expected_admin_manifest_path),
            &["hydrate", "ssr"],
        )?;
        for (dependency_slug, dependency_ui_crate) in
            collect_dependency_ui_crates_for_surface(&root_manifest, spec, manifest_path, "admin")?
        {
            let dependency_spec =
                root_manifest
                    .modules
                    .get(&dependency_slug)
                    .with_context(|| {
                        format!(
                            "Module dependency '{dependency_slug}' is missing from modules.toml"
                        )
                    })?;
            let expected_dependency_manifest_path =
                expected_module_ui_manifest_path(manifest_path, dependency_spec, "admin")?;
            validate_host_ui_crate_wiring(
                slug,
                &dependency_ui_crate,
                &admin_manifest_path,
                Some(&expected_dependency_manifest_path),
                &["hydrate", "ssr"],
            )
            .with_context(|| {
                format!(
                    "Module '{slug}' admin host composition is missing UI dependency from module '{dependency_slug}'"
                )
            })?;
        }
    }

    if let Some(crate_name) = storefront_ui_crate
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let storefront_manifest_path = workspace_root
            .join("apps")
            .join("storefront")
            .join("Cargo.toml");
        let expected_storefront_manifest_path =
            expected_module_ui_manifest_path(manifest_path, spec, "storefront")?;
        validate_host_ui_crate_wiring(
            slug,
            crate_name,
            &storefront_manifest_path,
            Some(&expected_storefront_manifest_path),
            &["ssr"],
        )?;
        for (dependency_slug, dependency_ui_crate) in collect_dependency_ui_crates_for_surface(
            &root_manifest,
            spec,
            manifest_path,
            "storefront",
        )? {
            let dependency_spec =
                root_manifest
                    .modules
                    .get(&dependency_slug)
                    .with_context(|| {
                        format!(
                            "Module dependency '{dependency_slug}' is missing from modules.toml"
                        )
                    })?;
            let expected_dependency_manifest_path =
                expected_module_ui_manifest_path(manifest_path, dependency_spec, "storefront")?;
            validate_host_ui_crate_wiring(
                slug,
                &dependency_ui_crate,
                &storefront_manifest_path,
                Some(&expected_dependency_manifest_path),
                &["ssr"],
            )
            .with_context(|| {
                format!(
                    "Module '{slug}' storefront host composition is missing UI dependency from module '{dependency_slug}'"
                )
            })?;
        }
    }

    Ok(())
}

fn collect_dependency_ui_crates_for_surface(
    manifest: &Manifest,
    spec: &ModuleSpec,
    manifest_path: &Path,
    surface: &str,
) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();

    for dependency_slug in spec.depends_on.as_deref().unwrap_or(&[]) {
        let dependency_slug = dependency_slug.trim();
        if dependency_slug.is_empty() {
            continue;
        }

        let dependency_spec = manifest.modules.get(dependency_slug).with_context(|| {
            format!("Module dependency '{dependency_slug}' is missing from modules.toml")
        })?;
        let Some(dependency_manifest_path) =
            module_package_manifest_path(manifest_path, dependency_spec)
        else {
            continue;
        };
        let dependency_manifest = load_module_package_manifest(&dependency_manifest_path)?;
        let dependency_ui_crate = match surface {
            "admin" => dependency_manifest
                .provides
                .admin_ui
                .as_ref()
                .and_then(|ui| ui.leptos_crate.as_deref()),
            "storefront" => dependency_manifest
                .provides
                .storefront_ui
                .as_ref()
                .and_then(|ui| ui.leptos_crate.as_deref()),
            _ => None,
        };

        if let Some(crate_name) = dependency_ui_crate
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            result.push((dependency_slug.to_string(), crate_name.to_string()));
        }
    }

    Ok(result)
}

pub(crate) fn validate_module_docs_navigation_contract(
    manifest_path: &Path,
    slug: &str,
    spec: &ModuleSpec,
    admin_ui_crate: Option<&str>,
    storefront_ui_crate: Option<&str>,
    showcase_admin_surfaces: &[String],
) -> Result<()> {
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let docs_modules_dir = workspace_root.join("docs").join("modules");
    let module_index_path = docs_modules_dir.join("_index.md");
    let ui_index_path = docs_modules_dir.join("UI_PACKAGES_INDEX.md");
    let module_index = fs::read_to_string(&module_index_path)
        .with_context(|| format!("Failed to read {}", module_index_path.display()))?;
    let ui_index = fs::read_to_string(&ui_index_path)
        .with_context(|| format!("Failed to read {}", ui_index_path.display()))?;
    let module_path = spec
        .path
        .as_deref()
        .with_context(|| format!("Module '{slug}' has source='path' but no path specified"))?;

    let docs_link = format!("../../{module_path}/docs/README.md");
    let plan_link = format!("../../{module_path}/docs/implementation-plan.md");
    if !module_index.contains(&docs_link) {
        anyhow::bail!(
            "Module '{slug}' is missing docs navigation link '{}' in {}",
            docs_link,
            module_index_path.display()
        );
    }
    if !module_index.contains(&plan_link) {
        anyhow::bail!(
            "Module '{slug}' is missing implementation plan link '{}' in {}",
            plan_link,
            module_index_path.display()
        );
    }

    if admin_ui_crate.is_some() {
        let admin_link = format!("../../{module_path}/admin/README.md");
        if !ui_index.contains(&admin_link) {
            anyhow::bail!(
                "Module '{slug}' declares admin UI, but {} is missing '{}'",
                ui_index_path.display(),
                admin_link
            );
        }
    }

    if storefront_ui_crate.is_some() {
        let storefront_link = format!("../../{module_path}/storefront/README.md");
        if !ui_index.contains(&storefront_link) {
            anyhow::bail!(
                "Module '{slug}' declares storefront UI, but {} is missing '{}'",
                ui_index_path.display(),
                storefront_link
            );
        }
    }

    if showcase_admin_surfaces
        .iter()
        .any(|surface| surface.trim() == "next-admin")
    {
        let next_admin_path_fragment = format!("apps/next-admin/packages/{slug}/");
        if !ui_index.contains(&next_admin_path_fragment) {
            anyhow::bail!(
                "Module '{slug}' declares showcase_admin_surfaces=['next-admin'], but {} is missing '{}'",
                ui_index_path.display(),
                next_admin_path_fragment
            );
        }

        let next_admin_package_dir = workspace_root
            .join("apps")
            .join("next-admin")
            .join("packages")
            .join(slug);
        if !next_admin_package_dir.exists() {
            anyhow::bail!(
                "Module '{slug}' declares showcase_admin_surfaces=['next-admin'], but {} does not exist",
                next_admin_package_dir.display()
            );
        }
    }

    Ok(())
}

fn validate_host_ui_crate_wiring(
    slug: &str,
    crate_name: &str,
    host_manifest_path: &Path,
    expected_ui_manifest_path: Option<&Path>,
    propagated_features: &[&str],
) -> Result<()> {
    let host_manifest = load_toml_value(host_manifest_path)?;
    let host_dependencies = host_manifest
        .get("dependencies")
        .and_then(TomlValue::as_table)
        .with_context(|| format!("{} is missing [dependencies]", host_manifest_path.display()))?;
    if !host_dependencies.contains_key(crate_name) {
        anyhow::bail!(
            "Module '{slug}' declares UI crate '{}', but {} is missing dependency '{}'",
            crate_name,
            host_manifest_path.display(),
            crate_name
        );
    }

    let ui_crate_manifest_path =
        resolve_host_ui_crate_manifest_path(host_manifest_path, crate_name)?;
    if let Some(expected_ui_manifest_path) = expected_ui_manifest_path {
        let actual_ui_manifest_path =
            fs::canonicalize(&ui_crate_manifest_path).unwrap_or(ui_crate_manifest_path.clone());
        let expected_ui_manifest_path = fs::canonicalize(expected_ui_manifest_path)
            .unwrap_or_else(|_| expected_ui_manifest_path.to_path_buf());
        if actual_ui_manifest_path != expected_ui_manifest_path {
            anyhow::bail!(
                "Module '{slug}' declares UI crate '{}', but {} points to '{}' instead of canonical '{}'",
                crate_name,
                host_manifest_path.display(),
                actual_ui_manifest_path.display(),
                expected_ui_manifest_path.display()
            );
        }
    }

    let ui_crate_manifest = load_toml_value(&ui_crate_manifest_path)?;
    let ui_features = ui_crate_manifest
        .get("features")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();
    let host_features = host_manifest
        .get("features")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();

    for feature_name in propagated_features {
        if !ui_features.contains_key(*feature_name) {
            continue;
        }

        let Some(host_feature_entries) = host_features
            .get(*feature_name)
            .and_then(TomlValue::as_array)
        else {
            anyhow::bail!(
                "Module '{slug}' requires host feature '{}' in {} for UI crate '{}'",
                feature_name,
                host_manifest_path.display(),
                crate_name
            );
        };

        let expected_entry = format!("{crate_name}/{feature_name}");
        let has_entry = host_feature_entries.iter().any(|entry| {
            entry
                .as_str()
                .map(|value| value.trim() == expected_entry)
                .unwrap_or(false)
        });
        if !has_entry {
            anyhow::bail!(
                "Module '{slug}' declares UI crate '{}', but {} feature '{}' is missing '{}'",
                crate_name,
                host_manifest_path.display(),
                feature_name,
                expected_entry
            );
        }
    }

    Ok(())
}

fn expected_module_ui_manifest_path(
    manifest_path: &Path,
    spec: &ModuleSpec,
    surface: &str,
) -> Result<PathBuf> {
    let module_path = spec.path.as_deref().with_context(|| {
        format!(
            "Module '{}' has source='path' but no path specified",
            spec.crate_name
        )
    })?;
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    Ok(workspace_root
        .join(module_path)
        .join(surface)
        .join("Cargo.toml"))
}

fn resolve_host_ui_crate_manifest_path(
    host_manifest_path: &Path,
    crate_name: &str,
) -> Result<PathBuf> {
    let host_manifest = load_toml_value(host_manifest_path)?;
    let dependency = host_manifest
        .get("dependencies")
        .and_then(TomlValue::as_table)
        .and_then(|dependencies| dependencies.get(crate_name))
        .with_context(|| {
            format!(
                "{} is missing dependency '{}'",
                host_manifest_path.display(),
                crate_name
            )
        })?;
    let dependency_table = dependency.as_table().with_context(|| {
        format!(
            "Dependency '{}' in {} must be declared as an inline table",
            crate_name,
            host_manifest_path.display()
        )
    })?;
    let relative_path = dependency_table
        .get("path")
        .and_then(TomlValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .with_context(|| {
            format!(
                "Dependency '{}' in {} must declare path for host UI wiring validation",
                crate_name,
                host_manifest_path.display()
            )
        })?;

    let base_dir = host_manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve parent directory for {}",
            host_manifest_path.display()
        )
    })?;
    Ok(base_dir.join(relative_path).join("Cargo.toml"))
}

pub(crate) fn validate_host_ui_inventory_contract(
    manifest_path: &Path,
    manifest: &Manifest,
) -> Result<()> {
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let admin_manifest_path = workspace_root.join("apps").join("admin").join("Cargo.toml");
    let storefront_manifest_path = workspace_root
        .join("apps")
        .join("storefront")
        .join("Cargo.toml");

    validate_host_ui_surface_inventory(manifest_path, manifest, &admin_manifest_path, "admin")?;
    validate_host_ui_surface_inventory(
        manifest_path,
        manifest,
        &storefront_manifest_path,
        "storefront",
    )?;

    Ok(())
}

fn validate_host_ui_surface_inventory(
    manifest_path: &Path,
    manifest: &Manifest,
    host_manifest_path: &Path,
    surface: &str,
) -> Result<()> {
    let declared_ui_crates =
        collect_declared_ui_crates_for_surface(manifest_path, manifest, surface)?;
    let host_manifest = load_toml_value(host_manifest_path)?;
    let Some(host_dependencies) = host_manifest
        .get("dependencies")
        .and_then(TomlValue::as_table)
    else {
        return Ok(());
    };

    for (crate_name, dependency) in host_dependencies {
        let Some(dependency_table) = dependency.as_table() else {
            continue;
        };
        let Some(relative_path) = dependency_table
            .get("path")
            .and_then(TomlValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let base_dir = host_manifest_path.parent().with_context(|| {
            format!(
                "Failed to resolve parent directory for {}",
                host_manifest_path.display()
            )
        })?;
        let ui_crate_dir = base_dir.join(relative_path);
        let actual_ui_manifest_path = ui_crate_dir.join("Cargo.toml");
        if !actual_ui_manifest_path.exists() {
            continue;
        }

        let surface_dir = actual_ui_manifest_path
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str());
        if surface_dir != Some(surface) {
            continue;
        }

        let Some(module_root) = actual_ui_manifest_path.parent().and_then(Path::parent) else {
            continue;
        };
        if !module_root.join("rustok-module.toml").exists() {
            continue;
        }

        let Some((owner_slug, expected_ui_manifest_path)) = declared_ui_crates.get(crate_name)
        else {
            anyhow::bail!(
                "Host UI inventory drift: {} depends on '{}' at '{}', but no module manifest declares it as {} UI",
                host_manifest_path.display(),
                crate_name,
                actual_ui_manifest_path.display(),
                surface
            );
        };

        let actual_ui_manifest_path =
            fs::canonicalize(&actual_ui_manifest_path).unwrap_or(actual_ui_manifest_path);
        let expected_ui_manifest_path = fs::canonicalize(expected_ui_manifest_path)
            .unwrap_or_else(|_| expected_ui_manifest_path.clone());
        if actual_ui_manifest_path != expected_ui_manifest_path {
            anyhow::bail!(
                "Host UI inventory drift: {} depends on '{}' for module '{}', but path '{}' does not match canonical '{}'",
                host_manifest_path.display(),
                crate_name,
                owner_slug,
                actual_ui_manifest_path.display(),
                expected_ui_manifest_path.display()
            );
        }
    }

    let host_features = host_manifest
        .get("features")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();
    for host_feature_name in ["hydrate", "ssr"] {
        let Some(entries) = host_features
            .get(host_feature_name)
            .and_then(TomlValue::as_array)
        else {
            continue;
        };

        for entry in entries {
            let Some(entry) = entry
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let Some((crate_name, crate_feature)) = entry.split_once('/') else {
                continue;
            };
            if crate_feature != host_feature_name {
                continue;
            }

            let declared_ui = declared_ui_crates.get(crate_name);
            let host_dependency = host_dependencies
                .get(crate_name)
                .and_then(TomlValue::as_table)
                .and_then(|dependency| dependency.get("path"))
                .and_then(TomlValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());

            let Some(host_dependency_path) = host_dependency else {
                if declared_ui.is_some() {
                    anyhow::bail!(
                        "Host UI feature drift: {} feature '{}' references '{}', but {} is missing dependency '{}'",
                        host_manifest_path.display(),
                        host_feature_name,
                        entry,
                        host_manifest_path.display(),
                        crate_name
                    );
                }
                continue;
            };

            let base_dir = host_manifest_path.parent().with_context(|| {
                format!(
                    "Failed to resolve parent directory for {}",
                    host_manifest_path.display()
                )
            })?;
            let actual_ui_manifest_path = base_dir.join(host_dependency_path).join("Cargo.toml");
            if !actual_ui_manifest_path.exists() {
                continue;
            }
            let surface_dir = actual_ui_manifest_path
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str());
            if surface_dir != Some(surface) {
                continue;
            }

            let Some(module_root) = actual_ui_manifest_path.parent().and_then(Path::parent) else {
                continue;
            };
            if !module_root.join("rustok-module.toml").exists() {
                continue;
            }

            let Some((owner_slug, expected_ui_manifest_path)) = declared_ui else {
                anyhow::bail!(
                    "Host UI feature drift: {} feature '{}' references '{}', but no module manifest declares it as {} UI",
                    host_manifest_path.display(),
                    host_feature_name,
                    entry,
                    surface
                );
            };

            let actual_ui_manifest_path =
                fs::canonicalize(&actual_ui_manifest_path).unwrap_or(actual_ui_manifest_path);
            let expected_ui_manifest_path = fs::canonicalize(expected_ui_manifest_path)
                .unwrap_or_else(|_| expected_ui_manifest_path.clone());
            if actual_ui_manifest_path != expected_ui_manifest_path {
                anyhow::bail!(
                    "Host UI feature drift: {} feature '{}' references '{}' for module '{}', but path '{}' does not match canonical '{}'",
                    host_manifest_path.display(),
                    host_feature_name,
                    entry,
                    owner_slug,
                    actual_ui_manifest_path.display(),
                    expected_ui_manifest_path.display()
                );
            }
        }
    }

    Ok(())
}

fn collect_declared_ui_crates_for_surface(
    manifest_path: &Path,
    manifest: &Manifest,
    surface: &str,
) -> Result<HashMap<String, (String, PathBuf)>> {
    let mut declared = HashMap::new();

    for (slug, spec) in &manifest.modules {
        let Some(package_manifest_path) = module_package_manifest_path(manifest_path, spec) else {
            continue;
        };
        if !package_manifest_path.exists() {
            continue;
        }

        let package_manifest = load_module_package_manifest(&package_manifest_path)?;
        let ui_crate = match surface {
            "admin" => package_manifest
                .provides
                .admin_ui
                .as_ref()
                .and_then(|ui| ui.leptos_crate.as_deref()),
            "storefront" => package_manifest
                .provides
                .storefront_ui
                .as_ref()
                .and_then(|ui| ui.leptos_crate.as_deref()),
            _ => None,
        };
        let Some(ui_crate) = ui_crate.map(str::trim).filter(|value| !value.is_empty()) else {
            continue;
        };

        let expected_manifest_path =
            expected_module_ui_manifest_path(manifest_path, spec, surface)?;
        if let Some((existing_slug, _)) = declared.insert(
            ui_crate.to_string(),
            (slug.to_string(), expected_manifest_path),
        ) {
            anyhow::bail!(
                "UI crate '{}' is declared by multiple module manifests: '{}' and '{}'",
                ui_crate,
                existing_slug,
                slug
            );
        }
    }

    Ok(declared)
}
