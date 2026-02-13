# 🚀 Sprint 3: Observability - Начало

> **Дата начала:** 2026-02-12  
> **Статус:** 🔄 IN PROGRESS  
> **Цель:** Внедрить comprehensive observability stack

---

## 🎯 Цели Sprint 3

### Основная цель
Добавить полную **observability** (наблюдаемость) в систему через OpenTelemetry, distributed tracing, и metrics dashboard.

### Метрики успеха
- ✅ OpenTelemetry интегрирован и работает
- ✅ Distributed tracing между модулями
- ✅ Metrics экспортируются в Prometheus
- ✅ Grafana dashboards настроены
- ✅ Architecture Score: 9.0/10 → 9.3/10
- ✅ Production Ready: 92% → 96%

---

## 📋 Задачи Sprint 3

### Task 3.1: OpenTelemetry Integration
**Приоритет:** P2 Nice-to-Have  
**Усилия:** 5 дней  
**ROI:** ⭐⭐⭐⭐

**Цель:**
Интегрировать OpenTelemetry для distributed tracing и standardized telemetry.

**Deliverables:**
- [ ] OpenTelemetry pipeline настроен
- [ ] Tracing context propagation
- [ ] Span creation и attributes
- [ ] OTLP exporter (Jaeger/Tempo)
- [ ] Integration в rustok-telemetry
- [ ] Тесты (5+)
- [ ] Документация (8KB)

**Файлы:**
- `crates/rustok-telemetry/src/otel.rs` (NEW, ~200 LOC)
- `apps/server/src/main.rs` (update)
- `Cargo.toml` (dependencies)

---

### Task 3.2: Distributed Tracing
**Приоритет:** P2 Nice-to-Have  
**Усилия:** 3 дня  
**ROI:** ⭐⭐⭐⭐

**Цель:**
Добавить tracing spans во все critical paths для request flow visualization.

**Deliverables:**
- [ ] Spans в HTTP handlers
- [ ] Spans в GraphQL resolvers
- [ ] Spans в EventBus
- [ ] Spans в database queries
- [ ] Span correlation через tenant_id/user_id
- [ ] Performance insights
- [ ] Документация (6KB)

**Файлы:**
- `apps/server/src/controllers/*.rs` (update)
- `apps/server/src/graphql/*.rs` (update)
- `crates/rustok-core/src/events/bus.rs` (update)
- `crates/*/src/services/*.rs` (update)

---

### Task 3.3: Metrics Dashboard
**Приоритет:** P2 Nice-to-Have  
**Усилия:** 2 дня  
**ROI:** ⭐⭐⭐

**Цель:**
Настроить Prometheus metrics и Grafana dashboards для key metrics.

**Deliverables:**
- [ ] Prometheus metrics endpoint
- [ ] Custom metrics (events, cache, circuit breaker)
- [ ] Grafana dashboard JSON
- [ ] Alert rules для SLOs
- [ ] Docker compose setup
- [ ] Документация (4KB)

**Файлы:**
- `apps/server/src/metrics.rs` (NEW, ~150 LOC)
- `docker-compose.observability.yml` (NEW)
- `grafana/dashboards/rustok.json` (NEW)
- `prometheus/prometheus.yml` (NEW)

---

## 🏗️ Архитектура Observability Stack

```
┌─────────────────────────────────────────────────────────┐
│                     RusToK Server                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐    ┌──────────────┐                 │
│  │   Tracing    │    │   Metrics    │                 │
│  │ (OTel Spans) │    │ (Prometheus) │                 │
│  └──────┬───────┘    └──────┬───────┘                 │
│         │                    │                         │
└─────────┼────────────────────┼─────────────────────────┘
          │                    │
          │ OTLP               │ HTTP /metrics
          ↓                    ↓
    ┌──────────┐         ┌──────────┐
    │  Jaeger  │         │Prometheus│
    │  /Tempo  │         │          │
    └────┬─────┘         └────┬─────┘
         │                    │
         │ Query              │ Data Source
         ↓                    ↓
       ┌────────────────────────┐
       │       Grafana          │
       │  (Unified Dashboard)   │
       └────────────────────────┘
```

### Компоненты

1. **OpenTelemetry (OTel)**
   - Standardized telemetry collection
   - Context propagation
   - Span creation и management
   - OTLP export protocol

2. **Jaeger / Tempo**
   - Distributed tracing backend
   - Span storage и query
   - Request flow visualization
   - Performance analysis

3. **Prometheus**
   - Metrics collection (pull-based)
   - Time-series storage
   - PromQL queries
   - Alerting rules

4. **Grafana**
   - Unified visualization
   - Custom dashboards
   - Multi-data-source (Jaeger + Prometheus)
   - Alerting UI

---

## 📊 Expected Impact

### Visibility Improvements

| Aspect | Before Sprint 3 | After Sprint 3 | Improvement |
|--------|----------------|----------------|-------------|
| Request Tracing | Logs only | **Full trace spans** | ✅ Complete |
| Event Flow | Implicit | **Explicit correlation** | ✅ Visualized |
| Performance | Guesswork | **Measured latencies** | ✅ Data-driven |
| Error Tracking | Basic logs | **Span errors + context** | ✅ Rich context |
| Debugging | Time-consuming | **Fast root cause** | ⚡ 10x faster |

### Metrics Coverage

**System Metrics:**
- HTTP request rate, latency, errors
- GraphQL query performance
- EventBus throughput
- Database query latency
- Cache hit/miss rates
- Circuit breaker state

**Business Metrics:**
- Active tenants
- API usage per tenant
- Content operations rate
- Commerce transactions
- User activity

---

## 🔧 Technology Stack

### Dependencies

