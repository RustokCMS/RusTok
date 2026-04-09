use super::*;

pub(crate) fn validate_module_package_metadata(
    slug: &str,
    metadata: &ModulePackageMetadata,
) -> Result<()> {
    let ownership = metadata.ownership.trim();
    if !is_valid_module_ownership(ownership) {
        anyhow::bail!("Module '{}' has invalid ownership '{}'", slug, ownership);
    }

    let trust_level = metadata.trust_level.trim();
    if !is_valid_trust_level(trust_level) {
        anyhow::bail!(
            "Module '{}' has invalid trust level '{}'",
            slug,
            trust_level
        );
    }

    let recommended = validate_admin_surfaces(
        slug,
        "recommended_admin_surfaces",
        &metadata.recommended_admin_surfaces,
    )?;
    let showcase = validate_admin_surfaces(
        slug,
        "showcase_admin_surfaces",
        &metadata.showcase_admin_surfaces,
    )?;

    if let Some(surface) = recommended.intersection(&showcase).next() {
        anyhow::bail!(
            "Module '{}' lists admin surface '{}' as both recommended and showcase",
            slug,
            surface
        );
    }

    Ok(())
}

pub(crate) fn validate_module_semantics_contract(
    slug: &str,
    spec: &ModuleSpec,
    metadata: &ModulePackageMetadata,
) -> Result<()> {
    let ownership = metadata.ownership.trim();
    let trust_level = metadata.trust_level.trim();

    if spec.source == "path" && ownership != "first_party" {
        anyhow::bail!(
            "Module '{slug}' uses source='path' and must declare ownership='first_party', got '{}'",
            ownership
        );
    }

    if spec.required && trust_level != "core" {
        anyhow::bail!(
            "Module '{slug}' is required in modules.toml and must declare trust_level='core', got '{}'",
            trust_level
        );
    }

    if !spec.required && trust_level == "core" {
        anyhow::bail!(
            "Module '{slug}' is optional in modules.toml and must not declare trust_level='core'"
        );
    }

    Ok(())
}

pub(crate) fn validate_module_local_docs_file(
    slug: &str,
    path: &Path,
    required_headings: &[&str],
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!(
            "Module '{slug}' requires local docs file {}",
            path.display()
        );
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    for heading in required_headings {
        if !content.contains(heading) {
            anyhow::bail!(
                "Module '{slug}' local docs {} must contain heading '{}'",
                path.display(),
                heading
            );
        }
    }

    Ok(())
}

pub(crate) fn validate_module_entry_type_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
    module_root: &Path,
) -> Result<()> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    let has_runtime_module_impl = content.contains("impl RusToKModule for");
    let entry_type = manifest
        .crate_contract
        .entry_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if !has_runtime_module_impl {
        if entry_type.is_some() {
            anyhow::bail!(
                "Module '{slug}' declares crate.entry_type in rustok-module.toml, but {} does not implement RusToKModule",
                lib_path.display()
            );
        }
        return Ok(());
    }

    let Some(entry_type) = entry_type else {
        anyhow::bail!(
            "Module '{slug}' must declare [crate].entry_type in rustok-module.toml because {} implements RusToKModule",
            lib_path.display()
        );
    };

    let has_entry_struct = content.contains(&format!("pub struct {entry_type};"))
        || content.contains(&format!("pub struct {entry_type} {{"))
        || content.contains(&format!("pub struct {entry_type}<"))
        || content.contains(&format!("struct {entry_type};"))
        || content.contains(&format!("struct {entry_type} {{"))
        || content.contains(&format!("struct {entry_type}<"));
    if !has_entry_struct {
        anyhow::bail!(
            "Module '{slug}' declares crate.entry_type='{}', but {} is missing runtime struct '{}'",
            entry_type,
            lib_path.display(),
            entry_type
        );
    }

    if !content.contains(&format!("impl RusToKModule for {entry_type}")) {
        anyhow::bail!(
            "Module '{slug}' declares crate.entry_type='{}', but {} is missing 'impl RusToKModule for {}'",
            entry_type,
            lib_path.display(),
            entry_type
        );
    }

    Ok(())
}

