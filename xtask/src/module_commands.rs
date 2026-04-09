use super::*;

pub(crate) fn module_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_module_usage();
        anyhow::bail!("Missing module subcommand");
    }

    match args[0].as_str() {
        "validate" => module_validate_command(&args[1..]),
        "test" => module_test_command(&args[1..]),
        "stage-run" => module_stage_run_command(&args[1..]),
        "runner" => module_runner_command(&args[1..]),
        "publish" => module_publish_command(&args[1..]),
        "request-changes" => module_request_changes_command(&args[1..]),
        "hold" => module_hold_command(&args[1..]),
        "resume" => module_resume_command(&args[1..]),
        "stage" => module_stage_command(&args[1..]),
        "owner-transfer" => module_owner_transfer_command(&args[1..]),
        "yank" => module_yank_command(&args[1..]),
        other => {
            print_module_usage();
            anyhow::bail!("Unknown module subcommand: {other}");
        }
    }
}

pub(crate) fn print_module_usage() {
    println!("Usage:");
    println!("  cargo xtask module validate [slug]");
    println!("  cargo xtask module test <slug> [--dry-run]");
    println!(
        "  cargo xtask module stage-run <slug> <request-id> <stage> [--dry-run] [--registry-url <url>] [--actor <actor>] [--detail <text>] [--confirm-manual-review]"
    );
    println!(
        "  cargo xtask module runner <runner-id> [--dry-run] [--once] [--registry-url <url>] [--runner-token <token>] [--poll-interval-ms <ms>] [--heartbeat-interval-ms <ms>] [--confirm-manual-review]"
    );
    println!(
        "  cargo xtask module publish <slug> [--dry-run] [--registry-url <url>] [--actor <actor>] [--auto-approve] [--approve-reason <text>] [--approve-reason-code <code>] [--confirm-manual-review]"
    );
    println!(
        "  cargo xtask module request-changes <request-id> [--dry-run] [--registry-url <url>] --actor <actor> --reason <text> --reason-code <code>"
    );
    println!(
        "  cargo xtask module hold <request-id> [--dry-run] [--registry-url <url>] --actor <actor> --reason <text> --reason-code <code>"
    );
    println!(
        "  cargo xtask module resume <request-id> [--dry-run] [--registry-url <url>] --actor <actor> --reason <text> --reason-code <code>"
    );
    println!(
        "  cargo xtask module stage <request-id> <stage> <status> [--dry-run] [--detail <text>] [--reason-code <code>] [--registry-url <url>] [--actor <actor>] [--requeue]"
    );
    println!(
        "  cargo xtask module owner-transfer <slug> <new-owner-actor> [--dry-run] [--reason <text>] [--reason-code <code>] [--registry-url <url>] [--actor <actor>]"
    );
    println!(
        "  cargo xtask module yank <slug> <version> [--dry-run] [--reason <text>] [--reason-code <code>] [--registry-url <url>] [--actor <actor>]"
    );
}

pub(crate) fn module_publish_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_module_usage();
        anyhow::bail!("module publish requires a module slug");
    }

    let slug = args[0].as_str();
    let dry_run = args.iter().skip(1).any(|arg| arg == "--dry-run");
    let actor = actor_argument(&args[1..]);
    let auto_approve = auto_approve_argument(&args[1..]);
    let approve_reason = approve_reason_argument(&args[1..]);
    let approve_reason_code = approve_reason_code_argument(&args[1..])?;
    let confirm_manual_review = manual_review_confirmation_argument(&args[1..]);

    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let spec = manifest
        .modules
        .get(slug)
        .with_context(|| format!("Unknown module slug '{slug}'"))?;
    let preview = build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
    let registry_url = registry_url_argument(&args[1..]);
    if dry_run {
        if let Some(registry_url) = registry_url {
            let payload = publish_via_registry_dry_run(&registry_url, &preview)?;
            println!("{payload}");
        } else {
            let payload = serde_json::to_string_pretty(&preview)?;
            println!("{payload}");
        }
    } else {
        let registry_url = registry_url.with_context(|| {
            "Live module publish requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
        })?;
        let actor = actor.with_context(|| "Live module publish requires --actor <actor>")?;
        let payload = publish_via_registry_live(
            &registry_url,
            &preview,
            &actor,
            auto_approve,
            approve_reason,
            approve_reason_code,
            confirm_manual_review,
        )?;
        println!("{payload}");
    }

    Ok(())
}

