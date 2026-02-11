# RusToK - Documentation Index

> **Purpose**: Central index for all project documentation  
> **Last Updated**: February 11, 2026

---

## 📖 Quick Navigation

### 🎯 Start Here
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - Master status, progress, and implementation plan
- **[README.md](README.md)** - Project overview and quick start

### 📋 Planning & Progress
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** ⭐ **MAIN** - Consolidated status and plan
- [PHASE2_COMPLETE.md](PHASE2_COMPLETE.md) - Phase 2 completion summary
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) - Detailed technical plans (legacy)
- [IMPLEMENTATION_CHECKLIST.md](IMPLEMENTATION_CHECKLIST.md) - Phase checklists (legacy)

### 🛠️ Implementation Guides

#### Security & Reliability
- **[docs/rate-limiting.md](docs/rate-limiting.md)** - Rate limiting middleware guide
- **[docs/input-validation.md](docs/input-validation.md)** - Input validation patterns
- [docs/rbac-enforcement.md](docs/rbac-enforcement.md) - RBAC implementation

#### Observability
- **[docs/structured-logging.md](docs/structured-logging.md)** - Logging best practices
- **[docs/grafana-setup.md](docs/grafana-setup.md)** - Monitoring stack setup
- **[docs/module-metrics.md](docs/module-metrics.md)** - Prometheus metrics guide
- [docs/grafana-dashboard-example.json](docs/grafana-dashboard-example.json) - Dashboard template

#### Testing
- [docs/testing-guide.md](docs/testing-guide.md) - Test infrastructure guide
- [docs/test-utils.md](docs/test-utils.md) - rustok-test-utils documentation

#### Architecture
- [docs/event-system.md](docs/event-system.md) - Event-driven architecture
- [docs/cqrs-read-model.md](docs/cqrs-read-model.md) - CQRS and indexing
- [docs/multi-tenant.md](docs/multi-tenant.md) - Multi-tenancy design
- [docs/ROADMAP.md](docs/ROADMAP.md) - Long-term strategy

### 📚 Reference Documentation

#### Configuration
- **[.cargo/config.toml](.cargo/config.toml)** - Cargo aliases (40+ commands)
- [config/](config/) - Application configuration files

#### API Documentation
- `/api/docs` - Swagger UI (when server running)
- [docs/api/](docs/api/) - API specifications

#### Database
- [crates/migration/](crates/migration/) - Database migrations
- [docs/database-schema.md](docs/database-schema.md) - Schema documentation

---

## 📁 Documentation Structure

```
rustok/
├── PROJECT_STATUS.md              ⭐ Main status and plan
├── DOCUMENTATION_INDEX.md         📖 This file
├── PHASE2_COMPLETE.md             ✅ Phase 2 summary
├── IMPLEMENTATION_PLAN.md         📋 Legacy detailed plan
├── IMPLEMENTATION_CHECKLIST.md    ☑️  Legacy checklist
│
├── docs/
│   ├── ROADMAP.md                 🗺️  Long-term strategy
│   │
│   ├── Security & Reliability/
│   │   ├── rate-limiting.md       🛡️  Rate limiting
│   │   ├── input-validation.md    ✅ Validation
│   │   └── rbac-enforcement.md    🔐 RBAC
│   │
│   ├── Observability/
│   │   ├── structured-logging.md  📝 Logging
│   │   ├── grafana-setup.md       📊 Monitoring
│   │   ├── module-metrics.md      📈 Metrics
│   │   └── grafana-dashboard-example.json
│   │
│   ├── Architecture/
│   │   ├── event-system.md        📡 Events
│   │   ├── cqrs-read-model.md     📚 CQRS
│   │   └── multi-tenant.md        🏢 Multi-tenancy
│   │
│   └── Testing/
│       ├── testing-guide.md       🧪 Tests
│       └── test-utils.md          🔧 Utilities
│
└── .cargo/
    └── config.toml                ⚙️  Cargo aliases
```

---

## 🎯 Documentation by Role

### For Developers
**Start**: [PROJECT_STATUS.md](PROJECT_STATUS.md)  
**Then**:
1. [docs/structured-logging.md](docs/structured-logging.md) - Learn logging patterns
2. [docs/input-validation.md](docs/input-validation.md) - Add validation to DTOs
3. [docs/testing-guide.md](docs/testing-guide.md) - Write tests
4. [.cargo/config.toml](.cargo/config.toml) - Use aliases for productivity

### For DevOps
**Start**: [docs/grafana-setup.md](docs/grafana-setup.md)  
**Then**:
1. [docs/module-metrics.md](docs/module-metrics.md) - Understand metrics
2. [docs/rate-limiting.md](docs/rate-limiting.md) - Configure protection
3. [docs/database-schema.md](docs/database-schema.md) - Database operations

