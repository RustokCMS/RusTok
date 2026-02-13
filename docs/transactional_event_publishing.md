# Transactional Event Publishing

## Overview

The `TransactionalEventBus` provides atomic event publishing that is guaranteed to be persisted in the database if and only if the surrounding transaction commits successfully. This prevents event loss during transaction rollbacks and ensures data consistency between domain operations and published events.

## Architecture

```
┌─────────────────┐    ┌─────────────────────┐    ┌──────────────────┐
│   Application   │    │  Transactional      │    │   OutboxTransport│
│      Code       │───▶│     EventBus        │───▶│                  │
└─────────────────┘    └─────────────────────┘    └──────────────────┘
                                │
                                ▼
                       ┌─────────────────────┐
                       │   Event Envelope    │
                       │   (with versioning) │
                       └─────────────────────┘
```

## Usage

### Basic Usage

```rust
use rustok_outbox::TransactionalEventBus;
use rustok_core::DomainEvent;
use uuid::Uuid;

async fn create_node_with_events(
    db: &DatabaseConnection,
    event_bus: &TransactionalEventBus,
    tenant_id: Uuid,
    user_id: Uuid,
    node_id: Uuid,
) -> Result<()> {
    // Start database transaction
    let txn = db.begin().await?;
    
    // Perform domain operations
    // ... create node, translations, etc.
    
    // Publish event transactionally
    event_bus.publish_in_tx(
        &txn,
        tenant_id,
        Some(user_id),
        DomainEvent::NodeCreated {
            node_id,
            kind: "post".to_string(),
            author_id: Some(user_id),
        },
    ).await?;
    
    // Commit both domain operations and event
    txn.commit().await?;
    
    Ok(())
}
```

### Integration with Services

Services are configured to use `TransactionalEventBus` automatically:

```rust
// In NodeService
impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }
    
    pub async fn create_node(&self, ...) -> Result<NodeResponse> {
        let txn = self.db.begin().await?;
        
        // Domain logic...
        
        self.event_bus
            .publish_in_tx(&txn, tenant_id, user_id, event)
            .await?;
            
        txn.commit().await?;
        // ...
    }
}
```

## Configuration

The `TransactionalEventBus` works with any `EventTransport` implementation that supports transactional writes:

### Outbox Transport (Recommended)

```yaml
settings:
  rustok:
    events:
      transport: outbox
      relay_interval_ms: 1000
```

Outbox transport provides the strongest guarantees:
- Events are persisted to `sys_events` table within the same transaction
- Automatic retry logic via outbox relay worker
- Event versioning support
- Idempotent event processing

### Memory Transport (Development)

```yaml
settings:
  rustok:
    events:
      transport: memory
```

Memory transport provides weaker guarantees:
- Events published immediately outside transaction context
- No persistence guarantees
- Warning logged if transport doesn't support transactional writes

## Event Versioning

All events are automatically versioned with schema versioning:

```rust
use rustok_core::events::DomainEvent;

// Each DomainEvent has schema_version() method
assert_eq!(
    DomainEvent::NodeCreated { /* ... */ }.schema_version(),
    1
);
```

Events are persisted with:
- `event_type`: String identifier (e.g., "node.created")
- `schema_version`: Integer version (starts at 1)
- `tenant_id`: Tenant context
- `actor_id`: User who triggered the event
- `event_data`: Serialized event payload
- `metadata`: Additional context

## Error Handling

### Transaction Rollback

If the containing transaction is rolled back, events are **not** persisted:

```rust
async fn example_with_rollback(event_bus: &TransactionalEventBus) -> Result<()> {
    let txn = db.begin().await?;
    
    event_bus.publish_in_tx(&txn, tenant_id, user_id, event).await?;
    
    // Something fails...
    txn.rollback().await?; // Event is NOT persisted
    
    Ok(())
}
```

### Transport Fallback

If the transport doesn't support transactional writes, a warning is logged:

