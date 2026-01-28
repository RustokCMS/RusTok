# RusToK — System Architecture Manifest v3.0

**Target:** AI Assistants (Cursor, Windsurf, Copilot, Claude)
**Role:** Senior Rust Architect
**Philosophy:** "Rust is ON. WordPress is OFF."

---

## 1. PROJECT IDENTITY

| Property | Value |
|----------|-------|
| **Name** | RusToK |
| **Type** | Enterprise Modular Headless Platform |
| **Language** | Rust 100% |
| **License** | AGPL-3.0 (core) + MIT (modules) |
| **Version** | 1.0 (The Tank) |
| **Repository** | https://github.com/RustokCMS/RusToK |

---

## 2. CORE PHILOSOPHY: "The Tank Strategy"

1. **Stability First:** Мы строим "Танк", а не хрупкую экосистему плагинов.
2. **Compile-Time Safety:** Если компилируется — работает. Никакой runtime магии в v1.0.
3. **Monorepo:** Backend, Admin и Storefront живут вместе. Общие типы обязательны.
4. **Laravel DX in Rust:** Опыт разработчика как в Laravel (CLI, структура), но с производительностью Rust.
5. **No "Spaghetti Data":** Строгая реляционная схема PostgreSQL. Никакого EAV.
6. **Headless Architecture:** Storefront физически отделён, связь только через GraphQL.

---

## 3. TECHNOLOGY STACK

| Layer | Technology | Details |
|-------|------------|---------|
| **Repository** | Cargo Workspace | Monorepo for all apps & crates |
| **Runtime** | Tokio | Async runtime |
| **Backend Framework** | Loco.rs | Axum-based, Rails-like MVC |
| **Admin UI** | Leptos CSR | Client-Side WASM, встроена в server |
| **Storefront** | Leptos SSR | Server-Side Rendering, отдельный деплой |
| **Database** | PostgreSQL | Strict schema, UUID primary keys |
| **ORM** | SeaORM | Async, fully typed |
| **API** | async-graphql | Schema Federation (MergedObject) |
| **IDs** | ULID | Generated via `ulid` crate, stored as `Uuid` |
| **Events** | In-Memory Channel | Pub/Sub для развязки модулей |

---

## 4. PROJECT STRUCTURE (Monorepo)

```text
rustok/
├── .github/
│   └── workflows/
│       └── ci.yml
├── apps/
│   ├── server/                    # Loco.rs backend
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── app.rs
│   │   │   └── controllers/
│   │   │       ├── mod.rs
│   │   │       ├── health.rs
│   │   │       └── graphql.rs
│   │   ├── config/
│   │   │   ├── development.yaml
│   │   │   └── test.yaml
│   │   └── migration/
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs
│   │           ├── m20240101_000001_create_tenants.rs
│   │           └── m20240101_000002_create_users.rs
│   ├── admin/                     # Leptos CSR
│   └── storefront/                # Leptos SSR
├── crates/
│   ├── rustok-core/
│   ├── rustok-commerce/
│   └── rustok-blog/
├── Cargo.toml
├── rust-toolchain.toml
└── .gitignore
```

---

## 5. DATABASE SCHEMA & ID STRATEGY

### 5.1 ID Generation Rule

**CRITICAL:** All Primary Keys and Foreign Keys are `UUID`.
Application generates `ULID`, converts to `UUID`, stores in PostgreSQL.

```rust
// crates/rustok-core/src/id.rs
use ulid::Ulid;
use uuid::Uuid;

/// Generate a new ID (ULID as UUID)
pub fn generate_id() -> Uuid {
    Uuid::from(Ulid::new())
}

/// Parse string to UUID (for API inputs)
pub fn parse_id(s: &str) -> Result<Uuid, IdError> {
    // Try ULID format first, then UUID format
    s.parse::<Ulid>()
        .map(Uuid::from)
        .or_else(|_| s.parse::<Uuid>())
        .map_err(|_| IdError::InvalidFormat(s.to_string()))
}
```

### 5.2 Core Tables

```sql
-- =============================================
-- CORE: Tenants (Multi-tenancy root)
-- =============================================
CREATE TABLE tenants (
    id              UUID PRIMARY KEY,
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(64) NOT NULL UNIQUE,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================
-- CORE: Users
-- =============================================
CREATE TABLE users (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email           VARCHAR(255) NOT NULL,
    password_hash   VARCHAR(255) NOT NULL,
    role            VARCHAR(32) NOT NULL DEFAULT 'customer',
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (tenant_id, email)
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);

-- =============================================
-- CORE: Module Toggles (Feature Flags)
-- =============================================
CREATE TABLE tenant_modules (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module_slug     VARCHAR(64) NOT NULL,  -- 'commerce', 'blog'
    enabled         BOOLEAN NOT NULL DEFAULT true,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (tenant_id, module_slug)
);
```

