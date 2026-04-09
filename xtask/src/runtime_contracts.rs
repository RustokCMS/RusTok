use super::*;

pub(crate) fn validate_module_event_listener_contract(
    slug: &str,
    module_root: &Path,
) -> Result<()> {
    let expected_fragments: &[&str] = match slug {
        "index" => &[
            "fn register_event_listeners(",
            "IndexerRuntimeConfig",
            "ContentIndexer::with_runtime",
            "ProductIndexer::with_runtime",
        ],
        "search" => &[
            "fn register_event_listeners(",
            "SearchIngestionHandler::new",
        ],
        "workflow" => &[
            "fn register_event_listeners(",
            "WorkflowTriggerHandler::new",
        ],
        _ => return Ok(()),
    };

    let lib_rs_path = module_root.join("src").join("lib.rs");
    let content = fs::read_to_string(&lib_rs_path)
        .with_context(|| format!("Failed to read {}", lib_rs_path.display()))?;

    if slug == "index" {
        let legacy_listener_path = module_root.join("src").join("listener.rs");
        if legacy_listener_path.exists() {
            anyhow::bail!(
                "Module '{slug}' event listener contract drift: legacy file {} must be removed after registry-driven migration",
                legacy_listener_path.display()
            );
        }
    }

    for fragment in expected_fragments {
        if !content.contains(fragment) {
            anyhow::bail!(
                "Module '{slug}' event listener contract drift: {} must contain '{}'",
                lib_rs_path.display(),
                fragment
            );
        }
    }

    Ok(())
}

pub(crate) fn validate_module_event_ingress_contract(
    manifest_path: &Path,
    slug: &str,
    module_root: &Path,
) -> Result<()> {
    if slug != "workflow" {
        return Ok(());
    }

    let controllers_mod_path = module_root.join("src").join("controllers").join("mod.rs");
    let trigger_handler_path = module_root
        .join("src")
        .join("services")
        .join("trigger_handler.rs");
    let controllers_mod = fs::read_to_string(&controllers_mod_path)
        .with_context(|| format!("Failed to read {}", controllers_mod_path.display()))?;
    let trigger_handler = fs::read_to_string(&trigger_handler_path)
        .with_context(|| format!("Failed to read {}", trigger_handler_path.display()))?;

    if !controllers_mod.contains("pub fn webhook_routes()") {
        anyhow::bail!(
            "Module '{slug}' event ingress contract drift: {} must export webhook_routes()",
            controllers_mod_path.display()
        );
    }
    if !controllers_mod.contains(".prefix(\"webhooks\")") {
        anyhow::bail!(
            "Module '{slug}' event ingress contract drift: {} must keep webhook ingress under the 'webhooks' prefix",
            controllers_mod_path.display()
        );
    }
    if !trigger_handler.contains("impl EventHandler for WorkflowTriggerHandler") {
        anyhow::bail!(
            "Module '{slug}' event ingress contract drift: {} must implement EventHandler for WorkflowTriggerHandler",
            trigger_handler_path.display()
        );
    }

    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let server_shim_path = workspace_root
        .join("apps")
        .join("server")
        .join("src")
        .join("controllers")
        .join("workflow")
        .join("mod.rs");
    let server_shim = fs::read_to_string(&server_shim_path)
        .with_context(|| format!("Failed to read {}", server_shim_path.display()))?;
    if !server_shim.contains("rustok_workflow::controllers::webhook_routes()") {
        anyhow::bail!(
            "Module '{slug}' event ingress contract drift: {} must re-export rustok_workflow::controllers::webhook_routes()",
            server_shim_path.display()
        );
    }

    Ok(())
}

pub(crate) fn validate_module_index_search_boundary_contract(
    slug: &str,
    module_root: &Path,
) -> Result<()> {
    match slug {
        "index" => {
            let lib_rs_path = module_root.join("src").join("lib.rs");
            let readme_path = module_root.join("README.md");
            let lib_rs = fs::read_to_string(&lib_rs_path)
                .with_context(|| format!("Failed to read {}", lib_rs_path.display()))?;
            let readme = fs::read_to_string(&readme_path)
                .with_context(|| format!("Failed to read {}", readme_path.display()))?;

            for fragment in [
                "IndexerRuntimeConfig",
                "ContentIndexer::with_runtime",
                "ProductIndexer::with_runtime",
                "read-model substrate",
            ] {
                if !(lib_rs.contains(fragment) || readme.contains(fragment)) {
                    anyhow::bail!(
                        "Module '{slug}' boundary contract drift: expected '{}' in {} or {}",
                        fragment,
                        lib_rs_path.display(),
                        readme_path.display()
                    );
                }
            }

            for forbidden in [
                "SearchEngineKind",
                "PgSearchEngine",
                "SearchIngestionHandler",
            ] {
                if lib_rs.contains(forbidden) {
                    anyhow::bail!(
                        "Module '{slug}' boundary contract drift: {} must not expose search-owned symbol '{}'",
                        lib_rs_path.display(),
                        forbidden
                    );
                }
            }
        }
        "search" => {
            let lib_rs_path = module_root.join("src").join("lib.rs");
            let readme_path = module_root.join("README.md");
            let lib_rs = fs::read_to_string(&lib_rs_path)
                .with_context(|| format!("Failed to read {}", lib_rs_path.display()))?;
            let readme = fs::read_to_string(&readme_path)
                .with_context(|| format!("Failed to read {}", readme_path.display()))?;

            for fragment in [
                "SearchEngineKind",
                "PgSearchEngine",
                "SearchIngestionHandler",
                "search_documents",
            ] {
                if !(lib_rs.contains(fragment) || readme.contains(fragment)) {
                    anyhow::bail!(
                        "Module '{slug}' boundary contract drift: expected '{}' in {} or {}",
                        fragment,
                        lib_rs_path.display(),
                        readme_path.display()
                    );
                }
            }

            for forbidden in [
                "IndexerRuntimeConfig",
                "ContentIndexer::with_runtime",
                "ProductIndexer::with_runtime",
            ] {
                if lib_rs.contains(forbidden) {
                    anyhow::bail!(
                        "Module '{slug}' boundary contract drift: {} must not expose index-owned symbol '{}'",
                        lib_rs_path.display(),
                        forbidden
                    );
                }
            }
        }
        _ => {}
    }

    Ok(())
}

