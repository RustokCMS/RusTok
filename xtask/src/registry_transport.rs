use super::*;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::de::DeserializeOwned;
use std::net::IpAddr;

pub(crate) fn publish_via_registry_dry_run(
    registry_url: &str,
    preview: &ModulePublishDryRunPreview,
) -> Result<String> {
    let endpoint = format!("{}/v2/catalog/publish", registry_url.trim_end_matches('/'));
    let request = build_publish_registry_request(preview);

    post_registry_json(&endpoint, &request)
}

pub(crate) fn publish_via_registry_live(
    registry_url: &str,
    preview: &ModulePublishDryRunPreview,
    actor: &str,
    auto_approve: bool,
    approve_reason: Option<String>,
    approve_reason_code: Option<String>,
    confirm_manual_review: bool,
) -> Result<String> {
    let publisher = format!("publisher:{}", preview.slug);
    let create_endpoint = format!("{}/v2/catalog/publish", registry_url.trim_end_matches('/'));
    let create_request = build_live_publish_registry_request(preview);
    let create_response: RegistryMutationHttpResponse = post_registry_json_parsed(
        &create_endpoint,
        &create_request,
        Some(actor),
        Some(&publisher),
    )?;
    if !create_response.accepted {
        anyhow::bail!(
            "Registry publish request was not accepted: {}",
            join_registry_errors(&create_response.errors)
        );
    }

    let request_id = create_response
        .request_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| {
            format!(
                "Registry publish did not return request_id; status={:?}, next_step={:?}",
                create_response.status, create_response.next_step
            )
        })?;

    let artifact_bytes = build_publish_artifact_bytes(preview)?;
    let upload_endpoint = format!(
        "{}/v2/catalog/publish/{request_id}/artifact",
        registry_url.trim_end_matches('/')
    );
    let upload_response: RegistryMutationHttpResponse = put_registry_bytes_parsed(
        &upload_endpoint,
        &artifact_bytes,
        "application/json",
        Some(actor),
        Some(&publisher),
    )?;
    ensure_publish_step_accepted(
        "artifact upload",
        upload_response.accepted,
        upload_response.status.as_deref(),
        &upload_response.errors,
    )?;

    let validate_endpoint = format!(
        "{}/v2/catalog/publish/{request_id}/validate",
        registry_url.trim_end_matches('/')
    );
    let validate_request = RegistryPublishValidationHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run: false,
    };
    let validate_response: RegistryMutationHttpResponse = post_registry_json_parsed(
        &validate_endpoint,
        &validate_request,
        Some(actor),
        Some(&publisher),
    )?;
    ensure_publish_step_accepted(
        "validation",
        validate_response.accepted,
        validate_response.status.as_deref(),
        &validate_response.errors,
    )?;

    let readiness_endpoint = format!(
        "{}/v2/catalog/publish/{request_id}",
        registry_url.trim_end_matches('/')
    );
    let readiness = poll_registry_publish_status_until(
        &readiness_endpoint,
        Some(actor),
        &["approved", "published", "rejected"],
    )?;
    ensure_publish_status_not_rejected(&readiness)?;
    if readiness.status == "published" {
        return pretty_json(&readiness);
    }
    if readiness.status != "approved" {
        anyhow::bail!(
            "Registry publish request '{}' did not reach approved status after validation; current status is '{}'",
            readiness.request_id,
            readiness.status
        );
    }
    if !publish_status_action_available(&readiness, "approve") {
        anyhow::bail!(
            "Registry publish request '{}' reached status '{}', but the status contract does not advertise approve in governanceActions. Next step: {}",
            readiness.request_id,
            readiness.status,
            readiness
                .next_step
                .clone()
                .unwrap_or_else(|| "no next_step returned".to_string())
        );
    }
    if !auto_approve {
        return pretty_json(&readiness);
    }
    let readiness = run_publish_follow_up_stages_if_needed(
        registry_url,
        preview,
        readiness,
        actor,
        confirm_manual_review,
    )?;
    if readiness.approval_override_required
        && (approve_reason.as_deref().is_none_or(str::is_empty)
            || approve_reason_code.as_deref().is_none_or(str::is_empty))
    {
        let pending_stages = readiness
            .validation_stages
            .iter()
            .filter(|stage| !stage.status.eq_ignore_ascii_case("passed"))
            .map(|stage| format!("{} ({})", stage.key, stage.status))
            .collect::<Vec<_>>();
        let security_policy_hint = if publish_status_stage_requires_action(
            &readiness,
            "security_policy_review",
        ) {
            "; rerun with --confirm-manual-review to complete the local security/policy review path, or provide explicit approve override fields"
        } else {
            ""
        };
        anyhow::bail!(
            "Registry publish request '{}' still has non-passed follow-up stages [{}]; rerun with --approve-reason and --approve-reason-code <{}> or mark the remaining stages as passed first{}",
            readiness.request_id,
            pending_stages.join(", "),
            if readiness.approval_override_reason_codes.is_empty() {
                REGISTRY_APPROVE_OVERRIDE_REASON_CODES.join("|")
            } else {
                readiness.approval_override_reason_codes.join("|")
            },
            security_policy_hint
        );
    }

    let approve_endpoint = format!(
        "{}/v2/catalog/publish/{request_id}/approve",
        registry_url.trim_end_matches('/')
    );
    let approve_request = RegistryPublishDecisionHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run: false,
        reason: approve_reason,
        reason_code: approve_reason_code,
    };
    let approve_response: RegistryMutationHttpResponse = post_registry_json_parsed(
        &approve_endpoint,
        &approve_request,
        Some(actor),
        Some(&publisher),
    )?;
    ensure_publish_step_accepted(
        "approval",
        approve_response.accepted,
        approve_response.status.as_deref(),
        &approve_response.errors,
    )?;

    let status_endpoint = format!(
        "{}/v2/catalog/publish/{request_id}",
        registry_url.trim_end_matches('/')
    );
    let status = poll_registry_publish_status_until(
        &status_endpoint,
        Some(actor),
        &["published", "rejected"],
    )?;
    ensure_publish_status_not_rejected(&status)?;
    pretty_json(&status)
}

