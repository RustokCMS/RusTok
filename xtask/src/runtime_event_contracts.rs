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