pub(crate) fn module_test_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_module_usage();
        anyhow::bail!("module test requires a module slug");
    }

    let slug = args[0].as_str();
    let dry_run = args.iter().skip(1).any(|arg| arg == "--dry-run");
    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let spec = manifest
        .modules
        .get(slug)
        .with_context(|| format!("Unknown module slug '{slug}'"))?;
    let preview = build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
    let plan = build_module_test_plan(&preview);

    if dry_run {
        let payload = serde_json::to_string_pretty(&plan)?;
        println!("{payload}");
        return Ok(());
    }

    println!(
        "Running local module smoke checks for {slug} v{}...",
        preview.version
    );
    for command in &plan.commands {
        println!("  > {}", command.argv.join(" "));
        run_command(command)?;
    }

    Ok(())
}

pub(crate) fn module_stage_run_command(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        print_module_usage();
        anyhow::bail!("module stage-run requires a module slug, request id, and stage key");
    }

    let slug = args[0].trim();
    let request_id = args[1].trim();
    let stage = args[2].trim().to_ascii_lowercase();
    if slug.is_empty() {
        anyhow::bail!("module stage-run requires a non-empty module slug");
    }
    if request_id.is_empty() {
        anyhow::bail!("module stage-run requires a non-empty request id");
    }

    let dry_run = args.iter().skip(3).any(|arg| arg == "--dry-run");
    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let spec = manifest
        .modules
        .get(slug)
        .with_context(|| format!("Unknown module slug '{slug}'"))?;
    let preview = build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
    let plan = build_module_validation_stage_run_preview(
        &preview,
        request_id,
        &stage,
        detail_argument(&args[3..]),
    )?;
    let confirm_manual_review = manual_review_confirmation_argument(&args[3..]);
    let registry_url = registry_url_argument(&args[3..]);
    let actor = actor_argument(&args[3..]);

    if dry_run {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    let registry_url = registry_url.with_context(|| {
        "Live module stage-run requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
    })?;
    let actor = actor.with_context(|| "Live module stage-run requires --actor <actor>")?;
    if plan.requires_manual_confirmation && !confirm_manual_review {
        anyhow::bail!(
            "module stage-run '{}' requires --confirm-manual-review before persisting a passed security/policy review",
            plan.stage
        );
    }
    run_validation_stage_plan_via_registry(&registry_url, &plan, &actor)?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

pub(crate) fn module_runner_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_module_usage();
        anyhow::bail!("module runner requires a non-empty runner id");
    }

    let runner_id = args[0].trim();
    if runner_id.is_empty() {
        anyhow::bail!("module runner requires a non-empty runner id");
    }

    let dry_run = args.iter().skip(1).any(|arg| arg == "--dry-run");
    let once = args.iter().skip(1).any(|arg| arg == "--once");
    let confirm_manual_review = manual_review_confirmation_argument(&args[1..]);
    let poll_interval_ms =
        positive_u64_argument(&args[1..], "--poll-interval-ms", "module runner")?
            .unwrap_or(DEFAULT_REMOTE_RUNNER_POLL_INTERVAL_MS);
    let heartbeat_interval_ms =
        positive_u64_argument(&args[1..], "--heartbeat-interval-ms", "module runner")?
            .unwrap_or(DEFAULT_REMOTE_RUNNER_HEARTBEAT_INTERVAL_MS);
    let supported_stages = supported_remote_runner_stages(confirm_manual_review);
    let preview = ModuleRunnerPreview {
        action: "remote_validation_runner".to_string(),
        runner_id: runner_id.to_string(),
        supported_stages: supported_stages
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        poll_interval_ms,
        heartbeat_interval_ms,
        once,
        confirm_manual_review,
    };

    if dry_run {
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(());
    }

    let registry_url = registry_url_argument(&args[1..]).with_context(|| {
        "Live module runner requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
    })?;
    let runner_token = runner_token_argument(&args[1..]).with_context(|| {
        format!("Live module runner requires --runner-token or {REMOTE_RUNNER_TOKEN_ENV}")
    })?;

    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;

    if once {
        let processed = run_remote_validation_runner_once(
            &registry_url,
            runner_id,
            &runner_token,
            &manifest_path,
            &manifest,
            &workspace_manifest,
            confirm_manual_review,
            heartbeat_interval_ms,
        )?;
        if !processed {
            println!("No queued remote validation stages were available.");
        }
        return Ok(());
    }

    println!(
        "Starting remote validation runner '{runner_id}' against {}...",
        registry_url
    );
    loop {
        let processed = run_remote_validation_runner_once(
            &registry_url,
            runner_id,
            &runner_token,
            &manifest_path,
            &manifest,
            &workspace_manifest,
            confirm_manual_review,
            heartbeat_interval_ms,
        )?;
        if !processed {
            thread::sleep(Duration::from_millis(poll_interval_ms));
        }
    }
}