pub(crate) fn yank_via_registry_dry_run(
    registry_url: &str,
    preview: &ModuleYankDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> Result<String> {
    let endpoint = format!("{}/v2/catalog/yank", registry_url.trim_end_matches('/'));
    let request = build_yank_registry_request(preview, reason, reason_code);

    post_registry_json(&endpoint, &request)
}

pub(crate) fn yank_via_registry_live(
    registry_url: &str,
    preview: &ModuleYankDryRunPreview,
    actor: &str,
    reason: String,
    reason_code: String,
) -> Result<String> {
    let endpoint = format!("{}/v2/catalog/yank", registry_url.trim_end_matches('/'));
    let request = build_live_yank_registry_request(preview, Some(reason), Some(reason_code));
    let publisher = format!("publisher:{}", preview.slug);
    let response: RegistryMutationHttpResponse =
        post_registry_json_parsed(&endpoint, &request, Some(actor), Some(&publisher))?;
    if !response.accepted {
        anyhow::bail!(
            "Registry yank request was not accepted: {}",
            join_registry_errors(&response.errors)
        );
    }

    pretty_json(&response)
}

pub(crate) fn owner_transfer_via_registry_dry_run(
    registry_url: &str,
    preview: &ModuleOwnerTransferDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> Result<String> {
    let endpoint = format!(
        "{}/v2/catalog/owner-transfer",
        registry_url.trim_end_matches('/')
    );
    let request = build_owner_transfer_registry_request(preview, reason, reason_code);

    post_registry_json(&endpoint, &request)
}

pub(crate) fn owner_transfer_via_registry_live(
    registry_url: &str,
    preview: &ModuleOwnerTransferDryRunPreview,
    actor: &str,
    reason: String,
    reason_code: String,
) -> Result<String> {
    let endpoint = format!(
        "{}/v2/catalog/owner-transfer",
        registry_url.trim_end_matches('/')
    );
    let request =
        build_live_owner_transfer_registry_request(preview, Some(reason), Some(reason_code));
    let response: RegistryMutationHttpResponse =
        post_registry_json_parsed(&endpoint, &request, Some(actor), None)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry owner transfer request was not accepted: {}",
            join_registry_errors(&response.errors)
        );
    }

    pretty_json(&response)
}

