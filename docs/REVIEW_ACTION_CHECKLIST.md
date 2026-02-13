# 📋 Architecture Review — Action Checklist

> Пошаговый чеклист для имплементации рекомендаций  
> **Дата создания:** 2026-02-12  
> **Estimated time to completion:** 3-4 weeks

---

## 🔴 SPRINT 1: Critical Fixes (Week 1)

**Goal:** Закрыть все P0 security и reliability issues

### Task 1.1: Event Validation Framework ⚡ HIGH PRIORITY

**Estimated time:** 1 day

- [ ] Создать `crates/rustok-core/src/events/validation.rs`
- [ ] Добавить `ValidateEvent` trait
- [ ] Имплементировать валидацию для всех `DomainEvent` вариантов
- [ ] Добавить валидацию в `TransactionalEventBus::publish_in_tx()`
- [ ] Написать unit tests для всех edge cases
- [ ] Добавить integration test для invalid events

**Files to modify:**
- `crates/rustok-core/src/events/validation.rs` (NEW)
- `crates/rustok-core/src/events/types.rs`
- `crates/rustok-outbox/src/transactional.rs`
- `crates/rustok-core/src/events/mod.rs`

**Test coverage target:** 95%+

**Acceptance criteria:**
- [ ] Все events проходят валидацию перед сохранением
- [ ] Invalid events rejected с понятной ошибкой
- [ ] Zero regression в существующих тестах

---

### Task 1.2: Tenant Identifier Sanitization ⚡ CRITICAL SECURITY

**Estimated time:** 1 day

- [ ] Создать `crates/rustok-core/src/tenant_validation.rs`
- [ ] Добавить `TenantIdentifierValidator`
- [ ] Имплементировать regex patterns для slug/UUID/host
- [ ] Добавить RESERVED_SLUGS whitelist
- [ ] Интегрировать в `apps/server/src/middleware/tenant.rs`
- [ ] Написать security tests

**Files to modify:**
- `crates/rustok-core/src/tenant_validation.rs` (NEW)
- `apps/server/src/middleware/tenant.rs`
- `crates/rustok-core/src/lib.rs`

**Security checklist:**
- [ ] SQL injection protected
- [ ] XSS attempts blocked
- [ ] Reserved names blocked
- [ ] Length limits enforced
- [ ] Invalid characters rejected

**Acceptance criteria:**
- [ ] All malicious inputs rejected
- [ ] Valid inputs sanitized properly
- [ ] Security audit passed

---

### Task 1.3: EventDispatcher Rate Limiting ⚡ HIGH PRIORITY

**Estimated time:** 2 days

- [ ] Создать `crates/rustok-core/src/events/backpressure.rs`
- [ ] Имплементировать `BackpressureController`
- [ ] Добавить `BackpressureConfig` в `DispatcherConfig`
- [ ] Интегрировать в `EventDispatcher`
- [ ] Добавить metrics для queue depth
- [ ] Написать load tests

**Files to modify:**
- `crates/rustok-core/src/events/backpressure.rs` (NEW)
- `crates/rustok-core/src/events/handler.rs`
- `crates/rustok-core/src/events/mod.rs`

**Performance targets:**
- [ ] Handle 10K events/sec without OOM
- [ ] Backpressure activates at 80% capacity
- [ ] Graceful degradation under load

**Acceptance criteria:**
- [ ] No OOM under high load
- [ ] Backpressure metrics exposed
- [ ] Load tests passing

---

### Task 1.4: EventBus Consistency Audit 📋 VERIFICATION

**Estimated time:** 0.5 day

- [ ] Audit all services for EventBus usage
- [ ] Verify all use `TransactionalEventBus`
- [ ] Check all `publish_in_tx` inside transactions
- [ ] Document any edge cases
- [ ] Update module READMEs

**Services to audit:**
- [x] `rustok-content` ✅ (already uses TransactionalEventBus)
- [x] `rustok-commerce` ✅ (already uses TransactionalEventBus)
- [x] `rustok-blog` ✅ (already uses TransactionalEventBus)
- [x] `rustok-forum` ✅ (already uses TransactionalEventBus)
- [x] `rustok-pages` ✅ (already uses TransactionalEventBus)
- [ ] Custom services in `apps/server` (need verification)

**Acceptance criteria:**
- [ ] All services use TransactionalEventBus
- [ ] No direct EventBus usage in services
- [ ] Documentation updated

---

## 🟠 SPRINT 2: Simplification (Week 2-3)

**Goal:** Упростить сложные абстракции и улучшить maintainability

### Task 2.1: Simplified Tenant Resolver with Moka 🔧 REFACTORING

**Estimated time:** 3 days

- [ ] Создать `apps/server/src/middleware/tenant_v2.rs`
- [ ] Имплементировать `SimplifiedTenantResolver`
- [ ] Migrate from custom cache to `moka`
- [ ] Remove `TenantCacheInfrastructure` complexity
- [ ] Add metrics endpoints
- [ ] Performance benchmarks
- [ ] Gradual rollout (feature flag)