pub(crate) fn module_yank_command(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        print_module_usage();
        anyhow::bail!("module yank requires a module slug and version");
    }

    let slug = args[0].as_str();
    let version = args[1].trim();
    let dry_run = args.iter().skip(2).any(|arg| arg == "--dry-run");
    Version::parse(version)
        .with_context(|| format!("module yank version '{version}' is not valid semver"))?;

    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let spec = manifest
        .modules
        .get(slug)
        .with_context(|| format!("Unknown module slug '{slug}'"))?;
    let preview = build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
    let payload = ModuleYankDryRunPreview {
        action: "yank".to_string(),
        slug: slug.to_string(),
        version: version.to_string(),
        crate_name: preview.crate_name.clone(),
        current_local_version: preview.version.clone(),
        matches_local_version: preview.version == version,
        package_manifest_path: preview.package_manifest_path,
    };
    let registry_url = registry_url_argument(&args[2..]);
    let actor = actor_argument(&args[2..]);
    let reason = reason_argument(&args[2..]);
    let reason_code = reason_code_argument(&args[2..])?;
    if dry_run {
        if let Some(registry_url) = registry_url {
            let remote_payload =
                yank_via_registry_dry_run(&registry_url, &payload, reason, reason_code)?;
            println!("{remote_payload}");
        } else {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
    } else {
        let registry_url = registry_url.with_context(|| {
            "Live module yank requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
        })?;
        let actor = actor.with_context(|| "Live module yank requires --actor <actor>")?;
        let reason = reason.with_context(|| "Live module yank requires --reason <text>")?;
        let reason_code = reason_code.with_context(|| {
            format!(
                "Live module yank requires --reason-code <{}>",
                REGISTRY_YANK_REASON_CODES.join("|")
            )
        })?;
        let remote_payload =
            yank_via_registry_live(&registry_url, &payload, &actor, reason, reason_code)?;
        println!("{remote_payload}");
    }

    Ok(())
}

pub(crate) fn module_request_changes_command(args: &[String]) -> Result<()> {
    module_publish_governance_command(
        args,
        "request-changes",
        "request_changes",
        REGISTRY_REQUEST_CHANGES_REASON_CODES,
    )
}

pub(crate) fn module_hold_command(args: &[String]) -> Result<()> {
    module_publish_governance_command(args, "hold", "hold", REGISTRY_HOLD_REASON_CODES)
}

pub(crate) fn module_resume_command(args: &[String]) -> Result<()> {
    module_publish_governance_command(args, "resume", "resume", REGISTRY_RESUME_REASON_CODES)
}

