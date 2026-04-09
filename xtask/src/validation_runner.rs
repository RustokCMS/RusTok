use super::*;

pub(crate) fn build_module_test_plan(
    preview: &ModulePublishDryRunPreview,
) -> ModuleTestPlanPreview {
    let mut commands = vec![ModuleCommandPreview {
        label: "module validate".to_string(),
        argv: vec![
            "cargo".to_string(),
            "run".to_string(),
            "-p".to_string(),
            "xtask".to_string(),
            "--".to_string(),
            "module".to_string(),
            "validate".to_string(),
            preview.slug.clone(),
        ],
    }];

    commands.push(package_check_command("module crate", &preview.crate_name));

    if let Some(admin) = preview.ui_packages.admin.as_ref() {
        commands.push(package_check_command("admin ui crate", &admin.crate_name));
    }

    if let Some(storefront) = preview.ui_packages.storefront.as_ref() {
        commands.push(package_check_command(
            "storefront ui crate",
            &storefront.crate_name,
        ));
    }

    ModuleTestPlanPreview {
        slug: preview.slug.clone(),
        version: preview.version.clone(),
        commands,
    }
}

pub(crate) fn build_module_validation_stage_run_preview(
    preview: &ModulePublishDryRunPreview,
    request_id: &str,
    stage: &str,
    detail_override: Option<String>,
) -> Result<ModuleValidationStageRunPreview> {
    let normalized_stage = normalize_executable_validation_stage(stage)?;
    let (commands, requires_manual_confirmation) = match normalized_stage {
        "compile_smoke" => (build_compile_smoke_commands(preview), false),
        "targeted_tests" => (build_module_test_plan(preview).commands, false),
        "security_policy_review" => (build_security_policy_review_commands(preview), true),
        _ => unreachable!(),
    };
    let running_detail = detail_override.unwrap_or_else(|| {
        executable_validation_stage_running_detail(normalized_stage, &preview.slug)
    });

    Ok(ModuleValidationStageRunPreview {
        action: "validation_stage_run".to_string(),
        slug: preview.slug.clone(),
        request_id: request_id.to_string(),
        stage: normalized_stage.to_string(),
        requires_manual_confirmation,
        running_detail,
        success_detail: executable_validation_stage_success_detail(normalized_stage, &preview.slug),
        failure_detail_prefix: executable_validation_stage_failure_prefix(normalized_stage),
        commands,
    })
}

pub(crate) fn normalize_executable_validation_stage(stage: &str) -> Result<&'static str> {
    let stage = stage.trim().to_ascii_lowercase();
    match stage.as_str() {
        "compile_smoke" => Ok("compile_smoke"),
        "targeted_tests" => Ok("targeted_tests"),
        "security_policy_review" => Ok("security_policy_review"),
        _ => anyhow::bail!(
            "module stage-run stage '{}' is not supported; expected one of compile_smoke, targeted_tests, or security_policy_review",
            stage
        ),
    }
}

fn build_compile_smoke_commands(preview: &ModulePublishDryRunPreview) -> Vec<ModuleCommandPreview> {
    let mut commands = vec![package_check_command("module crate", &preview.crate_name)];

    if let Some(admin) = preview.ui_packages.admin.as_ref() {
        commands.push(package_check_command("admin ui crate", &admin.crate_name));
    }

    if let Some(storefront) = preview.ui_packages.storefront.as_ref() {
        commands.push(package_check_command(
            "storefront ui crate",
            &storefront.crate_name,
        ));
    }

    commands
}

fn build_security_policy_review_commands(
    preview: &ModulePublishDryRunPreview,
) -> Vec<ModuleCommandPreview> {
    vec![
        ModuleCommandPreview {
            label: "module validate".to_string(),
            argv: vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "xtask".to_string(),
                "--".to_string(),
                "module".to_string(),
                "validate".to_string(),
                preview.slug.clone(),
            ],
        },
        ModuleCommandPreview {
            label: "module test dry-run".to_string(),
            argv: vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "xtask".to_string(),
                "--".to_string(),
                "module".to_string(),
                "test".to_string(),
                preview.slug.clone(),
                "--dry-run".to_string(),
            ],
        },
    ]
}

pub(crate) fn executable_validation_stage_running_detail(stage: &str, slug: &str) -> String {
    match stage {
        "compile_smoke" => format!("Running local compile smoke checks for module '{slug}'."),
        "targeted_tests" => format!("Running local targeted tests for module '{slug}'."),
        "security_policy_review" => {
            format!("Running local security/policy review preflight for module '{slug}'.")
        }
        _ => format!("Running local validation stage '{stage}' for module '{slug}'."),
    }
}