pub(crate) fn validate_module_runtime_metadata_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
    module_root: &Path,
) -> Result<()> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    if !content.contains("impl RusToKModule for") {
        return Ok(());
    }

    let runtime_slug = extract_runtime_string_method(&content, "slug").with_context(|| {
        format!(
            "Module '{slug}' must expose fn slug(&self) -> &'static str in {}",
            lib_path.display()
        )
    })?;
    let runtime_name = extract_runtime_string_method(&content, "name").with_context(|| {
        format!(
            "Module '{slug}' must expose fn name(&self) -> &'static str in {}",
            lib_path.display()
        )
    })?;
    let runtime_description =
        extract_runtime_string_method(&content, "description").with_context(|| {
            format!(
                "Module '{slug}' must expose fn description(&self) -> &'static str in {}",
                lib_path.display()
            )
        })?;

    if manifest.module.slug.trim() != runtime_slug {
        anyhow::bail!(
            "Module '{slug}' slug mismatch between rustok-module.toml ('{}') and RusToKModule::slug() ('{}')",
            manifest.module.slug.trim(),
            runtime_slug
        );
    }

    if manifest.module.name.trim() != runtime_name {
        anyhow::bail!(
            "Module '{slug}' name mismatch between rustok-module.toml ('{}') and RusToKModule::name() ('{}')",
            manifest.module.name.trim(),
            runtime_name
        );
    }

    if manifest.module.description.trim() != runtime_description {
        anyhow::bail!(
            "Module '{slug}' description mismatch between rustok-module.toml ('{}') and RusToKModule::description() ('{}')",
            manifest.module.description.trim(),
            runtime_description
        );
    }

    Ok(())
}