### For Product Managers
**Start**: [PROJECT_STATUS.md](PROJECT_STATUS.md)  
**Then**:
1. [docs/ROADMAP.md](docs/ROADMAP.md) - Long-term vision
2. [PHASE2_COMPLETE.md](PHASE2_COMPLETE.md) - Recent deliverables

### For Security Reviewers
**Start**: [docs/rbac-enforcement.md](docs/rbac-enforcement.md)  
**Then**:
1. [docs/rate-limiting.md](docs/rate-limiting.md) - DoS protection
2. [docs/input-validation.md](docs/input-validation.md) - Input sanitization
3. [docs/multi-tenant.md](docs/multi-tenant.md) - Tenant isolation

---

## 📝 Documentation Standards

### File Naming
- Use kebab-case: `structured-logging.md`
- Be descriptive: `grafana-setup.md` not `monitoring.md`
- Status files: `PROJECT_STATUS.md`, `PHASE2_COMPLETE.md`

### Structure
Every guide should include:
1. **Title and Purpose** - What is this about?
2. **Table of Contents** - For long docs
3. **Quick Start** - Get running fast
4. **Detailed Guide** - Step-by-step
5. **Examples** - Real code samples
6. **Troubleshooting** - Common issues
7. **References** - Related docs

### Code Examples
- Include imports
- Show full context
- Add comments for clarity
- Provide both success and error cases

---

## 🔄 Document Lifecycle

### Active Documents (Update Frequently)
- `PROJECT_STATUS.md` - After each milestone
- Phase completion docs - When phase finishes
- Implementation guides - When features change

### Stable Documents (Update Rarely)
- Architecture docs - Only on major changes
- ROADMAP - Quarterly reviews
- Testing guides - When patterns change

### Legacy Documents (Archived)
- Detailed plans superseded by PROJECT_STATUS
- Old status files - kept for historical reference

---

## 🔍 Finding Documentation

### By Topic

**Authentication & Authorization**
- `docs/rbac-enforcement.md`
- `docs/multi-tenant.md`

**API Development**
- `docs/input-validation.md`
- `docs/rate-limiting.md`
- Swagger UI at `/api/docs`

**Events & Messaging**
- `docs/event-system.md`
- `docs/cqrs-read-model.md`

**Monitoring & Debugging**
- `docs/structured-logging.md`
- `docs/grafana-setup.md`
- `docs/module-metrics.md`

**Testing**
- `docs/testing-guide.md`
- `docs/test-utils.md`

**Database**
- `crates/migration/`
- `docs/database-schema.md`

### By Phase

**Phase 1 (Critical Fixes)**
- Event versioning → `docs/event-system.md`
- Test utilities → `docs/test-utils.md`
- RBAC → `docs/rbac-enforcement.md`

**Phase 2 (Stability)**
- Rate limiting → `docs/rate-limiting.md`
- Validation → `docs/input-validation.md`
- Logging → `docs/structured-logging.md`
- Metrics → `docs/grafana-setup.md`
- Aliases → `.cargo/config.toml`

**Phase 3 (Production)** - Coming soon
- Error handling guide
- API documentation
- Database optimization guide

---

## 📚 External Resources

### Rust Ecosystem
- [Tokio Docs](https://tokio.rs/) - Async runtime
- [SeaORM Guide](https://www.sea-ql.org/SeaORM/) - ORM
- [Loco.rs](https://loco.rs/) - Framework
- [Axum Docs](https://docs.rs/axum/) - HTTP framework

### Observability
- [Tracing Docs](https://docs.rs/tracing/) - Structured logging
- [Prometheus Docs](https://prometheus.io/docs/) - Metrics
- [Grafana Docs](https://grafana.com/docs/) - Dashboards

### Testing
- [Criterion Guide](https://bheisler.github.io/criterion.rs/) - Benchmarks
- [cargo-nextest](https://nexte.st/) - Test runner

---

## 🤝 Contributing to Documentation

### Adding New Documentation
1. Create file in appropriate `docs/` subdirectory
2. Follow documentation standards above
3. Add entry to this index
4. Update related documents
5. Submit PR with `docs:` prefix

### Updating Existing Documentation
1. Check if information is still accurate
2. Update examples if API changed
3. Add "Last Updated" date
4. Review related documents for consistency

### Documentation Review Checklist
- [ ] Clear purpose statement
- [ ] Code examples are tested
- [ ] Links are not broken
- [ ] Follows naming conventions
- [ ] Added to this index
- [ ] Cross-references updated

---

## 📞 Documentation Support

**Missing documentation?** Open an issue with `docs` label  
**Found errors?** Submit a PR or open an issue  
**Need clarification?** Check related documents first, then ask

---

**Index Maintained By**: Development team  
**Last Review**: February 11, 2026  
**Next Review**: After Phase 3 completion