pub(crate) fn module_publish_governance_command(
    args: &[String],
    command_name: &str,
    action_key: &str,
    allowed_reason_codes: &[&str],
) -> Result<()> {
    if args.is_empty() {
        print_module_usage();
        anyhow::bail!("module {command_name} requires a request id");
    }

    let request_id = args[0].trim();
    if request_id.is_empty() {
        anyhow::bail!("module {command_name} requires a non-empty request id");
    }

    let dry_run = args.iter().skip(1).any(|arg| arg == "--dry-run");
    let actor = actor_argument(&args[1..]);
    let reason = reason_argument(&args[1..]);
    let reason_code =
        governance_reason_code_argument(&args[1..], command_name, allowed_reason_codes)?;
    let preview = ModulePublishGovernanceDryRunPreview {
        action: action_key.to_string(),
        request_id: request_id.to_string(),
        actor: actor.clone(),
        reason: reason.clone(),
        reason_code: reason_code.clone(),
    };
    let registry_url = registry_url_argument(&args[1..]);

    if dry_run {
        if let Some(registry_url) = registry_url {
            let payload = publish_request_governance_via_registry(
                &registry_url,
                request_id,
                command_name,
                action_key,
                actor.as_deref(),
                reason,
                reason_code,
                true,
            )?;
            println!("{payload}");
        } else {
            println!("{}", serde_json::to_string_pretty(&preview)?);
        }
        return Ok(());
    }

    let registry_url = registry_url.with_context(|| {
        format!("Live module {command_name} requires --registry-url or RUSTOK_MODULE_REGISTRY_URL")
    })?;
    let actor =
        actor.with_context(|| format!("Live module {command_name} requires --actor <actor>"))?;
    let reason =
        reason.with_context(|| format!("Live module {command_name} requires --reason <text>"))?;
    let reason_code = reason_code.with_context(|| {
        format!(
            "Live module {command_name} requires --reason-code <{}>",
            allowed_reason_codes.join("|")
        )
    })?;

    let payload = publish_request_governance_via_registry(
        &registry_url,
        request_id,
        command_name,
        action_key,
        Some(&actor),
        Some(reason),
        Some(reason_code),
        false,
    )?;
    println!("{payload}");
    Ok(())
}

pub(crate) fn module_owner_transfer_command(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        print_module_usage();
        anyhow::bail!("module owner-transfer requires a module slug and new owner actor");
    }

    let slug = args[0].trim();
    let new_owner_actor = args[1].trim();
    if slug.is_empty() {
        anyhow::bail!("module owner-transfer requires a non-empty module slug");
    }
    if new_owner_actor.is_empty() {
        anyhow::bail!("module owner-transfer requires a non-empty new owner actor");
    }

    let dry_run = args.iter().skip(2).any(|arg| arg == "--dry-run");
    let manifest_path = manifest_path();
    let manifest = load_manifest_from(&manifest_path)?;
    let workspace_manifest = load_workspace_manifest()?;
    let spec = manifest
        .modules
        .get(slug)
        .with_context(|| format!("Unknown module slug '{slug}'"))?;
    let preview = build_module_publish_preview(&manifest_path, slug, spec, &workspace_manifest)?;
    let reason = reason_argument(&args[2..]);
    let reason_code = owner_transfer_reason_code_argument(&args[2..])?;
    let payload = ModuleOwnerTransferDryRunPreview {
        action: "owner_transfer".to_string(),
        slug: slug.to_string(),
        crate_name: preview.crate_name,
        current_local_version: preview.version,
        package_manifest_path: preview.package_manifest_path,
        new_owner_actor: new_owner_actor.to_string(),
        reason: reason.clone(),
        reason_code: reason_code.clone(),
    };
    let registry_url = registry_url_argument(&args[2..]);
    let actor = actor_argument(&args[2..]);
    if dry_run {
        if let Some(registry_url) = registry_url {
            let remote_payload = owner_transfer_via_registry_dry_run(
                &registry_url,
                &payload,
                reason.clone(),
                reason_code.clone(),
            )?;
            println!("{remote_payload}");
        } else {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        return Ok(());
    }

    let registry_url = registry_url.with_context(|| {
        "Live module owner-transfer requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
    })?;
    let actor = actor.with_context(|| "Live module owner-transfer requires --actor <actor>")?;
    let reason = reason.with_context(|| "Live module owner-transfer requires --reason <text>")?;
    let reason_code = reason_code.with_context(|| {
        format!(
            "Live module owner-transfer requires --reason-code <{}>",
            REGISTRY_OWNER_TRANSFER_REASON_CODES.join("|")
        )
    })?;
    let remote_payload =
        owner_transfer_via_registry_live(&registry_url, &payload, &actor, reason, reason_code)?;
    println!("{remote_payload}");
    Ok(())
}

