# RusToK Server — Loco.rs Feature Support Analysis

**Date:** 2026-02-13  
**Loco.rs Version:** Latest compatible with Rust 1.80+  
**Status:** Core features implemented, some advanced features pending

---

## ✅ Implemented Loco.rs Features

### 1. Application Lifecycle Hooks (`app.rs`)

| Feature | Status | Implementation |
|---------|--------|----------------|
| `app_name()` | ✅ | Returns CARGO_PKG_NAME |
| `app_version()` | ✅ | Returns version + build SHA |
| `boot()` | ✅ | Creates app with Migrator |
| `routes()` | ✅ | All controllers registered |
| `after_routes()` | ✅ | Event runtime, tenant cache, registry, Alloy scripting |
| `truncate()` | ✅ | Stub implementation |
| `register_tasks()` | ✅ | Cleanup task registered |
| `initializers()` | ✅ | Telemetry initializer registered |
| `connect_workers()` | ✅ | Outbox relay worker spawned |
| `seed()` | ✅ | Seeds module with dev/test/minimal data |
| `shutdown()` | ✅ | Graceful shutdown with worker cleanup |

### 2. Configuration System

| Feature | Status | File |
|---------|--------|------|
| Environment-based config | ✅ | `development.yaml`, `test.yaml` |
| Logger configuration | ✅ | Level, format, backtrace |
| Server configuration | ✅ | Binding, port |
| Database configuration | ✅ | URI, connections, migration |
| Auth configuration | ✅ | JWT secret, expiration |
| Custom settings | ✅ | `settings.rustok.*` section |

### 3. Controllers & Routing

| Feature | Status | Implementation |
|---------|--------|----------------|
| REST controllers | ✅ | Health, Auth, Metrics, Swagger, Pages |
| GraphQL controller | ✅ | `/graphql` endpoint |
| Module controllers | ✅ | Commerce, Content, Blog, Forum |
| Middleware integration | ✅ | Tenant resolution, rate limiting |

### 4. Models & ORM

| Feature | Status | Implementation |
|---------|--------|----------------|
| Sea-ORM integration | ✅ | Full integration |
| Migrations | ✅ | `migration/` crate |
| Entities | ✅ | Users, Tenants, Sessions, etc. |

### 5. Authentication

| Feature | Status | Implementation |
|---------|--------|----------------|
| JWT auth | ✅ | `config.auth.jwt` |
| Argon2 password hashing | ✅ | Auth service |
| Session management | ✅ | Sessions model |
| RBAC | ✅ | `rustok-rbac` crate |

### 6. Middleware

| Feature | Status | Implementation |
|---------|--------|----------------|
| Custom middleware | ✅ | Tenant resolution |
| Rate limiting | ✅ | Custom implementation |
| Cache layers | ✅ | Tenant cache v2/v3 with moka |
| Axum middleware | ✅ | `after_routes` layering |

### 7. Background Processing

| Feature | Status | Implementation |
|---------|--------|----------------|
| Outbox relay worker | ✅ | Event transport factory |
| Graceful shutdown | ✅ | Worker handle cleanup |
| Custom background tasks | ⚠️ | Outbox only, no general worker queue |

### 8. Event System

| Feature | Status | Implementation |
|---------|--------|----------------|
| Event bus | ✅ | `EventBus` with backpressure |
| Transactional events | ✅ | `TransactionalEventBus` |
| Event validation | ✅ | 50+ domain events validated |
| Multiple transports | ✅ | Memory, Iggy |

### 9. Testing

| Feature | Status | Implementation |
|---------|--------|----------------|
| Unit tests | ✅ | Across all modules |
| Integration tests | ✅ | `tests/integration/` |
| Test config | ✅ | `test.yaml` |
| Loco testing features | ✅ | `testing` feature enabled |

---

## ❌ Missing Loco.rs Features

### 1. Workers (Background Jobs)

**Priority:** Medium  
**Use Case:** Background processing for emails, exports, imports

**What's Missing:**
- Worker trait implementations
- Job queue (Redis/SQLite backed)
- Job scheduling
- Worker monitoring

**Loco.rs Way:**
```rust
// In app.rs
fn register_workers(queue: &Queue) -> Result<()> {
    queue.register(DownloadWorker)?;
    queue.register(EmailWorker)?;
    Ok(())
}

// Worker implementation
#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadWorkerArgs {
    pub url: String,
}

pub struct DownloadWorker;
#[async_trait]
impl Worker for DownloadWorker {
    type Args = DownloadWorkerArgs;
    
    async fn perform(&self, args: DownloadWorkerArgs) -> Result<()> {
        // Background work
        Ok(())
    }
}
```

**Current RusToK Alternative:**
- Outbox relay worker for events
- Custom task spawning for specific needs

### 2. Mailers (Email)

**Priority:** Low-Medium  
**Use Case:** Transactional emails, notifications

**What's Missing:**
- Mailer trait implementations
- Email template system
- SMTP integration
- Multi-provider support (SendGrid, AWS SES, etc.)

**Loco.rs Way:**
```rust
// In app.rs
async fn after_context(ctx: &AppContext) -> Result<AppContext> {
    ctx.add_mailer(Box::new(SmtpMailer::new()))?;
    Ok(ctx)
}

// Mailer implementation
pub struct WelcomeMailer;
impl Mailer for WelcomeMailer {
    fn subject(&self) -> String { "Welcome!".to_string() }
    fn body(&self) -> String { /* template */ }
}
```

