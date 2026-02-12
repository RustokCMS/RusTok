# Implementation of Module Installation System

## Overview

This document describes the foundational implementation of the WordPress/NodeBB-style module installation system for RusToK.

## What Was Implemented

### 1. Module Manifest (`modules.toml`)

Located at project root, defines the module composition:

```toml
schema = 1
app = "rustok-server"

[build]
target = "x86_64-unknown-linux-gnu"
profile = "release"
deployment_profile = "monolith"

[modules]
content = { crate = "rustok-content", source = "path", path = "crates/rustok-content" }
commerce = { crate = "rustok-commerce", source = "path", path = "crates/rustok-commerce" }
# ... etc

[settings]
default_enabled = ["content", "commerce", "pages"]
```

### 2. XTask Build Automation (`xtask/`)

Code generator for ModuleRegistry:

```bash
# Generate registry code from manifest
cargo xtask generate-registry

# Validate manifest schema
cargo xtask validate-manifest

# List configured modules
cargo xtask list-modules
```

Generates `apps/server/src/modules/generated.rs` with automatic module registration.

### 3. Database Models

#### Build Model (`apps/server/src/models/build.rs`)

Tracks build lifecycle from request to deployment:

| Field | Description |
|-------|-------------|
| `id` | UUID primary key |
| `status` | queued/running/success/failed/cancelled |
| `stage` | pending/checkout/build/test/deploy/complete |
| `progress` | 0-100 percentage |
| `profile` | monolith/headless deployment |
| `manifest_hash` | SHA256 hash of module config (for deduplication) |
| `modules_delta` | JSON of added/removed/changed modules |
| `release_id` | Associated release (on success) |

#### Release Model (`apps/server/src/models/release.rs`)

Immutable deployment artifacts:

| Field | Description |
|-------|-------------|
| `id` | Generated release ID (e.g., `rel_20250212_120000`) |
| `status` | pending/deploying/active/rolled_back/failed |
| `build_id` | Source build |
| `environment` | prod/staging/etc |
| `container_image` | Docker image URL |
| `previous_release_id` | Rollback chain link |
| `modules` | JSON list of modules in release |

### 4. Build Service (`apps/server/src/services/build_service.rs`)

Core business logic for module installation:

```rust
pub struct BuildService {
    db: DatabaseConnection,
}

// Key methods:
- request_build()    // Create and queue build
- get_build()        // Get build status
- list_builds()      // List recent builds
- create_release()   // Create release from build
- rollback()         // Rollback to previous release
```

### 5. Database Migration

`m20250212_000001_create_builds_and_releases.rs` creates:
- `builds` table with indexes on status and manifest_hash
- `releases` table with indexes on status and build_id

## Architecture Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│  Admin UI   │────▶│  BuildService │────▶│  builds table   │
│ Install btn │     │ request_build()│     │  (queued)       │
└─────────────┘     └──────────────┘     └─────────────────┘
                                                   │
                           ┌───────────────────────┘
                           ▼
                    ┌─────────────────┐
                    │ Build Runner    │
                    │ (async worker)  │
                    └─────────────────┘
                           │
                           ▼
                    ┌─────────────────┐
                    │  releases table │
                    │  (on success)   │
                    └─────────────────┘
```

## Integration Points

### For Admin API (to be implemented):

```rust
// POST /admin/builds
async fn create_build(
    State(service): State<BuildService>,
    Json(request): Json<BuildRequest>,
) -> Result<Json<Build>, Error> {
    let build = service.request_build(request).await?;
    Ok(Json(build))
}

// GET /admin/builds/:id
async fn get_build_status(
    State(service): State<BuildService>,
    Path(id): Path<Uuid>,
) -> Result<Json<Build>, Error> {
    let build = service.get_build(id).await?;
    Ok(Json(build))
}

// POST /admin/builds/:id/deploy
async fn deploy_build(
    State(service): State<BuildService>,
    Path(id): Path<Uuid>,
) -> Result<Json<Release>, Error> {
    let release = service.create_release(id, "prod", modules).await?;
    Ok(Json(release))
}

// POST /admin/builds/:id/rollback
async fn rollback(
    State(service): State<BuildService>,
    Path(id): Path<String>,
) -> Result<Json<Release>, Error> {
    let release = service.rollback(&id).await?;
    Ok(Json(release))
}
```

### For Build Runner (to be implemented):

```rust
// Async worker that processes BuildRequested events
pub struct BuildRunner {
    service: BuildService,
}

impl BuildRunner {
    pub async fn process_build(&self, build_id: Uuid) -> Result<()> {
        // 1. Update status to Running
        self.service.update_build_status(build_id, Running, Some(Checkout), Some(0)).await?;
        
        // 2. Git checkout
        // 3. Generate registry
        // 4. cargo build --locked
        // 5. Create Docker image
        // 6. Update progress
        // 7. Mark as Success
        
        Ok(())
    }
}
```

## Next Steps for Full Implementation

### Phase 1: Admin API (1 week)
- [ ] Create admin controllers for builds/releases
- [ ] Add RBAC protection (only admins can trigger builds)
- [ ] Integrate BuildService into GraphQL

### Phase 2: Build Runner (1-2 weeks)
- [ ] Implement async build worker
- [ ] Docker build integration
- [ ] Progress tracking and logs

### Phase 3: Admin UI (1 week)
- [ ] Module marketplace page
- [ ] Build status modal
- [ ] Release history

### Phase 4: Deploy Integration (1 week)
- [ ] K8s/Docker Compose deployment
- [ ] Blue-green deployment
- [ ] Smoke tests

## Files Created/Modified

### New Files:
- `modules.toml` - Module manifest
- `xtask/Cargo.toml` - Build automation
- `xtask/src/main.rs` - Registry generator
- `apps/server/src/models/build.rs` - Build entity
- `apps/server/src/models/release.rs` - Release entity
- `apps/server/src/models/mod.rs` - Models module
- `apps/server/src/services/build_service.rs` - Build orchestration
- `apps/server/migration/src/m20250212_000001_create_builds_and_releases.rs` - Migration

### Modified Files:
- `.cargo/config.toml` - Added xtask aliases
- `Cargo.toml` - Added xtask to workspace
- `apps/server/migration/src/lib.rs` - Registered new migration

## Usage Example

```bash
# 1. Add module to manifest
# Edit modules.toml, add new module

# 2. Validate manifest
cargo xtask validate-manifest

# 3. Generate registry (in CI/CD)
cargo xtask generate-registry

# 4. Build project
cargo build --release

# 5. Run migrations
cargo loco db migrate

# 6. Start server
cargo run -p rustok-server
```

## References

- Original spec: `docs/modules/module-rebuild-plan.md`
- API blueprint: `docs/modules/module-manifest.md`
- Registry docs: `docs/modules/module-registry.md`