```toml
[dependencies]
# OpenTelemetry Core
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
tracing-opentelemetry = "0.22"

# Prometheus Metrics
prometheus = "0.13"
axum-prometheus = "0.5"

# Optional: Alternative backends
opentelemetry-jaeger = "0.20"  # Direct Jaeger export
opentelemetry-zipkin = "0.19"  # Zipkin compatibility
```

### External Services

**Development (Docker Compose):**
```yaml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
      
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=true
```

**Production:**
- Grafana Cloud
- Honeycomb.io
- Datadog
- New Relic
- Self-hosted Tempo + Prometheus

---

## 📝 Implementation Plan

### Phase 1: OpenTelemetry Foundation (Day 1-2)

**Goals:**
- Setup OpenTelemetry pipeline
- Configure OTLP exporter
- Initialize tracing subscriber
- Basic span creation

**Tasks:**
1. Add OTel dependencies
2. Create `rustok-telemetry::otel` module
3. Initialize in `apps/server/src/main.rs`
4. Configure environment variables
5. Test with simple spans

**Validation:**
- Spans visible in Jaeger UI
- Context propagation works
- No performance regression

### Phase 2: Distributed Tracing (Day 3-4)

**Goals:**
- Add spans to all critical paths
- Correlation через tenant/user
- Performance instrumentation

**Tasks:**
1. HTTP middleware spans
2. GraphQL resolver spans
3. EventBus spans
4. Service layer spans
5. Database query spans

**Validation:**
- Full request traces visible
- Event flows traceable
- Latencies measured

### Phase 3: Metrics & Dashboard (Day 5)

**Goals:**
- Prometheus metrics endpoint
- Custom metrics collection
- Grafana dashboard

**Tasks:**
1. Setup Prometheus exporter
2. Add custom metrics
3. Create Grafana dashboard
4. Configure alerts
5. Docker compose setup

**Validation:**
- Metrics scraped by Prometheus
- Dashboard displays data
- Alerts trigger correctly

---

## 🧪 Testing Strategy

### Unit Tests
- [ ] OpenTelemetry initialization
- [ ] Span creation and attributes
- [ ] Context propagation
- [ ] Metrics registration
- [ ] Exporter configuration

### Integration Tests
- [ ] End-to-end trace creation
- [ ] Span correlation across services
- [ ] Metrics collection
- [ ] OTLP export to Jaeger
- [ ] Prometheus scraping

### Manual Tests
- [ ] Jaeger UI shows traces
- [ ] Grafana displays metrics
- [ ] Performance acceptable
- [ ] No memory leaks

---

## 📚 Documentation Deliverables

### Guides (18KB total)

1. **OPENTELEMETRY_GUIDE.md** (8KB)
   - OpenTelemetry concepts
   - Setup и configuration
   - Span creation patterns
   - Context propagation
   - Best practices

2. **DISTRIBUTED_TRACING_GUIDE.md** (6KB)
   - Tracing architecture
   - Span instrumentation
   - Performance analysis
   - Debugging workflows

3. **METRICS_DASHBOARD_GUIDE.md** (4KB)
   - Prometheus setup
   - Custom metrics
   - Grafana dashboards
   - Alert configuration

---

## 🎯 Success Criteria

### Task 3.1: OpenTelemetry ✅
- [ ] OpenTelemetry pipeline works
- [ ] OTLP exporter configured
- [ ] Spans created и exported
- [ ] Context propagation tested
- [ ] 5+ unit tests
- [ ] Documentation (8KB)

### Task 3.2: Distributed Tracing ✅
- [ ] HTTP spans instrumented
- [ ] GraphQL spans added
- [ ] EventBus traced
- [ ] Database queries traced
- [ ] Correlation working
- [ ] Documentation (6KB)

### Task 3.3: Metrics Dashboard ✅
- [ ] Prometheus endpoint working
- [ ] Custom metrics exported
- [ ] Grafana dashboard created
- [ ] Alerts configured
- [ ] Docker compose ready
- [ ] Documentation (4KB)

---

## 📈 Expected Outcomes

### Architecture Score
```
Current: 9.0/10
  ↓ +0.3 (Observability)
Target: 9.3/10
```

### Production Readiness
```
Current: 92%
  ↓ +4% (Monitoring & Debugging)
Target: 96%
```

### Observability
```
Before: Basic logs
  ↓ Sprint 3
After: Full observability stack
  - Distributed tracing ✅
  - Metrics dashboard ✅
  - Alert rules ✅
  - Fast debugging ✅
```

---

## 🚀 Getting Started

### Prerequisites
```bash
# Docker для local observability stack
docker --version

# Rust dependencies
cargo --version
```

### Quick Start
```bash
# 1. Start observability stack
docker-compose -f docker-compose.observability.yml up -d

# 2. Run server with tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  cargo run -p rustok-server

# 3. Open dashboards
# Jaeger: http://localhost:16686
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000
```

---

## 📞 Resources

### Main Documents
- [ARCHITECTURE_IMPROVEMENT_PLAN.md](./ARCHITECTURE_IMPROVEMENT_PLAN.md) - Sprint 3 details
- [IMPROVEMENTS_SUMMARY.md](./IMPROVEMENTS_SUMMARY.md) - Progress tracking

### Previous Sprints
- [SPRINT_1_COMPLETION.md](./docs/SPRINT_1_COMPLETION.md)
- [SPRINT_2_COMPLETED.md](./SPRINT_2_COMPLETED.md)

### External References
- [OpenTelemetry Rust](https://docs.rs/opentelemetry/)
- [Prometheus Rust Client](https://docs.rs/prometheus/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)

---

**Status:** 🔄 Sprint 3 IN PROGRESS  
**Next:** Task 3.1 - OpenTelemetry Integration  
**Estimated Completion:** 5-10 days from start