```rust
// From TransactionalEventBus::publish_in_tx
if !outbox_transport.supports_transactional_writes() {
    tracing::warn!(
        "EventTransport doesn't support transactional writes. \
         Event may be lost if transaction fails."
    );
    // Falls back to immediate publish (not recommended for production)
}
```

## Performance Considerations

1. **Transaction Scope**: Keep transaction scope minimal to reduce lock contention
2. **Event Size**: Large events impact transaction performance
3. **Batch Publishing**: Consider publishing multiple events in single transaction
4. **Outbox Relay**: Configure appropriate relay intervals for production load

## Testing

Use the provided test utilities:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_persistence() {
        let (db, _pool) = setup_test_db().await;
        let transport = Arc::new(OutboxTransport::new(db.clone()));
        let event_bus = TransactionalEventBus::new(transport);
        
        let txn = db.begin().await.unwrap();
        event_bus.publish_in_tx(&txn, tenant_id, user_id, event).await.unwrap();
        txn.commit().await.unwrap();
        
        // Verify event persistence...
    }
}
```

## Migration Support

When upgrading existing systems:

1. Existing events in `sys_events` table are preserved
2. New events include version metadata
3. Outbox relay processes both old and new format events
4. Backward compatibility maintained for event consumers

## Best Practices

1. **Always use transactions** for domain operations that publish events
2. **Keep events small** - reference large data via IDs
3. **Use idempotency** - design events to be safely replayed
4. **Monitor outbox backlog** - set up alerts for stuck events
5. **Test rollback scenarios** - ensure events are not lost on failures

## Troubleshooting

### Events not being persisted

1. Check transaction commits - events are only persisted on successful commits
2. Verify transport configuration - outbox transport required for guarantees
3. Check database connectivity and permissions
4. Review logs for transport fallback warnings

### Performance issues

1. Reduce transaction scope and duration
2. Configure outbox relay intervals appropriately
3. Monitor database connection pool usage
4. Consider event batching for high-volume scenarios

### Event processing failures

1. Enable outbox relay logging
2. Implement dead letter handling for failed events
3. Monitor event delivery rates and latencies
4. Set up alerting for stuck outbox entries

---

## Modules Using TransactionalEventBus

**Status (2026-02-11)**: The following modules have been migrated to use `TransactionalEventBus` from `rustok-outbox`:

### Content Modules

| Module | Services | Dependency Added | Status |
|--------|----------|------------------|--------|
| `rustok-content` | `NodeService` | ✅ Yes | ✅ Migrated |
| `rustok-blog` | `PostService` | ✅ Yes | ✅ Migrated |
| `rustok-forum` | `CategoryService`, `TopicService`, `ReplyService`, `ModerationService` | ✅ Yes | ✅ Migrated |
| `rustok-pages` | `PageService`, `BlockService`, `MenuService` | ✅ Yes | ✅ Migrated |

### Migration Details

All service constructors now accept `TransactionalEventBus` instead of `EventBus`:

```rust
// Before (deprecated)
use rustok_core::EventBus;

impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: EventBus) -> Self { ... }
}

// After (current)
use rustok_outbox::TransactionalEventBus;

impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self { ... }
}
```

### Required Changes for New Modules

When creating new modules that publish events:

1. **Add dependency** in `Cargo.toml`:
   ```toml
   [dependencies]
   rustok-outbox.workspace = true
   ```

2. **Import TransactionalEventBus**:
   ```rust
   use rustok_outbox::TransactionalEventBus;
   ```

3. **Update service constructor**:
   ```rust
   pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self
   ```

4. **Use transactional publishing**:
   ```rust
   event_bus.publish_in_tx(&txn, tenant_id, user_id, event).await?;
   ```

See module READMEs for specific implementation details:
- [rustok-blog/README.md](../crates/rustok-blog/README.md)
- [rustok-forum/README.md](../crates/rustok-forum/README.md)
- [rustok-pages/README.md](../crates/rustok-pages/README.md)