pub(crate) fn executable_validation_stage_success_detail(stage: &str, slug: &str) -> String {
    match stage {
        "compile_smoke" => {
            format!("Local compile smoke checks passed for module '{slug}'.")
        }
        "targeted_tests" => {
            format!("Local targeted tests completed successfully for module '{slug}'.")
        }
        "security_policy_review" => format!(
            "Local security/policy preflight completed and manual review was confirmed for module '{slug}'."
        ),
        _ => format!("Local validation stage '{stage}' completed successfully for '{slug}'."),
    }
}

pub(crate) fn executable_validation_stage_failure_prefix(stage: &str) -> String {
    match stage {
        "compile_smoke" => "Local compile smoke failed".to_string(),
        "targeted_tests" => "Local targeted tests failed".to_string(),
        "security_policy_review" => "Local security/policy review preflight failed".to_string(),
        _ => format!("Local validation stage '{stage}' failed"),
    }
}

pub(crate) fn validation_stage_success_reason_code(stage: &str) -> &'static str {
    match stage {
        "security_policy_review" => "manual_review_complete",
        _ => "local_runner_passed",
    }
}

pub(crate) fn validation_stage_failure_reason_code(stage: &str) -> &'static str {
    match stage {
        "compile_smoke" => "build_failure",
        "targeted_tests" => "test_failure",
        "security_policy_review" => "policy_preflight_failed",
        _ => "other",
    }
}

pub(crate) fn validation_stage_status_requires_reason_code(status: &str) -> bool {
    matches!(status, "passed" | "failed" | "blocked")
}

fn package_check_command(label: &str, package: &str) -> ModuleCommandPreview {
    ModuleCommandPreview {
        label: label.to_string(),
        argv: vec![
            "cargo".to_string(),
            "check".to_string(),
            "-p".to_string(),
            package.to_string(),
        ],
    }
}

pub(crate) fn run_command(command: &ModuleCommandPreview) -> Result<()> {
    let program = command
        .argv
        .first()
        .with_context(|| format!("Command '{}' has empty argv", command.label))?;
    let status = Command::new(program)
        .args(&command.argv[1..])
        .current_dir(workspace_root())
        .status()
        .with_context(|| format!("Failed to launch '{}'", command.argv.join(" ")))?;
    if !status.success() {
        anyhow::bail!(
            "Command '{}' failed with status {}",
            command.argv.join(" "),
            status
        );
    }
    Ok(())
}

pub(crate) fn run_validation_stage_plan_via_registry(
    registry_url: &str,
    plan: &ModuleValidationStageRunPreview,
    actor: &str,
) -> Result<()> {
    let running_preview = ModuleValidationStageDryRunPreview {
        action: "validation_stage".to_string(),
        request_id: plan.request_id.clone(),
        stage: plan.stage.clone(),
        status: "running".to_string(),
        detail: Some(plan.running_detail.clone()),
        reason_code: None,
        requeue: false,
    };
    validation_stage_via_registry_live(registry_url, &running_preview, actor)?;

    for command in &plan.commands {
        println!("  > {}", command.argv.join(" "));
        if let Err(error) = run_command(command) {
            let failed_preview = ModuleValidationStageDryRunPreview {
                action: "validation_stage".to_string(),
                request_id: plan.request_id.clone(),
                stage: plan.stage.clone(),
                status: "failed".to_string(),
                detail: Some(format!("{}: {error}", plan.failure_detail_prefix)),
                reason_code: Some(validation_stage_failure_reason_code(&plan.stage).to_string()),
                requeue: false,
            };
            let report_error =
                validation_stage_via_registry_live(registry_url, &failed_preview, actor)
                    .err()
                    .map(|report_error| {
                        format!("; failed to persist failed stage status: {report_error}")
                    })
                    .unwrap_or_default();
            anyhow::bail!("{error}{report_error}");
        }
    }

    let passed_preview = ModuleValidationStageDryRunPreview {
        action: "validation_stage".to_string(),
        request_id: plan.request_id.clone(),
        stage: plan.stage.clone(),
        status: "passed".to_string(),
        detail: Some(plan.success_detail.clone()),
        reason_code: Some(validation_stage_success_reason_code(&plan.stage).to_string()),
        requeue: false,
    };
    validation_stage_via_registry_live(registry_url, &passed_preview, actor)?;
    Ok(())
}

