# RusToK — Implementation Checklist

Практический чеклист для отслеживания прогресса по внедрению рекомендаций.

---

## 🎯 Quick Progress Overview

```
Phase 1 (Critical):       [ ] 0/6 completed
Phase 2 (Stability):      [ ] 0/5 completed  
Phase 3 (Production):     [ ] 0/6 completed
Phase 4 (Advanced):       [ ] 0/5 completed

Total Progress: 0/22 (0%)
```

---

## Phase 1: Critical Fixes (2-3 weeks) 🔴

**Цель:** Исправить критичные проблемы безопасности и consistency

### Testing

- [ ] **Day 1-2:** Create `rustok-test-utils` crate
  - [ ] Add `fixtures` module with sample data
  - [ ] Add `db::setup_test_db()` helper
  - [ ] Add `events::mock_event_bus()` helper
  
- [ ] **Day 3-5:** Add unit tests for `rustok-content`
  - [ ] Test `NodeService::create_node()`
  - [ ] Test `NodeService::update_node()`
  - [ ] Test `NodeService::delete_node()`
  - [ ] Test RBAC enforcement (Own scope)
  - [ ] Test validation errors
  
- [ ] **Day 6-8:** Add unit tests for `rustok-commerce`
  - [ ] Test `CatalogService` CRUD
  - [ ] Test `PricingService` calculations
  - [ ] Test `InventoryService` stock management
  
- [ ] **Day 9-10:** Add integration tests
  - [ ] Test Node creation → Event → Index update
  - [ ] Test Product creation → Event → Index update
  - [ ] Test multi-tenant isolation
  
- [ ] **Verification:**
  ```bash
  cargo test --workspace
  # Target: 30%+ coverage
  ```

### Transaction Safety

- [ ] **Day 1-2:** Extend `OutboxTransport`
  - [ ] Add `write_to_outbox()` method accepting transaction
  - [ ] Add `publish_in_tx()` to `EventTransport` trait
  - [ ] Update migration for `sys_events` table
  
- [ ] **Day 3-4:** Create `TransactionalEventBus`
  - [ ] Wrap `OutboxTransport` with convenient API
  - [ ] Add error handling
  - [ ] Add tests
  
- [ ] **Day 5-7:** Refactor services
  - [ ] Update `NodeService` to use transactional bus
  - [ ] Update `CatalogService` to use transactional bus
  - [ ] Update other services
  - [ ] Add integration tests for atomicity
  
- [ ] **Verification:**
  ```bash
  # Test that events are written in same transaction
  cargo test test_transactional_event_publishing
  ```

### Event Schema Versioning

- [ ] **Day 1:** Update `EventEnvelope`
  - [ ] Add `schema_version: u16` field
  - [ ] Add `event_type: String` field
  - [ ] Update `EventEnvelope::new()` to populate fields
  
- [ ] **Day 2:** Update `DomainEvent`
  - [ ] Add `schema_version()` method
  - [ ] Document version policy in docstring
  - [ ] Add tests
  
- [ ] **Day 3:** Update storage
  - [ ] Update Outbox entity to store version
  - [ ] Update Iggy serialization to include version
  - [ ] Add migration
  
- [ ] **Verification:**
  ```bash
  # Verify version is stored
  psql $DATABASE_URL -c "SELECT event_type, schema_version FROM sys_events LIMIT 5;"
  ```

### Tenant Cache Stampede Protection

- [ ] **Day 1-2:** Implement singleflight pattern
  - [ ] Add `in_flight: Arc<Mutex<HashMap<String, Arc<Notify>>>>` to resolver
  - [ ] Implement `get_or_load()` method with coalescing
  - [ ] Add metrics for coalescing effectiveness
  
- [ ] **Day 3:** Load testing
  - [ ] Write load test script (1000 concurrent requests)
  - [ ] Verify only 1 DB query per cache miss
  - [ ] Verify metrics show coalescing
  
- [ ] **Verification:**
  ```bash
  # Should show high coalescing rate
  curl http://localhost:3000/metrics | grep tenant_cache_coalesced
  ```

### RBAC Enforcement

- [ ] **Day 1:** Audit endpoints
  - [ ] List all REST endpoints
  - [ ] List all GraphQL resolvers
  - [ ] Map to required permissions
  - [ ] Document in spreadsheet
  
- [ ] **Day 2:** Create middleware
  - [ ] Implement `enforce_permission` middleware
  - [ ] Add to router layer
  - [ ] Handle 403 Forbidden responses
  