### 5.3 Commerce Tables

```sql
-- =============================================
-- COMMERCE: Products
-- =============================================
CREATE TABLE commerce_products (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    title           VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    description     TEXT,
    sku             VARCHAR(64),
    
    price           BIGINT NOT NULL DEFAULT 0,      -- In cents
    compare_price   BIGINT,                          -- Original price (for sales)
    currency        CHAR(3) NOT NULL DEFAULT 'USD',
    
    stock_qty       INT NOT NULL DEFAULT 0,
    track_inventory BOOLEAN NOT NULL DEFAULT true,
    
    status          VARCHAR(32) NOT NULL DEFAULT 'draft', -- draft, active, archived
    
    metadata        JSONB NOT NULL DEFAULT '{}',
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,
    
    UNIQUE (tenant_id, slug),
    UNIQUE (tenant_id, sku) WHERE sku IS NOT NULL
);

CREATE INDEX idx_products_tenant ON commerce_products(tenant_id);
CREATE INDEX idx_products_status ON commerce_products(tenant_id, status);

-- =============================================
-- COMMERCE: Categories
-- =============================================
CREATE TABLE commerce_categories (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES commerce_categories(id) ON DELETE SET NULL,
    
    title           VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    description     TEXT,
    position        INT NOT NULL DEFAULT 0,
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (tenant_id, slug)
);

-- =============================================
-- COMMERCE: Product-Category (Many-to-Many)
-- =============================================
CREATE TABLE commerce_product_categories (
    product_id      UUID NOT NULL REFERENCES commerce_products(id) ON DELETE CASCADE,
    category_id     UUID NOT NULL REFERENCES commerce_categories(id) ON DELETE CASCADE,
    position        INT NOT NULL DEFAULT 0,
    
    PRIMARY KEY (product_id, category_id)
);

-- =============================================
-- COMMERCE: Orders
-- =============================================
CREATE TABLE commerce_orders (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    
    order_number    VARCHAR(32) NOT NULL,           -- Human-readable: ORD-2025-00001
    status          VARCHAR(32) NOT NULL DEFAULT 'pending',
    
    subtotal        BIGINT NOT NULL,
    tax             BIGINT NOT NULL DEFAULT 0,
    shipping        BIGINT NOT NULL DEFAULT 0,
    discount        BIGINT NOT NULL DEFAULT 0,
    total           BIGINT NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'USD',
    
    shipping_address JSONB,
    billing_address  JSONB,
    
    metadata        JSONB NOT NULL DEFAULT '{}',
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (tenant_id, order_number)
);

CREATE INDEX idx_orders_tenant ON commerce_orders(tenant_id);
CREATE INDEX idx_orders_user ON commerce_orders(user_id);
CREATE INDEX idx_orders_status ON commerce_orders(tenant_id, status);

-- =============================================
-- COMMERCE: Order Items
-- =============================================
CREATE TABLE commerce_order_items (
    id              UUID PRIMARY KEY,
    order_id        UUID NOT NULL REFERENCES commerce_orders(id) ON DELETE CASCADE,
    product_id      UUID REFERENCES commerce_products(id) ON DELETE SET NULL,
    
    title           VARCHAR(255) NOT NULL,          -- Snapshot at purchase time
    sku             VARCHAR(64),
    quantity        INT NOT NULL,
    unit_price      BIGINT NOT NULL,
    total           BIGINT NOT NULL,
    
    metadata        JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_order_items_order ON commerce_order_items(order_id);
```

### 5.4 Blog Tables

```sql
-- =============================================
-- BLOG: Posts
-- =============================================
CREATE TABLE blog_posts (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
    
    title           VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    excerpt         TEXT,
    content         TEXT NOT NULL,
    
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,
    
    UNIQUE (tenant_id, slug)
);
```

---

## 6. MODULE ARCHITECTURE

### 6.1 Module Definition

Each module is a Cargo crate that implements `RusToKModule` trait.

