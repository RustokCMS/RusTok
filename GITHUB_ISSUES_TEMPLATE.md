# GitHub Issues Templates — Code Review Recommendations

Эти шаблоны можно скопировать напрямую в GitHub Issues для трекинга улучшений.

---

## 🔴 CRITICAL Priority

### Issue #1: Add Unit Tests Coverage (Critical)

**Title:** Add unit tests for core services (target 30% coverage)

**Labels:** `critical`, `testing`, `tech-debt`

**Description:**

Currently, the project has minimal test coverage (~5%), which is a significant risk for production deployment.

**Scope:**
- [ ] Add unit tests for `NodeService` (rustok-content)
- [ ] Add unit tests for `CatalogService` (rustok-commerce)
- [ ] Add unit tests for RBAC enforcement
- [ ] Add unit tests for Event Bus
- [ ] Create `rustok-test-utils` crate with fixtures

**Expected outcome:**
- Minimum 30% test coverage
- All critical business logic tested
- CI pipeline runs tests automatically

**Resources:**
- See `QUICK_WINS.md` §1 for implementation examples
- Target: 10-15 days

**Acceptance criteria:**
- [ ] `cargo test --workspace` passes
- [ ] Coverage report shows 30%+ coverage
- [ ] All new PRs require tests

---

### Issue #2: Implement Transactional Event Publishing

**Title:** Ensure transaction safety for event publishing (prevent data/event inconsistency)

**Labels:** `critical`, `architecture`, `events`

**Description:**

Current implementation publishes events after transaction commit, which can lead to data inconsistency if event publishing fails.

**Problem:**
```rust
txn.commit().await?;
self.event_bus.publish(event)?;  // ⚠️ Can fail after commit!
```

**Solution:**
Implement transactional event publishing via Outbox pattern.

**Tasks:**
- [ ] Extend `OutboxTransport` with `publish_in_tx()` method
- [ ] Create `TransactionalEventBus` wrapper
- [ ] Refactor all services to use transactional publishing
- [ ] Add integration tests for atomicity

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §1.2 for implementation
- Target: 5-7 days

**Acceptance criteria:**
- [ ] All service methods use transactional event bus
- [ ] Integration tests verify atomicity
- [ ] Outbox relay worker processes pending events

---

### Issue #3: Add Event Schema Versioning

**Title:** Implement event schema versioning for evolution safety

**Labels:** `critical`, `architecture`, `events`

**Description:**

Events don't have explicit schema versions, which will cause issues when evolving event structures.

**Tasks:**
- [ ] Add `schema_version` field to `EventEnvelope`
- [ ] Add `event_type` field for fast filtering
- [ ] Implement `DomainEvent::schema_version()` method
- [ ] Update Outbox and Iggy to store version
- [ ] Document version evolution policy

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §1.1
- Target: 2-3 days

**Acceptance criteria:**
- [ ] All events have version field
- [ ] Outbox stores version with payload
- [ ] Handler can filter by version

---

### Issue #4: Tenant Cache Stampede Protection

**Title:** Add singleflight pattern to prevent cache stampede

**Labels:** `critical`, `performance`, `multi-tenant`

**Description:**

When tenant cache expires, multiple concurrent requests will hit the database simultaneously.

**Tasks:**
- [ ] Implement singleflight/coalescing pattern
- [ ] Add in-flight request tracking
- [ ] Add metrics for stampede prevention
- [ ] Load test with 1000 concurrent requests

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §1.4
- Target: 2-3 days

**Acceptance criteria:**
- [ ] Only one DB query per cache miss
- [ ] Concurrent requests wait for in-flight load
- [ ] Metrics show coalescing effectiveness

---

### Issue #5: Add RBAC Enforcement Middleware

**Title:** Implement global RBAC enforcement layer for all API endpoints

**Labels:** `critical`, `security`, `rbac`

**Description:**

Not all endpoints enforce RBAC permissions, creating security vulnerabilities.

**Tasks:**
- [ ] Audit all controllers for permission checks
- [ ] Create `enforce_permission` middleware
- [ ] Map endpoints to required permissions
- [ ] Add integration tests for authorization
- [ ] Document permission requirements per endpoint

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §5.3
- Target: 3-4 days

**Acceptance criteria:**
- [ ] All endpoints check permissions
- [ ] 403 Forbidden returned for unauthorized access
- [ ] Tests verify enforcement

---

## 🟡 HIGH Priority

### Issue #6: Add Rate Limiting Middleware

**Title:** Implement rate limiting to prevent API abuse

**Labels:** `high`, `security`, `performance`

**Description:**

No rate limiting exists, making the API vulnerable to abuse and DDoS.

**Tasks:**
- [ ] Implement rate limiter (100 req/min default)
- [ ] Add per-tenant rate limits
- [ ] Add Redis-backed distributed rate limiting
- [ ] Return 429 Too Many Requests with Retry-After

**Resources:**
- See `QUICK_WINS.md` §3
- Target: 1 day

