# RusToK Quick Start Guide

## Getting Started

### Prerequisites

- Rust 1.80+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 14+ (or use Docker Compose)
- Node.js 18+ (for admin/storefront)

### Clone and Setup

```bash
# Clone repository
git clone https://github.com/RustokCMS/RusToK.git
cd RusToK

# Start PostgreSQL (Docker)
docker-compose up -d postgres

# Run database migrations
cargo db-migrate

# Start development server
cargo dev
```

The server will start on `http://localhost:3000`.

## Using Cargo Aliases

RusToK includes 40+ cargo aliases for common tasks. See `.cargo/config.toml` for the full list.

### Development

```bash
# Start development server with auto-reload
cargo dev

# Start admin panel (Leptos CSR)
cargo dev-admin

# Start storefront (Leptos SSR)
cargo dev-storefront
```

### Testing

```bash
# Run all tests
cargo test-all

# Quick unit tests only (fast for TDD)
cargo test-fast

# Integration tests
cargo test-integration

# Test specific module
cargo test-content
cargo test-commerce
```

### Code Quality

```bash
# Lint with clippy
cargo lint

# Auto-fix clippy issues
cargo lint-fix

# Check formatting
cargo fmt-check

# Format code
cargo fmt-fix

# Run all CI checks locally
cargo ci
```

### Database

```bash
# Run migrations
cargo db-migrate

# Reset database (drop + create + migrate)
cargo db-reset

# Check migration status
cargo db-status

# Create new migration
cargo db-new
```

### Building

```bash
# Build all packages in release mode
cargo build-release

# Build server only
cargo build-server

# Build admin panel (WASM)
cargo build-admin

# Build storefront (WASM)
cargo build-storefront
```

### Security

```bash
# Run security audit
cargo audit

# Full security check (audit + deny)
cargo audit-all

# Check for outdated dependencies
cargo outdated
```

## Project Structure

```
RusToK/
├── apps/
│   ├── server/         # Main backend server (Axum + Loco)
│   ├── admin/          # Admin panel (Leptos CSR)
│   ├── storefront/     # Storefront (Leptos SSR)
│   └── mcp/            # MCP server
├── crates/
│   ├── rustok-core/    # Core types, events, RBAC
│   ├── rustok-content/ # Content management
│   ├── rustok-commerce/# E-commerce
│   ├── rustok-blog/    # Blog functionality
│   ├── rustok-pages/   # Pages module
│   ├── rustok-forum/   # Forum module
│   ├── rustok-index/   # CQRS read models
│   ├── rustok-outbox/  # Outbox pattern
│   └── rustok-test-utils/ # Testing utilities
└── docs/               # Documentation

```

## API Access

### REST API

```bash
# Health check
curl http://localhost:3000/api/health

# List nodes
curl http://localhost:3000/api/nodes

# Create node (requires authentication)
curl -X POST http://localhost:3000/api/nodes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "kind": "post",
    "translations": [{
      "locale": "en",
      "title": "My First Post"
    }],
    "bodies": []
  }'
```

### GraphQL API

GraphQL Playground: `http://localhost:3000/graphql`

```graphql
# Query nodes
query {
  nodes(filter: { kind: "post" }) {
    items {
      id
      kind
      translations {
        locale
        title
        slug
      }
    }
  }
}

# Create node
mutation {
  createNode(input: {
    kind: "post"
    translations: [{
      locale: "en"
      title: "My First Post"
    }]
  }) {
    id
    kind
  }
}
```

### Swagger UI

OpenAPI documentation: `http://localhost:3000/swagger-ui`

## Key Features

### ✅ Rate Limiting

Protect your API from abuse with built-in rate limiting:

```rust
// Default: 100 requests per minute
// Automatically applied to all routes
```

Responses include standard headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 73
X-RateLimit-Reset: 1704063600
```

See [docs/rate-limiting.md](./rate-limiting.md) for configuration.

### ✅ Input Validation

All inputs are validated using declarative rules:

```rust
#[derive(Validate)]
pub struct CreateNodeInput {
    #[validate(length(min = 1, max = 64))]
    pub kind: String,
    