pub(crate) fn validate_module_search_operator_surface_contract(
    slug: &str,
    module_root: &Path,
) -> Result<()> {
    if slug != "search" {
        return Ok(());
    }

    let lib_rs_path = module_root.join("src").join("lib.rs");
    let readme_path = module_root.join("README.md");
    let runbook_path = module_root.join("docs").join("observability-runbook.md");
    let lib_rs = fs::read_to_string(&lib_rs_path)
        .with_context(|| format!("Failed to read {}", lib_rs_path.display()))?;
    let readme = fs::read_to_string(&readme_path)
        .with_context(|| format!("Failed to read {}", readme_path.display()))?;

    for fragment in [
        "SearchDiagnosticsService",
        "SearchAnalyticsService",
        "SearchSettingsService",
        "SearchDictionaryService",
    ] {
        if !lib_rs.contains(fragment) {
            anyhow::bail!(
                "Module '{slug}' operator surface contract drift: {} must expose '{}'",
                lib_rs_path.display(),
                fragment
            );
        }
    }

    for fragment in [
        "searchDiagnostics",
        "searchAnalytics",
        "searchSettingsPreview",
        "triggerSearchRebuild",
    ] {
        if !readme.contains(fragment) {
            anyhow::bail!(
                "Module '{slug}' operator surface contract drift: {} must document '{}'",
                readme_path.display(),
                fragment
            );
        }
    }

    if !runbook_path.exists() {
        anyhow::bail!(
            "Module '{slug}' operator surface contract drift: {} must exist",
            runbook_path.display()
        );
    }

    Ok(())
}

pub(crate) fn validate_server_event_runtime_contract(manifest_path: &Path) -> Result<()> {
    let workspace_root = manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve workspace root from modules manifest {}",
            manifest_path.display()
        )
    })?;
    let services_dir = workspace_root
        .join("apps")
        .join("server")
        .join("src")
        .join("services");
    let app_runtime_path = services_dir.join("app_runtime.rs");
    let services_mod_path = services_dir.join("mod.rs");
    let module_dispatcher_path = services_dir.join("module_event_dispatcher.rs");
    let legacy_index_dispatcher_path = services_dir.join("index_dispatcher.rs");
    let legacy_search_dispatcher_path = services_dir.join("search_dispatcher.rs");

    let app_runtime = fs::read_to_string(&app_runtime_path)
        .with_context(|| format!("Failed to read {}", app_runtime_path.display()))?;
    let services_mod = fs::read_to_string(&services_mod_path)
        .with_context(|| format!("Failed to read {}", services_mod_path.display()))?;

    if !module_dispatcher_path.exists() {
        anyhow::bail!(
            "Server event runtime contract drift: {} is missing",
            module_dispatcher_path.display()
        );
    }
    if legacy_index_dispatcher_path.exists() {
        anyhow::bail!(
            "Server event runtime contract drift: legacy dispatcher {} must not exist",
            legacy_index_dispatcher_path.display()
        );
    }
    if legacy_search_dispatcher_path.exists() {
        anyhow::bail!(
            "Server event runtime contract drift: legacy dispatcher {} must not exist",
            legacy_search_dispatcher_path.display()
        );
    }
    if !services_mod.contains("pub mod module_event_dispatcher;") {
        anyhow::bail!(
            "Server event runtime contract drift: {} must export module_event_dispatcher",
            services_mod_path.display()
        );
    }
    if app_runtime.contains("spawn_index_dispatcher")
        || app_runtime.contains("spawn_search_dispatcher")
    {
        anyhow::bail!(
            "Server event runtime contract drift: {} still references legacy index/search dispatchers",
            app_runtime_path.display()
        );
    }
    if !app_runtime.contains("spawn_module_event_dispatcher") {
        anyhow::bail!(
            "Server event runtime contract drift: {} must bootstrap spawn_module_event_dispatcher",
            app_runtime_path.display()
        );
    }
    if app_runtime.contains("WorkflowTriggerHandler") {
        anyhow::bail!(
            "Server event runtime contract drift: {} must not wire WorkflowTriggerHandler directly",
            app_runtime_path.display()
        );
    }
    if !app_runtime.contains("WorkflowCronScheduler") {
        anyhow::bail!(
            "Server event runtime contract drift: {} must keep WorkflowCronScheduler as separate background runtime",
            app_runtime_path.display()
        );
    }

    Ok(())
}
