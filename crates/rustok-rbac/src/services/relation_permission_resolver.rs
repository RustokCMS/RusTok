use rustok_core::Permission;
use std::collections::HashSet;

fn normalize_permissions(permissions: Vec<Permission>) -> Vec<Permission> {
    let mut unique = permissions
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    unique.sort_by_key(|permission| permission.to_string());
    unique
}

#[async_trait::async_trait]
pub trait RelationPermissionStore {
    type Error;

    async fn load_user_role_ids(
        &self,
        user_id: &uuid::Uuid,
    ) -> Result<Vec<uuid::Uuid>, Self::Error>;

    async fn load_tenant_role_ids(
        &self,
        tenant_id: &uuid::Uuid,
        role_ids: &[uuid::Uuid],
    ) -> Result<Vec<uuid::Uuid>, Self::Error>;

    async fn load_permissions_for_roles(
        &self,
        tenant_id: &uuid::Uuid,
        role_ids: &[uuid::Uuid],
    ) -> Result<Vec<Permission>, Self::Error>;
}

pub async fn resolve_permissions_from_relations<S: RelationPermissionStore>(
    store: &S,
    tenant_id: &uuid::Uuid,
    user_id: &uuid::Uuid,
) -> Result<Vec<Permission>, S::Error> {
    let user_role_ids = store.load_user_role_ids(user_id).await?;
    if user_role_ids.is_empty() {
        return Ok(vec![]);
    }

    let tenant_role_ids = store
        .load_tenant_role_ids(tenant_id, &user_role_ids)
        .await?;
    if tenant_role_ids.is_empty() {
        return Ok(vec![]);
    }

    let permissions = store
        .load_permissions_for_roles(tenant_id, &tenant_role_ids)
        .await?;

    Ok(normalize_permissions(permissions))
}

#[cfg(test)]
mod tests {
    use super::{resolve_permissions_from_relations, RelationPermissionStore};
    use async_trait::async_trait;
    use rustok_core::Permission;

    struct StubStore {
        role_ids: Vec<uuid::Uuid>,
        tenant_role_ids: Vec<uuid::Uuid>,
        permissions: Vec<Permission>,
    }

    #[async_trait]
    impl RelationPermissionStore for StubStore {
        type Error = String;

        async fn load_user_role_ids(
            &self,
            _user_id: &uuid::Uuid,
        ) -> Result<Vec<uuid::Uuid>, Self::Error> {
            Ok(self.role_ids.clone())
        }

        async fn load_tenant_role_ids(
            &self,
            _tenant_id: &uuid::Uuid,
            _role_ids: &[uuid::Uuid],
        ) -> Result<Vec<uuid::Uuid>, Self::Error> {
            Ok(self.tenant_role_ids.clone())
        }

        async fn load_permissions_for_roles(
            &self,
            _tenant_id: &uuid::Uuid,
            _role_ids: &[uuid::Uuid],
        ) -> Result<Vec<Permission>, Self::Error> {
            Ok(self.permissions.clone())
        }
    }

    #[tokio::test]
    async fn returns_empty_when_user_has_no_roles() {
        let store = StubStore {
            role_ids: vec![],
            tenant_role_ids: vec![uuid::Uuid::new_v4()],
            permissions: vec![Permission::USERS_READ],
        };

        let resolved = resolve_permissions_from_relations(
            &store,
            &uuid::Uuid::new_v4(),
            &uuid::Uuid::new_v4(),
        )
        .await
        .unwrap();

        assert!(resolved.is_empty());
    }

    #[tokio::test]
    async fn returns_stable_sorted_permissions() {
        let role_id = uuid::Uuid::new_v4();
        let store = StubStore {
            role_ids: vec![role_id],
            tenant_role_ids: vec![role_id],
            permissions: vec![
                Permission::USERS_UPDATE,
                Permission::USERS_READ,
                Permission::USERS_MANAGE,
            ],
        };

        let resolved = resolve_permissions_from_relations(
            &store,
            &uuid::Uuid::new_v4(),
            &uuid::Uuid::new_v4(),
        )
        .await
        .unwrap();

        assert_eq!(
            resolved,
            vec![
                Permission::USERS_MANAGE,
                Permission::USERS_READ,
                Permission::USERS_UPDATE,
            ]
        );
    }

    #[tokio::test]
    async fn deduplicates_permissions() {
        let role_id = uuid::Uuid::new_v4();
        let store = StubStore {
            role_ids: vec![role_id],
            tenant_role_ids: vec![role_id],
            permissions: vec![Permission::USERS_READ, Permission::USERS_READ],
        };

        let resolved = resolve_permissions_from_relations(
            &store,
            &uuid::Uuid::new_v4(),
            &uuid::Uuid::new_v4(),
        )
        .await
        .unwrap();

        assert_eq!(resolved, vec![Permission::USERS_READ]);
    }
}