pub(crate) fn validation_stage_via_registry_dry_run(
    registry_url: &str,
    preview: &ModuleValidationStageDryRunPreview,
) -> Result<String> {
    let endpoint = format!(
        "{}/v2/catalog/publish/{}/stages",
        registry_url.trim_end_matches('/'),
        preview.request_id
    );
    let request = build_validation_stage_registry_request(preview);

    post_registry_json(&endpoint, &request)
}

pub(crate) fn validation_stage_via_registry_live(
    registry_url: &str,
    preview: &ModuleValidationStageDryRunPreview,
    actor: &str,
) -> Result<String> {
    let endpoint = format!(
        "{}/v2/catalog/publish/{}/stages",
        registry_url.trim_end_matches('/'),
        preview.request_id
    );
    let request = build_live_validation_stage_registry_request(preview);
    let response: RegistryMutationHttpResponse =
        post_registry_json_parsed(&endpoint, &request, Some(actor), None)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry validation stage update was not accepted: {}",
            join_registry_errors(&response.errors)
        );
    }

    pretty_json(&response)
}

pub(crate) fn publish_request_governance_via_registry(
    registry_url: &str,
    request_id: &str,
    command_name: &str,
    action_key: &str,
    actor: Option<&str>,
    reason: Option<String>,
    reason_code: Option<String>,
    dry_run: bool,
) -> Result<String> {
    if !dry_run {
        let status = fetch_registry_publish_status_with_actor(registry_url, request_id, actor)?;
        if !publish_status_action_available(&status, action_key) {
            anyhow::bail!(
                "Registry publish request '{}' does not advertise '{}' in governanceActions. Current status: '{}'. Next step: {}",
                status.request_id,
                action_key,
                status.status,
                status
                    .next_step
                    .clone()
                    .unwrap_or_else(|| "no next_step returned".to_string())
            );
        }
    }

    let endpoint = format!(
        "{}/v2/catalog/publish/{request_id}/{}",
        registry_url.trim_end_matches('/'),
        command_name
    );
    let request = RegistryPublishDecisionHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run,
        reason,
        reason_code,
    };
    let response: RegistryMutationHttpResponse =
        post_registry_json_parsed(&endpoint, &request, actor, None)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry publish {} request was not accepted: {}",
            action_key,
            join_registry_errors(&response.errors)
        );
    }

    pretty_json(&response)
}

pub(crate) fn claim_remote_validation_stage_via_registry(
    registry_url: &str,
    runner_id: &str,
    runner_token: &str,
    supported_stages: &[&str],
) -> Result<Option<RegistryRunnerClaimHttpPayload>> {
    let endpoint = format!(
        "{}/v2/catalog/runner/claim",
        registry_url.trim_end_matches('/')
    );
    let request = RegistryRunnerClaimHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        runner_id: runner_id.to_string(),
        supported_stages: supported_stages
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    };
    let response: RegistryRunnerClaimHttpResponse =
        post_registry_json_with_runner_token_parsed(&endpoint, &request, runner_token)?;
    if !response.accepted {
        anyhow::bail!("Registry remote runner claim was not accepted");
    }

    Ok(response.claim)
}

pub(crate) fn heartbeat_remote_validation_stage_via_registry(
    registry_url: &str,
    claim_id: &str,
    runner_id: &str,
    runner_token: &str,
) -> Result<()> {
    let endpoint = format!(
        "{}/v2/catalog/runner/{claim_id}/heartbeat",
        registry_url.trim_end_matches('/')
    );
    let request = RegistryRunnerHeartbeatHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        runner_id: runner_id.to_string(),
    };
    let response: RegistryRunnerMutationHttpResponse =
        post_registry_json_with_runner_token_parsed(&endpoint, &request, runner_token)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry remote runner heartbeat for claim '{}' was not accepted",
            claim_id
        );
    }

    Ok(())
}