```rust
// crates/rustok-core/src/module.rs

use async_trait::async_trait;

#[async_trait]
pub trait RusToKModule: Send + Sync {
    /// Unique identifier (e.g., "commerce", "blog")
    fn slug(&self) -> &'static str;
    
    /// Human-readable name
    fn name(&self) -> &'static str;
    
    /// Module version
    fn version(&self) -> &'static str;
    
    /// Dependencies on other modules
    fn dependencies(&self) -> &[&'static str] {
        &[]
    }
    
    /// Called when module is enabled for a tenant
    async fn on_enable(&self, ctx: &Context, tenant_id: Uuid) -> Result<()> {
        Ok(())
    }
    
    /// Called when module is disabled for a tenant
    async fn on_disable(&self, ctx: &Context, tenant_id: Uuid) -> Result<()> {
        Ok(())
    }
    
    /// Register GraphQL schema fragment
    fn graphql_schema(&self) -> Option<Box<dyn GraphQLModule>> {
        None
    }
    
    /// Register Admin UI resources
    fn admin_resources(&self, registry: &mut AdminRegistry) {}
    
    /// Register hooks
    fn hooks(&self) -> Option<Box<dyn HookProvider>> {
        None
    }
}
```

### 6.2 Module Registration (Server)

```rust
// apps/server/src/app.rs

use rustok_core::RusToKModule;
use rustok_commerce::CommerceModule;
use rustok_blog::BlogModule;

pub fn register_modules() -> Vec<Box<dyn RusToKModule>> {
    vec![
        Box::new(CommerceModule),
        Box::new(BlogModule),
        // Add new modules here
    ]
}
```

### 6.3 Feature Toggles

Modules are compiled into binary but can be toggled per-tenant via `tenant_modules` table.

```rust
// crates/rustok-core/src/module.rs

impl ModuleRegistry {
    /// Check if module is enabled for tenant
    pub async fn is_enabled(&self, tenant_id: Uuid, module_slug: &str) -> Result<bool> {
        let result = tenant_modules::Entity::find()
            .filter(tenant_modules::Column::TenantId.eq(tenant_id))
            .filter(tenant_modules::Column::ModuleSlug.eq(module_slug))
            .one(&self.db)
            .await?;
        
        Ok(result.map(|m| m.enabled).unwrap_or(false))
    }
    
    /// Toggle module for tenant
    pub async fn set_enabled(
        &self, 
        tenant_id: Uuid, 
        module_slug: &str, 
        enabled: bool
    ) -> Result<()> {
        // Upsert logic
        let model = tenant_modules::ActiveModel {
            id: Set(generate_id()),
            tenant_id: Set(tenant_id),
            module_slug: Set(module_slug.to_string()),
            enabled: Set(enabled),
            ..Default::default()
        };
        
        tenant_modules::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([
                    tenant_modules::Column::TenantId,
                    tenant_modules::Column::ModuleSlug,
                ])
                .update_column(tenant_modules::Column::Enabled)
                .to_owned()
            )
            .exec(&self.db)
            .await?;
        
        Ok(())
    }
}
```

---

## 7. HOOKS API (Future-Proofing)

Hooks allow modules to react to events without direct dependencies.

```rust
// crates/rustok-core/src/hooks/traits.rs

use async_trait::async_trait;

#[async_trait]
pub trait HookProvider: Send + Sync {
    // ============ Commerce Hooks ============
    
    /// Called when a new order is placed
    async fn on_order_placed(&self, _ctx: &Context, _order_id: Uuid) -> Result<()> {
        Ok(())
    }
    
    /// Called when order status changes
    async fn on_order_status_changed(
        &self, 
        _ctx: &Context, 
        _order_id: Uuid, 
        _old_status: &str, 
        _new_status: &str
    ) -> Result<()> {
        Ok(())
    }
    
    /// Called when inventory changes
    async fn on_inventory_changed(
        &self, 
        _ctx: &Context, 
        _product_id: Uuid, 
        _old_qty: i32, 
        _new_qty: i32
    ) -> Result<()> {
        Ok(())
    }
    
    // ============ User Hooks ============
    
    /// Called when a new user registers
    async fn on_user_registered(&self, _ctx: &Context, _user_id: Uuid) -> Result<()> {
        Ok(())
    }
    
    /// Called when user logs in
    async fn on_user_login(&self, _ctx: &Context, _user_id: Uuid) -> Result<()> {
        Ok(())
    }
    
    // ============ Content Hooks ============
    
    /// Called when content is published
    async fn on_content_published(
        &self, 
        _ctx: &Context, 
        _content_type: &str, 
        _content_id: Uuid
    ) -> Result<()> {
        Ok(())
    }
}
```

### 7.1 Hook Registry

