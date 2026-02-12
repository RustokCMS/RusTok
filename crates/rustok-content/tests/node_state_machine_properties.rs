//! Property-based tests for Content Node State Machine
//!
//! These tests verify that state machine invariants hold for all inputs.

use proptest::prelude::*;
use rustok_content::state_machine::{Archived, ContentNode, Draft, Published};
use rustok_test_utils::{non_empty_string, slug_strategy, uuid_strategy};
use uuid::Uuid;

// ============================================================================
// Helper Strategies
// ============================================================================

fn node_kind_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["post", "page", "article", "product"]).prop_map(|s| s.to_string())
}

fn create_test_node(
    id: Uuid,
    tenant_id: Uuid,
    author_id: Option<Uuid>,
    kind: String,
) -> ContentNode<Draft> {
    ContentNode::new_draft(id, tenant_id, author_id, kind)
}

// ============================================================================
// PROPERTY 1: Valid State Transitions Only
// ============================================================================

proptest! {
    /// Property: Draft nodes can only transition to Published
    #[test]
    fn draft_can_only_transition_to_published(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);

        // Can publish
        let published = node.publish();
        prop_assert_eq!(published.id, id, "Published node should preserve ID");
        prop_assert_eq!(published.tenant_id, tenant_id, "Published node should preserve tenant_id");

        // Note: Draft cannot directly archive (compile-time prevention)
        // This property is enforced by the type system
    }

    /// Property: Published nodes can only transition to Archived
    #[test]
    fn published_can_only_transition_to_archived(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let published = node.publish();

        // Can archive
        let archived = published.archive(reason.clone());
        prop_assert_eq!(archived.id, id, "Archived node should preserve ID");
        prop_assert_eq!(archived.state.reason, reason, "Archived node should preserve reason");

        // Note: Published cannot transition back to Draft (compile-time prevention)
    }
}

// ============================================================================
// PROPERTY 2: Published State Immutability
// ============================================================================

proptest! {
    /// Property: Once published, a node cannot transition back to Draft
    /// (This is enforced at compile-time, but we verify the one-way nature)
    #[test]
    fn published_state_is_one_way(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let published = node.publish();

        // Published node has published_at timestamp
        let published_at = published.state.published_at;
        
        // Update doesn't change published status
        let updated = published.update();
        prop_assert_eq!(
            updated.state.published_at, 
            published_at,
            "Update should not change published_at timestamp"
        );

        // Note: There is no unpublish() method - this is compile-time safe
    }

    /// Property: Published nodes maintain their published_at timestamp
    #[test]
    fn published_at_immutable_after_publish(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let published = node.publish();
        let published_at = published.state.published_at;

        // Update multiple times
        let updated1 = published.update();
        let updated2 = updated1.update();

        prop_assert_eq!(
            updated2.state.published_at,
            published_at,
            "published_at should remain constant across updates"
        );
    }
}

// ============================================================================
// PROPERTY 3: Node ID Immutability
// ============================================================================

proptest! {
    /// Property: Node ID is immutable across all state transitions
    #[test]
    fn node_id_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let original_id = node.id;

        // Publish
        let published = node.publish();
        prop_assert_eq!(published.id, original_id, "ID should be immutable after publish");

        // Archive
        let archived = published.archive(reason);
        prop_assert_eq!(archived.id, original_id, "ID should be immutable after archive");
    }

    /// Property: Tenant ID is immutable across all state transitions
    #[test]
    fn tenant_id_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let original_tenant = node.tenant_id;

        let published = node.publish();
        prop_assert_eq!(published.tenant_id, original_tenant, "tenant_id immutable after publish");

        let archived = published.archive(reason);
        prop_assert_eq!(archived.tenant_id, original_tenant, "tenant_id immutable after archive");
    }

    /// Property: Kind is immutable across all state transitions
    #[test]
    fn kind_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind.clone());
        let original_kind = node.kind.clone();

        let published = node.publish();
        prop_assert_eq!(published.kind, original_kind, "kind immutable after publish");

        let archived = published.archive(reason);
        prop_assert_eq!(archived.kind, original_kind, "kind immutable after archive");
    }
}

// ============================================================================
// PROPERTY 4: Draft Update Properties
// ============================================================================

proptest! {
    /// Property: Updating draft updates the updated_at timestamp
    #[test]
    fn draft_update_changes_timestamp(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let original_updated = node.state.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let updated = node.update();

        // updated_at should change (or at least not decrease)
        prop_assert!(
            updated.state.updated_at >= original_updated,
            "Update should advance or maintain timestamp"
        );
    }

    /// Property: Multiple draft updates are allowed
    #[test]
    fn draft_allows_multiple_updates(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);

        // Apply multiple updates
        let updated1 = node.update();
        let updated2 = updated1.update();
        let updated3 = updated2.update();

        // Node should still be valid
        prop_assert_eq!(updated3.id, id, "Node ID preserved through updates");
        prop_assert!(
            updated3.state.updated_at >= updated3.state.created_at,
            "updated_at should be >= created_at"
        );
    }
}

// ============================================================================
// PROPERTY 5: Archived State Properties
// ============================================================================

proptest! {
    /// Property: Archived nodes always have a non-empty reason
    #[test]
    fn archived_nodes_have_reason(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let published = node.publish();
        let archived = published.archive(reason.clone());

        prop_assert_eq!(archived.state.reason, reason);
        prop_assert!(!archived.state.reason.is_empty(), "Archive reason should not be empty");
    }

    /// Property: Archive timestamp is after publish timestamp
    #[test]
    fn archive_timestamp_after_publish(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        reason in non_empty_string()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);
        let published = node.publish();
        let published_at = published.state.published_at;

        std::thread::sleep(std::time::Duration::from_millis(10));

        let archived = published.archive(reason);

        prop_assert!(
            archived.state.archived_at >= published_at,
            "Archive timestamp should be >= publish timestamp"
        );
    }
}

// ============================================================================
// PROPERTY 6: Type System Guarantees
// ============================================================================

proptest! {
    /// Property: Node creation always produces Draft state
    #[test]
    fn new_nodes_always_draft(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy()
    ) {
        let node = create_test_node(id, tenant_id, author_id, kind);

        // Node should have created_at and updated_at
        prop_assert!(
            node.state.updated_at >= node.state.created_at,
            "updated_at should be >= created_at for new nodes"
        );
    }

    /// Property: State transitions preserve core node properties
    #[test]
    fn transitions_preserve_core_properties(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        author_id in proptest::option::of(uuid_strategy()),
        kind in node_kind_strategy(),
        parent_id in proptest::option::of(uuid_strategy()),
        category_id in proptest::option::of(uuid_strategy()),
        reason in non_empty_string()
    ) {
        let mut node = create_test_node(id, tenant_id, author_id, kind.clone());
        node.parent_id = parent_id;
        node.category_id = category_id;

        let published = node.publish();
        prop_assert_eq!(published.parent_id, parent_id);
        prop_assert_eq!(published.category_id, category_id);
        prop_assert_eq!(published.author_id, author_id);

        let archived = published.archive(reason);
        prop_assert_eq!(archived.parent_id, parent_id);
        prop_assert_eq!(archived.category_id, category_id);
        prop_assert_eq!(archived.author_id, author_id);
    }
}