pub(crate) fn validate_module_transport_surface_contract(
    slug: &str,
    manifest: &ModulePackageManifest,
    module_root: &Path,
) -> Result<()> {
    let source_files = collect_module_source_files(module_root)?;

    if let Some(graphql) = manifest.provides.graphql.as_ref() {
        if let Some(query) = graphql.query.as_deref() {
            let symbol = provided_path_symbol(query)?;
            validate_declared_symbol_exists(
                slug,
                "provides.graphql.query",
                query,
                symbol,
                &source_files,
            )?;
        }
        if let Some(mutation) = graphql.mutation.as_deref() {
            let symbol = provided_path_symbol(mutation)?;
            validate_declared_symbol_exists(
                slug,
                "provides.graphql.mutation",
                mutation,
                symbol,
                &source_files,
            )?;
        }
    }

    if let Some(http) = manifest.provides.http.as_ref() {
        if let Some(routes) = http.routes.as_deref() {
            let symbol = provided_path_symbol(routes)?;
            validate_declared_symbol_exists(
                slug,
                "provides.http.routes",
                routes,
                symbol,
                &source_files,
            )?;
        }
        if let Some(webhook_routes) = http.webhook_routes.as_deref() {
            let symbol = provided_path_symbol(webhook_routes)?;
            validate_declared_symbol_exists(
                slug,
                "provides.http.webhook_routes",
                webhook_routes,
                symbol,
                &source_files,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn validate_module_server_http_surface_contract(
    manifest_path: &Path,
    slug: &str,
    manifest: &ModulePackageManifest,
) -> Result<()> {
    let Some(http) = manifest.provides.http.as_ref() else {
        return Ok(());
    };

    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let Some(server_controller_path) = find_existing_server_controller_path(workspace_root, slug)
    else {
        return Ok(());
    };

    let controller_content = fs::read_to_string(&server_controller_path)
        .with_context(|| format!("Failed to read {}", server_controller_path.display()))?;

    if http.routes.as_deref().is_some()
        && !server_controller_exports_symbol(&controller_content, "routes")
    {
        anyhow::bail!(
            "Module '{slug}' declares provides.http.routes, but server controller shim {} does not export pub routes() for build.rs/codegen",
            server_controller_path.display()
        );
    }

    if http.webhook_routes.as_deref().is_some()
        && !server_controller_exports_symbol(&controller_content, "webhook_routes")
    {
        anyhow::bail!(
            "Module '{slug}' declares provides.http.webhook_routes, but server controller shim {} does not export pub webhook_routes() for build.rs/codegen",
            server_controller_path.display()
        );
    }

    Ok(())
}

fn find_existing_server_controller_path(workspace_root: &Path, slug: &str) -> Option<PathBuf> {
    [
        workspace_root
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers")
            .join(slug)
            .join("mod.rs"),
        workspace_root
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers")
            .join(format!("{slug}.rs")),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn server_controller_exports_symbol(content: &str, symbol: &str) -> bool {
    content.contains(&format!("pub fn {symbol}("))
        || content.contains("pub use ")
            && (content.contains("controllers::*")
                || content.contains(&format!("::{symbol};"))
                || content.contains(&format!("::{symbol},"))
                || content.contains(&format!("{symbol}::*")))
}

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

pub(crate) fn normalize_module_ui_classification(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "dual_surface" | "admin_only" | "storefront_only" | "no_ui" | "capability_only"
        | "future_ui" => Ok(normalized),
        _ => anyhow::bail!("unsupported value"),
    }
}

pub(crate) fn validate_module_dependency_contract(
    slug: &str,
    spec: &ModuleSpec,
    manifest: &ModulePackageManifest,
    module_root: &Path,
) -> Result<()> {
    let manifest_dependencies = normalize_dependency_set(spec.depends_on.as_deref().unwrap_or(&[]));
    let package_dependencies = normalize_dependency_set(
        &manifest
            .dependencies
            .keys()
            .map(|dependency| dependency.to_string())
            .collect::<Vec<_>>(),
    );

    if manifest_dependencies != package_dependencies {
        anyhow::bail!(
            "Module '{slug}' dependency mismatch between modules.toml and rustok-module.toml: modules.toml={:?}, rustok-module.toml={:?}",
            manifest_dependencies,
            package_dependencies
        );
    }

    if let Some(runtime_dependencies) = extract_runtime_module_dependencies(module_root)? {
        if manifest_dependencies != runtime_dependencies {
            anyhow::bail!(
                "Module '{slug}' dependency mismatch between modules.toml and RusToKModule::dependencies(): modules.toml={:?}, runtime={:?}",
                manifest_dependencies,
                runtime_dependencies
            );
        }
    }

    Ok(())
}

pub(crate) fn validate_module_permission_contract(slug: &str, module_root: &Path) -> Result<()> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    if !content.contains("impl RusToKModule for") {
        return Ok(());
    }

    let Some(permission_body) = extract_runtime_method_body(&content, "permissions") else {
        return Ok(());
    };

    let permission_contract =
        load_core_permission_contract(&workspace_root().join("crates").join("rustok-core"))?;
    let mut seen = HashSet::new();

    for permission in extract_permission_constants(permission_body) {
        let Some(canonical) = permission_contract.constants.get(&permission) else {
            anyhow::bail!(
                "Module '{slug}' declares unknown Permission::{} in {}",
                permission,
                lib_path.display()
            );
        };
        if !seen.insert(canonical.clone()) {
            anyhow::bail!(
                "Module '{slug}' declares duplicate permission '{}' in {}",
                canonical,
                lib_path.display()
            );
        }
    }

    for (resource, action) in extract_permission_constructors(permission_body) {
        if !permission_contract.resources.contains(&resource) {
            anyhow::bail!(
                "Module '{slug}' declares unknown Resource::{} in Permission::new(...) at {}",
                resource,
                lib_path.display()
            );
        }
        if !permission_contract.actions.contains(&action) {
            anyhow::bail!(
                "Module '{slug}' declares unknown Action::{} in Permission::new(...) at {}",
                action,
                lib_path.display()
            );
        }

        let canonical = format!(
            "{}:{}",
            type_name_to_permission_segment(&resource),
            type_name_to_permission_segment(&action)
        );
        if !seen.insert(canonical.clone()) {
            anyhow::bail!(
                "Module '{slug}' declares duplicate permission '{}' in {}",
                canonical,
                lib_path.display()
            );
        }
    }

    for expected in expected_minimum_module_permissions(slug) {
        if !seen.contains(*expected) {
            anyhow::bail!(
                "Module '{slug}' must declare minimum runtime permission '{}' in {}",
                expected,
                lib_path.display()
            );
        }
    }

    Ok(())
}

pub(crate) fn validate_module_kind_contract(
    slug: &str,
    spec: &ModuleSpec,
    module_root: &Path,
) -> Result<()> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    if !content.contains("impl RusToKModule for") {
        return Ok(());
    }

    let runtime_kind = extract_runtime_module_kind(&content);
    if spec.required {
        if runtime_kind != Some("Core") {
            anyhow::bail!(
                "Module '{slug}' is required in modules.toml and must declare fn kind(&self) -> ModuleKind {{ ModuleKind::Core }} in {}",
                lib_path.display()
            );
        }
    } else if runtime_kind == Some("Core") {
        anyhow::bail!(
            "Module '{slug}' is optional in modules.toml and must not declare ModuleKind::Core in {}",
            lib_path.display()
        );
    }

    Ok(())
}

fn normalize_dependency_set(dependencies: &[String]) -> HashSet<String> {
    dependencies
        .iter()
        .map(|dependency| dependency.trim())
        .filter(|dependency| !dependency.is_empty())
        .map(|dependency| dependency.to_string())
        .collect()
}

#[derive(Debug, Default)]
struct CorePermissionContract {
    constants: HashMap<String, String>,
    resources: HashSet<String>,
    actions: HashSet<String>,
}

fn load_core_permission_contract(core_root: &Path) -> Result<CorePermissionContract> {
    let permissions_path = core_root.join("src").join("permissions.rs");
    let content = fs::read_to_string(&permissions_path)
        .with_context(|| format!("Failed to read {}", permissions_path.display()))?;

    let mut contract = CorePermissionContract::default();
    for capture in regex::Regex::new(
        r"pub const ([A-Z0-9_]+): Self = Self::new\(Resource::([A-Za-z0-9_]+), Action::([A-Za-z0-9_]+)\);",
    )
    .expect("permission constant regex should compile")
    .captures_iter(&content)
    {
        contract.constants.insert(
            capture[1].to_string(),
            format!(
                "{}:{}",
                type_name_to_permission_segment(&capture[2]),
                type_name_to_permission_segment(&capture[3])
            ),
        );
    }

    contract.resources = extract_enum_variants(&content, "pub enum Resource");
    contract.actions = extract_enum_variants(&content, "pub enum Action");
    Ok(contract)
}

fn expected_minimum_module_permissions(slug: &str) -> &'static [&'static str] {
    match slug {
        "auth" => &["users:manage"],
        "tenant" => &["tenants:manage", "modules:manage"],
        "rbac" => &["settings:manage", "logs:read"],
        "blog" => &["blog_posts:manage"],
        "forum" => &["forum_topics:manage"],
        "customer" => &["customers:manage"],
        "product" => &["products:manage"],
        "profiles" => &["profiles:manage"],
        "region" => &["regions:manage"],
        "order" => &["orders:manage"],
        "payment" => &["payments:manage"],
        "fulfillment" => &["fulfillments:manage"],
        "media" => &["media:manage"],
        "pages" => &["pages:manage"],
        "taxonomy" => &["taxonomy:manage"],
        "workflow" => &["workflows:manage"],
        "alloy" => &["scripts:manage"],
        _ => &[],
    }
}

fn extract_enum_variants(content: &str, enum_marker: &str) -> HashSet<String> {
    let Some(marker_index) = content.find(enum_marker) else {
        return HashSet::new();
    };
    let tail = &content[marker_index..];
    let Some(body_start_offset) = tail.find('{') else {
        return HashSet::new();
    };
    let body = &tail[(body_start_offset + 1)..];
    let mut depth = 1usize;
    let mut end_index = None;
    for (index, ch) in body.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end_index = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(end_index) = end_index else {
        return HashSet::new();
    };
    let body = &body[..end_index];

    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .filter_map(|line| line.strip_suffix(','))
        .filter(|line| {
            line.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
        .map(|line| line.to_string())
        .collect()
}

fn type_name_to_permission_segment(value: &str) -> String {
    let mut result = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn extract_permission_constants(content: &str) -> Vec<String> {
    regex::Regex::new(r"Permission::([A-Z0-9_]+)")
        .expect("permission constant regex should compile")
        .captures_iter(content)
        .map(|capture| capture[1].to_string())
        .collect()
}

fn extract_permission_constructors(content: &str) -> Vec<(String, String)> {
    regex::Regex::new(r"Permission::new\(Resource::([A-Za-z0-9_]+),\s*Action::([A-Za-z0-9_]+)\)")
        .expect("permission ctor regex should compile")
        .captures_iter(content)
        .map(|capture| (capture[1].to_string(), capture[2].to_string()))
        .collect()
}

pub(crate) fn validate_module_ui_surface_contract(
    slug: &str,
    module_root: &Path,
    surface: &str,
    crate_name: Option<&str>,
) -> Result<()> {
    let manifest_path = module_root.join(surface).join("Cargo.toml");
    let has_subcrate = manifest_path.exists();
    let crate_name = crate_name.map(str::trim).filter(|value| !value.is_empty());

    if has_subcrate && crate_name.is_none() {
        anyhow::bail!(
            "Module '{slug}' contains {}, but rustok-module.toml is missing [provides.{surface}_ui].leptos_crate",
            manifest_path.display()
        );
    }

    if !has_subcrate && crate_name.is_some() {
        anyhow::bail!(
            "Module '{slug}' declares [provides.{surface}_ui].leptos_crate, but {} is missing",
            manifest_path.display()
        );
    }

    Ok(())
}

pub(crate) fn validate_module_ui_package(
    slug: &str,
    module_root: &Path,
    surface: &str,
    crate_name: Option<&str>,
    expected_version: &str,
    workspace_manifest: &TomlValue,
) -> Result<Option<ModuleUiPackagePreview>> {
    let Some(crate_name) = crate_name else {
        return Ok(None);
    };

    let manifest_path = module_root.join(surface).join("Cargo.toml");
    if !manifest_path.exists() {
        anyhow::bail!(
            "Module '{slug}' declares provides.{surface}_ui.leptos_crate='{crate_name}', but {} is missing",
            manifest_path.display()
        );
    }

    let package = load_resolved_cargo_package(&manifest_path, workspace_manifest)?;
    if package.name != crate_name {
        anyhow::bail!(
            "Module '{slug}' declares provides.{surface}_ui.leptos_crate='{crate_name}', but {} declares '{}'",
            manifest_path.display(),
            package.name
        );
    }
    if package.version != expected_version {
        anyhow::bail!(
            "Module '{slug}' {surface} package version mismatch: expected '{expected_version}', got '{}'",
            package.version
        );
    }

    Ok(Some(ModuleUiPackagePreview {
        crate_name: package.name,
        manifest_path: manifest_path.display().to_string(),
    }))
}