```rust
// crates/rustok-core/src/hooks/registry.rs

pub struct HookRegistry {
    providers: Vec<Box<dyn HookProvider>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { providers: vec![] }
    }
    
    pub fn register(&mut self, provider: Box<dyn HookProvider>) {
        self.providers.push(provider);
    }
    
    /// Execute hook across all providers
    pub async fn emit_order_placed(&self, ctx: &Context, order_id: Uuid) -> Result<()> {
        for provider in &self.providers {
            provider.on_order_placed(ctx, order_id).await?;
        }
        Ok(())
    }
    
    // ... similar methods for other hooks
}
```

---

## 8. EVENT BUS (Module Communication)

Modules communicate via events, not direct imports.

```rust
// crates/rustok-core/src/events/types.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainEvent {
    // Commerce Events
    OrderPlaced { order_id: Uuid, tenant_id: Uuid, total: i64 },
    OrderStatusChanged { order_id: Uuid, old_status: String, new_status: String },
    PaymentReceived { order_id: Uuid, amount: i64 },
    InventoryLow { product_id: Uuid, tenant_id: Uuid, remaining: i32 },
    
    // User Events
    UserRegistered { user_id: Uuid, tenant_id: Uuid, email: String },
    UserLoggedIn { user_id: Uuid },
    
    // Content Events
    ContentPublished { content_type: String, content_id: Uuid, tenant_id: Uuid },
}
```

```rust
// crates/rustok-core/src/events/bus.rs

use tokio::sync::broadcast;

pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    
    pub fn publish(&self, event: DomainEvent) {
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}
```

---

## 9. GRAPHQL API

### 9.1 Schema Federation

Each module provides its own Query/Mutation, merged at server level.

```rust
// apps/server/src/graphql/schema.rs

use async_graphql::{MergedObject, MergedSubscription, Schema, EmptySubscription};
use rustok_core::graphql::{CoreQuery, CoreMutation};
use rustok_commerce::graphql::{CommerceQuery, CommerceMutation};
use rustok_blog::graphql::{BlogQuery, BlogMutation};

#[derive(MergedObject, Default)]
pub struct Query(CoreQuery, CommerceQuery, BlogQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(CoreMutation, CommerceMutation, BlogMutation);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(ctx: AppContext) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(ctx)
        .finish()
}
```

### 9.2 Example: Commerce GraphQL

```rust
// crates/rustok-commerce/src/graphql/query.rs

use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct CommerceQuery;

#[Object]
impl CommerceQuery {
    /// Get all products for current tenant
    async fn products(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] offset: i32,
        status: Option<ProductStatus>,
    ) -> Result<Vec<Product>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant_id = app_ctx.current_tenant_id()?;
        
        let products = ProductService::new(&app_ctx.db)
            .list(tenant_id, limit, offset, status)
            .await?;
        
        Ok(products)
    }
    
    /// Get single product by ID or slug
    async fn product(
        &self,
        ctx: &Context<'_>,
        id: Option<ID>,
        slug: Option<String>,
    ) -> Result<Option<Product>> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant_id = app_ctx.current_tenant_id()?;
        
        let product = match (id, slug) {
            (Some(id), _) => {
                ProductService::new(&app_ctx.db)
                    .find_by_id(tenant_id, id.parse()?)
                    .await?
            }
            (_, Some(slug)) => {
                ProductService::new(&app_ctx.db)
                    .find_by_slug(tenant_id, &slug)
                    .await?
            }
            _ => return Err("Either id or slug required".into()),
        };
        
        Ok(product)
    }
}
```

---

## 10. ADMIN UI ARCHITECTURE (Leptos CSR)

### 10.1 Schema-Driven + Custom Widgets

Backend crates describe data and metadata only. The Admin UI owns rendering.
The schema-driven approach means:

- **Backend modules** provide field definitions, types, permissions, and widgets metadata.
- **Admin frontend** interprets this schema to render forms and tables.
- **UI is swappable:** Admin can be reimplemented in another framework without touching backend crates.