**Files to modify:**
- `apps/server/src/middleware/tenant_v2.rs` (NEW)
- `apps/server/src/middleware/tenant.rs` (DEPRECATE)
- `apps/server/src/middleware/mod.rs`

**Lines of code:**
- Current: ~580 lines
- Target: ~150 lines
- Reduction: 74%

**Performance targets:**
- [ ] P95 latency < 10ms
- [ ] Cache hit rate > 95%
- [ ] Zero stampede issues

**Acceptance criteria:**
- [ ] Performance equal or better than current
- [ ] All tests passing
- [ ] Gradual rollout successful
- [ ] Old implementation removed

---

### Task 2.2: Circuit Breaker Implementation 🛡️ RELIABILITY

**Estimated time:** 2 days

- [ ] Создать `crates/rustok-core/src/circuit_breaker.rs`
- [ ] Имплементировать `CircuitBreaker`
- [ ] Add `CircuitBreakerConfig`
- [ ] Wrap Redis calls
- [ ] Wrap Iggy calls
- [ ] Add fallback handlers
- [ ] Metrics and alerting

**Files to modify:**
- `crates/rustok-core/src/circuit_breaker.rs` (NEW)
- `crates/rustok-core/src/cache.rs`
- `crates/rustok-iggy/src/transport.rs`

**Targets:**
- [ ] Redis failures don't cascade
- [ ] Automatic recovery after timeout
- [ ] Metrics exposed to Prometheus

**Acceptance criteria:**
- [ ] Chaos testing passed
- [ ] Circuit opens on failures
- [ ] Auto-recovery works
- [ ] Fallbacks functional

---

### Task 2.3: Type-Safe State Machines 🔒 TYPE SAFETY

**Estimated time:** 2 days

- [ ] Identify critical state machines (Order, Product)
- [ ] Implement typestate pattern for Order
- [ ] Implement typestate pattern for Product
- [ ] Update services to use typed states
- [ ] Remove runtime state validation
- [ ] Add compile-time tests

**Files to modify:**
- `crates/rustok-commerce/src/domain/order.rs`
- `crates/rustok-commerce/src/domain/product.rs`
- `crates/rustok-commerce/src/services/order_service.rs`

**Acceptance criteria:**
- [ ] Invalid transitions don't compile
- [ ] All state changes type-safe
- [ ] No runtime state errors

---

### Task 2.4: Error Handling Policy 📋 STANDARDIZATION

**Estimated time:** 1 day

- [ ] Создать `crates/rustok-core/src/error_policy.rs`
- [ ] Define `ErrorPolicy` trait
- [ ] Implement `DefaultErrorPolicy`
- [ ] Add retry strategies
- [ ] Add fallback handlers
- [ ] Document error handling guidelines

**Files to modify:**
- `crates/rustok-core/src/error_policy.rs` (NEW)
- `crates/rustok-core/src/error.rs`

**Acceptance criteria:**
- [ ] Consistent error handling across modules
- [ ] Documented retry policies
- [ ] Tests for all error scenarios

---

### Task 2.5: Test Coverage Increase 🧪 QUALITY

**Estimated time:** Ongoing throughout sprint

**Current:** 31%  
**Target:** 40%  
**Stretch:** 50%

**Priority areas:**
1. [ ] Event validation (target: 95%+)
2. [ ] Tenant isolation (target: 100%)
3. [ ] State machines (target: 90%+)
4. [ ] Error handling (target: 80%+)
5. [ ] Circuit breakers (target: 85%+)

**Test types:**
- [ ] Unit tests for new code
- [ ] Integration tests for critical paths
- [ ] Security tests for inputs
- [ ] Performance tests for bottlenecks

**Acceptance criteria:**
- [ ] Overall coverage ≥ 40%
- [ ] Critical paths ≥ 80%
- [ ] No new code without tests

---

## 🟡 SPRINT 3: Observability & Polish (Week 4)

**Goal:** Production-ready observability and final polish

### Task 3.1: OpenTelemetry Integration 📊 OBSERVABILITY

**Estimated time:** 2 days

- [ ] Add `opentelemetry` dependencies
- [ ] Update `rustok-telemetry`
- [ ] Integrate tracing in services
- [ ] Add span context to events
- [ ] Configure exporters (Jaeger/Tempo)
- [ ] Create sample traces
- [ ] Documentation

**Files to modify:**
- `crates/rustok-telemetry/src/tracing.rs`
- `crates/rustok-outbox/src/transactional.rs`
- `Cargo.toml`

**Targets:**
- [ ] All critical paths traced
- [ ] Event flows traceable end-to-end
- [ ] Performance overhead < 5%

**Acceptance criteria:**
- [ ] Traces visible in Jaeger/Grafana
- [ ] Event correlation working
- [ ] Documentation complete

---

### Task 3.2: Integration Tests Suite 🧪 QUALITY

**Estimated time:** 2 days

