use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use toml::Value as TomlValue;

mod module_commands;
mod module_contracts;
mod parsing_helpers;
mod preview_builders;
mod registry_transport;
mod runtime_contracts;
mod server_contracts;
mod ui_contracts;
mod validation_runner;

use module_commands::*;
use module_contracts::*;
use parsing_helpers::*;
use preview_builders::*;
use registry_transport::*;
use runtime_contracts::*;
use server_contracts::*;
use ui_contracts::*;
use validation_runner::*;

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: u32,
    #[allow(dead_code)]
    app: String,
    #[allow(dead_code)]
    build: Option<BuildConfig>,
    modules: HashMap<String, ModuleSpec>,
    settings: Option<Settings>,
}

#[derive(Debug, Deserialize)]
struct BuildConfig {
    #[allow(dead_code)]
    target: Option<String>,
    #[allow(dead_code)]
    profile: Option<String>,
    #[allow(dead_code)]
    deployment_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModuleSpec {
    #[serde(rename = "crate")]
    crate_name: String,
    source: String,
    path: Option<String>,
    #[serde(default)]
    required: bool,
    version: Option<String>,
    git: Option<String>,
    #[allow(dead_code)]
    rev: Option<String>,
    #[allow(dead_code)]
    depends_on: Option<Vec<String>>,
    #[allow(dead_code)]
    features: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Settings {
    default_enabled: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageManifest {
    #[serde(default)]
    module: ModulePackageMetadata,
    #[serde(default)]
    marketplace: ModulePackageMarketplaceMetadata,
    #[serde(default)]
    dependencies: HashMap<String, ModulePackageDependency>,
    #[serde(rename = "crate", default)]
    crate_contract: ModulePackageCrateContract,
    #[serde(default)]
    provides: ModulePackageProvides,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageMetadata {
    #[serde(default)]
    slug: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    ownership: String,
    #[serde(default)]
    trust_level: String,
    #[serde(default)]
    ui_classification: String,
    #[serde(default)]
    recommended_admin_surfaces: Vec<String>,
    #[serde(default)]
    showcase_admin_surfaces: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageDependency {
    #[allow(dead_code)]
    #[serde(default)]
    version_req: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageMarketplaceMetadata {
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageCrateContract {
    #[serde(default)]
    entry_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModulePackageProvides {
    #[serde(default)]
    graphql: Option<ModuleGraphqlProvides>,
    #[serde(default)]
    http: Option<ModuleHttpProvides>,
    #[serde(default)]
    admin_ui: Option<ModuleUiProvides>,
    #[serde(default)]
    storefront_ui: Option<ModuleUiProvides>,
}

#[derive(Debug, Deserialize, Default)]
struct ModuleGraphqlProvides {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    mutation: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModuleHttpProvides {
    #[serde(default)]
    routes: Option<String>,
    #[serde(default)]
    webhook_routes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ModuleUiProvides {
    #[serde(default)]
    leptos_crate: Option<String>,
    #[serde(default)]
    route_segment: Option<String>,
    #[serde(default)]
    nav_label: Option<String>,
    #[serde(default)]
    slot: Option<String>,
    #[serde(default)]
    page_title: Option<String>,
    #[serde(default)]
    i18n: Option<ModuleUiI18nProvides>,
}

#[derive(Debug, Deserialize, Default)]
struct ModuleUiI18nProvides {
    #[serde(default)]
    default_locale: Option<String>,
    #[serde(default)]
    supported_locales: Vec<String>,
    #[serde(default)]
    leptos_locales_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModulePublishDryRunPreview {
    slug: String,
    version: String,
    crate_name: String,
    module_name: String,
    module_description: String,
    ownership: String,
    trust_level: String,
    license: String,
    manifest_path: String,
    package_manifest_path: String,
    module_entry_type: Option<String>,
    marketplace: ModuleMarketplacePreview,
    ui_packages: ModuleUiPackagesPreview,
}

#[derive(Debug, Serialize)]
struct ModuleMarketplacePreview {
    category: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ModuleUiPackagesPreview {
    admin: Option<ModuleUiPackagePreview>,
    storefront: Option<ModuleUiPackagePreview>,
}

#[derive(Debug, Serialize)]
struct ModuleUiPackagePreview {
    crate_name: String,
    manifest_path: String,
}

#[derive(Debug, Serialize)]
struct RegistryPublishHttpRequest {
    schema_version: u32,
    dry_run: bool,
    module: RegistryPublishModuleHttpRequest,
}

#[derive(Debug, Serialize)]
struct RegistryPublishModuleHttpRequest {
    slug: String,
    version: String,
    crate_name: String,
    name: String,
    description: String,
    ownership: String,
    trust_level: String,
    license: String,
    entry_type: Option<String>,
    marketplace: RegistryPublishMarketplaceHttpRequest,
    ui_packages: RegistryPublishUiPackagesHttpRequest,
}

#[derive(Debug, Serialize)]
struct RegistryPublishMarketplaceHttpRequest {
    category: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegistryPublishUiPackagesHttpRequest {
    admin: Option<RegistryPublishUiPackageHttpRequest>,
    storefront: Option<RegistryPublishUiPackageHttpRequest>,
}

#[derive(Debug, Serialize)]
struct RegistryPublishUiPackageHttpRequest {
    crate_name: String,
}

#[derive(Debug, Serialize)]
struct RegistryYankHttpRequest {
    schema_version: u32,
    dry_run: bool,
    slug: String,
    version: String,
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegistryOwnerTransferHttpRequest {
    schema_version: u32,
    dry_run: bool,
    slug: String,
    new_owner_actor: String,
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegistryValidationStageHttpRequest {
    schema_version: u32,
    dry_run: bool,
    stage: String,
    status: String,
    detail: Option<String>,
    reason_code: Option<String>,
    requeue: bool,
}

#[derive(Debug, Serialize)]
struct RegistryRunnerClaimHttpRequest {
    schema_version: u32,
    runner_id: String,
    #[serde(rename = "supportedStages")]
    supported_stages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegistryRunnerHeartbeatHttpRequest {
    schema_version: u32,
    runner_id: String,
}

#[derive(Debug, Serialize)]
struct RegistryRunnerCompletionHttpRequest {
    schema_version: u32,
    runner_id: String,
    detail: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryMutationHttpResponse {
    accepted: bool,
    #[allow(dead_code)]
    action: Option<String>,
    #[allow(dead_code)]
    dry_run: Option<bool>,
    request_id: Option<String>,
    status: Option<String>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    errors: Vec<String>,
    next_step: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryPublishStatusHttpResponse {
    request_id: String,
    slug: String,
    version: String,
    status: String,
    accepted: bool,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default, rename = "followUpGates")]
    follow_up_gates: Vec<RegistryPublishStatusFollowUpGate>,
    #[serde(default, rename = "validationStages")]
    validation_stages: Vec<RegistryPublishStatusValidationStage>,
    #[serde(default, rename = "approvalOverrideRequired")]
    approval_override_required: bool,
    #[serde(default, rename = "approvalOverrideReasonCodes")]
    approval_override_reason_codes: Vec<String>,
    #[serde(default, rename = "governanceActions")]
    governance_actions: Vec<RegistryGovernanceActionHttpResponse>,
    next_step: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryGovernanceActionHttpResponse {
    key: String,
    #[serde(rename = "reasonRequired")]
    reason_required: bool,
    #[serde(rename = "reasonCodeRequired")]
    reason_code_required: bool,
    #[serde(default, rename = "reasonCodes")]
    reason_codes: Vec<String>,
    destructive: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryPublishStatusFollowUpGate {
    key: String,
    status: String,
    detail: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryPublishStatusValidationStage {
    key: String,
    status: String,
    detail: String,
    #[serde(rename = "attemptNumber")]
    attempt_number: i32,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "startedAt")]
    started_at: Option<String>,
    #[serde(rename = "finishedAt")]
    finished_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryRunnerClaimHttpResponse {
    accepted: bool,
    claim: Option<RegistryRunnerClaimHttpPayload>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryRunnerClaimHttpPayload {
    #[serde(rename = "claimId")]
    claim_id: String,
    #[serde(rename = "requestId")]
    request_id: String,
    slug: String,
    version: String,
    #[serde(rename = "stageKey")]
    stage_key: String,
    #[serde(rename = "executionMode")]
    execution_mode: String,
    runnable: bool,
    #[serde(rename = "requiresManualConfirmation")]
    requires_manual_confirmation: bool,
    #[serde(default, rename = "allowedTerminalReasonCodes")]
    allowed_terminal_reason_codes: Vec<String>,
    #[serde(default, rename = "suggestedPassReasonCode")]
    suggested_pass_reason_code: Option<String>,
    #[serde(default, rename = "suggestedFailureReasonCode")]
    suggested_failure_reason_code: Option<String>,
    #[serde(default, rename = "suggestedBlockedReasonCode")]
    suggested_blocked_reason_code: Option<String>,
    #[serde(rename = "artifactUrl")]
    artifact_url: String,
    #[serde(rename = "artifactChecksumSha256")]
    artifact_checksum_sha256: String,
    #[serde(rename = "crateName")]
    crate_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryRunnerMutationHttpResponse {
    accepted: bool,
    #[serde(rename = "claimId")]
    claim_id: String,
    status: String,
    #[serde(default)]
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegistryPublishValidationHttpRequest {
    schema_version: u32,
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct RegistryPublishDecisionHttpRequest {
    schema_version: u32,
    dry_run: bool,
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModulePublishGovernanceDryRunPreview {
    action: String,
    request_id: String,
    actor: Option<String>,
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModuleYankDryRunPreview {
    action: String,
    slug: String,
    version: String,
    crate_name: String,
    current_local_version: String,
    matches_local_version: bool,
    package_manifest_path: String,
}

#[derive(Debug, Serialize)]
struct ModuleOwnerTransferDryRunPreview {
    action: String,
    slug: String,
    crate_name: String,
    current_local_version: String,
    package_manifest_path: String,
    new_owner_actor: String,
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModuleValidationStageDryRunPreview {
    action: String,
    request_id: String,
    stage: String,
    status: String,
    detail: Option<String>,
    reason_code: Option<String>,
    requeue: bool,
}

#[derive(Debug, Serialize)]
struct ModuleValidationStageRunPreview {
    action: String,
    slug: String,
    request_id: String,
    stage: String,
    requires_manual_confirmation: bool,
    running_detail: String,
    success_detail: String,
    failure_detail_prefix: String,
    commands: Vec<ModuleCommandPreview>,
}

#[derive(Debug, Serialize)]
struct ModuleRunnerPreview {
    action: String,
    runner_id: String,
    supported_stages: Vec<String>,
    poll_interval_ms: u64,
    heartbeat_interval_ms: u64,
    once: bool,
    confirm_manual_review: bool,
}

#[derive(Debug, Serialize)]
struct ModuleTestPlanPreview {
    slug: String,
    version: String,
    commands: Vec<ModuleCommandPreview>,
}

#[derive(Debug, Serialize)]
struct ModuleCommandPreview {
    label: String,
    argv: Vec<String>,
}

#[derive(Debug)]
struct ResolvedCargoPackage {
    name: String,
    version: String,
    license: Option<String>,
    manifest_path: PathBuf,
}

const REGISTRY_MUTATION_SCHEMA_VERSION: u32 = 1;
const REGISTRY_OWNER_TRANSFER_REASON_CODES: &[&str] = &[
    "maintenance_handoff",
    "team_restructure",
    "publisher_rotation",
    "security_emergency",
    "governance_override",
    "other",
];
const REGISTRY_APPROVE_OVERRIDE_REASON_CODES: &[&str] = &[
    "manual_review_complete",
    "trusted_first_party",
    "expedited_release",
    "governance_override",
    "other",
];
const REGISTRY_REQUEST_CHANGES_REASON_CODES: &[&str] = &[
    "artifact_mismatch",
    "quality_gap",
    "policy_gap",
    "docs_gap",
    "other",
];
const REGISTRY_HOLD_REASON_CODES: &[&str] = &[
    "release_window",
    "incident",
    "legal_hold",
    "security_review",
    "other",
];
const REGISTRY_RESUME_REASON_CODES: &[&str] = &[
    "review_complete",
    "incident_closed",
    "legal_cleared",
    "other",
];
const REGISTRY_VALIDATION_STAGE_REASON_CODES: &[&str] = &[
    "local_runner_passed",
    "manual_review_complete",
    "build_failure",
    "test_failure",
    "policy_preflight_failed",
    "security_findings",
    "policy_exception",
    "license_issue",
    "manual_override",
    "other",
];
const REGISTRY_YANK_REASON_CODES: &[&str] = &[
    "security",
    "legal",
    "malware",
    "critical_regression",
    "rollback",
    "other",
];
const REMOTE_RUNNER_TOKEN_ENV: &str = "RUSTOK_MODULE_RUNNER_TOKEN";
const DEFAULT_REMOTE_RUNNER_POLL_INTERVAL_MS: u64 = 5_000;
const DEFAULT_REMOTE_RUNNER_HEARTBEAT_INTERVAL_MS: u64 = 5_000;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate-registry" => generate_registry()?,
        "validate-manifest" => validate_manifest()?,
        "list-modules" => list_modules()?,
        "module" => module_command(&args[2..])?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: cargo xtask <command>");
    println!();
    println!("Commands:");
    println!("  generate-registry   Generate ModuleRegistry from modules.toml");
    println!("  validate-manifest   Validate modules.toml and rustok-module.toml files");
    println!("  list-modules        List all configured modules");
    println!("  module validate     Validate module publish-readiness contracts");
    println!("  module test         Run or preview local module smoke checks");
    println!("  module stage-run    Execute a local follow-up validation stage and report it");
    println!("  module runner       Run a thin remote validation worker against runner/* API");
    println!("  module publish      Create/preview a publish request and stop at review-ready unless --auto-approve is set");
    println!("  module request-changes Request a fresh artifact revision for an approved publish request");
    println!("  module hold         Place a publish request on hold");
    println!("  module resume       Resume a held publish request");
    println!("  module stage        Record or requeue a follow-up validation stage");
    println!("  module owner-transfer Emit a dry-run owner transfer payload preview");
    println!("  module yank         Emit a dry-run yank payload preview");
}

fn is_valid_module_ownership(value: &str) -> bool {
    matches!(value, "first_party" | "third_party")
}

fn is_valid_trust_level(value: &str) -> bool {
    matches!(value, "core" | "verified" | "unverified" | "private")
}

fn is_valid_admin_surface(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn validate_admin_surfaces(
    slug: &str,
    field: &str,
    surfaces: &[String],
) -> Result<HashSet<String>> {
    let mut normalized = HashSet::new();

    for surface in surfaces {
        let surface = surface.trim();
        if !is_valid_admin_surface(surface) {
            anyhow::bail!(
                "Module '{}' has invalid admin surface '{}' in {}",
                slug,
                surface,
                field
            );
        }
        normalized.insert(surface.to_string());
    }

    Ok(normalized)
}

fn module_validate_command(args: &[String]) -> Result<()> {
    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let explicit_slug = args.first().map(String::as_str);
    let targets = selected_modules(&manifest, explicit_slug)?;
    let mut skipped_without_package_manifest = 0usize;

    println!("Validating module publish-readiness contracts...");

    for (slug, spec) in targets {
        if explicit_slug.is_none() {
            let Some(package_manifest_path) = module_package_manifest_path(&manifest_path, spec)
            else {
                continue;
            };
            if !package_manifest_path.exists() {
                skipped_without_package_manifest += 1;
                continue;
            }
        }

        let preview =
            build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
        println!(
            "  PASS {slug} -> {} v{}",
            preview.crate_name, preview.version
        );
    }

    if skipped_without_package_manifest > 0 {
        println!(
            "  Skipped {skipped_without_package_manifest} path modules without rustok-module.toml publish contract"
        );
    }

    Ok(())
}

fn selected_modules<'a>(
    manifest: &'a Manifest,
    slug: Option<&'a str>,
) -> Result<Vec<(&'a str, &'a ModuleSpec)>> {
    if let Some(slug) = slug {
        let spec = manifest
            .modules
            .get(slug)
            .with_context(|| format!("Unknown module slug '{slug}'"))?;
        return Ok(vec![(slug, spec)]);
    }

    let mut modules = manifest
        .modules
        .iter()
        .filter(|(_, spec)| spec.source == "path")
        .map(|(slug, spec)| (slug.as_str(), spec))
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| left.0.cmp(right.0));
    Ok(modules)
}

fn validate_module_local_docs_contract(slug: &str, module_root: &Path) -> Result<()> {
    validate_module_local_docs_file(
        slug,
        &module_root.join("docs").join("README.md"),
        &[
            "## Назначение",
            "## Зона ответственности",
            "## Интеграция",
            "## Проверка",
            "## Связанные документы",
        ],
    )?;
    validate_module_local_docs_file(
        slug,
        &module_root.join("docs").join("implementation-plan.md"),
        &[
            "## Область работ",
            "## Текущее состояние",
            "## Этапы",
            "## Проверка",
            "## Правила обновления",
        ],
    )?;

    Ok(())
}

fn generate_registry() -> Result<()> {
    println!("Generating ModuleRegistry from modules.toml...");

    let manifest = load_manifest()?;
    let output_path = Path::new("apps/server/src/modules/generated.rs");

    fs::create_dir_all(output_path.parent().unwrap())
        .context("Failed to create modules directory")?;

    let mut code = String::new();
    code.push_str("// AUTO-GENERATED by `cargo xtask generate-registry`\n");
    code.push_str("// DO NOT EDIT MANUALLY\n");
    code.push_str("// Generated from modules.toml\n\n");
    code.push_str("use rustok_core::ModuleRegistry;\n\n");

    for (slug, spec) in &manifest.modules {
        let module_struct = to_pascal_case(slug);
        let crate_name = spec.crate_name.replace("-", "_");
        code.push_str(&format!("use {}::{}Module;\n", crate_name, module_struct));
    }

    code.push_str("\n/// Build ModuleRegistry from configured modules\n");
    code.push_str("pub fn build_registry() -> ModuleRegistry {\n");
    code.push_str("    let mut registry = ModuleRegistry::new();\n\n");

    for slug in manifest.modules.keys() {
        let module_struct = to_pascal_case(slug);
        code.push_str(&format!("    // Register {} module\n", slug));
        code.push_str(&format!(
            "    registry.register(Box::new({}Module::new()));\n\n",
            module_struct
        ));
    }

    code.push_str("    registry\n");
    code.push_str("}\n");

    fs::write(output_path, code).context("Failed to write generated.rs")?;

    println!("вњ“ Generated: {}", output_path.display());
    println!("  Registered {} modules", manifest.modules.len());

    Ok(())
}

fn validate_manifest() -> Result<()> {
    println!("Validating modules.toml and rustok-module.toml...");

    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let installed = manifest.modules.keys().cloned().collect::<HashSet<_>>();

    let missing_defaults = manifest
        .settings
        .as_ref()
        .and_then(|settings| settings.default_enabled.as_ref())
        .into_iter()
        .flatten()
        .filter(|slug| !installed.contains(*slug))
        .cloned()
        .collect::<Vec<_>>();

    if !missing_defaults.is_empty() {
        anyhow::bail!(
            "default_enabled contains modules not present in modules.toml: {}",
            missing_defaults.join(", ")
        );
    }

    validate_default_enabled_server_contract(&manifest_path, &manifest)?;
    validate_host_ui_inventory_contract(&manifest_path, &manifest)?;
    validate_server_event_runtime_contract(&manifest_path)?;

    let mut module_manifest_count = 0usize;

    for (slug, spec) in &manifest.modules {
        match spec.source.as_str() {
            "path" => {
                if spec.path.is_none() {
                    anyhow::bail!("Module '{}' has source='path' but no path specified", slug);
                }
            }
            "git" => {
                if spec.git.is_none() {
                    anyhow::bail!(
                        "Module '{}' has source='git' but no git URL specified",
                        slug
                    );
                }
            }
            "registry" | "crates-io" => {
                if spec.version.is_none() {
                    anyhow::bail!(
                        "Module '{}' has source='{}' but no version specified",
                        slug,
                        spec.source
                    );
                }
            }
            other => anyhow::bail!("Module '{}' has invalid source '{}'", slug, other),
        }

        let missing_dependencies = spec
            .depends_on
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|dependency| !installed.contains(*dependency))
            .cloned()
            .collect::<Vec<_>>();

        if !missing_dependencies.is_empty() {
            anyhow::bail!(
                "Module '{}' depends on missing modules: {}",
                slug,
                missing_dependencies.join(", ")
            );
        }

        if spec.source == "path" {
            let package_path =
                module_package_manifest_path(&manifest_path, spec).with_context(|| {
                    format!("Module '{}' has source='path' but no path specified", slug)
                })?;
            if !package_path.exists() {
                anyhow::bail!(
                    "Module '{}' requires rustok-module.toml at {}",
                    slug,
                    package_path.display()
                );
            }

            let package_manifest = load_module_package_manifest(&package_path)?;
            validate_module_package_metadata(slug, &package_manifest.module)?;
            validate_module_semantics_contract(slug, spec, &package_manifest.module)?;
            let module_root = package_path.parent().with_context(|| {
                format!(
                    "Failed to resolve module root for '{}'",
                    package_path.display()
                )
            })?;
            validate_module_event_listener_contract(slug, module_root)?;
            module_manifest_count += 1;
        }
    }

    println!("вњ“ Manifest is valid");
    println!("  Schema: {}", manifest.schema);
    println!("  Modules: {}", manifest.modules.len());
    println!("  Module manifests: {}", module_manifest_count);

    Ok(())
}

fn list_modules() -> Result<()> {
    let manifest = load_manifest()?;

    println!("Configured modules:");
    println!();

    for (slug, spec) in &manifest.modules {
        println!("  {}:", slug);
        println!("    crate: {}", spec.crate_name);
        println!("    source: {}", spec.source);
        if let Some(ref path) = spec.path {
            println!("    path: {}", path);
        }
        if let Some(ref version) = spec.version {
            println!("    version: {}", version);
        }
        if let Some(ref depends_on) = spec.depends_on {
            println!("    depends_on: {:?}", depends_on);
        }
        println!();
    }

    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        actor_argument, auto_approve_argument, build_live_owner_transfer_registry_request,
        build_live_publish_registry_request, build_live_validation_stage_registry_request,
        build_live_yank_registry_request, build_module_test_plan,
        build_owner_transfer_registry_request, build_publish_registry_request,
        build_validation_stage_registry_request, build_yank_registry_request, detail_argument,
        extract_runtime_module_dependencies, load_manifest_from, module_hold_command,
        module_owner_transfer_command, module_publish_command, module_request_changes_command,
        module_resume_command, module_runner_command, module_stage_command,
        module_stage_run_command, module_test_command, module_yank_command,
        normalize_module_ui_classification, positive_u64_argument, publish_status_action_available,
        reason_argument, reason_code_argument, registry_endpoint_uses_loopback,
        registry_url_argument, resolve_workspace_inherited_string, runner_token_argument,
        supported_remote_runner_stages, validate_default_enabled_server_contract,
        validate_host_ui_inventory_contract, validate_module_admin_surface_contract,
        validate_module_docs_navigation_contract, validate_module_entry_type_contract,
        validate_module_event_ingress_contract, validate_module_event_listener_contract,
        validate_module_host_ui_contract, validate_module_index_search_boundary_contract,
        validate_module_kind_contract, validate_module_local_docs_file,
        validate_module_permission_contract, validate_module_publish_contract,
        validate_module_runtime_metadata_contract,
        validate_module_search_operator_surface_contract, validate_module_semantics_contract,
        validate_module_server_http_surface_contract, validate_module_server_registry_contract,
        validate_module_transport_surface_contract, validate_module_ui_classification_contract,
        validate_module_ui_metadata_contract, validate_module_ui_surface_contract,
        validate_server_event_runtime_contract, workspace_root, Manifest, ModuleMarketplacePreview,
        ModuleOwnerTransferDryRunPreview, ModulePackageManifest, ModulePackageMetadata,
        ModulePublishDryRunPreview, ModuleSpec, ModuleUiPackagePreview, ModuleUiPackagesPreview,
        ModuleValidationStageDryRunPreview, ModuleYankDryRunPreview,
        RegistryGovernanceActionHttpResponse, RegistryPublishStatusHttpResponse,
        REGISTRY_MUTATION_SCHEMA_VERSION, REGISTRY_YANK_REASON_CODES, REMOTE_RUNNER_TOKEN_ENV,
    };
    use std::{
        collections::HashMap,
        env,
        path::{Path, PathBuf},
        sync::{Mutex, MutexGuard, OnceLock},
    };

    struct WorkspaceRootGuard {
        previous_dir: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl WorkspaceRootGuard {
        fn enter() -> Self {
            static WORKSPACE_CWD_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

            let lock = WORKSPACE_CWD_GUARD
                .get_or_init(|| Mutex::new(()))
                .lock()
                .expect("workspace cwd guard should lock");
            let previous_dir = env::current_dir().expect("current dir should resolve");
            env::set_current_dir(workspace_root()).expect("workspace root should be accessible");

            Self {
                previous_dir,
                _lock: lock,
            }
        }
    }

    impl Drop for WorkspaceRootGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous_dir)
                .expect("current dir should restore after xtask test");
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        previous_value: Option<String>,
        _lock: MutexGuard<'static, ()>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            static ENV_VAR_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

            let lock = ENV_VAR_GUARD
                .get_or_init(|| Mutex::new(()))
                .lock()
                .expect("env var guard should lock");
            let previous_value = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }

            Self {
                key,
                previous_value,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous_value {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn resolve_workspace_inherited_string_uses_workspace_package_value() {
        let package_manifest: toml::Value = toml::from_str(
            r#"
                [package]
                version.workspace = true
            "#,
        )
        .expect("package manifest should parse");
        let workspace_manifest: toml::Value = toml::from_str(
            r#"
                [workspace.package]
                version = "1.2.3"
            "#,
        )
        .expect("workspace manifest should parse");

        let resolved = resolve_workspace_inherited_string(
            package_manifest
                .get("package")
                .and_then(toml::Value::as_table)
                .and_then(|table| table.get("version")),
            &workspace_manifest,
            "version",
        )
        .expect("version should resolve");

        assert_eq!(resolved.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn validate_module_publish_contract_rejects_short_description() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "blog"
                name = "Blog"
                version = "1.2.3"
                description = "Too short"
                ownership = "first_party"
                trust_level = "verified"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_publish_contract("blog", &manifest)
            .expect_err("short descriptions must fail");
        assert!(error.to_string().contains("at least 20 characters"));
    }

    #[test]
    fn validate_module_local_docs_file_rejects_missing_heading() {
        let path =
            env::temp_dir().join(format!("xtask-local-docs-{}-README.md", std::process::id()));
        std::fs::write(&path, "# Heading\n\n## Назначение\n")
            .expect("temporary docs file should be writable");

        let error = validate_module_local_docs_file(
            "blog",
            &path,
            &["## Назначение", "## Зона ответственности"],
        )
        .expect_err("missing heading must fail");

        assert!(error.to_string().contains("must contain heading"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn normalize_module_ui_classification_accepts_supported_values() {
        assert_eq!(
            normalize_module_ui_classification("admin-only").expect("admin-only should normalize"),
            "admin_only"
        );
        assert_eq!(
            normalize_module_ui_classification("dual_surface")
                .expect("dual_surface should normalize"),
            "dual_surface"
        );
    }

    #[test]
    fn validate_module_ui_metadata_contract_rejects_missing_admin_metadata() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"
                recommended_admin_surfaces = ["leptos-admin"]

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_ui_metadata_contract("demo", &manifest)
            .expect_err("missing admin metadata must fail");
        assert!(error
            .to_string()
            .contains("provides.admin_ui.route_segment"));
    }

    #[test]
    fn validate_module_ui_metadata_contract_accepts_dual_surface_manifest() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "dual_surface"
                recommended_admin_surfaces = ["leptos-admin"]

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
                route_segment = "demo"
                nav_label = "Demo"

                [provides.admin_ui.i18n]
                default_locale = "en"
                supported_locales = ["en", "ru"]
                leptos_locales_path = "admin/locales"

                [provides.storefront_ui]
                leptos_crate = "rustok-demo-storefront"
                slot = "home_after_catalog"
                route_segment = "demo"
                page_title = "Demo"

                [provides.storefront_ui.i18n]
                default_locale = "en"
                supported_locales = ["en", "ru"]
                leptos_locales_path = "storefront/locales"
            "#,
        )
        .expect("module manifest should parse");

        validate_module_ui_metadata_contract("demo", &manifest)
            .expect("valid UI metadata should pass");
    }

    #[test]
    fn validate_module_admin_surface_contract_rejects_surfaces_without_admin_ui() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"
                recommended_admin_surfaces = ["leptos-admin"]
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_admin_surface_contract("demo", &manifest)
            .expect_err("admin surfaces without provides.admin_ui must fail");
        assert!(error
            .to_string()
            .contains("does not declare [provides.admin_ui]"));
    }

    #[test]
    fn validate_module_admin_surface_contract_rejects_missing_recommended_surface() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_admin_surface_contract("demo", &manifest)
            .expect_err("admin ui without recommended surface must fail");
        assert!(error
            .to_string()
            .contains("must declare at least one recommended_admin_surface"));
    }

    #[test]
    fn validate_module_admin_surface_contract_rejects_missing_leptos_admin_recommendation() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"
                recommended_admin_surfaces = ["next-admin"]

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_admin_surface_contract("demo", &manifest)
            .expect_err("leptos admin ui without leptos-admin recommendation must fail");
        assert!(error.to_string().contains("must include 'leptos-admin'"));
    }

    #[test]
    fn validate_module_admin_surface_contract_accepts_leptos_admin_manifest() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"
                recommended_admin_surfaces = ["leptos-admin"]
                showcase_admin_surfaces = ["next-admin"]

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("module manifest should parse");

        validate_module_admin_surface_contract("demo", &manifest)
            .expect("valid admin surface manifest should pass");
    }

    #[test]
    fn validate_module_ui_classification_contract_rejects_surface_drift() {
        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "cart"
                name = "Cart"
                version = "1.2.3"
                description = "Default cart submodule in the ecommerce family"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"

                [provides.storefront_ui]
                leptos_crate = "rustok-cart-storefront"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_ui_classification_contract("cart", &manifest)
            .expect_err("ui classification drift must fail");
        assert!(error
            .to_string()
            .contains("manifest UI surfaces resolve to 'storefront_only'"));
    }

    #[test]
    fn extract_runtime_module_dependencies_reads_dependency_array() {
        let base = env::temp_dir().join(format!("xtask-runtime-deps-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            r#"
                impl RusToKModule for DemoModule {
                    fn dependencies(&self) -> &[&'static str] {
                        &["content", "taxonomy"]
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let dependencies = extract_runtime_module_dependencies(&base)
            .expect("dependencies should parse")
            .expect("runtime implementation should be detected");
        assert!(dependencies.contains("content"));
        assert!(dependencies.contains("taxonomy"));
        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_entry_type_contract_rejects_missing_entry_type_for_runtime_module() {
        let base = env::temp_dir().join(format!("xtask-entry-type-missing-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            "pub struct DemoModule;\nimpl RusToKModule for DemoModule {}\n",
        )
        .expect("temporary lib.rs should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_entry_type_contract("demo", &manifest, &base)
            .expect_err("missing entry_type must fail");
        assert!(error
            .to_string()
            .contains("must declare [crate].entry_type"));
        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_entry_type_contract_accepts_non_runtime_module_without_entry_type() {
        let base = env::temp_dir().join(format!(
            "xtask-entry-type-capability-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn helper() {}\n")
            .expect("temporary lib.rs should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"
            "#,
        )
        .expect("module manifest should parse");

        validate_module_entry_type_contract("demo", &manifest, &base)
            .expect("capability-style module without RusToKModule impl should pass");
        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_runtime_metadata_contract_rejects_description_drift() {
        let base = env::temp_dir().join(format!("xtask-runtime-meta-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            r#"
                pub struct DemoModule;
                impl RusToKModule for DemoModule {
                    fn slug(&self) -> &'static str { "demo" }
                    fn name(&self) -> &'static str { "Demo" }
                    fn description(&self) -> &'static str { "Runtime description" }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "Manifest description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "DemoModule"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_runtime_metadata_contract("demo", &manifest, &base)
            .expect_err("description drift must fail");
        assert!(error.to_string().contains("description mismatch"));
        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_semantics_contract_rejects_path_module_with_non_first_party_ownership() {
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };
        let metadata = ModulePackageMetadata {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            version: "0.1.0".to_string(),
            description: "A sufficiently long demo module description".to_string(),
            ownership: "third_party".to_string(),
            trust_level: "verified".to_string(),
            ui_classification: "no_ui".to_string(),
            recommended_admin_surfaces: Vec::new(),
            showcase_admin_surfaces: Vec::new(),
        };

        let error = validate_module_semantics_contract("demo", &spec, &metadata)
            .expect_err("path module with non-first-party ownership must fail");
        assert!(error
            .to_string()
            .contains("must declare ownership='first_party'"));
    }

    #[test]
    fn validate_module_semantics_contract_rejects_required_module_without_core_trust_level() {
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: true,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };
        let metadata = ModulePackageMetadata {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            version: "0.1.0".to_string(),
            description: "A sufficiently long demo module description".to_string(),
            ownership: "first_party".to_string(),
            trust_level: "verified".to_string(),
            ui_classification: "no_ui".to_string(),
            recommended_admin_surfaces: Vec::new(),
            showcase_admin_surfaces: Vec::new(),
        };

        let error = validate_module_semantics_contract("demo", &spec, &metadata)
            .expect_err("required module without core trust level must fail");
        assert!(error
            .to_string()
            .contains("must declare trust_level='core'"));
    }

    #[test]
    fn validate_module_semantics_contract_rejects_optional_module_with_core_trust_level() {
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };
        let metadata = ModulePackageMetadata {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            version: "0.1.0".to_string(),
            description: "A sufficiently long demo module description".to_string(),
            ownership: "first_party".to_string(),
            trust_level: "core".to_string(),
            ui_classification: "no_ui".to_string(),
            recommended_admin_surfaces: Vec::new(),
            showcase_admin_surfaces: Vec::new(),
        };

        let error = validate_module_semantics_contract("demo", &spec, &metadata)
            .expect_err("optional module with core trust level must fail");
        assert!(error
            .to_string()
            .contains("must not declare trust_level='core'"));
    }

    #[test]
    fn validate_module_kind_contract_rejects_required_module_without_core_kind() {
        let base = env::temp_dir().join(format!("xtask-kind-required-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            r#"
                pub struct DemoModule;
                impl RusToKModule for DemoModule {
                    fn kind(&self) -> ModuleKind { ModuleKind::Optional }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: true,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_kind_contract("demo", &spec, &base)
            .expect_err("required module without ModuleKind::Core must fail");
        assert!(error
            .to_string()
            .contains("must declare fn kind(&self) -> ModuleKind { ModuleKind::Core }"));

        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_kind_contract_rejects_optional_module_declaring_core_kind() {
        let base = env::temp_dir().join(format!("xtask-kind-optional-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            r#"
                pub struct DemoModule;
                impl RusToKModule for DemoModule {
                    fn kind(&self) -> ModuleKind { ModuleKind::Core }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_kind_contract("demo", &spec, &base)
            .expect_err("optional module declaring ModuleKind::Core must fail");
        assert!(error
            .to_string()
            .contains("must not declare ModuleKind::Core"));

        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_transport_surface_contract_rejects_missing_declared_symbol() {
        let base = env::temp_dir().join(format!("xtask-surface-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub struct ExistingQuery;\n")
            .expect("temporary lib.rs should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "DemoModule"

                [provides.graphql]
                query = "graphql::MissingQuery"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_transport_surface_contract("demo", &manifest, &base)
            .expect_err("missing declared symbol must fail");
        assert!(error.to_string().contains("MissingQuery"));
        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_entry_type_contract_accepts_runtime_struct_with_fields() {
        let base = env::temp_dir().join(format!("xtask-entry-type-fields-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(
            &lib_path,
            r#"
                pub struct DemoModule {
                    service: usize,
                }

                impl RusToKModule for DemoModule {}
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "DemoModule"
            "#,
        )
        .expect("module manifest should parse");

        validate_module_entry_type_contract("demo", &manifest, &base)
            .expect("runtime struct with fields should satisfy entry_type contract");

        let _ = std::fs::remove_file(&lib_path);
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_http_surface_contract_rejects_missing_server_routes_export() {
        let base = env::temp_dir().join(format!("xtask-server-http-routes-{}", std::process::id()));
        let controller_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers")
            .join("demo");
        std::fs::create_dir_all(&controller_dir)
            .expect("temporary server controller dir should exist");
        std::fs::write(controller_dir.join("mod.rs"), "pub mod api;\n")
            .expect("temporary controller mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "DemoModule"

                [provides.http]
                routes = "controllers::routes"
            "#,
        )
        .expect("module manifest should parse");

        let error = validate_module_server_http_surface_contract(
            &base.join("modules.toml"),
            "demo",
            &manifest,
        )
        .expect_err("missing server routes export must fail");
        assert!(error.to_string().contains("does not export pub routes()"));

        let _ = std::fs::remove_file(controller_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&controller_dir);
        let _ = std::fs::remove_dir(
            base.join("apps")
                .join("server")
                .join("src")
                .join("controllers"),
        );
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_http_surface_contract_accepts_reexported_routes_and_webhooks() {
        let base =
            env::temp_dir().join(format!("xtask-server-http-reexport-{}", std::process::id()));
        let controllers_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers");
        std::fs::create_dir_all(&controllers_dir)
            .expect("temporary server controllers dir should exist");
        std::fs::write(
            controllers_dir.join("demo.rs"),
            "pub use rustok_demo::controllers::*;\n",
        )
        .expect("temporary controller re-export should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "DemoModule"

                [provides.http]
                routes = "controllers::routes"
                webhook_routes = "controllers::webhook_routes"
            "#,
        )
        .expect("module manifest should parse");

        validate_module_server_http_surface_contract(&base.join("modules.toml"), "demo", &manifest)
            .expect("re-exported controller shim should satisfy host HTTP contract");

        let _ = std::fs::remove_file(controllers_dir.join("demo.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&controllers_dir);
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_registry_contract_rejects_missing_server_feature() {
        let base = env::temp_dir().join(format!(
            "xtask-server-registry-missing-{}",
            std::process::id()
        ));
        let module_root = base.join("crates").join("demo-module");
        let src_dir = module_root.join("src");
        let server_dir = base.join("apps").join("server");
        let server_modules_dir = server_dir.join("src").join("modules");
        std::fs::create_dir_all(&src_dir).expect("temporary module src dir should exist");
        std::fs::create_dir_all(&server_modules_dir)
            .expect("temporary server modules dir should exist");

        std::fs::write(
            src_dir.join("lib.rs"),
            "pub struct DemoModule;\nimpl RusToKModule for DemoModule {}\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = []
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        std::fs::write(
            server_modules_dir.join("mod.rs"),
            "pub fn build_registry() {}\n",
        )
        .expect("temporary server modules mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"

                [crate]
                entry_type = "DemoModule"
            "#,
        )
        .expect("module manifest should parse");
        let spec = ModuleSpec {
            crate_name: "demo-module".to_string(),
            source: "path".to_string(),
            path: Some("crates/demo-module".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_server_registry_contract(
            &base.join("modules.toml"),
            "demo",
            &spec,
            &manifest,
            &module_root,
        )
        .expect_err("missing server feature must fail");
        assert!(error.to_string().contains("must expose feature 'mod-demo'"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(server_modules_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&module_root);
        let _ = std::fs::remove_dir(&server_modules_dir);
        let _ = std::fs::remove_dir(server_dir.join("src"));
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_registry_contract_accepts_optional_runtime_module() {
        let base = env::temp_dir().join(format!("xtask-server-registry-ok-{}", std::process::id()));
        let module_root = base.join("crates").join("alloy");
        let src_dir = module_root.join("src");
        let server_dir = base.join("apps").join("server");
        let server_modules_dir = server_dir.join("src").join("modules");
        std::fs::create_dir_all(&src_dir).expect("temporary module src dir should exist");
        std::fs::create_dir_all(&server_modules_dir)
            .expect("temporary server modules dir should exist");

        std::fs::write(
            src_dir.join("lib.rs"),
            "pub struct AlloyModule;\nimpl RusToKModule for AlloyModule {}\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = []
                mod-alloy = ["dep:alloy"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        std::fs::write(
            server_modules_dir.join("mod.rs"),
            "pub fn build_registry() {}\n",
        )
        .expect("temporary server modules mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "alloy"
                name = "Alloy"
                version = "0.1.0"
                description = "A sufficiently long alloy module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"

                [crate]
                entry_type = "AlloyModule"
            "#,
        )
        .expect("module manifest should parse");
        let spec = ModuleSpec {
            crate_name: "alloy".to_string(),
            source: "path".to_string(),
            path: Some("crates/alloy".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        validate_module_server_registry_contract(
            &base.join("modules.toml"),
            "alloy",
            &spec,
            &manifest,
            &module_root,
        )
        .expect("optional runtime module should map into server registry");

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(server_modules_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&module_root);
        let _ = std::fs::remove_dir(&server_modules_dir);
        let _ = std::fs::remove_dir(server_dir.join("src"));
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_registry_contract_rejects_required_module_missing_direct_registration(
    ) {
        let base = env::temp_dir().join(format!(
            "xtask-server-registry-required-{}",
            std::process::id()
        ));
        let module_root = base.join("crates").join("rustok-auth");
        let src_dir = module_root.join("src");
        let server_dir = base.join("apps").join("server");
        let server_modules_dir = server_dir.join("src").join("modules");
        std::fs::create_dir_all(&src_dir).expect("temporary module src dir should exist");
        std::fs::create_dir_all(&server_modules_dir)
            .expect("temporary server modules dir should exist");

        std::fs::write(
            src_dir.join("lib.rs"),
            "pub struct AuthModule;\nimpl RusToKModule for AuthModule { fn kind(&self) -> ModuleKind { ModuleKind::Core } }\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = []
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        std::fs::write(
            server_modules_dir.join("mod.rs"),
            "pub fn build_registry() {}\n",
        )
        .expect("temporary server modules mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "auth"
                name = "Auth"
                version = "0.1.0"
                description = "A sufficiently long auth module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"

                [crate]
                entry_type = "AuthModule"
            "#,
        )
        .expect("module manifest should parse");
        let spec = ModuleSpec {
            crate_name: "rustok-auth".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-auth".to_string()),
            required: true,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_server_registry_contract(
            &base.join("modules.toml"),
            "auth",
            &spec,
            &manifest,
            &module_root,
        )
        .expect_err("required module missing direct registration must fail");
        assert!(error
            .to_string()
            .contains("must be registered directly in apps/server/src/modules/mod.rs"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(server_modules_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&module_root);
        let _ = std::fs::remove_dir(&server_modules_dir);
        let _ = std::fs::remove_dir(server_dir.join("src"));
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_registry_contract_rejects_optional_module_direct_registration() {
        let base = env::temp_dir().join(format!(
            "xtask-server-registry-direct-optional-{}",
            std::process::id()
        ));
        let module_root = base.join("crates").join("alloy");
        let src_dir = module_root.join("src");
        let server_dir = base.join("apps").join("server");
        let server_modules_dir = server_dir.join("src").join("modules");
        std::fs::create_dir_all(&src_dir).expect("temporary module src dir should exist");
        std::fs::create_dir_all(&server_modules_dir)
            .expect("temporary server modules dir should exist");

        std::fs::write(
            src_dir.join("lib.rs"),
            "pub struct AlloyModule;\nimpl RusToKModule for AlloyModule {}\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = []
                mod-alloy = ["dep:alloy"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        std::fs::write(
            server_modules_dir.join("mod.rs"),
            "pub fn build_registry() { let _ = ModuleRegistry::new().register(AlloyModule); }\n",
        )
        .expect("temporary server modules mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "alloy"
                name = "Alloy"
                version = "0.1.0"
                description = "A sufficiently long alloy module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"

                [crate]
                entry_type = "AlloyModule"
            "#,
        )
        .expect("module manifest should parse");
        let spec = ModuleSpec {
            crate_name: "alloy".to_string(),
            source: "path".to_string(),
            path: Some("crates/alloy".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_server_registry_contract(
            &base.join("modules.toml"),
            "alloy",
            &spec,
            &manifest,
            &module_root,
        )
        .expect_err("optional module direct registration must fail");
        assert!(error
            .to_string()
            .contains("must not be registered directly"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(server_modules_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&module_root);
        let _ = std::fs::remove_dir(&server_modules_dir);
        let _ = std::fs::remove_dir(server_dir.join("src"));
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_server_registry_contract_rejects_server_feature_dependency_drift() {
        let base = env::temp_dir().join(format!(
            "xtask-server-registry-drift-{}",
            std::process::id()
        ));
        let module_root = base.join("crates").join("blog");
        let src_dir = module_root.join("src");
        let server_dir = base.join("apps").join("server");
        let server_modules_dir = server_dir.join("src").join("modules");
        std::fs::create_dir_all(&src_dir).expect("temporary module src dir should exist");
        std::fs::create_dir_all(&server_modules_dir)
            .expect("temporary server modules dir should exist");

        std::fs::write(
            src_dir.join("lib.rs"),
            "pub struct BlogModule;\nimpl RusToKModule for BlogModule { fn dependencies(&self) -> &[&'static str] { &[\"content\", \"taxonomy\"] } }\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = []
                mod-blog = ["dep:rustok-blog", "mod-taxonomy"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        std::fs::write(
            server_modules_dir.join("mod.rs"),
            "pub fn build_registry() {}\n",
        )
        .expect("temporary server modules mod.rs should be writable");
        std::fs::write(
            base.join("modules.toml"),
            "app = \"rustok-server\"\nschema = 2\n",
        )
        .expect("temporary modules.toml should be writable");

        let manifest: ModulePackageManifest = toml::from_str(
            r#"
                [module]
                slug = "blog"
                name = "Blog"
                version = "0.1.0"
                description = "A sufficiently long blog module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "capability_only"

                [crate]
                entry_type = "BlogModule"

                [dependencies]
                content = {}
                taxonomy = {}
            "#,
        )
        .expect("module manifest should parse");
        let spec = ModuleSpec {
            crate_name: "rustok-blog".to_string(),
            source: "path".to_string(),
            path: Some("crates/blog".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: Some(vec!["content".to_string(), "taxonomy".to_string()]),
            features: None,
        };

        let error = validate_module_server_registry_contract(
            &base.join("modules.toml"),
            "blog",
            &spec,
            &manifest,
            &module_root,
        )
        .expect_err("missing mod-content must fail");
        assert!(error.to_string().contains("server feature graph drift"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(server_modules_dir.join("mod.rs"));
        let _ = std::fs::remove_file(base.join("modules.toml"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&module_root);
        let _ = std::fs::remove_dir(&server_modules_dir);
        let _ = std::fs::remove_dir(server_dir.join("src"));
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_default_enabled_server_contract_rejects_missing_server_default_feature() {
        let base = env::temp_dir().join(format!(
            "xtask-default-enabled-missing-{}",
            std::process::id()
        ));
        let server_dir = base.join("apps").join("server");
        std::fs::create_dir_all(&server_dir).expect("temporary server dir should exist");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = ["mod-content"]
                mod-content = ["dep:rustok-content"]
                mod-pages = ["dep:rustok-pages", "mod-content"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let manifest = Manifest {
            schema: 2,
            app: "rustok-server".to_string(),
            build: None,
            modules: HashMap::from([
                (
                    "content".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-content".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-content".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: None,
                        features: None,
                    },
                ),
                (
                    "pages".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-pages".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-pages".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: Some(vec!["content".to_string()]),
                        features: None,
                    },
                ),
            ]),
            settings: Some(super::Settings {
                default_enabled: Some(vec!["content".to_string(), "pages".to_string()]),
            }),
        };

        let error = validate_default_enabled_server_contract(&manifest_path, &manifest)
            .expect_err("missing mod-pages in server defaults must fail");
        assert!(error
            .to_string()
            .contains("default_enabled modules must be present"));

        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_default_enabled_server_contract_rejects_required_module_in_default_enabled() {
        let base = env::temp_dir().join(format!(
            "xtask-default-enabled-required-{}",
            std::process::id()
        ));
        let server_dir = base.join("apps").join("server");
        std::fs::create_dir_all(&server_dir).expect("temporary server dir should exist");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = ["mod-content"]
                mod-content = ["dep:rustok-content"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let manifest = Manifest {
            schema: 2,
            app: "rustok-server".to_string(),
            build: None,
            modules: HashMap::from([(
                "channel".to_string(),
                ModuleSpec {
                    crate_name: "rustok-channel".to_string(),
                    source: "path".to_string(),
                    path: Some("crates/rustok-channel".to_string()),
                    required: true,
                    version: None,
                    git: None,
                    rev: None,
                    depends_on: None,
                    features: None,
                },
            )]),
            settings: Some(super::Settings {
                default_enabled: Some(vec!["channel".to_string()]),
            }),
        };

        let error = validate_default_enabled_server_contract(&manifest_path, &manifest)
            .expect_err("required modules must not appear in default_enabled");
        assert!(error
            .to_string()
            .contains("default_enabled must list only optional modules"));

        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_default_enabled_server_contract_accepts_present_server_default_features() {
        let base = env::temp_dir().join(format!("xtask-default-enabled-ok-{}", std::process::id()));
        let server_dir = base.join("apps").join("server");
        std::fs::create_dir_all(&server_dir).expect("temporary server dir should exist");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = ["mod-content", "mod-pages"]
                mod-content = ["dep:rustok-content"]
                mod-pages = ["dep:rustok-pages", "mod-content"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let manifest = Manifest {
            schema: 2,
            app: "rustok-server".to_string(),
            build: None,
            modules: HashMap::from([
                (
                    "content".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-content".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-content".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: None,
                        features: None,
                    },
                ),
                (
                    "pages".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-pages".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-pages".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: Some(vec!["content".to_string()]),
                        features: None,
                    },
                ),
            ]),
            settings: Some(super::Settings {
                default_enabled: Some(vec!["content".to_string(), "pages".to_string()]),
            }),
        };

        validate_default_enabled_server_contract(&manifest_path, &manifest)
            .expect("default_enabled slugs present in server defaults should pass");

        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_default_enabled_server_contract_rejects_missing_optional_dependency_closure() {
        let base = env::temp_dir().join(format!(
            "xtask-default-enabled-closure-{}",
            std::process::id()
        ));
        let server_dir = base.join("apps").join("server");
        std::fs::create_dir_all(&server_dir).expect("temporary server dir should exist");
        std::fs::write(
            server_dir.join("Cargo.toml"),
            r#"
                [features]
                default = ["mod-blog"]
                mod-content = ["dep:rustok-content"]
                mod-blog = ["dep:rustok-blog", "mod-content"]
            "#,
        )
        .expect("temporary server Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let manifest = Manifest {
            schema: 2,
            app: "rustok-server".to_string(),
            build: None,
            modules: HashMap::from([
                (
                    "content".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-content".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-content".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: None,
                        features: None,
                    },
                ),
                (
                    "blog".to_string(),
                    ModuleSpec {
                        crate_name: "rustok-blog".to_string(),
                        source: "path".to_string(),
                        path: Some("crates/rustok-blog".to_string()),
                        required: false,
                        version: None,
                        git: None,
                        rev: None,
                        depends_on: Some(vec!["content".to_string()]),
                        features: None,
                    },
                ),
            ]),
            settings: Some(super::Settings {
                default_enabled: Some(vec!["blog".to_string()]),
            }),
        };

        let error = validate_default_enabled_server_contract(&manifest_path, &manifest)
            .expect_err("missing optional dependency closure must fail");
        assert!(error
            .to_string()
            .contains("default_enabled must include optional dependency closure"));

        let _ = std::fs::remove_file(server_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&server_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_host_ui_inventory_contract_rejects_orphan_module_ui_dependency() {
        let base = env::temp_dir().join(format!("xtask-host-inventory-{}", std::process::id()));
        let admin_dir = base.join("apps").join("admin");
        let storefront_dir = base.join("apps").join("storefront");
        let demo_admin_dir = base.join("crates").join("rustok-demo").join("admin");
        let demo_root = base.join("crates").join("rustok-demo");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&storefront_dir).expect("temporary storefront dir should exist");
        std::fs::create_dir_all(&demo_admin_dir).expect("temporary demo admin dir should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = ["rustok-demo-admin/hydrate"]
                ssr = ["rustok-demo-admin/ssr"]

                [dependencies]
                rustok-demo-admin = { path = "../../crates/rustok-demo/admin", default-features = false }
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(storefront_dir.join("Cargo.toml"), "[dependencies]\n")
            .expect("temporary storefront Cargo.toml should be writable");
        std::fs::write(
            demo_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-admin"
                version = "0.1.0"
            "#,
        )
        .expect("temporary demo admin Cargo.toml should be writable");
        std::fs::write(
            demo_root.join("rustok-module.toml"),
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "no_ui"
            "#,
        )
        .expect("temporary rustok-module.toml should be writable");

        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\" }\n",
        )
        .expect("temporary modules.toml should be writable");
        let manifest = load_manifest_from(&manifest_path).expect("manifest should parse");

        let error = validate_host_ui_inventory_contract(&manifest_path, &manifest)
            .expect_err("orphan module ui dependency must fail");
        assert!(error
            .to_string()
            .contains("but no module manifest declares it as admin UI"));

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(storefront_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_root.join("rustok-module.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&demo_admin_dir);
        let _ = std::fs::remove_dir(&demo_root);
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(&storefront_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_host_ui_inventory_contract_ignores_support_ui_dependency_without_module_manifest() {
        let base = env::temp_dir().join(format!(
            "xtask-host-inventory-support-{}",
            std::process::id()
        ));
        let admin_dir = base.join("apps").join("admin");
        let storefront_dir = base.join("apps").join("storefront");
        let support_admin_dir = base.join("crates").join("rustok-ai").join("admin");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&storefront_dir).expect("temporary storefront dir should exist");
        std::fs::create_dir_all(&support_admin_dir)
            .expect("temporary support admin dir should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = ["rustok-ai-admin/hydrate"]
                ssr = ["rustok-ai-admin/ssr"]

                [dependencies]
                rustok-ai-admin = { path = "../../crates/rustok-ai/admin", default-features = false }
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(storefront_dir.join("Cargo.toml"), "[dependencies]\n")
            .expect("temporary storefront Cargo.toml should be writable");
        std::fs::write(
            support_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-ai-admin"
                version = "0.1.0"
            "#,
        )
        .expect("temporary support admin Cargo.toml should be writable");

        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\n",
        )
        .expect("temporary modules.toml should be writable");
        let manifest = load_manifest_from(&manifest_path).expect("manifest should parse");

        validate_host_ui_inventory_contract(&manifest_path, &manifest)
            .expect("support ui dependency without rustok-module.toml should be ignored");

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(storefront_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(support_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&support_admin_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-ai"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(&storefront_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_host_ui_inventory_contract_rejects_orphan_feature_entry_for_declared_module_ui() {
        let base = env::temp_dir().join(format!(
            "xtask-host-inventory-feature-{}",
            std::process::id()
        ));
        let admin_dir = base.join("apps").join("admin");
        let storefront_dir = base.join("apps").join("storefront");
        let demo_root = base.join("crates").join("rustok-demo");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&storefront_dir).expect("temporary storefront dir should exist");
        std::fs::create_dir_all(&demo_root).expect("temporary demo root should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = []
                ssr = ["rustok-demo-admin/ssr"]

                [dependencies]
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(storefront_dir.join("Cargo.toml"), "[dependencies]\n")
            .expect("temporary storefront Cargo.toml should be writable");
        std::fs::write(
            demo_root.join("rustok-module.toml"),
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("temporary rustok-module.toml should be writable");

        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\" }\n",
        )
        .expect("temporary modules.toml should be writable");
        let manifest = load_manifest_from(&manifest_path).expect("manifest should parse");

        let error = validate_host_ui_inventory_contract(&manifest_path, &manifest)
            .expect_err("orphan feature entry must fail");
        assert!(error
            .to_string()
            .contains("feature 'ssr' references 'rustok-demo-admin/ssr'"));

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(storefront_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_root.join("rustok-module.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&demo_root);
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(&storefront_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_server_event_runtime_contract_rejects_legacy_dispatcher_references() {
        let base = env::temp_dir().join(format!(
            "xtask-server-event-runtime-legacy-{}",
            std::process::id()
        ));
        let services_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("services");
        std::fs::create_dir_all(&services_dir).expect("temporary services dir should exist");
        std::fs::write(
            services_dir.join("app_runtime.rs"),
            "fn bootstrap() { spawn_index_dispatcher(); let _ = WorkflowCronScheduler::new(todo!()); }",
        )
        .expect("temporary app_runtime.rs should be writable");
        std::fs::write(
            services_dir.join("mod.rs"),
            "pub mod module_event_dispatcher;\n",
        )
        .expect("temporary services mod.rs should be writable");
        std::fs::write(
            services_dir.join("module_event_dispatcher.rs"),
            "pub fn spawn_module_event_dispatcher() {}\n",
        )
        .expect("temporary module_event_dispatcher.rs should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let error = validate_server_event_runtime_contract(&manifest_path)
            .expect_err("legacy dispatcher references must fail");
        assert!(error
            .to_string()
            .contains("legacy index/search dispatchers"));

        let _ = std::fs::remove_file(services_dir.join("app_runtime.rs"));
        let _ = std::fs::remove_file(services_dir.join("mod.rs"));
        let _ = std::fs::remove_file(services_dir.join("module_event_dispatcher.rs"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&services_dir);
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_server_event_runtime_contract_accepts_module_owned_dispatcher_path() {
        let base = env::temp_dir().join(format!(
            "xtask-server-event-runtime-ok-{}",
            std::process::id()
        ));
        let services_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("services");
        std::fs::create_dir_all(&services_dir).expect("temporary services dir should exist");
        std::fs::write(
            services_dir.join("app_runtime.rs"),
            "use crate::services::module_event_dispatcher::spawn_module_event_dispatcher;\nfn bootstrap() { spawn_module_event_dispatcher(); }\nfn workflow() { let _ = WorkflowCronScheduler::new(todo!()); }",
        )
        .expect("temporary app_runtime.rs should be writable");
        std::fs::write(
            services_dir.join("mod.rs"),
            "pub mod module_event_dispatcher;\n",
        )
        .expect("temporary services mod.rs should be writable");
        std::fs::write(
            services_dir.join("module_event_dispatcher.rs"),
            "pub fn spawn_module_event_dispatcher() {}\n",
        )
        .expect("temporary module_event_dispatcher.rs should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        validate_server_event_runtime_contract(&manifest_path)
            .expect("module-owned dispatcher path should validate");

        let _ = std::fs::remove_file(services_dir.join("app_runtime.rs"));
        let _ = std::fs::remove_file(services_dir.join("mod.rs"));
        let _ = std::fs::remove_file(services_dir.join("module_event_dispatcher.rs"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&services_dir);
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_host_ui_contract_rejects_missing_admin_feature_wiring() {
        let base = env::temp_dir().join(format!("xtask-host-ui-missing-{}", std::process::id()));
        let admin_dir = base.join("apps").join("admin");
        let ui_crate_dir = base.join("crates").join("rustok-demo").join("admin");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&ui_crate_dir).expect("temporary ui crate dir should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = []
                ssr = []

                [dependencies]
                rustok-demo-admin = { path = "../../crates/rustok-demo/admin", default-features = false }
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(
            ui_crate_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-admin"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary ui crate Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\" }\n",
        )
            .expect("temporary modules.toml should be writable");
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_host_ui_contract(
            &manifest_path,
            "demo",
            &spec,
            Some("rustok-demo-admin"),
            None,
        )
        .expect_err("missing host ui feature wiring must fail");
        assert!(error
            .to_string()
            .contains("feature 'hydrate' is missing 'rustok-demo-admin/hydrate'"));

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(ui_crate_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&ui_crate_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-demo"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_host_ui_contract_accepts_storefront_ssr_wiring() {
        let base = env::temp_dir().join(format!("xtask-host-ui-ok-{}", std::process::id()));
        let storefront_dir = base.join("apps").join("storefront");
        let ui_crate_dir = base.join("crates").join("rustok-demo").join("storefront");
        std::fs::create_dir_all(&storefront_dir).expect("temporary storefront dir should exist");
        std::fs::create_dir_all(&ui_crate_dir).expect("temporary ui crate dir should exist");

        std::fs::write(
            storefront_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = []
                ssr = ["rustok-demo-storefront/ssr"]

                [dependencies]
                rustok-demo-storefront = { path = "../../crates/rustok-demo/storefront", default-features = false }
            "#,
        )
        .expect("temporary storefront Cargo.toml should be writable");
        std::fs::write(
            ui_crate_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-storefront"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary ui crate Cargo.toml should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\" }\n",
        )
            .expect("temporary modules.toml should be writable");
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        validate_module_host_ui_contract(
            &manifest_path,
            "demo",
            &spec,
            None,
            Some("rustok-demo-storefront"),
        )
        .expect("storefront ssr wiring should validate");

        let _ = std::fs::remove_file(storefront_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(ui_crate_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&ui_crate_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-demo"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&storefront_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_host_ui_contract_rejects_missing_dependency_admin_ui_wiring() {
        let base = env::temp_dir().join(format!("xtask-host-ui-dependency-{}", std::process::id()));
        let admin_dir = base.join("apps").join("admin");
        let demo_admin_dir = base.join("crates").join("rustok-demo").join("admin");
        let comments_admin_dir = base.join("crates").join("rustok-comments").join("admin");
        let demo_root = base.join("crates").join("rustok-demo");
        let comments_root = base.join("crates").join("rustok-comments");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&demo_admin_dir).expect("temporary demo admin dir should exist");
        std::fs::create_dir_all(&comments_admin_dir)
            .expect("temporary comments admin dir should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = ["rustok-demo-admin/hydrate"]
                ssr = ["rustok-demo-admin/ssr"]

                [dependencies]
                rustok-demo-admin = { path = "../../crates/rustok-demo/admin", default-features = false }
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(
            demo_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-admin"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary demo admin Cargo.toml should be writable");
        std::fs::write(
            comments_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-comments-admin"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary comments admin Cargo.toml should be writable");
        std::fs::write(
            demo_root.join("rustok-module.toml"),
            r#"
                [module]
                slug = "demo"
                name = "Demo"
                version = "0.1.0"
                description = "A sufficiently long demo module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"

                [provides.admin_ui]
                leptos_crate = "rustok-demo-admin"
            "#,
        )
        .expect("temporary demo rustok-module.toml should be writable");
        std::fs::write(
            comments_root.join("rustok-module.toml"),
            r#"
                [module]
                slug = "comments"
                name = "Comments"
                version = "0.1.0"
                description = "A sufficiently long comments module description"
                ownership = "first_party"
                trust_level = "verified"
                ui_classification = "admin_only"

                [provides.admin_ui]
                leptos_crate = "rustok-comments-admin"
            "#,
        )
        .expect("temporary comments rustok-module.toml should be writable");

        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\", depends_on = [\"comments\"] }\ncomments = { crate = \"rustok-comments\", source = \"path\", path = \"crates/rustok-comments\" }\n",
        )
        .expect("temporary modules.toml should be writable");
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: Some(vec!["comments".to_string()]),
            features: None,
        };

        let error = validate_module_host_ui_contract(
            &manifest_path,
            "demo",
            &spec,
            Some("rustok-demo-admin"),
            None,
        )
        .expect_err("missing dependency admin UI wiring must fail");
        assert!(error
            .to_string()
            .contains("missing UI dependency from module 'comments'"));

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(comments_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_root.join("rustok-module.toml"));
        let _ = std::fs::remove_file(comments_root.join("rustok-module.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&demo_admin_dir);
        let _ = std::fs::remove_dir(&comments_admin_dir);
        let _ = std::fs::remove_dir(&demo_root);
        let _ = std::fs::remove_dir(&comments_root);
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_host_ui_contract_rejects_non_canonical_ui_dependency_path() {
        let base = env::temp_dir().join(format!("xtask-host-ui-canonical-{}", std::process::id()));
        let admin_dir = base.join("apps").join("admin");
        let demo_admin_dir = base.join("crates").join("rustok-demo").join("admin");
        let wrong_admin_dir = base.join("crates").join("wrong-demo").join("admin");
        std::fs::create_dir_all(&admin_dir).expect("temporary admin dir should exist");
        std::fs::create_dir_all(&demo_admin_dir).expect("temporary demo admin dir should exist");
        std::fs::create_dir_all(&wrong_admin_dir).expect("temporary wrong admin dir should exist");

        std::fs::write(
            admin_dir.join("Cargo.toml"),
            r#"
                [features]
                hydrate = ["rustok-demo-admin/hydrate"]
                ssr = ["rustok-demo-admin/ssr"]

                [dependencies]
                rustok-demo-admin = { path = "../../crates/wrong-demo/admin", default-features = false }
            "#,
        )
        .expect("temporary admin Cargo.toml should be writable");
        std::fs::write(
            demo_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-admin"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary demo admin Cargo.toml should be writable");
        std::fs::write(
            wrong_admin_dir.join("Cargo.toml"),
            r#"
                [package]
                name = "rustok-demo-admin"
                version = "0.1.0"

                [features]
                hydrate = []
                ssr = []
            "#,
        )
        .expect("temporary wrong admin Cargo.toml should be writable");

        let manifest_path = base.join("modules.toml");
        std::fs::write(
            &manifest_path,
            "app = \"rustok-server\"\nschema = 2\n[modules]\ndemo = { crate = \"rustok-demo\", source = \"path\", path = \"crates/rustok-demo\" }\n",
        )
        .expect("temporary modules.toml should be writable");
        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_host_ui_contract(
            &manifest_path,
            "demo",
            &spec,
            Some("rustok-demo-admin"),
            None,
        )
        .expect_err("non-canonical ui dependency path must fail");
        assert!(error.to_string().contains("instead of canonical"));

        let _ = std::fs::remove_file(admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(demo_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(wrong_admin_dir.join("Cargo.toml"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&demo_admin_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-demo"));
        let _ = std::fs::remove_dir(&wrong_admin_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("wrong-demo"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&admin_dir);
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_permission_contract_rejects_unknown_permission_constant() {
        let base =
            env::temp_dir().join(format!("xtask-permissions-unknown-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct DemoModule;
                impl RusToKModule for DemoModule {
                    fn permissions(&self) -> Vec<Permission> {
                        vec![Permission::DOES_NOT_EXIST]
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let error = validate_module_permission_contract("demo", &base)
            .expect_err("unknown permission constant must fail");
        assert!(error
            .to_string()
            .contains("unknown Permission::DOES_NOT_EXIST"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_permission_contract_rejects_duplicate_permission_semantics() {
        let base = env::temp_dir().join(format!(
            "xtask-permissions-duplicate-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct DemoModule;
                impl RusToKModule for DemoModule {
                    fn permissions(&self) -> Vec<Permission> {
                        vec![
                            Permission::USERS_READ,
                            Permission::new(Resource::Users, Action::Read),
                        ]
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let error = validate_module_permission_contract("demo", &base)
            .expect_err("duplicate permission semantics must fail");
        assert!(error
            .to_string()
            .contains("duplicate permission 'users:read'"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_permission_contract_rejects_missing_minimum_runtime_permission() {
        let base =
            env::temp_dir().join(format!("xtask-permissions-minimum-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct BlogModule;
                impl RusToKModule for BlogModule {
                    fn permissions(&self) -> Vec<Permission> {
                        vec![Permission::BLOG_POSTS_READ]
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let error = validate_module_permission_contract("blog", &base)
            .expect_err("missing minimum runtime permission must fail");
        assert!(error
            .to_string()
            .contains("must declare minimum runtime permission 'blog_posts:manage'"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_event_listener_contract_rejects_missing_registration_hook() {
        let base = env::temp_dir().join(format!(
            "xtask-event-listener-missing-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct SearchModule;

                impl SearchModule {
                    pub fn new() -> Self {
                        Self
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        let error = validate_module_event_listener_contract("search", &base)
            .expect_err("missing register_event_listeners hook must fail");
        assert!(error.to_string().contains("register_event_listeners"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_event_listener_contract_accepts_index_runtime_registration() {
        let base = env::temp_dir().join(format!("xtask-event-listener-ok-{}", std::process::id()));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct IndexModule;

                impl IndexModule {
                    pub fn register_event_listeners(&self) {
                        let runtime = IndexerRuntimeConfig::new(2, 100, 10);
                        let _content = ContentIndexer::with_runtime(todo!(), runtime.clone());
                        let _product = ProductIndexer::with_runtime(todo!(), runtime);
                    }
                }
            "#,
        )
        .expect("temporary lib.rs should be writable");

        validate_module_event_listener_contract("index", &base)
            .expect("index event listener fragments should validate");

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_event_ingress_contract_rejects_workflow_without_webhook_routes() {
        let base = env::temp_dir().join(format!(
            "xtask-event-ingress-missing-{}",
            std::process::id()
        ));
        let workflow_controllers_dir = base
            .join("crates")
            .join("rustok-workflow")
            .join("src")
            .join("controllers");
        let workflow_services_dir = base
            .join("crates")
            .join("rustok-workflow")
            .join("src")
            .join("services");
        let server_workflow_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers")
            .join("workflow");
        std::fs::create_dir_all(&workflow_controllers_dir)
            .expect("temporary workflow controllers dir should exist");
        std::fs::create_dir_all(&workflow_services_dir)
            .expect("temporary workflow services dir should exist");
        std::fs::create_dir_all(&server_workflow_dir)
            .expect("temporary server workflow dir should exist");
        std::fs::write(
            workflow_controllers_dir.join("mod.rs"),
            "pub fn routes() {}\n",
        )
        .expect("temporary workflow controllers mod.rs should be writable");
        std::fs::write(
            workflow_services_dir.join("trigger_handler.rs"),
            "impl EventHandler for WorkflowTriggerHandler {}\n",
        )
        .expect("temporary trigger_handler.rs should be writable");
        std::fs::write(
            server_workflow_dir.join("mod.rs"),
            "pub fn webhook_routes() -> Routes { rustok_workflow::controllers::webhook_routes() }\n",
        )
        .expect("temporary server workflow shim should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");
        let module_root = base.join("crates").join("rustok-workflow");

        let error =
            validate_module_event_ingress_contract(&manifest_path, "workflow", &module_root)
                .expect_err("missing workflow webhook routes must fail");
        assert!(error.to_string().contains("webhook_routes"));

        let _ = std::fs::remove_file(workflow_controllers_dir.join("mod.rs"));
        let _ = std::fs::remove_file(workflow_services_dir.join("trigger_handler.rs"));
        let _ = std::fs::remove_file(server_workflow_dir.join("mod.rs"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&workflow_controllers_dir);
        let _ = std::fs::remove_dir(&workflow_services_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-workflow").join("src"));
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-workflow"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&server_workflow_dir);
        let _ = std::fs::remove_dir(
            base.join("apps")
                .join("server")
                .join("src")
                .join("controllers"),
        );
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_event_ingress_contract_accepts_workflow_webhook_shim() {
        let base = env::temp_dir().join(format!("xtask-event-ingress-ok-{}", std::process::id()));
        let workflow_controllers_dir = base
            .join("crates")
            .join("rustok-workflow")
            .join("src")
            .join("controllers");
        let workflow_services_dir = base
            .join("crates")
            .join("rustok-workflow")
            .join("src")
            .join("services");
        let server_workflow_dir = base
            .join("apps")
            .join("server")
            .join("src")
            .join("controllers")
            .join("workflow");
        std::fs::create_dir_all(&workflow_controllers_dir)
            .expect("temporary workflow controllers dir should exist");
        std::fs::create_dir_all(&workflow_services_dir)
            .expect("temporary workflow services dir should exist");
        std::fs::create_dir_all(&server_workflow_dir)
            .expect("temporary server workflow dir should exist");
        std::fs::write(
            workflow_controllers_dir.join("mod.rs"),
            "pub fn routes() {}\npub fn webhook_routes() { Routes::new().prefix(\"webhooks\"); }\n",
        )
        .expect("temporary workflow controllers mod.rs should be writable");
        std::fs::write(
            workflow_services_dir.join("trigger_handler.rs"),
            "impl EventHandler for WorkflowTriggerHandler {}\n",
        )
        .expect("temporary trigger_handler.rs should be writable");
        std::fs::write(
            server_workflow_dir.join("mod.rs"),
            "pub fn webhook_routes() -> Routes { rustok_workflow::controllers::webhook_routes() }\n",
        )
        .expect("temporary server workflow shim should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");
        let module_root = base.join("crates").join("rustok-workflow");

        validate_module_event_ingress_contract(&manifest_path, "workflow", &module_root)
            .expect("workflow event ingress contract should validate");

        let _ = std::fs::remove_file(workflow_controllers_dir.join("mod.rs"));
        let _ = std::fs::remove_file(workflow_services_dir.join("trigger_handler.rs"));
        let _ = std::fs::remove_file(server_workflow_dir.join("mod.rs"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&workflow_controllers_dir);
        let _ = std::fs::remove_dir(&workflow_services_dir);
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-workflow").join("src"));
        let _ = std::fs::remove_dir(base.join("crates").join("rustok-workflow"));
        let _ = std::fs::remove_dir(base.join("crates"));
        let _ = std::fs::remove_dir(&server_workflow_dir);
        let _ = std::fs::remove_dir(
            base.join("apps")
                .join("server")
                .join("src")
                .join("controllers"),
        );
        let _ = std::fs::remove_dir(base.join("apps").join("server").join("src"));
        let _ = std::fs::remove_dir(base.join("apps").join("server"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_index_search_boundary_contract_rejects_index_exposing_search_engine() {
        let base = env::temp_dir().join(format!(
            "xtask-index-search-boundary-bad-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub use crate::search::PgSearchEngine;\npub struct IndexModule; pub struct ContentIndexer; pub struct ProductIndexer; pub struct IndexerRuntimeConfig;",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            base.join("README.md"),
            "read-model substrate\nContentIndexer::with_runtime\nProductIndexer::with_runtime\nIndexerRuntimeConfig\n",
        )
        .expect("temporary README.md should be writable");

        let error = validate_module_index_search_boundary_contract("index", &base)
            .expect_err("index must not expose search-owned symbols");
        assert!(error.to_string().contains("PgSearchEngine"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(base.join("README.md"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_index_search_boundary_contract_accepts_search_surface() {
        let base = env::temp_dir().join(format!(
            "xtask-index-search-boundary-ok-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub use crate::engine::SearchEngineKind;\npub use crate::pg_engine::PgSearchEngine;\npub use crate::ingestion::SearchIngestionHandler;\n",
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            base.join("README.md"),
            "search_documents\nproduct-facing search contracts\n",
        )
        .expect("temporary README.md should be writable");

        validate_module_index_search_boundary_contract("search", &base)
            .expect("search boundary surface should validate");

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(base.join("README.md"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_search_operator_surface_contract_rejects_missing_readme_marker() {
        let base = env::temp_dir().join(format!(
            "xtask-search-operator-surface-missing-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        let docs_dir = base.join("docs");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::create_dir_all(&docs_dir).expect("temporary docs dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct SearchDiagnosticsService;
                pub struct SearchAnalyticsService;
                pub struct SearchSettingsService;
                pub struct SearchDictionaryService;
            "#,
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            base.join("README.md"),
            "searchDiagnostics\nsearchAnalytics\nsearchSettingsPreview\n",
        )
        .expect("temporary README.md should be writable");
        std::fs::write(docs_dir.join("observability-runbook.md"), "# runbook\n")
            .expect("temporary observability-runbook.md should be writable");

        let error = validate_module_search_operator_surface_contract("search", &base)
            .expect_err("missing operator readme marker must fail");
        assert!(error.to_string().contains("triggerSearchRebuild"));

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(base.join("README.md"));
        let _ = std::fs::remove_file(docs_dir.join("observability-runbook.md"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&docs_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_search_operator_surface_contract_accepts_operator_plane() {
        let base = env::temp_dir().join(format!(
            "xtask-search-operator-surface-ok-{}",
            std::process::id()
        ));
        let src_dir = base.join("src");
        let docs_dir = base.join("docs");
        std::fs::create_dir_all(&src_dir).expect("temporary src dir should exist");
        std::fs::create_dir_all(&docs_dir).expect("temporary docs dir should exist");
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
                pub struct SearchDiagnosticsService;
                pub struct SearchAnalyticsService;
                pub struct SearchSettingsService;
                pub struct SearchDictionaryService;
            "#,
        )
        .expect("temporary lib.rs should be writable");
        std::fs::write(
            base.join("README.md"),
            "searchDiagnostics\nsearchAnalytics\nsearchSettingsPreview\ntriggerSearchRebuild\n",
        )
        .expect("temporary README.md should be writable");
        std::fs::write(docs_dir.join("observability-runbook.md"), "# runbook\n")
            .expect("temporary observability-runbook.md should be writable");

        validate_module_search_operator_surface_contract("search", &base)
            .expect("search operator-plane contract should validate");

        let _ = std::fs::remove_file(src_dir.join("lib.rs"));
        let _ = std::fs::remove_file(base.join("README.md"));
        let _ = std::fs::remove_file(docs_dir.join("observability-runbook.md"));
        let _ = std::fs::remove_dir(&src_dir);
        let _ = std::fs::remove_dir(&docs_dir);
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_docs_navigation_contract_rejects_missing_ui_navigation_link() {
        let base = env::temp_dir().join(format!("xtask-docs-nav-missing-{}", std::process::id()));
        let docs_modules_dir = base.join("docs").join("modules");
        std::fs::create_dir_all(&docs_modules_dir)
            .expect("temporary docs/modules dir should exist");
        std::fs::write(
            docs_modules_dir.join("_index.md"),
            "| `rustok-demo` | [docs](../../crates/rustok-demo/docs/README.md) | [plan](../../crates/rustok-demo/docs/implementation-plan.md) |\n",
        )
        .expect("temporary _index.md should be writable");
        std::fs::write(docs_modules_dir.join("UI_PACKAGES_INDEX.md"), "# ui\n")
            .expect("temporary UI index should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_docs_navigation_contract(
            &manifest_path,
            "demo",
            &spec,
            Some("rustok-demo-admin"),
            None,
            &[],
        )
        .expect_err("missing admin ui link must fail");
        assert!(error.to_string().contains("declares admin UI"));

        let _ = std::fs::remove_file(docs_modules_dir.join("_index.md"));
        let _ = std::fs::remove_file(docs_modules_dir.join("UI_PACKAGES_INDEX.md"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&docs_modules_dir);
        let _ = std::fs::remove_dir(base.join("docs"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_docs_navigation_contract_accepts_documented_storefront_module() {
        let base = env::temp_dir().join(format!("xtask-docs-nav-ok-{}", std::process::id()));
        let docs_modules_dir = base.join("docs").join("modules");
        std::fs::create_dir_all(&docs_modules_dir)
            .expect("temporary docs/modules dir should exist");
        std::fs::write(
            docs_modules_dir.join("_index.md"),
            "| `rustok-demo` | [docs](../../crates/rustok-demo/docs/README.md) | [plan](../../crates/rustok-demo/docs/implementation-plan.md) |\n",
        )
        .expect("temporary _index.md should be writable");
        std::fs::write(
            docs_modules_dir.join("UI_PACKAGES_INDEX.md"),
            "- `rustok-demo` storefront UI: [README](../../crates/rustok-demo/storefront/README.md)\n",
        )
        .expect("temporary UI index should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let spec = ModuleSpec {
            crate_name: "rustok-demo".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-demo".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        validate_module_docs_navigation_contract(
            &manifest_path,
            "demo",
            &spec,
            None,
            Some("rustok-demo-storefront"),
            &[],
        )
        .expect("documented storefront UI should validate");

        let _ = std::fs::remove_file(docs_modules_dir.join("_index.md"));
        let _ = std::fs::remove_file(docs_modules_dir.join("UI_PACKAGES_INDEX.md"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&docs_modules_dir);
        let _ = std::fs::remove_dir(base.join("docs"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_docs_navigation_contract_rejects_missing_next_admin_showcase_entry() {
        let base =
            env::temp_dir().join(format!("xtask-docs-nav-next-admin-{}", std::process::id()));
        let docs_modules_dir = base.join("docs").join("modules");
        let next_admin_package_dir = base
            .join("apps")
            .join("next-admin")
            .join("packages")
            .join("blog");
        std::fs::create_dir_all(&docs_modules_dir)
            .expect("temporary docs/modules dir should exist");
        std::fs::create_dir_all(&next_admin_package_dir)
            .expect("temporary next-admin package dir should exist");
        std::fs::write(
            docs_modules_dir.join("_index.md"),
            "| `rustok-blog` | [docs](../../crates/rustok-blog/docs/README.md) | [plan](../../crates/rustok-blog/docs/implementation-plan.md) |\n",
        )
        .expect("temporary _index.md should be writable");
        std::fs::write(
            docs_modules_dir.join("UI_PACKAGES_INDEX.md"),
            "- `rustok-blog` admin UI: [README](../../crates/rustok-blog/admin/README.md)\n",
        )
        .expect("temporary UI index should be writable");
        let manifest_path = base.join("modules.toml");
        std::fs::write(&manifest_path, "app = \"rustok-server\"\nschema = 2\n")
            .expect("temporary modules.toml should be writable");

        let spec = ModuleSpec {
            crate_name: "rustok-blog".to_string(),
            source: "path".to_string(),
            path: Some("crates/rustok-blog".to_string()),
            required: false,
            version: None,
            git: None,
            rev: None,
            depends_on: None,
            features: None,
        };

        let error = validate_module_docs_navigation_contract(
            &manifest_path,
            "blog",
            &spec,
            Some("rustok-blog-admin"),
            None,
            &["next-admin".to_string()],
        )
        .expect_err("missing next-admin showcase entry must fail");
        assert!(error
            .to_string()
            .contains("showcase_admin_surfaces=['next-admin']"));

        let _ = std::fs::remove_file(docs_modules_dir.join("_index.md"));
        let _ = std::fs::remove_file(docs_modules_dir.join("UI_PACKAGES_INDEX.md"));
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir(&docs_modules_dir);
        let _ = std::fs::remove_dir(base.join("docs"));
        let _ = std::fs::remove_dir(&next_admin_package_dir);
        let _ = std::fs::remove_dir(base.join("apps").join("next-admin").join("packages"));
        let _ = std::fs::remove_dir(base.join("apps").join("next-admin"));
        let _ = std::fs::remove_dir(base.join("apps"));
        let _ = std::fs::remove_dir(&base);
    }

    #[test]
    fn validate_module_ui_surface_contract_rejects_declared_missing_subcrate() {
        let error = validate_module_ui_surface_contract(
            "blog",
            Path::new("crates/rustok-blog"),
            "preview",
            Some("rustok-blog-preview"),
        )
        .expect_err("declared surface without subcrate must fail");

        assert!(error
            .to_string()
            .contains("declares [provides.preview_ui].leptos_crate"));
    }

    #[test]
    fn validate_module_ui_surface_contract_accepts_wired_existing_admin_subcrate() {
        validate_module_ui_surface_contract(
            "blog",
            &super::workspace_root().join("crates").join("rustok-blog"),
            "admin",
            Some("rustok-blog-admin"),
        )
        .expect("wired admin subcrate should validate");
    }

    #[test]
    fn build_module_test_plan_includes_main_and_ui_crates() {
        let preview = ModulePublishDryRunPreview {
            slug: "blog".to_string(),
            version: "1.2.3".to_string(),
            crate_name: "rustok-blog".to_string(),
            module_name: "Blog".to_string(),
            module_description: "A blog module description long enough.".to_string(),
            ownership: "first_party".to_string(),
            trust_level: "verified".to_string(),
            license: "MIT".to_string(),
            manifest_path: "modules.toml".to_string(),
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
            module_entry_type: Some("BlogModule".to_string()),
            marketplace: ModuleMarketplacePreview {
                category: Some("content".to_string()),
                tags: vec!["content".to_string()],
            },
            ui_packages: ModuleUiPackagesPreview {
                admin: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-admin".to_string(),
                    manifest_path: "crates/rustok-blog/admin/Cargo.toml".to_string(),
                }),
                storefront: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-storefront".to_string(),
                    manifest_path: "crates/rustok-blog/storefront/Cargo.toml".to_string(),
                }),
            },
        };

        let plan = build_module_test_plan(&preview);
        let commands = plan
            .commands
            .iter()
            .map(|command| command.argv.join(" "))
            .collect::<Vec<_>>();

        assert_eq!(commands.len(), 4);
        assert!(commands[0].contains("module validate blog"));
        assert!(commands[1].contains("cargo check -p rustok-blog"));
        assert!(commands[2].contains("cargo check -p rustok-blog-admin"));
        assert!(commands[3].contains("cargo check -p rustok-blog-storefront"));
    }

    #[test]
    fn build_publish_registry_request_serializes_v2_contract() {
        let preview = ModulePublishDryRunPreview {
            slug: "blog".to_string(),
            version: "1.2.3".to_string(),
            crate_name: "rustok-blog".to_string(),
            module_name: "Blog".to_string(),
            module_description: "A blog module description long enough.".to_string(),
            ownership: "first_party".to_string(),
            trust_level: "verified".to_string(),
            license: "MIT".to_string(),
            manifest_path: "modules.toml".to_string(),
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
            module_entry_type: Some("BlogModule".to_string()),
            marketplace: ModuleMarketplacePreview {
                category: Some("content".to_string()),
                tags: vec!["content".to_string(), "editorial".to_string()],
            },
            ui_packages: ModuleUiPackagesPreview {
                admin: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-admin".to_string(),
                    manifest_path: "crates/rustok-blog/admin/Cargo.toml".to_string(),
                }),
                storefront: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-storefront".to_string(),
                    manifest_path: "crates/rustok-blog/storefront/Cargo.toml".to_string(),
                }),
            },
        };

        let request_body = serde_json::to_value(build_publish_registry_request(&preview))
            .expect("request should serialize");
        assert_eq!(
            request_body["schema_version"],
            REGISTRY_MUTATION_SCHEMA_VERSION
        );
        assert_eq!(request_body["dry_run"], true);
        assert_eq!(request_body["module"]["slug"], "blog");
        assert_eq!(request_body["module"]["version"], "1.2.3");
        assert_eq!(request_body["module"]["crate_name"], "rustok-blog");
        assert_eq!(request_body["module"]["name"], "Blog");
        assert_eq!(
            request_body["module"]["description"],
            "A blog module description long enough."
        );
        assert_eq!(request_body["module"]["ownership"], "first_party");
        assert_eq!(request_body["module"]["trust_level"], "verified");
        assert_eq!(request_body["module"]["license"], "MIT");
        assert_eq!(request_body["module"]["entry_type"], "BlogModule");
        assert_eq!(request_body["module"]["marketplace"]["category"], "content");
        assert_eq!(
            request_body["module"]["marketplace"]["tags"],
            serde_json::json!(["content", "editorial"])
        );
        assert_eq!(
            request_body["module"]["ui_packages"]["admin"]["crate_name"],
            "rustok-blog-admin"
        );
        assert_eq!(
            request_body["module"]["ui_packages"]["storefront"]["crate_name"],
            "rustok-blog-storefront"
        );
    }

    #[test]
    fn build_live_publish_registry_request_turns_off_dry_run() {
        let preview = ModulePublishDryRunPreview {
            slug: "blog".to_string(),
            version: "1.2.3".to_string(),
            crate_name: "rustok-blog".to_string(),
            module_name: "Blog".to_string(),
            module_description: "A blog module description long enough.".to_string(),
            ownership: "first_party".to_string(),
            trust_level: "verified".to_string(),
            license: "MIT".to_string(),
            manifest_path: "modules.toml".to_string(),
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
            module_entry_type: Some("BlogModule".to_string()),
            marketplace: ModuleMarketplacePreview {
                category: Some("content".to_string()),
                tags: vec!["content".to_string(), "editorial".to_string()],
            },
            ui_packages: ModuleUiPackagesPreview {
                admin: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-admin".to_string(),
                    manifest_path: "crates/rustok-blog/admin/Cargo.toml".to_string(),
                }),
                storefront: Some(ModuleUiPackagePreview {
                    crate_name: "rustok-blog-storefront".to_string(),
                    manifest_path: "crates/rustok-blog/storefront/Cargo.toml".to_string(),
                }),
            },
        };

        let request_body = serde_json::to_value(build_live_publish_registry_request(&preview))
            .expect("live publish request should serialize");

        assert_eq!(request_body["dry_run"], false);
        assert_eq!(request_body["module"]["slug"], "blog");
    }

    #[test]
    fn module_publish_command_requires_slug() {
        let error = module_publish_command(&[])
            .expect_err("publish command without slug should fail immediately");

        assert!(error
            .to_string()
            .contains("module publish requires a module slug"));
    }

    #[test]
    fn module_publish_command_live_requires_registry_url() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let _env_guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", None);
        let args = vec!["blog".to_string()];

        let error = module_publish_command(&args)
            .expect_err("live publish should require registry url before any network call");

        assert!(error
            .to_string()
            .contains("Live module publish requires --registry-url"));
    }

    #[test]
    fn module_publish_command_live_requires_actor() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:18080".to_string(),
        ];

        let error = module_publish_command(&args)
            .expect_err("live publish should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module publish requires --actor <actor>"));
    }

    #[test]
    fn module_publish_command_rejects_unknown_slug() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let args = vec!["missing-module".to_string(), "--dry-run".to_string()];

        let error = module_publish_command(&args)
            .expect_err("unknown slug should fail before any network call");

        assert!(error
            .to_string()
            .contains("Unknown module slug 'missing-module'"));
    }

    #[test]
    fn module_test_command_requires_slug() {
        let error =
            module_test_command(&[]).expect_err("module test without slug should fail immediately");

        assert!(error
            .to_string()
            .contains("module test requires a module slug"));
    }

    #[test]
    fn build_validation_stage_registry_request_serializes_stage_contract() {
        let preview = ModuleValidationStageDryRunPreview {
            action: "validation_stage".to_string(),
            request_id: "rpr_test".to_string(),
            stage: "compile_smoke".to_string(),
            status: "passed".to_string(),
            detail: Some("External CI recorded the result.".to_string()),
            reason_code: None,
            requeue: false,
        };

        let request_body = serde_json::to_value(build_validation_stage_registry_request(&preview))
            .expect("stage request should serialize");

        assert_eq!(
            request_body
                .get("schema_version")
                .and_then(serde_json::Value::as_u64),
            Some(REGISTRY_MUTATION_SCHEMA_VERSION as u64)
        );
        assert_eq!(
            request_body
                .get("dry_run")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            request_body
                .get("stage")
                .and_then(serde_json::Value::as_str),
            Some("compile_smoke")
        );
        assert_eq!(
            request_body
                .get("status")
                .and_then(serde_json::Value::as_str),
            Some("passed")
        );
        assert_eq!(
            request_body
                .get("requeue")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn build_validation_stage_registry_request_serializes_requeue_contract() {
        let preview = ModuleValidationStageDryRunPreview {
            action: "validation_stage".to_string(),
            request_id: "rpr_retry".to_string(),
            stage: "targeted_tests".to_string(),
            status: "queued".to_string(),
            detail: Some("Waiting for external rerun.".to_string()),
            reason_code: None,
            requeue: true,
        };

        let request_body = serde_json::to_value(build_validation_stage_registry_request(&preview))
            .expect("stage request should serialize");

        assert_eq!(
            request_body
                .get("stage")
                .and_then(serde_json::Value::as_str),
            Some("targeted_tests")
        );
        assert_eq!(
            request_body
                .get("status")
                .and_then(serde_json::Value::as_str),
            Some("queued")
        );
        assert_eq!(
            request_body
                .get("requeue")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            request_body
                .get("detail")
                .and_then(serde_json::Value::as_str),
            Some("Waiting for external rerun.")
        );
        assert_eq!(
            request_body
                .get("dry_run")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn build_live_validation_stage_registry_request_turns_off_dry_run() {
        let preview = ModuleValidationStageDryRunPreview {
            action: "validation_stage".to_string(),
            request_id: "rpr_live".to_string(),
            stage: "security_policy_review".to_string(),
            status: "blocked".to_string(),
            detail: Some("Waiting for manual policy sign-off.".to_string()),
            reason_code: None,
            requeue: false,
        };

        let request_body =
            serde_json::to_value(build_live_validation_stage_registry_request(&preview))
                .expect("live stage request should serialize");

        assert_eq!(
            request_body
                .get("dry_run")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
        assert_eq!(
            request_body
                .get("status")
                .and_then(serde_json::Value::as_str),
            Some("blocked")
        );
    }

    #[test]
    fn build_validation_stage_registry_request_preserves_null_detail() {
        let preview = ModuleValidationStageDryRunPreview {
            action: "validation_stage".to_string(),
            request_id: "rpr_null".to_string(),
            stage: "compile_smoke".to_string(),
            status: "running".to_string(),
            detail: None,
            reason_code: None,
            requeue: false,
        };

        let request_body = serde_json::to_value(build_validation_stage_registry_request(&preview))
            .expect("stage request should serialize");

        assert!(
            request_body.get("detail").is_some(),
            "stage request keeps explicit detail field for API contract stability"
        );
        assert!(
            request_body
                .get("detail")
                .is_some_and(serde_json::Value::is_null),
            "detail should serialize as null when operator omitted it"
        );
    }

    #[test]
    fn module_stage_command_rejects_unknown_status() {
        let args = vec![
            "rpr_test".to_string(),
            "compile_smoke".to_string(),
            "skipped".to_string(),
        ];

        let error = module_stage_command(&args).expect_err("unsupported stage status must fail");

        assert!(error
            .to_string()
            .contains("module stage status 'skipped' is not supported"));
    }

    #[test]
    fn module_stage_command_requires_request_stage_and_status() {
        let args = vec!["rpr_test".to_string(), "compile_smoke".to_string()];

        let error = module_stage_command(&args)
            .expect_err("stage command without full argument set should fail immediately");

        assert!(error
            .to_string()
            .contains("module stage requires a request id, stage key, and status"));
    }

    #[test]
    fn module_stage_command_rejects_requeue_without_queued_status() {
        let args = vec![
            "rpr_test".to_string(),
            "compile_smoke".to_string(),
            "passed".to_string(),
            "--requeue".to_string(),
        ];

        let error = module_stage_command(&args)
            .expect_err("requeue should only be accepted for queued status");

        assert!(error
            .to_string()
            .contains("module stage --requeue requires status 'queued'"));
    }

    #[test]
    fn module_stage_command_rejects_empty_request_id() {
        let args = vec![
            "   ".to_string(),
            "compile_smoke".to_string(),
            "queued".to_string(),
        ];

        let error = module_stage_command(&args)
            .expect_err("empty request id should fail before any registry lookup");

        assert!(error
            .to_string()
            .contains("module stage requires a non-empty request id"));
    }

    #[test]
    fn module_stage_command_rejects_empty_stage_key() {
        let args = vec![
            "rpr_test".to_string(),
            "   ".to_string(),
            "queued".to_string(),
        ];

        let error = module_stage_command(&args)
            .expect_err("empty stage key should fail before any registry lookup");

        assert!(error
            .to_string()
            .contains("module stage requires a non-empty stage key"));
    }

    #[test]
    fn module_stage_command_live_requires_registry_url() {
        let args = vec![
            "rpr_test".to_string(),
            "compile_smoke".to_string(),
            "running".to_string(),
        ];

        let error = module_stage_command(&args)
            .expect_err("live stage update should require registry url before any network call");

        assert!(error
            .to_string()
            .contains("Live module stage update requires --registry-url"));
    }

    #[test]
    fn module_stage_command_live_requires_actor() {
        let args = vec![
            "rpr_test".to_string(),
            "compile_smoke".to_string(),
            "running".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:18080".to_string(),
        ];

        let error = module_stage_command(&args)
            .expect_err("live stage update should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module stage update requires --actor <actor>"));
    }

    #[test]
    fn module_stage_run_command_live_requires_actor() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "rpr_test".to_string(),
            "compile_smoke".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:18080".to_string(),
        ];

        let error = module_stage_run_command(&args)
            .expect_err("live stage-run should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module stage-run requires --actor <actor>"));
    }

    #[test]
    fn registry_url_argument_prefers_cli_value_over_env() {
        let _guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", Some("http://env.example"));

        assert_eq!(
            registry_url_argument(&[
                "--registry-url".to_string(),
                "  http://cli.example  ".to_string(),
            ]),
            Some("http://cli.example".to_string())
        );
    }

    #[test]
    fn registry_url_argument_uses_trimmed_env_and_ignores_blank_values() {
        let _guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", Some("  http://env.example  "));
        assert_eq!(
            registry_url_argument(&[]),
            Some("http://env.example".to_string())
        );

        drop(_guard);

        let _guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", Some("   "));
        assert_eq!(registry_url_argument(&[]), None);
    }

    #[test]
    fn runner_token_argument_prefers_cli_value_over_env() {
        let _guard = EnvVarGuard::set(REMOTE_RUNNER_TOKEN_ENV, Some("env-token"));

        assert_eq!(
            runner_token_argument(&["--runner-token".to_string(), "  cli-token  ".to_string(),]),
            Some("cli-token".to_string())
        );
    }

    #[test]
    fn runner_token_argument_uses_trimmed_env_and_ignores_blank_values() {
        let _guard = EnvVarGuard::set(REMOTE_RUNNER_TOKEN_ENV, Some("  env-token  "));
        assert_eq!(runner_token_argument(&[]), Some("env-token".to_string()));

        drop(_guard);

        let _guard = EnvVarGuard::set(REMOTE_RUNNER_TOKEN_ENV, Some("   "));
        assert_eq!(runner_token_argument(&[]), None);
    }

    #[test]
    fn actor_argument_trims_and_ignores_blank_values() {
        assert_eq!(
            actor_argument(&[
                "--actor".to_string(),
                "  governance:moderator  ".to_string()
            ]),
            Some("governance:moderator".to_string())
        );
        assert_eq!(
            actor_argument(&["--actor".to_string(), "   ".to_string()]),
            None
        );
    }

    #[test]
    fn supported_remote_runner_stages_include_manual_review_only_when_enabled() {
        assert_eq!(
            supported_remote_runner_stages(false),
            vec!["compile_smoke", "targeted_tests"]
        );
        assert_eq!(
            supported_remote_runner_stages(true),
            vec!["compile_smoke", "targeted_tests", "security_policy_review"]
        );
    }

    #[test]
    fn publish_status_action_available_matches_governance_actions_case_insensitively() {
        let status = RegistryPublishStatusHttpResponse {
            request_id: "rpr_case".to_string(),
            slug: "catalog".to_string(),
            version: "1.0.0".to_string(),
            status: "approved".to_string(),
            accepted: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            follow_up_gates: Vec::new(),
            validation_stages: Vec::new(),
            approval_override_required: false,
            approval_override_reason_codes: Vec::new(),
            governance_actions: vec![RegistryGovernanceActionHttpResponse {
                key: "request_changes".to_string(),
                reason_required: true,
                reason_code_required: true,
                reason_codes: vec!["docs_gap".to_string()],
                destructive: false,
            }],
            next_step: None,
        };

        assert!(publish_status_action_available(&status, "REQUEST_CHANGES"));
        assert!(!publish_status_action_available(&status, "approve"));
    }

    #[test]
    fn module_request_changes_command_live_requires_actor() {
        let args = vec![
            "rpr_123".to_string(),
            "--registry-url".to_string(),
            "http://localhost:5150".to_string(),
            "--reason".to_string(),
            "Needs a fresh artifact".to_string(),
            "--reason-code".to_string(),
            "artifact_mismatch".to_string(),
        ];

        let error = module_request_changes_command(&args)
            .expect_err("live request-changes should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module request-changes requires --actor <actor>"));
    }

    #[test]
    fn module_hold_command_live_requires_reason_code() {
        let args = vec![
            "rpr_123".to_string(),
            "--registry-url".to_string(),
            "http://localhost:5150".to_string(),
            "--actor".to_string(),
            "governance:moderator".to_string(),
            "--reason".to_string(),
            "Incident review".to_string(),
        ];

        let error = module_hold_command(&args)
            .expect_err("live hold should require reason code before any network call");

        assert!(error
            .to_string()
            .contains("Live module hold requires --reason-code"));
    }

    #[test]
    fn module_resume_command_live_requires_reason() {
        let args = vec![
            "rpr_123".to_string(),
            "--registry-url".to_string(),
            "http://localhost:5150".to_string(),
            "--actor".to_string(),
            "governance:moderator".to_string(),
            "--reason-code".to_string(),
            "review_complete".to_string(),
        ];

        let error = module_resume_command(&args)
            .expect_err("live resume should require reason before any network call");

        assert!(error
            .to_string()
            .contains("Live module resume requires --reason <text>"));
    }

    #[test]
    fn positive_u64_argument_rejects_zero_values() {
        let error = positive_u64_argument(
            &["--poll-interval-ms".to_string(), "0".to_string()],
            "--poll-interval-ms",
            "module runner",
        )
        .expect_err("zero interval must fail");

        assert!(error
            .to_string()
            .contains("module runner --poll-interval-ms must be > 0"));
    }

    #[test]
    fn module_runner_command_requires_runner_id() {
        let error =
            module_runner_command(&[]).expect_err("runner command without id should fail early");

        assert!(error
            .to_string()
            .contains("module runner requires a non-empty runner id"));
    }

    #[test]
    fn reason_argument_trims_and_ignores_blank_values() {
        assert_eq!(
            reason_argument(&["--reason".to_string(), "  ownership move  ".to_string()]),
            Some("ownership move".to_string())
        );
        assert_eq!(
            reason_argument(&["--reason".to_string(), "   ".to_string()]),
            None
        );
    }

    #[test]
    fn detail_argument_trims_and_ignores_blank_values() {
        assert_eq!(
            detail_argument(&["--detail".to_string(), "  external runner  ".to_string()]),
            Some("external runner".to_string())
        );
        assert_eq!(
            detail_argument(&["--detail".to_string(), "   ".to_string()]),
            None
        );
    }

    #[test]
    fn reason_code_argument_trims_and_normalizes_values() {
        assert_eq!(
            reason_code_argument(&[
                "--reason-code".to_string(),
                "  Critical_Regression  ".to_string(),
            ])
            .expect("supported reason code should parse"),
            Some("critical_regression".to_string())
        );
    }

    #[test]
    fn reason_code_argument_rejects_unknown_values() {
        let error = reason_code_argument(&["--reason-code".to_string(), "surprise".to_string()])
            .expect_err("unknown yank reason code must fail");

        assert!(error
            .to_string()
            .contains("module yank reason code 'surprise' is not supported"));
    }

    #[test]
    fn module_owner_transfer_command_rejects_empty_new_owner_actor() {
        let args = vec!["blog".to_string(), "   ".to_string()];

        let error = module_owner_transfer_command(&args)
            .expect_err("empty new owner actor must fail before manifest lookup");

        assert!(error
            .to_string()
            .contains("module owner-transfer requires a non-empty new owner actor"));
    }

    #[test]
    fn module_owner_transfer_command_requires_slug_and_new_owner_actor() {
        let args = vec!["blog".to_string()];

        let error = module_owner_transfer_command(&args)
            .expect_err("owner-transfer command without actor should fail immediately");

        assert!(error
            .to_string()
            .contains("module owner-transfer requires a module slug and new owner actor"));
    }

    #[test]
    fn module_owner_transfer_command_live_requires_reason() {
        let _guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "publisher:forum".to_string(),
            "--actor".to_string(),
            "registry:admin".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ];

        let error = module_owner_transfer_command(&args)
            .expect_err("live owner-transfer should require reason before any network call");

        assert!(error
            .to_string()
            .contains("Live module owner-transfer requires --reason <text>"));
    }

    #[test]
    fn module_owner_transfer_command_live_requires_registry_url() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let _env_guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", None);
        let args = vec![
            "blog".to_string(),
            "publisher:forum".to_string(),
            "--actor".to_string(),
            "registry:admin".to_string(),
            "--reason".to_string(),
            "ownership move".to_string(),
        ];

        let error = module_owner_transfer_command(&args)
            .expect_err("live owner-transfer should require registry url before any network call");

        assert!(error
            .to_string()
            .contains("Live module owner-transfer requires --registry-url"));
    }

    #[test]
    fn module_owner_transfer_command_live_requires_actor() {
        let _guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "publisher:forum".to_string(),
            "--reason".to_string(),
            "ownership move".to_string(),
            "--reason-code".to_string(),
            "maintenance_handoff".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ];

        let error = module_owner_transfer_command(&args)
            .expect_err("live owner-transfer should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module owner-transfer requires --actor <actor>"));
    }

    #[test]
    fn module_yank_command_live_requires_reason() {
        let _guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "1.2.3".to_string(),
            "--actor".to_string(),
            "registry:admin".to_string(),
            "--reason-code".to_string(),
            "rollback".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ];

        let error = module_yank_command(&args)
            .expect_err("live yank should require reason before any network call");

        assert!(error
            .to_string()
            .contains("Live module yank requires --reason <text>"));
    }

    #[test]
    fn module_yank_command_requires_slug_and_version() {
        let args = vec!["blog".to_string()];

        let error = module_yank_command(&args)
            .expect_err("yank command without version should fail immediately");

        assert!(error
            .to_string()
            .contains("module yank requires a module slug and version"));
    }

    #[test]
    fn module_yank_command_live_requires_registry_url() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let _env_guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", None);
        let args = vec![
            "blog".to_string(),
            "1.2.3".to_string(),
            "--actor".to_string(),
            "registry:admin".to_string(),
            "--reason".to_string(),
            "policy rollback".to_string(),
            "--reason-code".to_string(),
            "rollback".to_string(),
        ];

        let error = module_yank_command(&args)
            .expect_err("live yank should require registry url before any network call");

        assert!(error
            .to_string()
            .contains("Live module yank requires --registry-url"));
    }

    #[test]
    fn module_yank_command_live_requires_actor() {
        let _guard = WorkspaceRootGuard::enter();
        let args = vec![
            "blog".to_string(),
            "1.2.3".to_string(),
            "--reason".to_string(),
            "critical regression in production".to_string(),
            "--reason-code".to_string(),
            "rollback".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ];

        let error = module_yank_command(&args)
            .expect_err("live yank should require actor before any network call");

        assert!(error
            .to_string()
            .contains("Live module yank requires --actor <actor>"));
    }

    #[test]
    fn module_yank_command_live_requires_reason_code() {
        let _cwd_guard = WorkspaceRootGuard::enter();
        let _env_guard = EnvVarGuard::set("RUSTOK_MODULE_REGISTRY_URL", None);
        let args = vec![
            "blog".to_string(),
            "1.2.3".to_string(),
            "--actor".to_string(),
            "registry:admin".to_string(),
            "--reason".to_string(),
            "critical regression in production".to_string(),
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ];

        let error = module_yank_command(&args)
            .expect_err("live yank should require reason code before any network call");

        assert!(error
            .to_string()
            .contains("Live module yank requires --reason-code"));
        assert!(error
            .to_string()
            .contains(&REGISTRY_YANK_REASON_CODES.join("|")));
    }

    #[test]
    fn module_yank_command_rejects_invalid_semver() {
        let args = vec!["blog".to_string(), "not-a-version".to_string()];

        let error = module_yank_command(&args)
            .expect_err("invalid semver should fail before any manifest lookup");

        assert!(error
            .to_string()
            .contains("module yank version 'not-a-version' is not valid semver"));
    }

    #[test]
    fn build_yank_registry_request_serializes_v2_contract() {
        let preview = ModuleYankDryRunPreview {
            action: "yank".to_string(),
            slug: "blog".to_string(),
            version: "1.2.3".to_string(),
            crate_name: "rustok-blog".to_string(),
            current_local_version: "1.2.3".to_string(),
            matches_local_version: true,
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
        };

        let request_body = serde_json::to_value(build_yank_registry_request(
            &preview,
            Some("Accidental publish".to_string()),
            Some("rollback".to_string()),
        ))
        .expect("request should serialize");
        assert_eq!(
            request_body["schema_version"],
            REGISTRY_MUTATION_SCHEMA_VERSION
        );
        assert_eq!(request_body["dry_run"], true);
        assert_eq!(request_body["slug"], "blog");
        assert_eq!(request_body["version"], "1.2.3");
        assert_eq!(request_body["reason"], "Accidental publish");
        assert_eq!(request_body["reason_code"], "rollback");
    }

    #[test]
    fn build_live_yank_registry_request_turns_off_dry_run_and_keeps_reason_code() {
        let preview = ModuleYankDryRunPreview {
            action: "yank".to_string(),
            slug: "blog".to_string(),
            version: "1.2.3".to_string(),
            crate_name: "rustok-blog".to_string(),
            current_local_version: "1.2.3".to_string(),
            matches_local_version: true,
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
        };

        let request_body = serde_json::to_value(build_live_yank_registry_request(
            &preview,
            Some("Security takedown".to_string()),
            Some("security".to_string()),
        ))
        .expect("live yank request should serialize");

        assert_eq!(request_body["dry_run"], false);
        assert_eq!(request_body["reason_code"], "security");
    }

    #[test]
    fn build_owner_transfer_registry_request_serializes_v2_contract() {
        let preview = ModuleOwnerTransferDryRunPreview {
            action: "owner_transfer".to_string(),
            slug: "blog".to_string(),
            crate_name: "rustok-blog".to_string(),
            current_local_version: "1.2.3".to_string(),
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
            new_owner_actor: "publisher:forum".to_string(),
            reason: Some("Ownership transferred to the forum publisher".to_string()),
            reason_code: Some("maintenance_handoff".to_string()),
        };

        let request_body = serde_json::to_value(build_owner_transfer_registry_request(
            &preview,
            preview.reason.clone(),
            preview.reason_code.clone(),
        ))
        .expect("request should serialize");
        assert_eq!(
            request_body["schema_version"],
            REGISTRY_MUTATION_SCHEMA_VERSION
        );
        assert_eq!(request_body["dry_run"], true);
        assert_eq!(request_body["slug"], "blog");
        assert_eq!(request_body["new_owner_actor"], "publisher:forum");
        assert_eq!(
            request_body["reason"],
            "Ownership transferred to the forum publisher"
        );
        assert_eq!(request_body["reason_code"], "maintenance_handoff");
    }

    #[test]
    fn build_live_owner_transfer_registry_request_turns_off_dry_run() {
        let preview = ModuleOwnerTransferDryRunPreview {
            action: "owner_transfer".to_string(),
            slug: "blog".to_string(),
            crate_name: "rustok-blog".to_string(),
            current_local_version: "1.2.3".to_string(),
            package_manifest_path: "crates/rustok-blog/rustok-module.toml".to_string(),
            new_owner_actor: "publisher:comments".to_string(),
            reason: Some("Transfer to the comments publisher".to_string()),
            reason_code: Some("publisher_rotation".to_string()),
        };

        let request_body = serde_json::to_value(build_live_owner_transfer_registry_request(
            &preview,
            preview.reason.clone(),
            preview.reason_code.clone(),
        ))
        .expect("live owner transfer request should serialize");

        assert_eq!(request_body["dry_run"], false);
        assert_eq!(request_body["new_owner_actor"], "publisher:comments");
        assert_eq!(request_body["reason_code"], "publisher_rotation");
    }

    #[test]
    fn registry_endpoint_uses_loopback_for_local_urls() {
        assert!(registry_endpoint_uses_loopback(
            "http://127.0.0.1:5150/v2/catalog/publish"
        ));
        assert!(registry_endpoint_uses_loopback(
            "http://localhost:5150/v2/catalog/yank"
        ));
        assert!(!registry_endpoint_uses_loopback(
            "https://modules.rustok.dev/v2/catalog/publish"
        ));
        assert!(registry_endpoint_uses_loopback(
            "http://[::1]:5150/v2/catalog/publish"
        ));
        assert!(!registry_endpoint_uses_loopback(
            "http://0.0.0.0:5150/v2/catalog/publish"
        ));
    }

    #[test]
    fn auto_approve_argument_detects_flag() {
        assert!(auto_approve_argument(&[
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
            "--auto-approve".to_string(),
        ]));
        assert!(!auto_approve_argument(&[
            "--registry-url".to_string(),
            "http://127.0.0.1:5150".to_string(),
        ]));
    }
}