- [ ] **Day 3-4:** Add tests
  - [ ] Test unauthorized access returns 403
  - [ ] Test authorized access succeeds
  - [ ] Test per-endpoint permissions
  
- [ ] **Verification:**
  ```bash
  # Should return 403
  curl -X POST http://localhost:3000/api/products \
    -H "Authorization: Bearer <customer-token>" \
    -d '{"title":"Test"}'
  ```

### Rate Limiting

- [ ] **Day 1:** Implement rate limiter
  - [ ] Create `RateLimiter` struct with HashMap
  - [ ] Add sliding window logic
  - [ ] Add middleware
  - [ ] Configure limits in settings
  
- [ ] **Verification:**
  ```bash
  # Should return 429 after 100 requests
  for i in {1..101}; do
    curl http://localhost:3000/api/health
  done
  ```

---

## Phase 2: Stability (3-4 weeks) 🟡

**Цель:** Улучшить reliability и observability

### Event Handler Retry & DLQ

- [ ] **Week 1:** Implement retry logic
  - [ ] Add `EventDispatcherConfig` with retry settings
  - [ ] Implement exponential backoff
  - [ ] Add transient vs permanent error detection
  - [ ] Add tests
  
- [ ] **Week 2:** Implement DLQ
  - [ ] Create `event_dlq` table
  - [ ] Store failed events with error details
  - [ ] Add monitoring for DLQ depth
  - [ ] Add replay mechanism

### GraphQL DataLoaders

- [ ] **Day 1-2:** Create loaders
  - [ ] `NodeLoader`
  - [ ] `NodeTranslationLoader`
  - [ ] `UserLoader`
  - [ ] `ProductLoader`
  
- [ ] **Day 3:** Register loaders
  - [ ] Add to GraphQL schema context
  - [ ] Update resolvers to use loaders
  
- [ ] **Verification:**
  ```bash
  # Query count should be minimal
  # Before: 1 + N queries
  # After: 2 queries (nodes + translations batch)
  ```

### Input Validation

- [ ] **Day 1-2:** Add validator to DTOs
  - [ ] Update `CreateNodeInput`
  - [ ] Update `UpdateNodeInput`
  - [ ] Update Commerce DTOs
  - [ ] Add custom validators
  
- [ ] **Day 3:** Update services
  - [ ] Call `.validate()` before processing
  - [ ] Map validation errors
  - [ ] Add tests

### Index Rebuild with Checkpoints

- [ ] **Week 1:** Implement checkpoints
  - [ ] Create `index_checkpoints` table
  - [ ] Add `save_checkpoint()` method
  - [ ] Add `load_checkpoint()` method
  
- [ ] **Week 2:** Implement batch processing
  - [ ] Stream results from DB
  - [ ] Process in batches
  - [ ] Save checkpoint after each batch
  - [ ] Add progress monitoring

### Integration Tests

- [ ] **Week 1:** Create flow → Index
  - [ ] Test node creation → event → index update
  - [ ] Test node update → event → index update
  - [ ] Test node deletion → event → index removal
  
- [ ] **Week 2:** Product flow → Index
  - [ ] Test product creation → event → index update
  - [ ] Test variant update → event → index update
  - [ ] Test inventory change → event → index update

---

## Phase 3: Production Ready (2-3 weeks) 🟢

**Цель:** Production hardening и полная observability

### Structured Logging

- [ ] **Day 1-2:** Add `#[instrument]` to services
  - [ ] All `NodeService` methods
  - [ ] All `CatalogService` methods
  - [ ] All critical paths
  
- [ ] **Day 3:** Configure production logging
  - [ ] JSON output format
  - [ ] Correlation IDs
  - [ ] Log levels per module

### Module Metrics

- [ ] **Day 1-2:** Add Prometheus metrics
  - [ ] Operation counters per module
  - [ ] Duration histograms
  - [ ] Business metrics (nodes/products created)
  
- [ ] **Day 3:** Create dashboards
  - [ ] Grafana dashboard for content module
  - [ ] Grafana dashboard for commerce module
  - [ ] Alert rules

### Error Handling Standardization

- [ ] **Day 1-2:** Standardize error types
  - [ ] Use thiserror in libraries
  - [ ] Use anyhow in applications
  - [ ] Remove all `.unwrap()`
  - [ ] Add context everywhere

### API Documentation

- [ ] **Day 1-2:** Add OpenAPI examples
  - [ ] Request/response examples
  - [ ] Authentication flow
  - [ ] Error responses
  
- [ ] **Day 3:** Create Postman collection
  - [ ] All endpoints
  - [ ] Environment variables
  - [ ] Test scripts