**Current RusToK Alternative:**
- No email system implemented
- Would need custom implementation

### 3. Storage (File Uploads)

**Priority:** Low  
**Use Case:** File uploads, asset storage

**What's Missing:**
- Storage abstraction
- Local disk storage
- S3-compatible storage
- File upload handlers

**Loco.rs Way:**
```rust
// Configuration
storage:
  type: s3
  bucket: my-bucket
  region: us-east-1

// Usage
let storage = ctx.storage;
storage.upload(path, bytes).await?;
```

**Current RusToK Alternative:**
- Would need custom implementation
- No file upload features currently

### 4. Task System ✅ IMPLEMENTED

**Priority:** Low  
**Use Case:** One-off background tasks, CLI tasks

**Implementation:** `apps/server/src/tasks/`

**Available Tasks:**

| Task | Description | Usage |
|------|-------------|-------|
| `cleanup` | Remove old sessions and cache | `cargo loco task --name cleanup --args "sessions"` |

**Targets:**
- `sessions` - Clean expired sessions
- `cache` - Clear temporary cache
- (empty) - Full cleanup

**Loco.rs Pattern:**
```rust
// In app.rs
fn register_tasks(tasks: &mut Tasks) {
    tasks::register(tasks)?;
}

// Task implementation
#[async_trait]
impl Task for CleanupTask {
    fn task_name(&self) -> String { "cleanup".to_string() }
    async fn run(&self, ctx: &AppContext, args: &str) -> Result<()> {
        // Task logic
        Ok(())
    }
}
```

**Current RusToK Status:**
- ✅ `register_tasks` implemented
- ✅ Cleanup task available
- ✅ CLI task runner enabled

### 5. Initializers ✅ IMPLEMENTED

**Priority:** Low  
**Use Case:** Third-party service initialization

**Implementation:** `apps/server/src/initializers/`

**Available Initializers:**

| Initializer | Purpose |
|-------------|---------|
| `TelemetryInitializer` | OpenTelemetry and tracing setup |

**Loco.rs Pattern:**
```rust
async fn initializers(ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    initializers::create(ctx).await
}

// Implementation
pub struct TelemetryInitializer;
#[async_trait]
impl Initializer for TelemetryInitializer {
    fn name(&self) -> String { "telemetry".to_string() }
    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        // Setup logic
        Ok(())
    }
}
```

**Current RusToK Status:**
- ✅ `initializers()` implemented
- ✅ Telemetry initializer registered
- ✅ Proper separation of concerns

---

## 🔧 Recommendations

### Short Term (Low Effort, High Value)

1. **Add Basic Task Support**
   - Implement a few common tasks (cleanup, cache warmup)
   - Enable `cargo loco task` CLI

2. **Document Current Architecture**
   - Explain why Outbox pattern is used instead of general workers
   - Document event-driven approach

### Medium Term (Medium Effort)

1. **Worker Queue System**
   - Implement for background processing needs
   - Start with Redis-backed queue
   - Use for: exports, imports, bulk operations

2. **Email System**
   - SMTP mailer for transactional emails
   - Template system integration

### Long Term (Higher Effort)

1. **Storage Abstraction**
   - When file uploads become a requirement
   - S3-compatible storage for cloud deployments

2. **Full Initializer System**
   - Move third-party init from `after_routes` to initializers
   - Better separation of concerns

---

## 📊 Feature Coverage Summary

| Category | Implemented | Missing | Coverage |
|----------|-------------|---------|----------|
| Core App | 10/11 | 1 | 91% |
| Configuration | 6/6 | 0 | 100% |
| Controllers | 5/5 | 0 | 100% |
| Models/ORM | 3/3 | 0 | 100% |
| Auth | 4/4 | 0 | 100% |
| Middleware | 4/4 | 0 | 100% |
| Background | 2/3 | 1 | 67% |
| Events | 4/4 | 0 | 100% |
| Testing | 4/4 | 0 | 100% |
| **Workers** | 0/4 | 4 | 0% |
| **Mailers** | 0/3 | 3 | 0% |
| **Storage** | 0/3 | 3 | 0% |
| **Tasks** | 2/2 | 0 | 100% ✅ |
| **Initializers** | 1/1 | 0 | 100% ✅ |
| **TOTAL** | **45/57** | **12** | **79%** |

---

## 💡 Design Decisions

### Why Some Features Are Not Implemented

1. **Workers**: RusToK uses event-driven architecture with Outbox pattern instead of traditional job queues. This provides:
   - Better reliability (events are transactional)
   - Better observability
   - CQRS-lite compatibility

2. **Mailers**: Not a core requirement for headless CMS. When needed:
   - Can use external services via webhooks
   - Can be added as module-specific feature

3. **Storage**: Headless platforms typically don't handle file storage directly:
   - Assets served via CDN
   - Uploads handled by dedicated services
   - Can be added when needed

### Architecture Philosophy

RusToK prioritizes:
- ✅ Event-driven over job queues
- ✅ External services over built-in features
- ✅ Headless API over monolithic features
- ✅ Module-specific over framework-wide

---

## 🔗 Related Documentation

- [Loco.rs Docs Index](./loco/README.md)
- [Library Stack](./library-stack.md)
- [Server README](../README.md)
- [app.rs Implementation](../src/app.rs)

---

*Last Updated: 2026-02-13*  
*Update: Added Tasks and Initializers support*