```rust
// crates/rustok-core/src/admin/registry.rs

pub struct AdminRegistry {
    resources: HashMap<String, ResourceConfig>,
}

impl AdminRegistry {
    pub fn resource<E>(&mut self, slug: &str) -> ResourceBuilder<E>
    where
        E: EntityTrait + AdminEntity,
    {
        ResourceBuilder::new(self, slug)
    }
}

pub struct ResourceBuilder<'a, E> {
    registry: &'a mut AdminRegistry,
    config: ResourceConfig,
    _phantom: PhantomData<E>,
}

impl<'a, E: EntityTrait + AdminEntity> ResourceBuilder<'a, E> {
    pub fn label(mut self, label: &str) -> Self {
        self.config.label = label.to_string();
        self
    }
    
    pub fn icon(mut self, icon: &str) -> Self {
        self.config.icon = icon.to_string();
        self
    }
    
    /// Configure list view columns
    pub fn list<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ListBuilder) -> ListBuilder,
    {
        let builder = f(ListBuilder::new());
        self.config.list = builder.build();
        self
    }
    
    /// Configure form fields
    pub fn form<F>(mut self, f: F) -> Self
    where
        F: FnOnce(FormBuilder) -> FormBuilder,
    {
        let builder = f(FormBuilder::new());
        self.config.form = builder.build();
        self
    }
    
    /// Override detail view with custom component
    pub fn detail_view<C: Component>(mut self) -> Self {
        self.config.custom_detail = Some(TypeId::of::<C>());
        self
    }
    
    pub fn permissions(mut self, read: &str, write: &str) -> Self {
        self.config.read_permission = read.to_string();
        self.config.write_permission = write.to_string();
        self
    }
    
    pub fn build(self) {
        self.registry.resources.insert(
            self.config.slug.clone(),
            self.config,
        );
    }
}
```

### 10.2 Module Admin Registration Example

```rust
// crates/rustok-commerce/src/admin.rs

use rustok_core::admin::{AdminRegistry, Widget, Format};
use crate::entities::product;
use crate::admin_components::ProductVariantManager;

pub fn register(registry: &mut AdminRegistry) {
    registry.resource::<product::Entity>("products")
        .label("Товары")
        .icon("package")
        
        // List View
        .list(|list| {
            list.column("title", "Название")
                    .searchable()
                    .sortable()
                .column("sku", "Артикул")
                .column("price", "Цена")
                    .format(Format::Money { currency: "currency" })
                .column("stock_qty", "Остаток")
                    .badge(|qty| if qty < 10 { "warning" } else { "success" })
                .column("status", "Статус")
                    .badge_map([
                        ("draft", "secondary", "Черновик"),
                        ("active", "success", "Активен"),
                        ("archived", "muted", "В архиве"),
                    ])
        })
        
        // Form View
        .form(|form| {
            form.section("Основное", |s| {
                    s.field("title", Widget::Text { required: true })
                     .field("slug", Widget::Slug { from: "title" })
                     .field("description", Widget::RichText)
                })
                .section("Цена и склад", |s| {
                    s.field("price", Widget::Money)
                     .field("compare_price", Widget::Money)
                     .field("sku", Widget::Text)
                     .field("stock_qty", Widget::Number)
                })
                .section("Варианты", |s| {
                    // Custom Leptos component injection
                    s.custom::<ProductVariantManager>()
                })
                .sidebar(|s| {
                    s.field("status", Widget::Select {
                        options: vec!["draft", "active", "archived"]
                    })
                    .field("categories", Widget::MultiSelect {
                        resource: "categories"
                    })
                })
        })
        
        .permissions("products.read", "products.write")
        .build();
}
```

---

## 11. STOREFRONT ARCHITECTURE (Leptos SSR)

### 11.1 Separation from Backend

Storefront is a **separate binary** that communicates with Backend via GraphQL.

```rust
// apps/storefront/src/main.rs

use leptos::*;
use leptos_axum::*;

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    
    // GraphQL client configuration
    let graphql_url = std::env::var("RUSTOK_API_URL")
        .unwrap_or_else(|_| "http://localhost:3000/graphql".to_string());
    
    let app = Router::new()
        .leptos_routes(&leptos_options, routes(), App)
        .with_state(AppState { graphql_url });
    
    axum::serve(TcpListener::bind(&addr).await.unwrap(), app)
        .await
        .unwrap();
}
```

### 11.2 GraphQL Client

```rust
// apps/storefront/src/api/client.rs

use cynic::{QueryBuilder, MutationBuilder};

pub struct GraphQLClient {
    url: String,
    client: reqwest::Client,
}

impl GraphQLClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn query<Q: QueryBuilder>(&self, query: Q) -> Result<Q::Response> {
        let response = self.client
            .post(&self.url)
            .json(&query.build())
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
}
```

### 11.3 Server Functions (Leptos SSR)