**Acceptance criteria:**
- [ ] Rate limiting works per IP/user
- [ ] Redis backend for distributed limiting
- [ ] Configurable limits via settings

---

### Issue #7: Implement GraphQL DataLoaders

**Title:** Add DataLoaders to prevent N+1 queries in GraphQL

**Labels:** `high`, `performance`, `graphql`

**Description:**

GraphQL resolvers fetch related entities individually, causing N+1 query problems.

**Tasks:**
- [ ] Create DataLoader for Nodes
- [ ] Create DataLoader for Translations
- [ ] Create DataLoader for Users
- [ ] Create DataLoader for Products
- [ ] Add query count monitoring

**Resources:**
- See `QUICK_WINS.md` §7
- Target: 2-3 days

**Acceptance criteria:**
- [ ] N+1 queries eliminated
- [ ] Query count reduced by 50%+
- [ ] Performance tests verify improvement

---

### Issue #8: Add Event Handler Retry & DLQ

**Title:** Implement retry logic and dead letter queue for event handlers

**Labels:** `high`, `reliability`, `events`

**Description:**

Event handlers don't retry on transient failures, leading to lost events.

**Tasks:**
- [ ] Add retry config (max_retries, backoff)
- [ ] Implement exponential backoff
- [ ] Create DLQ for failed events
- [ ] Add monitoring for DLQ depth
- [ ] Document error handling strategy

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §1.5
- Target: 3-4 days

**Acceptance criteria:**
- [ ] Transient errors trigger retry
- [ ] Permanent errors go to DLQ
- [ ] Metrics track retry/DLQ rates

---

## 🟢 MEDIUM Priority

### Issue #9: Add Input Validation with validator crate

**Title:** Standardize input validation across all DTOs

**Labels:** `medium`, `validation`, `api`

**Description:**

Validation is inconsistent and spread across services.

**Tasks:**
- [ ] Add `validator` dependency
- [ ] Add validation to all Input DTOs
- [ ] Standardize error messages
- [ ] Add validation tests
- [ ] Document validation rules

**Resources:**
- See `QUICK_WINS.md` §2
- Target: 2-3 days

**Acceptance criteria:**
- [ ] All inputs validated before processing
- [ ] Clear error messages
- [ ] Tests verify validation

---

### Issue #10: Add Structured Logging with tracing

**Title:** Implement structured logging across all services

**Labels:** `medium`, `observability`, `logging`

**Description:**

Logs lack structure and don't always include tenant_id/trace_id context.

**Tasks:**
- [ ] Add `#[instrument]` to all service methods
- [ ] Include tenant_id and user_id in spans
- [ ] Configure JSON output for production
- [ ] Document logging best practices

**Resources:**
- See `QUICK_WINS.md` §4
- Target: 2-3 days

**Acceptance criteria:**
- [ ] All service calls logged
- [ ] Structured JSON in production
- [ ] Trace IDs propagate through system

---

### Issue #11: Add Module-Level Prometheus Metrics

**Title:** Implement Prometheus metrics for all modules

**Labels:** `medium`, `observability`, `metrics`

**Description:**

Only basic metrics exist; need detailed per-module metrics.

**Tasks:**
- [ ] Add operation counters per module
- [ ] Add duration histograms
- [ ] Add business metrics (nodes created, orders placed)
- [ ] Update `/metrics` endpoint
- [ ] Create Grafana dashboard templates

**Resources:**
- See `QUICK_WINS.md` §5
- Target: 2-3 days

**Acceptance criteria:**
- [ ] Metrics exported in Prometheus format
- [ ] Dashboards for key modules
- [ ] Alerts configured

---

### Issue #12: Implement Type-State Pattern for Order Flow

**Title:** Use type-state pattern for order state transitions

**Labels:** `medium`, `architecture`, `type-safety`

**Description:**

Order status is String/enum without compile-time safety for transitions.

**Tasks:**
- [ ] Create type-state structs (OrderPending, OrderPaid, etc.)
- [ ] Implement transition methods
- [ ] Refactor order services
- [ ] Add tests for invalid transitions

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §1.3
- Target: 3-4 days

**Acceptance criteria:**
- [ ] Invalid transitions don't compile
- [ ] All order flows use type-state
- [ ] Tests verify safety

---

### Issue #13: Standardize Error Handling

**Title:** Consistent error handling across all modules

**Labels:** `medium`, `error-handling`, `maintainability`

**Description:**

Mix of anyhow, thiserror, and custom Result types.

**Tasks:**
- [ ] Standardize on thiserror for libraries
- [ ] Use anyhow for applications
- [ ] Remove all `.unwrap()` and `.expect()`
- [ ] Add error context everywhere
- [ ] Document error handling policy

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §2.1
- Target: 1-2 days

**Acceptance criteria:**
- [ ] No unwrap() in production code
- [ ] Consistent error types per layer
- [ ] All errors have context

---

## 🔵 LOW Priority