pub(crate) fn complete_remote_validation_stage_via_registry(
    registry_url: &str,
    claim_id: &str,
    runner_id: &str,
    runner_token: &str,
    detail: String,
    reason_code: String,
) -> Result<()> {
    let endpoint = format!(
        "{}/v2/catalog/runner/{claim_id}/complete",
        registry_url.trim_end_matches('/')
    );
    let request = RegistryRunnerCompletionHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        runner_id: runner_id.to_string(),
        detail: Some(detail),
        reason_code: Some(reason_code),
    };
    let response: RegistryRunnerMutationHttpResponse =
        post_registry_json_with_runner_token_parsed(&endpoint, &request, runner_token)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry remote runner completion for claim '{}' was not accepted",
            claim_id
        );
    }

    Ok(())
}

pub(crate) fn fail_remote_validation_stage_via_registry(
    registry_url: &str,
    claim_id: &str,
    runner_id: &str,
    runner_token: &str,
    detail: String,
    reason_code: String,
) -> Result<()> {
    let endpoint = format!(
        "{}/v2/catalog/runner/{claim_id}/fail",
        registry_url.trim_end_matches('/')
    );
    let request = RegistryRunnerCompletionHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        runner_id: runner_id.to_string(),
        detail: Some(detail),
        reason_code: Some(reason_code),
    };
    let response: RegistryRunnerMutationHttpResponse =
        post_registry_json_with_runner_token_parsed(&endpoint, &request, runner_token)?;
    if !response.accepted {
        anyhow::bail!(
            "Registry remote runner failure report for claim '{}' was not accepted",
            claim_id
        );
    }

    Ok(())
}

pub(crate) fn build_publish_registry_request(
    preview: &ModulePublishDryRunPreview,
) -> RegistryPublishHttpRequest {
    build_publish_registry_request_with_dry_run(preview, true)
}

pub(crate) fn build_live_publish_registry_request(
    preview: &ModulePublishDryRunPreview,
) -> RegistryPublishHttpRequest {
    build_publish_registry_request_with_dry_run(preview, false)
}

fn build_publish_registry_request_with_dry_run(
    preview: &ModulePublishDryRunPreview,
    dry_run: bool,
) -> RegistryPublishHttpRequest {
    RegistryPublishHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run,
        module: RegistryPublishModuleHttpRequest {
            slug: preview.slug.clone(),
            version: preview.version.clone(),
            crate_name: preview.crate_name.clone(),
            name: preview.module_name.clone(),
            description: preview.module_description.clone(),
            ownership: preview.ownership.clone(),
            trust_level: preview.trust_level.clone(),
            license: preview.license.clone(),
            entry_type: preview.module_entry_type.clone(),
            marketplace: RegistryPublishMarketplaceHttpRequest {
                category: preview.marketplace.category.clone(),
                tags: preview.marketplace.tags.clone(),
            },
            ui_packages: RegistryPublishUiPackagesHttpRequest {
                admin: preview.ui_packages.admin.as_ref().map(|ui| {
                    RegistryPublishUiPackageHttpRequest {
                        crate_name: ui.crate_name.clone(),
                    }
                }),
                storefront: preview.ui_packages.storefront.as_ref().map(|ui| {
                    RegistryPublishUiPackageHttpRequest {
                        crate_name: ui.crate_name.clone(),
                    }
                }),
            },
        },
    }
}