    #[validate(length(min = 1))]
    pub translations: Vec<NodeTranslationInput>,
}
```

Validation errors return clear messages:
```json
{
  "error": "Validation failed: title must be 1-255 characters"
}
```

See [docs/input-validation.md](./input-validation.md) for details.

### ✅ RBAC Authorization

Fine-grained permission control:

```rust
// In your handler
pub async fn create_node(
    RequireNodesCreate(user): RequireNodesCreate,  // ✅ Enforced!
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<NodeResponse>> {
    // Only users with NODES_CREATE permission can proceed
}
```

Roles: SuperAdmin, Admin, Manager, Customer

See [docs/rbac-enforcement.md](./rbac-enforcement.md) for permissions.

### ✅ Event-Driven Architecture

Domain events drive the system:

```rust
// Publishing events
event_bus.publish(
    tenant_id,
    user_id,
    DomainEvent::NodeCreated {
        node_id,
        kind,
        author_id,
    }
)?;

// Handling events
impl EventHandler for MyHandler {
    async fn handle(&self, envelope: EventEnvelope) -> Result<()> {
        // Process event
    }
}
```

### ✅ Multi-Tenancy

Built-in tenant isolation:

```rust
// Tenant context automatically resolved from:
// 1. Subdomain (e.g., tenant1.rustok.io)
// 2. Header (X-Tenant-ID)
// 3. JWT claim (tenant_id)
```

### ✅ CQRS Read Models

Denormalized views for fast reads:

```rust
// Write side: rustok-content (source of truth)
// Read side: rustok-index (optimized for queries)
```

## Testing

### Run Tests

```bash
# All tests
cargo test-all

# Fast unit tests
cargo test-fast

# With coverage
cargo test-coverage
```

### Writing Tests

Use `rustok-test-utils` for fixtures:

```rust
use rustok_test_utils::{
    db::setup_test_db,
    fixtures::{UserFixture, NodeFixture},
    events::MockEventBus,
};

#[tokio::test]
async fn test_create_node() {
    let db = setup_test_db().await;
    let event_bus = MockEventBus::new();
    
    let user = UserFixture::admin().create(&db).await?;
    let node = NodeFixture::post()
        .title("Test Post")
        .create(&db).await?;
    
    // Assertions
}
```

## Configuration

### Environment Variables

Create `.env` file:

```bash
# Database
DATABASE_URL=postgres://postgres:postgres@localhost/rustok

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# JWT
JWT_SECRET=your-secret-key

# Cache
CACHE_BACKEND=memory  # or redis
REDIS_URL=redis://localhost:6379

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MAX_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60

# Logging
RUST_LOG=info,rustok=debug
LOG_FORMAT=pretty  # or json
```

### Rate Limiting Config

Adjust limits per endpoint:

```rust
// Strict limits for expensive operations
let strict_limiter = RateLimiter::new(
    RateLimitConfig::new(10, 60) // 10 req/min
);

// Relaxed limits for reads
let relaxed_limiter = RateLimiter::new(
    RateLimitConfig::new(1000, 60) // 1000 req/min
);
```

## Monitoring

### Metrics

Prometheus metrics available at `/metrics`:

```
# Rate limiting
rustok_rate_limit_hits_total

# Content operations
rustok_content_operations_total
rustok_content_operation_duration_seconds
rustok_content_nodes_total

# Commerce operations
rustok_commerce_operations_total
rustok_commerce_products_total
```

### Logs

Structured logging with tracing:

```rust
// In production
LOG_FORMAT=json

// In development
LOG_FORMAT=pretty
```

## Deployment

### Docker

```bash
# Build
docker build -t rustok:latest .

# Run
docker run -p 3000:3000 \
  -e DATABASE_URL=$DATABASE_URL \
  rustok:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f server
```

## Next Steps

1. **Read Architecture Docs**: [docs/ARCHITECTURE_RECOMMENDATIONS.md](../ARCHITECTURE_RECOMMENDATIONS.md)
2. **Explore Modules**: Each crate has its own README
3. **Review Examples**: Check `examples/` directory
4. **Join Community**: GitHub Discussions

## Getting Help

- 📖 **Documentation**: See `docs/` directory
- 🐛 **Issues**: GitHub Issues
- 💬 **Discussions**: GitHub Discussions
- 📧 **Email**: support@rustok.io

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](../LICENSE)

---

**Happy Hacking! 🦀**