### Issue #14: Add Pre-commit Hooks

**Title:** Implement pre-commit hooks for code quality

**Labels:** `low`, `devex`, `quality`

**Description:**

No automated checks before commits.

**Tasks:**
- [ ] Create pre-commit hook script
- [ ] Check formatting (cargo fmt)
- [ ] Run clippy
- [ ] Run fast tests
- [ ] Document setup

**Resources:**
- See `QUICK_WINS.md` §6
- Target: 0.5 days

**Acceptance criteria:**
- [ ] Hook prevents bad commits
- [ ] Team members use hooks
- [ ] CI mirrors local checks

---

### Issue #15: Add Cargo Aliases for Common Tasks

**Title:** Create convenient Cargo aliases for development

**Labels:** `low`, `devex`

**Description:**

Common tasks require long commands.

**Tasks:**
- [ ] Create `.cargo/config.toml`
- [ ] Add aliases for dev, test, lint, etc.
- [ ] Document aliases in README
- [ ] Share with team

**Resources:**
- See `QUICK_WINS.md` §10
- Target: 0.1 days

**Acceptance criteria:**
- [ ] Aliases work for all devs
- [ ] Documented in README
- [ ] CI uses same commands

---

### Issue #16: Add API Documentation Examples

**Title:** Improve OpenAPI/GraphQL docs with examples

**Labels:** `low`, `documentation`, `api`

**Description:**

API docs lack examples for common use cases.

**Tasks:**
- [ ] Add request/response examples to OpenAPI
- [ ] Add GraphQL query examples
- [ ] Document authentication flow
- [ ] Create Postman collection

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §8.1
- Target: 1-2 days

**Acceptance criteria:**
- [ ] All endpoints have examples
- [ ] Postman collection available
- [ ] Authentication documented

---

## 📋 Epic: Integration Tests Suite

**Title:** Build comprehensive integration test suite

**Labels:** `epic`, `testing`

**Description:**

Create end-to-end integration tests covering critical user flows.

**Sub-issues:**
- [ ] #17: Test: Create node → Publish → Index → Search
- [ ] #18: Test: Create product → Add to cart → Checkout → Order
- [ ] #19: Test: User registration → Login → RBAC enforcement
- [ ] #20: Test: Tenant creation → Module enable → Operations
- [ ] #21: Test: Event publishing → Outbox → Handlers → Index

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §3.1
- Target: 2 weeks

**Acceptance criteria:**
- [ ] All critical flows tested
- [ ] Tests run in CI
- [ ] Clear test documentation

---

## 📋 Epic: Performance Optimization

**Title:** Optimize database queries and caching

**Labels:** `epic`, `performance`

**Description:**

Improve query performance and caching strategies.

**Sub-issues:**
- [ ] #22: Add connection pool tuning
- [ ] #23: Optimize list queries with pagination
- [ ] #24: Add query result caching
- [ ] #25: Add index rebuild checkpoints
- [ ] #26: Load testing and profiling

**Resources:**
- See `ARCHITECTURE_RECOMMENDATIONS.md` §4
- Target: 2-3 weeks

**Acceptance criteria:**
- [ ] Response time < 100ms p99
- [ ] DB connections optimized
- [ ] Cache hit rate > 80%

---

## 🎯 Milestone: Production Ready v1.0

**Due:** 3 months from now

**Goals:**
- All CRITICAL issues resolved
- 50%+ test coverage
- Performance benchmarks met
- Security audit passed
- Documentation complete

**Issues:**
- All issues #1-5 (Critical)
- Issues #6-8 (High priority)
- Integration tests epic
- Basic performance epic

---

## 📝 Usage Instructions

1. **Create GitHub Project:**
   ```bash
   gh project create "RusToK v1.0 Production Ready"
   ```

2. **Create Issues:**
   - Copy issue templates above
   - Paste into GitHub Issues
   - Assign to appropriate team members

3. **Prioritize:**
   - Start with CRITICAL priority
   - Move to HIGH after critical issues resolved
   - MEDIUM and LOW can be addressed in parallel

4. **Track Progress:**
   - Use GitHub Projects board
   - Weekly standup to review progress
   - Update estimates as needed

5. **Link to PRs:**
   - Use "Fixes #N" in PR descriptions
   - GitHub will auto-close issues

---

## 🤝 Contributing

When working on these issues:

1. **Read the docs:**
   - `ARCHITECTURE_RECOMMENDATIONS.md` for details
   - `QUICK_WINS.md` for implementation examples

2. **Write tests:**
   - Add tests for all new code
   - Aim for 80%+ coverage on new features

3. **Follow conventions:**
   - Run `cargo fmt` and `cargo clippy`
   - Use structured logging
   - Add metrics where appropriate

4. **Update docs:**
   - Update relevant README files
   - Add ADRs for significant decisions
   - Update API documentation

---

**Template Version:** 1.0  
**Generated:** 11 февраля 2026  
**Based on:** Full code review and architecture analysis