```rust
// apps/storefront/src/pages/product.rs

use leptos::*;

#[server(GetProduct)]
pub async fn get_product(slug: String) -> Result<Option<Product>, ServerFnError> {
    let client = use_context::<GraphQLClient>()
        .ok_or_else(|| ServerFnError::new("No GraphQL client"))?;
    
    let query = ProductQuery::build(ProductQueryVariables { slug });
    let response = client.query(query).await?;
    
    Ok(response.product)
}

#[component]
pub fn ProductPage(slug: String) -> impl IntoView {
    let product = create_resource(
        move || slug.clone(),
        |slug| async move { get_product(slug).await }
    );
    
    view! {
        <Suspense fallback=|| view! { <LoadingSpinner/> }>
            {move || product.get().map(|result| match result {
                Ok(Some(product)) => view! {
                    <ProductDetail product=product/>
                }.into_view(),
                Ok(None) => view! { <NotFound/> }.into_view(),
                Err(e) => view! { <ErrorPage message=e.to_string()/> }.into_view(),
            })}
        </Suspense>
    }
}
```

---

## 12. ERROR HANDLING

### 12.1 Error Types

```rust
// crates/rustok-core/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RusToKError {
    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: Uuid },
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl RusToKError {
    pub fn not_found<E: EntityTrait>(id: Uuid) -> Self {
        Self::NotFound {
            entity: E::default().table_name(),
            id,
        }
    }
}
```

### 12.2 Result Type

```rust
// crates/rustok-core/src/error.rs

pub type Result<T> = std::result::Result<T, RusToKError>;
```

---

## 13. CODING STANDARDS

### 13.1 File Naming
- Entities: `snake_case.rs` (e.g., `order_item.rs`)
- Components: `snake_case.rs` (e.g., `product_card.rs`)
- Tests: `*_test.rs` in same directory

### 13.2 Entity Patterns

```rust
// Standard entity structure
// crates/rustok-commerce/src/entities/product.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "commerce_products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    
    pub price: i64,
    pub compare_price: Option<i64>,
    pub currency: String,
    
    pub stock_qty: i32,
    pub track_inventory: bool,
    
    pub status: ProductStatus,
    
    pub metadata: Json<serde_json::Value>,
    
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub published_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::order_item::Entity")]
    OrderItems,
    
    #[sea_orm(
        belongs_to = "crate::entities::tenant::Entity",
        from = "Column::TenantId",
        to = "crate::entities::tenant::Column::Id"
    )]
    Tenant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
#[sea_orm(rs_type = "String", db_type = "String(Some(32))")]
pub enum ProductStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "archived")]
    Archived,
}
```

### 13.3 Service Pattern

```rust
// crates/rustok-commerce/src/services/product_service.rs

pub struct ProductService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ProductService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
    
    pub async fn create(&self, input: CreateProductInput) -> Result<product::Model> {
        let model = product::ActiveModel {
            id: Set(generate_id()),
            tenant_id: Set(input.tenant_id),
            title: Set(input.title),
            slug: Set(input.slug),
            price: Set(input.price),
            status: Set(ProductStatus::Draft),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };
        
        let result = model.insert(self.db).await?;
        Ok(result)
    }
    
    pub async fn find_by_id(
        &self, 
        tenant_id: Uuid, 
        id: Uuid
    ) -> Result<Option<product::Model>> {
        let result = product::Entity::find_by_id(id)
            .filter(product::Column::TenantId.eq(tenant_id))
            .one(self.db)
            .await?;
        
        Ok(result)
    }
    
    pub async fn list(
        &self,
        tenant_id: Uuid,
        limit: i32,
        offset: i32,
        status: Option<ProductStatus>,
    ) -> Result<Vec<product::Model>> {
        let mut query = product::Entity::find()
            .filter(product::Column::TenantId.eq(tenant_id))
            .order_by_desc(product::Column::CreatedAt);
        
        if let Some(status) = status {
            query = query.filter(product::Column::Status.eq(status));
        }
        
        let results = query
            .limit(limit as u64)
            .offset(offset as u64)
            .all(self.db)
            .await?;
        
        Ok(results)
    }
}
```

---

## 14. DEVELOPMENT WORKFLOW

### 14.1 Scaffolding (Loco.rs First)

```bash
# Install Loco CLI
cargo install loco-cli

# Create repo root
mkdir rustok && cd rustok
git init

# Create backend (SaaS template with auth)
mkdir apps && cd apps
loco new server --template saas
cd ..

# Admin and Storefront are created manually (Leptos CSR/SSR)
```

### 14.2 Workspace Cargo.toml

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "apps/server",
    "apps/admin",
    "apps/storefront",
    "crates/rustok-core",
    "crates/rustok-commerce",
    "crates/rustok-blog",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "AGPL-3.0"
repository = "https://github.com/RustokCMS/RusToK"

[workspace.dependencies]
# Loco Framework
loco-rs = "0.12"

# Async Runtime
tokio = { version = "1.40", features = ["full"] }

