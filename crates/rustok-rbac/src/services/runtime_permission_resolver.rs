use crate::{
    resolve_permissions_with_cache, PermissionCache, PermissionResolution, PermissionResolver,
    RelationPermissionStore,
};
use async_trait::async_trait;
use rustok_core::UserRole;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RuntimePermissionResolver<S, C, A, E>
where
    S: RelationPermissionStore,
    C: PermissionCache,
    A: RoleAssignmentStore,
    S::Error: Into<E>,
    A::Error: Into<E>,
{
    store: S,
    cache: C,
    assignment_store: A,
    _error: PhantomData<E>,
}

impl<S, C, A, E> RuntimePermissionResolver<S, C, A, E>
where
    S: RelationPermissionStore,
    C: PermissionCache,
    A: RoleAssignmentStore,
    S::Error: Into<E>,
    A::Error: Into<E>,
{
    pub fn new(store: S, cache: C, assignment_store: A) -> Self {
        Self {
            store,
            cache,
            assignment_store,
            _error: PhantomData,
        }
    }
}

#[async_trait]
pub trait RoleAssignmentStore {
    type Error;

    async fn assign_role_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;

    async fn replace_user_role(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<S, C, A, E> PermissionResolver for RuntimePermissionResolver<S, C, A, E>
where
    S: RelationPermissionStore + Send + Sync,
    C: PermissionCache + Send + Sync,
    A: RoleAssignmentStore + Send + Sync,
    S::Error: Into<E> + Send + Sync,
    A::Error: Into<E> + Send + Sync,
    E: Send + Sync,
{
    type Error = E;

    async fn resolve_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<PermissionResolution, Self::Error> {
        resolve_permissions_with_cache(&self.store, &self.cache, tenant_id, user_id)
            .await
            .map_err(Into::into)
    }

    async fn assign_role_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error> {
        self.assignment_store
            .assign_role_permissions(tenant_id, user_id, role)
            .await
            .map_err(Into::into)
    }

    async fn replace_user_role(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error> {
        self.assignment_store
            .replace_user_role(tenant_id, user_id, role)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{RoleAssignmentStore, RuntimePermissionResolver};
    use crate::{PermissionCache, PermissionResolver, RelationPermissionStore};
    use async_trait::async_trait;
    use rustok_core::{Permission, UserRole};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct StubStore {
        role_ids: Vec<uuid::Uuid>,
        tenant_role_ids: Vec<uuid::Uuid>,
        permissions: Vec<Permission>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum StubStoreError {
        Load,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum StubAssignmentError {
        Assign,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ResolverError {
        Store(StubStoreError),
        Assignment(StubAssignmentError),
    }

    impl From<StubStoreError> for ResolverError {
        fn from(value: StubStoreError) -> Self {
            Self::Store(value)
        }
    }

    impl From<StubAssignmentError> for ResolverError {
        fn from(value: StubAssignmentError) -> Self {
            Self::Assignment(value)
        }
    }

    #[derive(Default)]
    struct StubCache {
        values: Arc<Mutex<HashMap<(uuid::Uuid, uuid::Uuid), Vec<Permission>>>>,
    }

    #[derive(Default)]
    struct StubAssignmentStore {
        assigned: Arc<Mutex<Vec<(uuid::Uuid, uuid::Uuid, UserRole)>>>,
        replaced: Arc<Mutex<Vec<(uuid::Uuid, uuid::Uuid, UserRole)>>>,
        fail_assign: bool,
    }

    #[async_trait]
    impl PermissionCache for StubCache {
        async fn get(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
        ) -> Option<Vec<Permission>> {
            self.values
                .lock()
                .await
                .get(&(*tenant_id, *user_id))
                .cloned()
        }

        async fn insert(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            permissions: Vec<Permission>,
        ) {
            self.values
                .lock()
                .await
                .insert((*tenant_id, *user_id), permissions);
        }

        async fn invalidate(&self, tenant_id: &uuid::Uuid, user_id: &uuid::Uuid) {
            self.values.lock().await.remove(&(*tenant_id, *user_id));
        }
    }

    #[async_trait]
    impl RelationPermissionStore for StubStore {
        type Error = StubStoreError;

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

    #[async_trait]
    impl RoleAssignmentStore for StubAssignmentStore {
        type Error = StubAssignmentError;

        async fn assign_role_permissions(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            role: UserRole,
        ) -> Result<(), Self::Error> {
            if self.fail_assign {
                return Err(StubAssignmentError::Assign);
            }
            self.assigned
                .lock()
                .await
                .push((*tenant_id, *user_id, role));
            Ok(())
        }

        async fn replace_user_role(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            role: UserRole,
        ) -> Result<(), Self::Error> {
            self.replaced
                .lock()
                .await
                .push((*tenant_id, *user_id, role));
            Ok(())
        }
    }

    #[tokio::test]
    async fn resolve_permissions_delegates_to_relation_and_cache_layer() {
        let role_id = uuid::Uuid::new_v4();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let resolver = RuntimePermissionResolver::new(
            StubStore {
                role_ids: vec![role_id],
                tenant_role_ids: vec![role_id],
                permissions: vec![Permission::USERS_READ],
            },
            StubCache::default(),
            StubAssignmentStore::default(),
        );

        let first = resolver
            .resolve_permissions(&tenant_id, &user_id)
            .await
            .unwrap();
        let second = resolver
            .resolve_permissions(&tenant_id, &user_id)
            .await
            .unwrap();

        assert!(!first.cache_hit);
        assert!(second.cache_hit);
        assert_eq!(second.permissions, vec![Permission::USERS_READ]);
    }

    #[tokio::test]
    async fn role_assignment_use_cases_delegate_to_assignment_store() {
        let assignment_store = StubAssignmentStore::default();
        let resolver = RuntimePermissionResolver::new(
            StubStore {
                role_ids: vec![],
                tenant_role_ids: vec![],
                permissions: vec![],
            },
            StubCache::default(),
            assignment_store,
        );
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();

        resolver
            .assign_role_permissions(&tenant_id, &user_id, UserRole::Editor)
            .await
            .unwrap();
        resolver
            .replace_user_role(&tenant_id, &user_id, UserRole::Admin)
            .await
            .unwrap();

        let assigned = resolver.assignment_store.assigned.lock().await.clone();
        let replaced = resolver.assignment_store.replaced.lock().await.clone();

        assert_eq!(assigned, vec![(tenant_id, user_id, UserRole::Editor)]);
        assert_eq!(replaced, vec![(tenant_id, user_id, UserRole::Admin)]);
    }

    #[tokio::test]
    async fn assignment_error_is_mapped_to_runtime_resolver_error_type() {
        let resolver: RuntimePermissionResolver<_, _, _, ResolverError> =
            RuntimePermissionResolver::new(
                StubStore {
                    role_ids: vec![],
                    tenant_role_ids: vec![],
                    permissions: vec![],
                },
                StubCache::default(),
                StubAssignmentStore {
                    fail_assign: true,
                    ..StubAssignmentStore::default()
                },
            );

        let result = resolver
            .assign_role_permissions(
                &uuid::Uuid::new_v4(),
                &uuid::Uuid::new_v4(),
                UserRole::Admin,
            )
            .await;

        assert_eq!(
            result.err(),
            Some(ResolverError::Assignment(StubAssignmentError::Assign))
        );
    }
}