pub(crate) fn module_stage_command(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        print_module_usage();
        anyhow::bail!("module stage requires a request id, stage key, and status");
    }

    let request_id = args[0].trim();
    let stage = args[1].trim();
    let status = args[2].trim();
    if request_id.is_empty() {
        anyhow::bail!("module stage requires a non-empty request id");
    }
    if stage.is_empty() {
        anyhow::bail!("module stage requires a non-empty stage key");
    }

    let stage_status = match status {
        "queued" | "running" | "passed" | "failed" | "manual_review" => status,
        _ => anyhow::bail!(
            "module stage status '{}' is invalid; expected queued|running|passed|failed|manual_review",
            status
        ),
    };
    let requeue = args.iter().skip(3).any(|arg| arg == "--requeue");
    if requeue && stage_status != "queued" {
        anyhow::bail!("module stage --requeue requires status 'queued'");
    }

    let dry_run = args.iter().skip(3).any(|arg| arg == "--dry-run");
    let detail = detail_argument(&args[3..]);
    let reason_code = validation_stage_reason_code_argument(&args[3..])?;
    let actor = actor_argument(&args[3..]);
    let registry_url = registry_url_argument(&args[3..]);

    let preview = ModuleValidationStageDryRunPreview {
        action: if requeue {
            "stage_requeue".to_string()
        } else {
            "stage_update".to_string()
        },
        request_id: request_id.to_string(),
        stage: stage.to_string(),
        status: stage_status.to_string(),
        detail,
        reason_code,
        requeue,
    };

    if dry_run {
        if let Some(registry_url) = registry_url {
            let payload = validation_stage_via_registry_dry_run(&registry_url, &preview)?;
            println!("{payload}");
        } else {
            println!("{}", serde_json::to_string_pretty(&preview)?);
        }
        return Ok(());
    }

    let registry_url = registry_url.with_context(|| {
        "Live module stage requires --registry-url or RUSTOK_MODULE_REGISTRY_URL"
    })?;
    let actor = actor.with_context(|| "Live module stage requires --actor <actor>")?;
    if validation_stage_status_requires_reason_code(&preview.status)
        && preview.reason_code.is_none()
    {
        anyhow::bail!(
            "Live module stage status '{}' requires --reason-code <{}>",
            preview.status,
            REGISTRY_VALIDATION_STAGE_REASON_CODES.join("|")
        );
    }
    let payload = validation_stage_via_registry_live(&registry_url, &preview, &actor)?;
    println!("{payload}");
    Ok(())
}

fn cli_option_value(args: &[String], flag: &str) -> Option<String> {
    let index = args.iter().position(|arg| arg == flag)?;
    args.get(index + 1).cloned()
}

pub(crate) fn registry_url_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--registry-url")
        .or_else(|| std::env::var("RUSTOK_MODULE_REGISTRY_URL").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn runner_token_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--runner-token")
        .or_else(|| std::env::var(REMOTE_RUNNER_TOKEN_ENV).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn positive_u64_argument(
    args: &[String],
    flag: &str,
    command_label: &str,
) -> Result<Option<u64>> {
    let Some(value) = cli_option_value(args, flag) else {
        return Ok(None);
    };
    let parsed = value.parse::<u64>().with_context(|| {
        format!("{command_label} flag {flag} expects a positive integer, got '{value}'")
    })?;
    if parsed == 0 {
        anyhow::bail!("{command_label} flag {flag} expects a positive integer, got '0'");
    }
    Ok(Some(parsed))
}

pub(crate) fn actor_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--actor")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn auto_approve_argument(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--auto-approve")
}

