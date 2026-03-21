# rustok-outbox docs

В этой папке хранится документация модуля `crates/rustok-outbox`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Canonical scope

This file is the canonical module-level documentation for `crates/rustok-outbox`.
Module-specific outbox and transactional-publishing documentation must stay in this crate, not under `docs/architecture/`.

## What this module owns

`rustok-outbox` owns transactional event publishing and the core outbox delivery contract:

- `TransactionalEventBus` for atomic publish-with-transaction semantics;
- persistence to `sys_events` through the configured transactional transport;
- relay/retry/DLQ semantics used by the server event runtime;
- the write-side guarantee that domain changes and event persistence succeed or fail together.

## Transactional publishing contract

`TransactionalEventBus` guarantees that an event is persisted if and only if the surrounding transaction commits successfully.

Typical flow:

1. open DB transaction;
2. perform domain writes;
3. call `publish_in_tx(...)`;
4. commit transaction;
5. relay/runtime delivers the persisted event asynchronously.

When the transaction rolls back, the event must not be persisted.

## Runtime and operations

Current production pattern in RusToK:

- write-side uses outbox (`transport = "outbox"`);
- downstream relay target is configured separately (`memory` for local/dev, `iggy` for replay/high-load scenarios);
- retry policy and logical DLQ are handled by the relay worker and `sys_events` statuses.

Operational concerns that stay attached to this module:

- backlog growth and delivery retry behavior;
- DLQ replay workflows;
- reindex decisions after consumer lag or downstream outages.

## Configuration

Current production pattern is L1 outbox persistence with a separately selected relay target:

```toml
[settings.rustok.events]
transport = "outbox"
relay_target = "memory" # memory | iggy

[settings.rustok.events.relay_retry_policy]
max_attempts = 5
base_backoff_ms = 1000
max_backoff_ms = 60000

[settings.rustok.events.dlq]
enabled = true
max_attempts = 10
```

Operational semantics:

- `transport = "outbox"` means the event is persisted atomically to `sys_events` in the DB transaction.
- `relay_target` selects downstream delivery independently from the write-side contract.
- If downstream is unavailable, the event remains in backlog and is retried according to retry policy.
- After retry exhaustion, the event is moved into logical DLQ state in `sys_events`.

## Incident runbook

### Backlog growth

1. Check `outbox_backlog_size` and its 15-30 minute trend.
2. Verify relay target availability (`iggy` or memory subscriber path).
3. Inspect `outbox_retries_total` growth and relay-worker delivery errors.

### Downstream outage

1. Confirm write-side continues persisting events to `sys_events`.
2. Confirm retry loop is active and backoff is not saturated.
3. After downstream recovery, verify `outbox_backlog_size` starts decreasing.

### DLQ replay

1. Filter DLQ events by `tenant_id` and `event_type`.
2. Confirm root cause before replaying.
3. Replay in batches and monitor `outbox_dlq_total` and delivery latency.
4. Record incident and remediation in postmortem notes.

### Consumer lag and reindex

1. Check `rustok_event_consumer_lagged_total{consumer="..."}` and correlated growth in `rustok_event_consumer_restarted_total`.
2. If lag affected only UI streaming or cache invalidation and did not touch read-model consumers, restore the loop first; reindex is not required by default.
3. Choose partial reindex only when the scope of missed events is tightly localized.
4. Choose full rebuild when the affected scope cannot be localized or read-model drift remains after partial recovery.
5. After reindex, verify dispatch latency stabilizes and lag counters stop growing.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Related docs

- [Server event transport runtime](../../../apps/server/docs/event-transport.md)
- [Platform event flow contract](../../../docs/architecture/event-flow-contract.md)