# Web
axum = "0.7"
tower = "0.5"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# Database (Loco uses SeaORM)
sea-orm = { version = "1.0", features = [
    "sqlx-postgres",
    "runtime-tokio-rustls",
    "macros",
    "with-uuid",
    "with-chrono",
    "with-json",
] }

# GraphQL
async-graphql = { version = "6.0", features = ["uuid", "chrono", "serde"] }

# IDs
uuid = { version = "1.10", features = ["v4", "serde"] }
ulid = "1.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Leptos
leptos = "0.6"
leptos_axum = "0.6"
leptos_router = "0.6"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Internal crates
rustok-core = { path = "crates/rustok-core" }
rustok-commerce = { path = "crates/rustok-commerce" }
rustok-blog = { path = "crates/rustok-blog" }
```

### 14.3 Server (Loco-based)

```toml
# apps/server/Cargo.toml
[package]
name = "rustok-server"
version.workspace = true
edition.workspace = true

[dependencies]
rustok-core.workspace = true
rustok-commerce.workspace = true
rustok-blog.workspace = true

loco-rs.workspace = true
tokio.workspace = true
axum.workspace = true
sea-orm.workspace = true
async-graphql.workspace = true
async-graphql-axum.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
tracing.workspace = true

# Loco extras
migration = { path = "migration" }

[dev-dependencies]
loco-rs = { workspace = true, features = ["testing"] }
serial_test = "3.1"
```

```rust
// apps/server/src/main.rs
use loco_rs::cli;
use migration::Migrator;
use rustok_server::app::App;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    cli::main::<App, Migrator>().await
}
```

```rust
// apps/server/src/lib.rs
pub mod app;
pub mod controllers;
pub mod models;
pub mod views;
pub mod graphql;
```

```rust
// apps/server/src/app.rs
use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    task::Tasks,
    Result,
};
use sea_orm::DatabaseConnection;

use crate::controllers;

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self>(mode, environment).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::health::routes())
            .add_route(controllers::graphql::routes())
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        // Очистка таблиц для тестов
        truncate_table(db, "users").await?;
        Ok(())
    }
}
```

```rust
// apps/server/src/controllers/mod.rs
pub mod health;
pub mod graphql;
```

```rust
// apps/server/src/controllers/health.rs
use axum::{routing::get, Router};
use loco_rs::prelude::*;

async fn health() -> Result<Response> {
    format::json(serde_json::json!({
        "status": "ok",
        "app": "rustok",
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("health")
        .add("/", get(health))
}
```

```rust
// apps/server/src/controllers/graphql.rs
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::{get, post}, Extension, Router};
use loco_rs::prelude::*;

// Временная заглушка Query
#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &str {
        "GraphQL is working!"
    }

    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(
        async_graphql::http::playground_source(
            async_graphql::http::GraphQLPlaygroundConfig::new("/api/graphql")
        )
    )
}