pub(crate) fn approve_reason_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--approve-reason")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn manual_review_confirmation_argument(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--confirm-manual-review")
}

pub(crate) fn reason_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--reason")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn detail_argument(args: &[String]) -> Option<String> {
    cli_option_value(args, "--detail")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn supported_remote_runner_stages(confirm_manual_review: bool) -> Vec<&'static str> {
    let mut stages = vec!["compile", "tests", "docs", "integrity"];
    if confirm_manual_review {
        stages.push("security_review");
    }
    stages
}

pub(crate) fn reason_code_argument(args: &[String]) -> Result<Option<String>> {
    let Some(reason_code) = cli_option_value(args, "--reason-code") else {
        return Ok(None);
    };
    let reason_code = reason_code.trim().to_ascii_lowercase();
    if reason_code.is_empty() {
        anyhow::bail!(
            "--reason-code expects one of: {}",
            REGISTRY_YANK_REASON_CODES.join("|")
        );
    }
    if !REGISTRY_YANK_REASON_CODES.contains(&reason_code.as_str()) {
        anyhow::bail!(
            "--reason-code '{}' is invalid; expected one of: {}",
            reason_code,
            REGISTRY_YANK_REASON_CODES.join("|")
        );
    }
    Ok(Some(reason_code.to_string()))
}

fn owner_transfer_reason_code_argument(args: &[String]) -> Result<Option<String>> {
    normalized_reason_code_argument(
        args,
        REGISTRY_OWNER_TRANSFER_REASON_CODES,
        "module owner-transfer",
        "--reason-code",
    )
}

pub(crate) fn approve_reason_code_argument(args: &[String]) -> Result<Option<String>> {
    let Some(reason_code) = cli_option_value(args, "--approve-reason-code") else {
        return Ok(None);
    };
    let reason_code = reason_code.trim().to_ascii_lowercase();
    if reason_code.is_empty() {
        anyhow::bail!(
            "--approve-reason-code expects one of: {}",
            REGISTRY_APPROVE_OVERRIDE_REASON_CODES.join("|")
        );
    }
    if !REGISTRY_APPROVE_OVERRIDE_REASON_CODES.contains(&reason_code.as_str()) {
        anyhow::bail!(
            "--approve-reason-code '{}' is invalid; expected one of: {}",
            reason_code,
            REGISTRY_APPROVE_OVERRIDE_REASON_CODES.join("|")
        );
    }
    Ok(Some(reason_code.to_string()))
}

fn validation_stage_reason_code_argument(args: &[String]) -> Result<Option<String>> {
    normalized_reason_code_argument(
        args,
        REGISTRY_VALIDATION_STAGE_REASON_CODES,
        "module stage",
        "--reason-code",
    )
}

pub(crate) fn governance_reason_code_argument(
    args: &[String],
    command_name: &str,
    allowed_reason_codes: &[&str],
) -> Result<Option<String>> {
    let Some(reason_code) = cli_option_value(args, "--reason-code") else {
        return Ok(None);
    };
    let reason_code = reason_code.trim().to_ascii_lowercase();
    if reason_code.is_empty() {
        anyhow::bail!(
            "module {command_name} --reason-code expects one of: {}",
            allowed_reason_codes.join("|")
        );
    }
    if !allowed_reason_codes.contains(&reason_code.as_str()) {
        anyhow::bail!(
            "module {command_name} --reason-code '{}' is invalid; expected one of: {}",
            reason_code,
            allowed_reason_codes.join("|")
        );
    }
    Ok(Some(reason_code.to_string()))
}

fn normalized_reason_code_argument(
    args: &[String],
    allowed: &[&str],
    command_label: &str,
    flag: &str,
) -> Result<Option<String>> {
    let Some(value) = cli_option_value(args, flag) else {
        return Ok(None);
    };
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return Ok(None);
    }
    if !allowed.iter().any(|candidate| *candidate == value) {
        anyhow::bail!(
            "{} reason code '{}' is not supported; expected one of {}",
            command_label,
            value,
            allowed.join(", ")
        );
    }
    Ok(Some(value))
}