### Pre-commit Hooks

- [ ] **Day 1:** Create hook script
  - [ ] Format check
  - [ ] Clippy
  - [ ] Fast tests
  
- [ ] **Rollout to team**

### Database Optimization

- [ ] **Day 1-2:** Connection pool tuning
  - [ ] Set max_connections
  - [ ] Set min_connections
  - [ ] Set timeouts
  
- [ ] **Day 3-4:** Query optimization
  - [ ] Add missing indexes
  - [ ] Optimize list queries
  - [ ] Add query result caching

---

## Phase 4: Advanced Features (4+ weeks) 🔵

**Цель:** Long-term improvements

### Type-State Pattern

- [ ] **Week 1:** Design type-state for orders
  - [ ] Create state structs
  - [ ] Design transition methods
  - [ ] Write tests
  
- [ ] **Week 2:** Refactor OrderService
  - [ ] Use type-state
  - [ ] Update tests
  - [ ] Document patterns

### Advanced RBAC

- [ ] **Week 1:** Design ABAC layer
  - [ ] Attribute-based policies
  - [ ] Policy evaluation engine
  - [ ] Tests
  
- [ ] **Week 2:** Integration
  - [ ] Extend RBAC with ABAC
  - [ ] Update middleware
  - [ ] Documentation

### E2E Tests

- [ ] **Week 1-2:** User flows
  - [ ] Registration → Login → Create content
  - [ ] Browse products → Add to cart → Checkout
  - [ ] Admin workflows

### Load Testing

- [ ] **Week 1:** Setup load testing
  - [ ] k6 or Gatling scripts
  - [ ] Realistic workloads
  - [ ] Metrics collection
  
- [ ] **Week 2:** Performance tuning
  - [ ] Profile bottlenecks
  - [ ] Optimize hot paths
  - [ ] Verify improvements

### Flex Module (Optional)

- [ ] **Week 1:** Design
  - [ ] Schema definition tables
  - [ ] Data storage tables
  - [ ] Validation rules
  
- [ ] **Week 2:** Implementation
  - [ ] CRUD operations
  - [ ] Validation
  - [ ] Events
  
- [ ] **Week 3:** Integration
  - [ ] Indexer
  - [ ] GraphQL resolvers
  - [ ] Tests

---

## 📊 Progress Tracking

### Update this section weekly:

**Week of [DATE]:**

Completed:
- [ ] Task 1
- [ ] Task 2

In Progress:
- [ ] Task 3

Blocked:
- [ ] Task 4 (waiting for X)

Next Week:
- [ ] Task 5
- [ ] Task 6

---

## 🎓 Learning Resources Checklist

- [ ] Read: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [ ] Read: [async-graphql Performance](https://async-graphql.github.io/async-graphql/en/performance.html)
- [ ] Read: [Event Sourcing Best Practices](https://www.eventstore.com/blog/what-is-event-sourcing)
- [ ] Watch: [CQRS and Event Sourcing](https://www.youtube.com/watch?v=JHGkaShoyNs)
- [ ] Read: RusToK `ARCHITECTURE_RECOMMENDATIONS.md`
- [ ] Read: RusToK `QUICK_WINS.md`

---

## 🚀 Deployment Checklist

Before going to production:

### Security
- [ ] All RBAC enforcement in place
- [ ] Rate limiting configured
- [ ] Input validation on all endpoints
- [ ] HTTPS enabled
- [ ] Secrets in environment variables
- [ ] Security audit passed

### Performance
- [ ] Load testing completed
- [ ] Database queries optimized
- [ ] Caching strategy validated
- [ ] P99 latency < 200ms
- [ ] Connection pools tuned

### Reliability
- [ ] 50%+ test coverage
- [ ] Integration tests passing
- [ ] Event handlers have retry + DLQ
- [ ] Health checks configured
- [ ] Backups configured

### Observability
- [ ] Structured logging enabled
- [ ] Metrics exported to Prometheus
- [ ] Dashboards created
- [ ] Alerts configured
- [ ] Runbook documented

### Documentation
- [ ] API documentation complete
- [ ] Architecture docs updated
- [ ] Deployment guide written
- [ ] Troubleshooting guide written
- [ ] ADRs captured

---

## 📝 Notes Section

Use this space for team notes, learnings, blockers, etc.

```
[DATE] - [NAME]:
-

[DATE] - [NAME]:
-
```

---

**Version:** 1.0  
**Last Updated:** 11 февраля 2026  
**Next Review:** [DATE]