pub(crate) fn run_remote_validation_runner_once(
    registry_url: &str,
    runner_id: &str,
    runner_token: &str,
    manifest_path: &Path,
    manifest: &Manifest,
    workspace_manifest: &TomlValue,
    confirm_manual_review: bool,
    heartbeat_interval_ms: u64,
) -> Result<bool> {
    let supported_stages = supported_remote_runner_stages(confirm_manual_review);
    let Some(claim) = claim_remote_validation_stage_via_registry(
        registry_url,
        runner_id,
        runner_token,
        &supported_stages,
    )?
    else {
        return Ok(false);
    };

    println!(
        "Claimed remote validation stage '{}' for {}@{} ({})",
        claim.stage_key, claim.slug, claim.version, claim.request_id
    );
    if !claim.runnable {
        anyhow::bail!(
            "Remote runner claim '{}' for stage '{}' is marked non-runnable",
            claim.claim_id,
            claim.stage_key
        );
    }

    let stop_heartbeats = Arc::new(AtomicBool::new(false));
    let heartbeat_stop = Arc::clone(&stop_heartbeats);
    let heartbeat_registry_url = registry_url.to_string();
    let heartbeat_runner_id = runner_id.to_string();
    let heartbeat_runner_token = runner_token.to_string();
    let heartbeat_claim_id = claim.claim_id.clone();
    let heartbeat_handle = thread::spawn(move || {
        while !heartbeat_stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(heartbeat_interval_ms));
            if heartbeat_stop.load(Ordering::Relaxed) {
                break;
            }
            if let Err(error) = heartbeat_remote_validation_stage_via_registry(
                &heartbeat_registry_url,
                &heartbeat_claim_id,
                &heartbeat_runner_id,
                &heartbeat_runner_token,
            ) {
                eprintln!(
                    "Remote validation runner heartbeat failed for claim '{}': {}",
                    heartbeat_claim_id, error
                );
                break;
            }
        }
    });

    let result =
        execute_remote_validation_claim(manifest_path, manifest, workspace_manifest, &claim)
            .and_then(|plan| {
                for command in &plan.commands {
                    println!("  > {}", command.argv.join(" "));
                    run_command(command)
                        .with_context(|| format!("remote runner stage '{}' failed", plan.stage))?;
                }

                let reason_code = claim
                    .suggested_pass_reason_code
                    .clone()
                    .filter(|value| {
                        claim.allowed_terminal_reason_codes.is_empty()
                            || claim
                                .allowed_terminal_reason_codes
                                .iter()
                                .any(|candidate| candidate.eq_ignore_ascii_case(value))
                    })
                    .unwrap_or_else(|| {
                        validation_stage_success_reason_code(&claim.stage_key).to_string()
                    });
                complete_remote_validation_stage_via_registry(
                    registry_url,
                    &claim.claim_id,
                    runner_id,
                    runner_token,
                    plan.success_detail,
                    reason_code,
                )
            });

    stop_heartbeats.store(true, Ordering::Relaxed);
    let _ = heartbeat_handle.join();

    if let Err(error) = result {
        let reason_code = claim
            .suggested_failure_reason_code
            .clone()
            .filter(|value| {
                claim.allowed_terminal_reason_codes.is_empty()
                    || claim
                        .allowed_terminal_reason_codes
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(value))
            })
            .unwrap_or_else(|| validation_stage_failure_reason_code(&claim.stage_key).to_string());
        let detail = format!(
            "{}: {}",
            executable_validation_stage_failure_prefix(&claim.stage_key),
            error
        );
        let report_error = fail_remote_validation_stage_via_registry(
            registry_url,
            &claim.claim_id,
            runner_id,
            runner_token,
            detail,
            reason_code,
        )
        .err()
        .map(|report_error| format!("; failed to report remote stage failure: {report_error}"))
        .unwrap_or_default();
        anyhow::bail!("{error}{report_error}");
    }

    Ok(true)
}

fn execute_remote_validation_claim(
    manifest_path: &Path,
    manifest: &Manifest,
    workspace_manifest: &TomlValue,
    claim: &RegistryRunnerClaimHttpPayload,
) -> Result<ModuleValidationStageRunPreview> {
    let spec = manifest.modules.get(&claim.slug).with_context(|| {
        format!(
            "Remote runner claimed unknown local module slug '{}'",
            claim.slug
        )
    })?;
    let preview =
        build_module_publish_preview(manifest_path, &claim.slug, spec, workspace_manifest)?;
    if preview.version != claim.version {
        anyhow::bail!(
            "Remote runner claimed '{}@{}', but local workspace resolves version '{}'",
            claim.slug,
            claim.version,
            preview.version
        );
    }

    build_module_validation_stage_run_preview(
        &preview,
        &claim.request_id,
        &claim.stage_key,
        Some(remote_claim_running_detail(claim)),
    )
}

fn remote_claim_running_detail(claim: &RegistryRunnerClaimHttpPayload) -> String {
    format!(
        "Remote runner '{}' executing '{}' for module '{}' from {}.",
        claim.execution_mode, claim.stage_key, claim.slug, claim.artifact_url
    )
}
