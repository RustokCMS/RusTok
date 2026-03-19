use casbin::{CoreApi, DefaultModel, Enforcer, StringAdapter};
use rustok_core::{Action, Permission};
use std::fmt::Write;

const DEFAULT_CASBIN_MODEL: &str = include_str!("../../config/casbin_model.conf");
const RESOLVED_PERMISSIONS_SUBJECT: &str = "__resolved_permissions_subject__";

pub fn default_casbin_model() -> &'static str {
    DEFAULT_CASBIN_MODEL
}

pub fn resolved_permissions_subject() -> &'static str {
    RESOLVED_PERMISSIONS_SUBJECT
}

pub fn build_casbin_policy_csv(
    tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
) -> String {
    let tenant_domain = tenant_id.to_string();
    let mut policy = String::new();

    for permission in resolved_permissions {
        let subject = permission_subject(permission);
        let object = permission.resource.to_string();
        let action = permission_action_token(permission);

        let _ = writeln!(
            &mut policy,
            "p, {subject}, {tenant_domain}, {object}, {action}"
        );
        let _ = writeln!(
            &mut policy,
            "g, {subject_user}, {subject}, {tenant_domain}",
            subject_user = resolved_permissions_subject(),
        );
    }

    policy
}

pub async fn build_enforcer_for_permissions(
    tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
) -> casbin::Result<Enforcer> {
    let model = DefaultModel::from_str(default_casbin_model()).await?;
    let adapter = StringAdapter::new(build_casbin_policy_csv(tenant_id, resolved_permissions));
    Enforcer::new(model, adapter).await
}

fn permission_subject(permission: &Permission) -> String {
    format!("perm::{permission}")
}

fn permission_action_token(permission: &Permission) -> String {
    match permission.action {
        Action::Manage => "*".to_string(),
        _ => permission.action.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_casbin_policy_csv, build_enforcer_for_permissions, default_casbin_model,
        resolved_permissions_subject,
    };
    use casbin::CoreApi;
    use rustok_core::Permission;

    #[test]
    fn model_contains_core_sections() {
        let model = default_casbin_model();
        assert!(model.contains("[request_definition]"));
        assert!(model.contains("[policy_definition]"));
        assert!(model.contains("[role_definition]"));
        assert!(model.contains("[matchers]"));
    }

    #[test]
    fn model_declares_tenant_domain_field() {
        let model = default_casbin_model();
        assert!(model.contains("r = sub, dom, obj, act"));
        assert!(model.contains("p = sub, dom, obj, act"));
        assert!(model.contains("g = _, _, _"));
    }

    #[test]
    fn policy_csv_maps_manage_to_wildcard_action() {
        let tenant_id = uuid::Uuid::new_v4();
        let policy = build_casbin_policy_csv(&tenant_id, &[Permission::USERS_MANAGE]);

        assert!(policy.contains("p, perm::users:manage,"));
        assert!(policy.contains(", users, *"));
        assert!(policy.contains(resolved_permissions_subject()));
    }

    #[tokio::test]
    async fn enforcer_allows_permissions_loaded_from_generated_policy() {
        let tenant_id = uuid::Uuid::new_v4();
        let tenant_domain = tenant_id.to_string();
        let enforcer = build_enforcer_for_permissions(
            &tenant_id,
            &[Permission::USERS_MANAGE, Permission::PAGES_READ],
        )
        .await
        .expect("generated Casbin enforcer should be valid");

        assert!(enforcer
            .enforce((
                resolved_permissions_subject(),
                tenant_domain.clone(),
                "users".to_string(),
                "update".to_string(),
            ))
            .expect("Casbin enforce should succeed"));
        assert!(enforcer
            .enforce((
                resolved_permissions_subject(),
                tenant_domain,
                "pages".to_string(),
                "read".to_string(),
            ))
            .expect("Casbin enforce should succeed"));
    }
}