pub fn routes() -> Routes {
    let schema = build_schema();
    
    Routes::new()
        .prefix("api/graphql")
        .add("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
}
```

```yaml
# apps/server/config/development.yaml
logger:
  enable: true
  pretty_backtrace: true
  level: debug
  format: compact

server:
  binding: 0.0.0.0
  port: 3000

database:
  uri: postgres://postgres:postgres@localhost:5432/rustok_dev
  enable_logging: true
  min_connections: 1
  max_connections: 5

auth:
  jwt:
    secret: change-this-in-production
    expiration: 604800 # 7 days
```

```yaml
# apps/server/config/test.yaml
logger:
  enable: true
  level: error
  format: compact

server:
  binding: 0.0.0.0
  port: 3001

database:
  uri: postgres://postgres:postgres@localhost:5432/rustok_test
  enable_logging: false
  min_connections: 1
  max_connections: 2
  auto_migrate: true

auth:
  jwt:
    secret: test-secret
    expiration: 3600
```

### 14.4 Migrations (Loco Style)

```toml
# apps/server/migration/Cargo.toml
[package]
name = "migration"
version = "0.1.0"
edition = "2021"

[dependencies]
sea-orm-migration.workspace = true
uuid.workspace = true

[dependencies.sea-orm-migration]
version = "1.0"
features = ["runtime-tokio-rustls", "sqlx-postgres"]
```

```rust
// apps/server/migration/src/lib.rs
#![allow(elided_lifetimes_in_paths)]

pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tenants;
mod m20240101_000002_create_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tenants::Migration),
            Box::new(m20240101_000002_create_users::Migration),
        ]
    }
}
```

```rust
// apps/server/migration/src/m20240101_000001_create_tenants.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tenants::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tenants::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Tenants::Slug).string_len(64).not_null().unique_key())
                    .col(ColumnDef::new(Tenants::Settings).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(Tenants::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Tenants::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
    Name,
    Slug,
    Settings,
    CreatedAt,
    UpdatedAt,
}
```

```rust
// apps/server/migration/src/m20240101_000002_create_users.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Users::Email).string_len(255).not_null())
                    .col(ColumnDef::new(Users::PasswordHash).string_len(255).not_null())
                    .col(ColumnDef::new(Users::Role).string_len(32).not_null().default("customer"))
                    .col(ColumnDef::new(Users::Metadata).json_binary().not_null().default("{}"))
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Users::Table, Users::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: email per tenant
        manager
            .create_index(
                Index::create()
                    .name("idx_users_tenant_email")
                    .table(Users::Table)
                    .col(Users::TenantId)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    TenantId,
    Email,
    PasswordHash,
    Role,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
}
```

### 14.5 Core Crate (Updated)

```toml
# crates/rustok-core/Cargo.toml
[package]
name = "rustok-core"
version.workspace = true
edition.workspace = true

[dependencies]
sea-orm.workspace = true
uuid.workspace = true
ulid.workspace = true
serde.workspace = true
thiserror.workspace = true
chrono.workspace = true
tracing.workspace = true
```

```rust
// crates/rustok-core/src/lib.rs
pub mod error;
pub mod id;

pub use error::{RusToKError, Result};
pub use id::generate_id;

/// Re-export common types
pub mod prelude {
    pub use crate::error::{RusToKError, Result};
    pub use crate::id::generate_id;
    pub use uuid::Uuid;
}
```

### 14.6 CI Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-Dwarnings"

jobs:
  fmt:
    name: Formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings

  build-crates:
    name: Build Crates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build -p rustok-core
      - run: cargo build -p rustok-commerce
      - run: cargo build -p rustok-blog

  build-server:
    name: Build Server (Loco)
    runs-on: ubuntu-latest
    needs: build-crates
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build -p rustok-server

  build-admin:
    name: Build Admin (WASM)
    runs-on: ubuntu-latest
    needs: build-crates
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - name: Install Trunk
        run: cargo install trunk --locked
      - name: Build Admin
        working-directory: apps/admin
        run: trunk build

  build-storefront:
    name: Build Storefront (SSR)
    runs-on: ubuntu-latest
    needs: build-crates
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-leptos
        run: cargo install cargo-leptos --locked
      - name: Build Storefront
        working-directory: apps/storefront
        run: cargo leptos build

  test:
    name: Tests
    runs-on: ubuntu-latest
    needs: build-crates
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: rustok_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    env:
      DATABASE_URL: postgres://postgres:postgres@localhost:5432/rustok_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace

  ci-success:
    name: CI Success
    runs-on: ubuntu-latest
    needs: [fmt, clippy, build-server, build-admin, build-storefront, test]
    if: always()
    steps:
      - name: Check all jobs
        run: |
          if [[ "${{ needs.fmt.result }}" != "success" ]] || \
             [[ "${{ needs.clippy.result }}" != "success" ]] || \
             [[ "${{ needs.build-server.result }}" != "success" ]] || \
             [[ "${{ needs.build-admin.result }}" != "success" ]] || \
             [[ "${{ needs.build-storefront.result }}" != "success" ]] || \
             [[ "${{ needs.test.result }}" != "success" ]]; then
            echo "One or more jobs failed"
            exit 1
          fi
          echo "All checks passed!"
```

### 14.7 Local Run

```bash
# Database
docker run -d --name rustok-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustok_dev \
  -p 5432:5432 \
  postgres:16

# Migrations (Loco CLI)
cd apps/server
cargo loco db migrate

# Run server
cargo loco start

# Admin (separate terminal)
cd apps/admin
trunk serve

# GraphQL Playground
open http://localhost:3000/api/graphql
```

---

## 15. SUMMARY CHECKLIST

Before implementing any feature, verify:

- [ ] Uses `Uuid` for all IDs (generated from ULID)
- [ ] Includes `tenant_id` for multi-tenant entities
- [ ] Implements proper error handling with `RusToKError`
- [ ] Has SeaORM entity with relations
- [ ] Has service layer (not direct DB access in handlers)
- [ ] GraphQL resolvers check tenant context
- [ ] Admin resource registered with proper permissions
- [ ] Hooks called where relevant
- [ ] Events published for cross-module communication

---

**END OF MANIFEST v3.0**
```