- [ ] Create `apps/server/tests/integration/`
- [ ] Tenant isolation tests
- [ ] Event flow tests
- [ ] CRUD operation tests
- [ ] Error scenario tests
- [ ] Performance benchmarks

**Test scenarios:**
1. [ ] Tenant cannot access other tenant's data
2. [ ] Events persist transactionally
3. [ ] Index updates from events
4. [ ] Circuit breaker behavior
5. [ ] Backpressure handling

**Acceptance criteria:**
- [ ] 20+ integration tests
- [ ] All critical paths covered
- [ ] CI/CD integrated

---

### Task 3.3: Performance Benchmarks 📈 OPTIMIZATION

**Estimated time:** 1 day

- [ ] Setup benchmark harness
- [ ] Tenant resolution benchmarks
- [ ] Event dispatch benchmarks
- [ ] Database query benchmarks
- [ ] Cache performance benchmarks
- [ ] Document baselines

**Targets:**
- Tenant resolution: < 10ms P95
- Event dispatch: > 5K events/sec
- Database queries: < 50ms P95
- Cache hit rate: > 95%

**Acceptance criteria:**
- [ ] Benchmarks established
- [ ] Baselines documented
- [ ] Regression tests in CI

---

### Task 3.4: Security Audit 🔒 SECURITY

**Estimated time:** 1 day

- [ ] Review input validation
- [ ] Check tenant isolation
- [ ] Audit authentication flows
- [ ] Review RBAC implementation
- [ ] Check for SQL injection vectors
- [ ] Test XSS prevention
- [ ] Document security measures

**Security checklist:**
- [ ] Input validation complete
- [ ] Tenant isolation verified
- [ ] No SQL injection vectors
- [ ] XSS prevention working
- [ ] Auth flows secure
- [ ] RBAC enforced

**Acceptance criteria:**
- [ ] Security score ≥ 95%
- [ ] All critical issues fixed
- [ ] Security documentation updated

---

### Task 3.5: Documentation Update 📚 DOCUMENTATION

**Estimated time:** 1 day

- [ ] Update ARCHITECTURE_GUIDE.md
- [ ] Update module READMEs
- [ ] Add runbook for operations
- [ ] Document new patterns
- [ ] Update API documentation
- [ ] Create migration guide

**Documents to update:**
- [ ] ARCHITECTURE_GUIDE.md
- [ ] RUSTOK_MANIFEST.md
- [ ] Module READMEs
- [ ] API docs
- [ ] Runbooks

**Acceptance criteria:**
- [ ] All new features documented
- [ ] Migration guide complete
- [ ] Runbooks reviewed by ops

---

## 📊 Progress Tracking

### Week 1 Metrics
- [ ] P0 issues closed: __/4
- [ ] Test coverage: __%
- [ ] Security score: __%

### Week 2 Metrics
- [ ] P1 issues closed: __/5
- [ ] Test coverage: __%
- [ ] Lines of code reduced: __

### Week 3 Metrics
- [ ] Complexity reduced: __%
- [ ] Performance improved: __%
- [ ] Documentation updated: __%

### Week 4 Metrics
- [ ] Observability score: __%
- [ ] Production readiness: __%
- [ ] Team confidence: __/10

---

## 🎯 Definition of Done

### Per Task
- [ ] Code implemented
- [ ] Tests written (≥80% coverage)
- [ ] Documentation updated
- [ ] Code review passed
- [ ] CI/CD passing
- [ ] Performance benchmarks met

### Per Sprint
- [ ] All tasks completed
- [ ] Sprint metrics achieved
- [ ] Demo to stakeholders
- [ ] Retrospective held

### Overall Project
- [ ] All P0 issues closed
- [ ] 50%+ P1 issues closed
- [ ] Test coverage ≥ 40%
- [ ] Security score ≥ 95%
- [ ] Production readiness ≥ 95%
- [ ] Documentation complete
- [ ] Team sign-off

---

## 🚀 Deployment Plan

### Phase 1: Critical Fixes (Week 1)
- Deploy event validation
- Deploy tenant sanitization
- Deploy rate limiting
- **Risk:** Low (additive changes)

### Phase 2: Refactoring (Week 2-3)
- Feature flag tenant_v2
- Gradual rollout (10% → 50% → 100%)
- Monitor metrics
- **Risk:** Medium (behavior changes)

### Phase 3: Polish (Week 4)
- Enable OpenTelemetry
- Deploy final tests
- Security hardening
- **Risk:** Low (observability only)

---

## 📞 Contacts & Support

**Architecture questions:**  
Review: [ARCHITECTURE_REVIEW_2026-02-12.md](./ARCHITECTURE_REVIEW_2026-02-12.md)

**Implementation help:**  
Guide: [REFACTORING_ROADMAP.md](./REFACTORING_ROADMAP.md)

**Module-specific:**  
Reference: [MODULE_IMPROVEMENTS.md](./MODULE_IMPROVEMENTS.md)

---

**Status:** 🔴 Not Started  
**Next Review:** Weekly standup  
**Completion Target:** 2026-03-12