pub(crate) fn build_yank_registry_request(
    preview: &ModuleYankDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> RegistryYankHttpRequest {
    build_yank_registry_request_with_dry_run(preview, reason, reason_code, true)
}

pub(crate) fn build_live_yank_registry_request(
    preview: &ModuleYankDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> RegistryYankHttpRequest {
    build_yank_registry_request_with_dry_run(preview, reason, reason_code, false)
}

fn build_yank_registry_request_with_dry_run(
    preview: &ModuleYankDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
    dry_run: bool,
) -> RegistryYankHttpRequest {
    RegistryYankHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run,
        slug: preview.slug.clone(),
        version: preview.version.clone(),
        reason,
        reason_code,
    }
}

pub(crate) fn build_owner_transfer_registry_request(
    preview: &ModuleOwnerTransferDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> RegistryOwnerTransferHttpRequest {
    build_owner_transfer_registry_request_with_dry_run(preview, reason, reason_code, true)
}

pub(crate) fn build_live_owner_transfer_registry_request(
    preview: &ModuleOwnerTransferDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
) -> RegistryOwnerTransferHttpRequest {
    build_owner_transfer_registry_request_with_dry_run(preview, reason, reason_code, false)
}

fn build_owner_transfer_registry_request_with_dry_run(
    preview: &ModuleOwnerTransferDryRunPreview,
    reason: Option<String>,
    reason_code: Option<String>,
    dry_run: bool,
) -> RegistryOwnerTransferHttpRequest {
    RegistryOwnerTransferHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run,
        slug: preview.slug.clone(),
        new_owner_actor: preview.new_owner_actor.clone(),
        reason,
        reason_code,
    }
}

pub(crate) fn build_validation_stage_registry_request(
    preview: &ModuleValidationStageDryRunPreview,
) -> RegistryValidationStageHttpRequest {
    build_validation_stage_registry_request_with_dry_run(preview, true)
}

pub(crate) fn build_live_validation_stage_registry_request(
    preview: &ModuleValidationStageDryRunPreview,
) -> RegistryValidationStageHttpRequest {
    build_validation_stage_registry_request_with_dry_run(preview, false)
}

fn build_validation_stage_registry_request_with_dry_run(
    preview: &ModuleValidationStageDryRunPreview,
    dry_run: bool,
) -> RegistryValidationStageHttpRequest {
    RegistryValidationStageHttpRequest {
        schema_version: REGISTRY_MUTATION_SCHEMA_VERSION,
        dry_run,
        stage: preview.stage.clone(),
        status: preview.status.clone(),
        detail: preview.detail.clone(),
        reason_code: preview.reason_code.clone(),
        requeue: preview.requeue,
    }
}

pub(crate) fn post_registry_json<T>(endpoint: &str, payload: &T) -> Result<String>
where
    T: Serialize,
{
    let value: serde_json::Value = post_registry_json_parsed(endpoint, payload, None, None)?;
    pretty_json(&value)
}

pub(crate) fn post_registry_json_parsed<T, U>(
    endpoint: &str,
    payload: &T,
    actor: Option<&str>,
    publisher: Option<&str>,
) -> Result<U>
where
    T: Serialize,
    U: DeserializeOwned,
{
    let client = build_registry_http_client(endpoint)?;
    let mut request = client.post(endpoint).json(payload);
    if let Some(actor) = actor {
        request = request.header("x-rustok-actor", actor);
    }
    if let Some(publisher) = publisher {
        request = request.header("x-rustok-publisher", publisher);
    }
    let response = request
        .send()
        .with_context(|| format!("Failed to call registry endpoint {endpoint}"))?;
    parse_registry_json_response(endpoint, response)
}

pub(crate) fn post_registry_json_with_runner_token_parsed<T, U>(
    endpoint: &str,
    payload: &T,
    runner_token: &str,
) -> Result<U>
where
    T: Serialize,
    U: DeserializeOwned,
{
    let client = build_registry_http_client(endpoint)?;
    let response = client
        .post(endpoint)
        .header("x-rustok-runner-token", runner_token)
        .json(payload)
        .send()
        .with_context(|| format!("Failed to call registry runner endpoint {endpoint}"))?;
    parse_registry_json_response(endpoint, response)
}

pub(crate) fn put_registry_bytes_parsed<U>(
    endpoint: &str,
    payload: &[u8],
    content_type: &str,
    actor: Option<&str>,
    publisher: Option<&str>,
) -> Result<U>
where
    U: DeserializeOwned,
{
    let client = build_registry_http_client(endpoint)?;
    let mut request = client
        .put(endpoint)
        .header("content-type", content_type)
        .body(payload.to_vec());
    if let Some(actor) = actor {
        request = request.header("x-rustok-actor", actor);
    }
    if let Some(publisher) = publisher {
        request = request.header("x-rustok-publisher", publisher);
    }
    let response = request
        .send()
        .with_context(|| format!("Failed to call registry upload endpoint {endpoint}"))?;
    parse_registry_json_response(endpoint, response)
}

pub(crate) fn get_registry_json_parsed<U>(endpoint: &str, actor: Option<&str>) -> Result<U>
where
    U: DeserializeOwned,
{
    let client = build_registry_http_client(endpoint)?;
    let mut request = client.get(endpoint);
    if let Some(actor) = actor {
        request = request.header("x-rustok-actor", actor);
    }
    let response = request
        .send()
        .with_context(|| format!("Failed to call registry endpoint {endpoint}"))?;
    parse_registry_json_response(endpoint, response)
}

fn parse_registry_json_response<U>(
    endpoint: &str,
    response: reqwest::blocking::Response,
) -> Result<U>
where
    U: DeserializeOwned,
{
    let response = response
        .error_for_status()
        .with_context(|| format!("Registry endpoint {endpoint} returned an error status"))?;
    response
        .json::<U>()
        .with_context(|| format!("Failed to parse registry response from {endpoint}"))
}

pub(crate) fn build_publish_artifact_bytes(
    preview: &ModulePublishDryRunPreview,
) -> Result<Vec<u8>> {
    let package_manifest_path = workspace_root().join(&preview.package_manifest_path);
    let package_manifest = fs::read_to_string(&package_manifest_path).with_context(|| {
        format!(
            "Failed to read publish package manifest {}",
            package_manifest_path.display()
        )
    })?;
    let module_root = package_manifest_path.parent().with_context(|| {
        format!(
            "Failed to resolve module root for {}",
            package_manifest_path.display()
        )
    })?;
    let crate_manifest_path = module_root.join("Cargo.toml");
    let crate_manifest = fs::read_to_string(&crate_manifest_path).with_context(|| {
        format!(
            "Failed to read crate manifest {}",
            crate_manifest_path.display()
        )
    })?;

    let admin_manifest = preview
        .ui_packages
        .admin
        .as_ref()
        .map(|ui| {
            let path = workspace_root().join(&ui.manifest_path);
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read admin UI manifest {}", path.display()))?;
            Ok::<String, anyhow::Error>(content)
        })
        .transpose()?;
    let storefront_manifest = preview
        .ui_packages
        .storefront
        .as_ref()
        .map(|ui| {
            let path = workspace_root().join(&ui.manifest_path);
            let content = fs::read_to_string(&path).with_context(|| {
                format!("Failed to read storefront UI manifest {}", path.display())
            })?;
            Ok::<String, anyhow::Error>(content)
        })
        .transpose()?;

    let payload = serde_json::json!({
        "schema_version": REGISTRY_MUTATION_SCHEMA_VERSION,
        "artifact_type": "rustok-module-publish-bundle",
        "module": preview,
        "files": {
            "rustok-module.toml": package_manifest,
            "Cargo.toml": crate_manifest,
            "admin/Cargo.toml": admin_manifest,
            "storefront/Cargo.toml": storefront_manifest,
        }
    });

    serde_json::to_vec_pretty(&payload).context("Failed to serialize publish artifact bundle")
}

pub(crate) fn run_publish_follow_up_stages_if_needed(
    registry_url: &str,
    preview: &ModulePublishDryRunPreview,
    mut status: RegistryPublishStatusHttpResponse,
    actor: &str,
    confirm_manual_review: bool,
) -> Result<RegistryPublishStatusHttpResponse> {
    let executable_stages = ["compile_smoke", "targeted_tests"];
    for stage in executable_stages {
        if publish_status_stage_requires_action(&status, stage) {
            let plan = build_module_validation_stage_run_preview(
                preview,
                &status.request_id,
                stage,
                None,
            )?;
            run_validation_stage_plan_via_registry(registry_url, &plan, actor)?;
            status = fetch_registry_publish_status_with_actor(
                registry_url,
                &status.request_id,
                Some(actor),
            )?;
            ensure_publish_status_not_rejected(&status)?;
        }
    }

    if publish_status_stage_requires_action(&status, "security_policy_review")
        && confirm_manual_review
    {
        let plan = build_module_validation_stage_run_preview(
            preview,
            &status.request_id,
            "security_policy_review",
            None,
        )?;
        run_validation_stage_plan_via_registry(registry_url, &plan, actor)?;
        status = fetch_registry_publish_status_with_actor(
            registry_url,
            &status.request_id,
            Some(actor),
        )?;
        ensure_publish_status_not_rejected(&status)?;
    }

    Ok(status)
}

pub(crate) fn fetch_registry_publish_status_with_actor(
    registry_url: &str,
    request_id: &str,
    actor: Option<&str>,
) -> Result<RegistryPublishStatusHttpResponse> {
    let endpoint = format!(
        "{}/v2/catalog/publish/{request_id}",
        registry_url.trim_end_matches('/')
    );
    get_registry_json_parsed(&endpoint, actor)
}

pub(crate) fn publish_status_stage_requires_action(
    status: &RegistryPublishStatusHttpResponse,
    stage_key: &str,
) -> bool {
    status
        .validation_stages
        .iter()
        .find(|stage| stage.key.eq_ignore_ascii_case(stage_key))
        .is_some_and(|stage| !stage.status.eq_ignore_ascii_case("passed"))
}

pub(crate) fn publish_status_action_available(
    status: &RegistryPublishStatusHttpResponse,
    action_key: &str,
) -> bool {
    status
        .governance_actions
        .iter()
        .any(|action| action.key.eq_ignore_ascii_case(action_key))
}

pub(crate) fn poll_registry_publish_status_until(
    endpoint: &str,
    actor: Option<&str>,
    desired_statuses: &[&str],
) -> Result<RegistryPublishStatusHttpResponse> {
    let mut last_status = None;

    for attempt in 0..10 {
        let status: RegistryPublishStatusHttpResponse = get_registry_json_parsed(endpoint, actor)?;
        let terminal = desired_statuses
            .iter()
            .any(|candidate| status.status == *candidate);
        last_status = Some(status);
        if terminal {
            break;
        }
        if attempt < 9 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    last_status.with_context(|| format!("Registry status endpoint {endpoint} returned no payload"))
}

pub(crate) fn ensure_publish_step_accepted(
    step: &str,
    accepted: bool,
    status: Option<&str>,
    errors: &[String],
) -> Result<()> {
    if accepted && status != Some("rejected") {
        return Ok(());
    }

    let status_label = status.unwrap_or("unknown");
    anyhow::bail!(
        "Registry publish {step} finished with status '{status_label}': {}",
        join_registry_errors(errors)
    );
}

pub(crate) fn ensure_publish_status_not_rejected(
    status: &RegistryPublishStatusHttpResponse,
) -> Result<()> {
    if status.accepted && status.status != "rejected" {
        return Ok(());
    }

    anyhow::bail!(
        "Registry publish request '{}' was rejected: {}",
        status.request_id,
        join_registry_errors(&status.errors)
    );
}

pub(crate) fn join_registry_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "no error details returned".to_string()
    } else {
        errors.join("; ")
    }
}

pub(crate) fn pretty_json<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).context("Failed to pretty-print registry payload")
}

fn build_registry_http_client(endpoint: &str) -> Result<Client> {
    let mut builder = Client::builder().timeout(std::time::Duration::from_secs(15));
    if registry_endpoint_uses_loopback(endpoint) {
        builder = builder.no_proxy();
    }

    builder
        .build()
        .context("Failed to build registry HTTP client")
}

pub(crate) fn registry_endpoint_uses_loopback(endpoint: &str) -> bool {
    Url::parse(endpoint)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_string()))
        .is_some_and(|host| {
            host.eq_ignore_ascii_case("localhost")
                || host
                    .trim_matches(|ch| ch == '[' || ch == ']')
                    .parse::<IpAddr>()
                    .map(|address| address.is_loopback())
                    .unwrap_or(false)
        })
}
