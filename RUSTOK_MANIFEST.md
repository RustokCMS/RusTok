# RusToK — System Architecture Manifest v4.0

**Target:** AI Assistants (Cursor, Windsurf, Copilot, Claude)
**Role:** Senior Rust Architect
**Philosophy:** "Rust is ON. WordPress is OFF."

---

## CHANGELOG v3.0 → v4.0

- **Unified Core:** базовые таблицы контента в ядре.
- **CQRS-lite:** разделение write/read paths.
- **Index Module:** денормализованные индексы для поиска.
- **Partitioning Strategy:** масштабирование таблиц.
- **Event-Driven:** модули общаются через события.
- **Microservice-Ready:** Index Module выносится отдельно.

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

## 2. CORE PHILOSOPHY

### 2.1 The Tank Strategy

- **Stability First:** Мы строим "Танк", а не хрупкую экосистему плагинов.
- **Compile-Time Safety:** Если компилируется — работает.
- **Monorepo:** Backend, Admin и Storefront живут вместе.

### 2.2 WordPress Simplicity, Highload Performance

- **Unified Core:** Базовые сущности в ядре, не раздутые.
- **Specialized Modules:** Товары ≠ статьи, разные таблицы.
- **Empty Tables Cost Zero:** Неиспользуемые таблицы не мешают.

### 2.3 CQRS-Lite

- **Write Path:** Нормализованные таблицы, быстрая запись.
- **Read Path:** Денормализованные индексы, быстрое чтение.
- **Event-Driven Sync:** Изменения propagate через события.

---

## 3. TECHNOLOGY STACK

| Layer | Technology | Details |
|-------|------------|---------|
| **Repository** | Cargo Workspace | Monorepo for all apps & crates |
| **Runtime** | Tokio | Async runtime |
| **Backend Framework** | Loco.rs | Axum-based, Rails-like MVC |
| **Admin UI** | Leptos CSR | Client-Side WASM |
| **Storefront** | Leptos SSR | Server-Side Rendering |
| **Database** | PostgreSQL 16+ | Partitioning, JSONB |
| **ORM** | SeaORM | Async, fully typed |
| **API** | async-graphql | Schema Federation |
| **IDs** | ULID | Generated via `ulid` crate, stored as `Uuid` |
| **Events** | tokio::broadcast | In-process pub/sub |
| **Search (optional)** | Meilisearch / Tantivy | Full-text search |

---

## 4. PROJECT STRUCTURE

```text
rustok/
├── apps/
│   ├── server/                     # Loco.rs backend
│   │   ├── src/
│   │   ├── config/
│   │   └── migration/
│   ├── admin/                      # Leptos CSR
│   └── storefront/                 # Leptos SSR
│
├── crates/
│   ├── rustok-core/                # Универсальное ядро
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── id.rs               # ULID → UUID
│   │   │   ├── error.rs
│   │   │   ├── events/             # Event Bus
│   │   │   ├── entities/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── user.rs
│   │   │   │   ├── tenant.rs
│   │   │   │   ├── node.rs         # Универсальный контент
│   │   │   │   ├── body.rs         # Тяжёлый текст
│   │   │   │   ├── category.rs     # Контентные категории
│   │   │   │   ├── tag.rs          # Универсальные теги
│   │   │   │   ├── taggable.rs     # Полиморфная связь
│   │   │   │   ├── meta.rs         # SEO
│   │   │   │   └── media.rs        # Файлы
│   │   │   └── services/
│   │   └── Cargo.toml
│   │
│   ├── rustok-commerce/            # E-commerce модуль
│   │   ├── src/
│   │   │   ├── entities/
│   │   │   │   ├── product.rs
│   │   │   │   ├── variant.rs
│   │   │   │   ├── option.rs
│   │   │   │   ├── price.rs
│   │   │   │   ├── category.rs     # СВОИ категории
│   │   │   │   ├── inventory.rs
│   │   │   │   ├── order.rs
│   │   │   │   └── order_item.rs
│   │   │   ├── services/
│   │   │   └── graphql/
│   │   └── Cargo.toml
│   │
│   ├── rustok-community/           # Социальные фичи
│   │   ├── src/
│   │   │   ├── entities/
│   │   │   │   ├── reaction.rs
│   │   │   │   ├── reputation.rs
│   │   │   │   └── follow.rs
│   │   │   └── services/
│   │   └── Cargo.toml
│   │
│   └── rustok-index/               # CQRS Read Models
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config.rs
│       │   ├── indexers/
│       │   │   ├── product_indexer.rs
│       │   │   └── content_indexer.rs
│       │   └── entities/
│       │       ├── search_product.rs
│       │       └── search_content.rs
│       └── Cargo.toml
│
├── Cargo.toml
├── rust-toolchain.toml
└── docker-compose.yml
```

---

## 5. DATABASE ARCHITECTURE

### 5.1 ID Generation (unchanged)

```rust
// crates/rustok-core/src/id.rs
use ulid::Ulid;
use uuid::Uuid;

pub fn generate_id() -> Uuid {
    Uuid::from(Ulid::new())
}

pub fn parse_id(s: &str) -> Result<Uuid, IdError> {
    s.parse::<Ulid>()
        .map(Uuid::from)
        .or_else(|_| s.parse::<Uuid>())
        .map_err(|_| IdError::InvalidFormat(s.to_string()))
}
```

### 5.2 Core Tables (Unified Foundation)

```sql
-- =============================================
-- CORE: Tenants
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

-- =============================================
-- CORE: Nodes (универсальный контент)
-- Страницы, посты, комментарии — всё здесь
-- =============================================
CREATE TABLE nodes (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES nodes(id) ON DELETE CASCADE,
    author_id       UUID REFERENCES users(id) ON DELETE SET NULL,

    kind            VARCHAR(32) NOT NULL,       -- 'page', 'post', 'comment'
    title           VARCHAR(255),
    slug            VARCHAR(255),
    excerpt         TEXT,

    category_id     UUID REFERENCES categories(id) ON DELETE SET NULL,
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',

    position        INT DEFAULT 0,              -- Сортировка
    depth           INT DEFAULT 0,              -- Уровень вложенности
    reply_count     INT DEFAULT 0,              -- Денормализация для скорости

    metadata        JSONB NOT NULL DEFAULT '{}',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,

    UNIQUE (tenant_id, kind, slug) WHERE slug IS NOT NULL
);

CREATE INDEX idx_nodes_tenant_kind ON nodes(tenant_id, kind, status);
CREATE INDEX idx_nodes_parent ON nodes(parent_id);
CREATE INDEX idx_nodes_category ON nodes(category_id);
CREATE INDEX idx_nodes_author ON nodes(author_id);
CREATE INDEX idx_nodes_published ON nodes(tenant_id, kind, published_at DESC)
    WHERE status = 'published';

-- =============================================
-- CORE: Bodies (тяжёлый контент отдельно)
-- =============================================
CREATE TABLE bodies (
    node_id         UUID PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    body            TEXT,
    format          VARCHAR(16) NOT NULL DEFAULT 'markdown',
    search_vector   TSVECTOR,

    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bodies_search ON bodies USING GIN(search_vector);

-- =============================================
-- CORE: Categories (контентные)
-- =============================================
CREATE TABLE categories (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES categories(id) ON DELETE CASCADE,

    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    description     TEXT,

    position        INT NOT NULL DEFAULT 0,
    depth           INT NOT NULL DEFAULT 0,
    node_count      INT NOT NULL DEFAULT 0,     -- Денормализация

    settings        JSONB NOT NULL DEFAULT '{}', -- Права, модерация, иконка

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, slug)
);

CREATE INDEX idx_categories_tenant ON categories(tenant_id, position);
CREATE INDEX idx_categories_parent ON categories(parent_id);

-- =============================================
-- CORE: Tags (универсальные ярлыки)
-- =============================================
CREATE TABLE tags (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL,

    use_count       INT NOT NULL DEFAULT 0,     -- Популярность

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, slug)
);

CREATE INDEX idx_tags_tenant ON tags(tenant_id);
CREATE INDEX idx_tags_popular ON tags(tenant_id, use_count DESC);

-- =============================================
-- CORE: Taggables (полиморфная связь)
-- =============================================
CREATE TABLE taggables (
    tag_id          UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    target_type     VARCHAR(32) NOT NULL,       -- 'node', 'product'
    target_id       UUID NOT NULL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (tag_id, target_type, target_id)
);

CREATE INDEX idx_taggables_target ON taggables(target_type, target_id);

-- =============================================
-- CORE: Meta (SEO, универсальное)
-- =============================================
CREATE TABLE meta (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    target_type     VARCHAR(32) NOT NULL,       -- 'node', 'product', 'category'
    target_id       UUID NOT NULL,

    -- Basic SEO
    title           VARCHAR(255),
    description     VARCHAR(500),
    keywords        VARCHAR(255),

    -- Open Graph
    og_title        VARCHAR(255),
    og_description  VARCHAR(500),
    og_image        VARCHAR(500),
    og_type         VARCHAR(32),

    -- Twitter
    twitter_card    VARCHAR(32),

    -- Robots
    no_index        BOOLEAN NOT NULL DEFAULT false,
    no_follow       BOOLEAN NOT NULL DEFAULT false,
    canonical_url   VARCHAR(500),

    -- Structured Data (JSON-LD)
    structured_data JSONB,

    UNIQUE (target_type, target_id)
);

CREATE INDEX idx_meta_target ON meta(target_type, target_id);

-- =============================================
-- CORE: Media (файлы)
-- =============================================
CREATE TABLE media (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    uploaded_by     UUID REFERENCES users(id) ON DELETE SET NULL,

    filename        VARCHAR(255) NOT NULL,
    original_name   VARCHAR(255) NOT NULL,
    mime_type       VARCHAR(100) NOT NULL,
    size            BIGINT NOT NULL,

    storage_path    VARCHAR(500) NOT NULL,      -- S3 path или local
    storage_driver  VARCHAR(32) NOT NULL DEFAULT 'local',

    -- Image specific
    width           INT,
    height          INT,

    alt_text        VARCHAR(255),
    metadata        JSONB NOT NULL DEFAULT '{}',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_media_tenant ON media(tenant_id);

-- =============================================
-- CORE: Module Toggles
-- =============================================
CREATE TABLE tenant_modules (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module_slug     VARCHAR(64) NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, module_slug)
);
```

### 5.3 Commerce Tables (Medusa-style)

```sql
-- =============================================
-- COMMERCE: Products
-- =============================================
CREATE TABLE commerce_products (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    title           VARCHAR(255) NOT NULL,
    subtitle        VARCHAR(255),
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,

    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    discountable    BOOLEAN NOT NULL DEFAULT true,

    metadata        JSONB NOT NULL DEFAULT '{}',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, handle)
);

CREATE INDEX idx_commerce_products_tenant ON commerce_products(tenant_id, status);

-- =============================================
-- COMMERCE: Variants
-- =============================================
CREATE TABLE commerce_variants (
    id              UUID PRIMARY KEY,
    product_id      UUID NOT NULL REFERENCES commerce_products(id) ON DELETE CASCADE,

    title           VARCHAR(255) NOT NULL,
    sku             VARCHAR(64),
    barcode         VARCHAR(64),

    manage_inventory BOOLEAN NOT NULL DEFAULT true,
    allow_backorder  BOOLEAN NOT NULL DEFAULT false,

    weight          INT,
    length          INT,
    height          INT,
    width           INT,

    position        INT NOT NULL DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_commerce_variants_product ON commerce_variants(product_id);
CREATE UNIQUE INDEX idx_commerce_variants_sku ON commerce_variants(sku) WHERE sku IS NOT NULL;

-- =============================================
-- COMMERCE: Options (Size, Color, etc.)
-- =============================================
CREATE TABLE commerce_options (
    id              UUID PRIMARY KEY,
    product_id      UUID NOT NULL REFERENCES commerce_products(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    position        INT NOT NULL DEFAULT 0
);

CREATE TABLE commerce_option_values (
    id              UUID PRIMARY KEY,
    option_id       UUID NOT NULL REFERENCES commerce_options(id) ON DELETE CASCADE,
    value           VARCHAR(255) NOT NULL,
    position        INT NOT NULL DEFAULT 0
);

CREATE TABLE commerce_variant_options (
    variant_id      UUID NOT NULL REFERENCES commerce_variants(id) ON DELETE CASCADE,
    option_value_id UUID NOT NULL REFERENCES commerce_option_values(id) ON DELETE CASCADE,
    PRIMARY KEY (variant_id, option_value_id)
);

-- =============================================
-- COMMERCE: Prices (мультивалютность)
-- =============================================
CREATE TABLE commerce_prices (
    id              UUID PRIMARY KEY,
    variant_id      UUID NOT NULL REFERENCES commerce_variants(id) ON DELETE CASCADE,

    amount          BIGINT NOT NULL,
    currency_code   CHAR(3) NOT NULL,

    price_list_id   UUID,                       -- Для разных прайсов
    min_quantity    INT NOT NULL DEFAULT 1,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (variant_id, currency_code, price_list_id, min_quantity)
);

CREATE INDEX idx_commerce_prices_variant ON commerce_prices(variant_id);

-- =============================================
-- COMMERCE: Categories (своя иерархия)
-- =============================================
CREATE TABLE commerce_categories (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES commerce_categories(id) ON DELETE SET NULL,

    name            VARCHAR(255) NOT NULL,
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,

    is_active       BOOLEAN NOT NULL DEFAULT true,
    is_internal     BOOLEAN NOT NULL DEFAULT false,
    rank            INT NOT NULL DEFAULT 0,

    metadata        JSONB NOT NULL DEFAULT '{}',

    UNIQUE (tenant_id, handle)
);

CREATE TABLE commerce_product_categories (
    product_id      UUID NOT NULL REFERENCES commerce_products(id) ON DELETE CASCADE,
    category_id     UUID NOT NULL REFERENCES commerce_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);

-- =============================================
-- COMMERCE: Inventory
-- =============================================
CREATE TABLE commerce_inventory_items (
    id              UUID PRIMARY KEY,
    sku             VARCHAR(64),
    requires_shipping BOOLEAN NOT NULL DEFAULT true,
    metadata        JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE commerce_stock_locations (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    address         JSONB,
    metadata        JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE commerce_inventory_levels (
    id              UUID PRIMARY KEY,
    inventory_item_id UUID NOT NULL REFERENCES commerce_inventory_items(id) ON DELETE CASCADE,
    location_id     UUID NOT NULL REFERENCES commerce_stock_locations(id) ON DELETE CASCADE,

    stocked_quantity  INT NOT NULL DEFAULT 0,
    reserved_quantity INT NOT NULL DEFAULT 0,
    incoming_quantity INT NOT NULL DEFAULT 0,

    UNIQUE (inventory_item_id, location_id)
);

CREATE TABLE commerce_variant_inventory (
    variant_id        UUID NOT NULL REFERENCES commerce_variants(id) ON DELETE CASCADE,
    inventory_item_id UUID NOT NULL REFERENCES commerce_inventory_items(id) ON DELETE CASCADE,
    PRIMARY KEY (variant_id, inventory_item_id)
);

-- =============================================
-- COMMERCE: Orders
-- =============================================
CREATE TABLE commerce_orders (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    customer_id     UUID REFERENCES users(id) ON DELETE SET NULL,

    display_id      SERIAL,                     -- Human-readable #1001
    status          VARCHAR(32) NOT NULL DEFAULT 'pending',

    email           VARCHAR(255),
    currency_code   CHAR(3) NOT NULL,

    subtotal        BIGINT NOT NULL,
    tax_total       BIGINT NOT NULL DEFAULT 0,
    shipping_total  BIGINT NOT NULL DEFAULT 0,
    discount_total  BIGINT NOT NULL DEFAULT 0,
    total           BIGINT NOT NULL,

    shipping_address JSONB,
    billing_address  JSONB,

    metadata        JSONB NOT NULL DEFAULT '{}',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_commerce_orders_tenant ON commerce_orders(tenant_id, created_at DESC);
CREATE INDEX idx_commerce_orders_customer ON commerce_orders(customer_id);
CREATE INDEX idx_commerce_orders_status ON commerce_orders(tenant_id, status);

CREATE TABLE commerce_order_items (
    id              UUID PRIMARY KEY,
    order_id        UUID NOT NULL REFERENCES commerce_orders(id) ON DELETE CASCADE,
    variant_id      UUID REFERENCES commerce_variants(id) ON DELETE SET NULL,

    title           VARCHAR(255) NOT NULL,
    sku             VARCHAR(64),
    quantity        INT NOT NULL,
    unit_price      BIGINT NOT NULL,
    total           BIGINT NOT NULL,

    metadata        JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_commerce_order_items_order ON commerce_order_items(order_id);
```

### 5.4 Index Tables (CQRS Read Models)

```sql
-- =============================================
-- INDEX: Денормализованные продукты для поиска
-- =============================================
CREATE TABLE index_products (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    product_id      UUID NOT NULL,

    -- Денормализованные данные
    title           VARCHAR(255) NOT NULL,
    subtitle        VARCHAR(255),
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,
    status          VARCHAR(32) NOT NULL,

    -- Агрегированные варианты
    min_price       BIGINT,
    max_price       BIGINT,
    currencies      CHAR(3)[],
    total_stock     INT,
    has_stock       BOOLEAN,

    -- Агрегированные связи
    categories      JSONB,                      -- [{id, name, handle}]
    tags            TEXT[],

    -- SEO
    meta_title      VARCHAR(255),
    meta_description VARCHAR(500),

    -- Поиск
    search_vector   TSVECTOR,

    -- Синхронизация
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (product_id)
);

CREATE INDEX idx_index_products_tenant ON index_products(tenant_id);
CREATE INDEX idx_index_products_search ON index_products USING GIN(search_vector);
CREATE INDEX idx_index_products_price ON index_products(tenant_id, min_price);
CREATE INDEX idx_index_products_stock ON index_products(tenant_id, has_stock);

-- =============================================
-- INDEX: Денормализованный контент для поиска
-- =============================================
CREATE TABLE index_content (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    node_id         UUID NOT NULL,

    -- Денормализованные данные
    kind            VARCHAR(32) NOT NULL,
    title           VARCHAR(255),
    slug            VARCHAR(255),
    excerpt         TEXT,
    body_preview    TEXT,                       -- Первые N символов
    status          VARCHAR(32) NOT NULL,

    -- Автор
    author_id       UUID,
    author_name     VARCHAR(255),

    -- Категория
    category_id     UUID,
    category_name   VARCHAR(255),
    category_slug   VARCHAR(255),

    -- Связи
    tags            TEXT[],
    parent_id       UUID,
    reply_count     INT,

    -- SEO
    meta_title      VARCHAR(255),
    meta_description VARCHAR(500),

    -- Поиск
    search_vector   TSVECTOR,

    -- Даты
    published_at    TIMESTAMPTZ,
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (node_id)
);

CREATE INDEX idx_index_content_tenant ON index_content(tenant_id, kind, status);
CREATE INDEX idx_index_content_search ON index_content USING GIN(search_vector);
CREATE INDEX idx_index_content_published ON index_content(tenant_id, kind, published_at DESC);
CREATE INDEX idx_index_content_category ON index_content(category_id);
```

### 5.5 Partitioning Strategy

```sql
-- =============================================
-- PARTITIONING: Orders по дате (highload)
-- =============================================

-- Основная таблица с партиционированием
CREATE TABLE commerce_orders_partitioned (
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    -- ... остальные поля
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Партиции по кварталам
CREATE TABLE commerce_orders_2025_q1
    PARTITION OF commerce_orders_partitioned
    FOR VALUES FROM ('2025-01-01') TO ('2025-04-01');

CREATE TABLE commerce_orders_2025_q2
    PARTITION OF commerce_orders_partitioned
    FOR VALUES FROM ('2025-04-01') TO ('2025-07-01');

-- Партиция для будущих данных (default)
CREATE TABLE commerce_orders_future
    PARTITION OF commerce_orders_partitioned
    DEFAULT;

-- =============================================
-- PARTITIONING: Nodes по tenant (multi-tenant highload)
-- =============================================

CREATE TABLE nodes_partitioned (
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    -- ... остальные поля
    PRIMARY KEY (id, tenant_id)
) PARTITION BY HASH (tenant_id);

-- 8 партиций для распределения нагрузки
CREATE TABLE nodes_p0 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 0);
CREATE TABLE nodes_p1 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 1);
CREATE TABLE nodes_p2 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 2);
CREATE TABLE nodes_p3 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 3);
CREATE TABLE nodes_p4 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 4);
CREATE TABLE nodes_p5 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 5);
CREATE TABLE nodes_p6 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 6);
CREATE TABLE nodes_p7 PARTITION OF nodes_partitioned FOR VALUES WITH (MODULUS 8, REMAINDER 7);
```

---

## 6. EVENT SYSTEM

### 6.1 Domain Events

```rust
// crates/rustok-core/src/events/types.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: DomainEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // ============ Content Events ============
    NodeCreated {
        node_id: Uuid,
        kind: String,
        author_id: Option<Uuid>,
    },
    NodeUpdated {
        node_id: Uuid,
    },
    NodePublished {
        node_id: Uuid,
        kind: String,
    },
    NodeDeleted {
        node_id: Uuid,
        kind: String,
    },

    // ============ Commerce Events ============
    ProductCreated {
        product_id: Uuid,
    },
    ProductUpdated {
        product_id: Uuid,
    },
    ProductPublished {
        product_id: Uuid,
    },
    ProductDeleted {
        product_id: Uuid,
    },

    VariantCreated {
        variant_id: Uuid,
        product_id: Uuid,
    },
    VariantUpdated {
        variant_id: Uuid,
        product_id: Uuid,
    },

    InventoryUpdated {
        variant_id: Uuid,
        location_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
    },
    InventoryLow {
        variant_id: Uuid,
        product_id: Uuid,
        remaining: i32,
        threshold: i32,
    },

    OrderPlaced {
        order_id: Uuid,
        customer_id: Option<Uuid>,
        total: i64,
    },
    OrderStatusChanged {
        order_id: Uuid,
        old_status: String,
        new_status: String,
    },
    OrderCompleted {
        order_id: Uuid,
    },
    OrderCancelled {
        order_id: Uuid,
        reason: Option<String>,
    },

    // ============ User Events ============
    UserRegistered {
        user_id: Uuid,
        email: String,
    },
    UserLoggedIn {
        user_id: Uuid,
    },

    // ============ Tag Events ============
    TagAttached {
        tag_id: Uuid,
        target_type: String,
        target_id: Uuid,
    },
    TagDetached {
        tag_id: Uuid,
        target_type: String,
        target_id: Uuid,
    },

    // ============ Index Events ============
    ReindexRequested {
        target_type: String,
        target_id: Option<Uuid>, // None = full reindex
    },
}
```

### 6.2 Event Bus

```rust
// crates/rustok-core/src/events/bus.rs

use tokio::sync::broadcast;
use std::sync::Arc;

pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    capacity: usize,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, capacity }
    }

    /// Publish event to all subscribers
    pub fn publish(&self, tenant_id: Uuid, event: DomainEvent) {
        let envelope = EventEnvelope {
            id: generate_id(),
            tenant_id,
            timestamp: Utc::now(),
            event,
        };

        // Log if no receivers (не критично)
        if self.sender.receiver_count() == 0 {
            tracing::debug!("No event subscribers for {:?}", envelope.event);
        }

        let _ = self.sender.send(envelope);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            capacity: self.capacity,
        }
    }
}
```

### 6.3 Event Handlers

```rust
// crates/rustok-core/src/events/handler.rs

use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Filter: какие события обрабатываем
    fn handles(&self, event: &DomainEvent) -> bool;

    /// Handle event
    async fn handle(&self, envelope: &EventEnvelope) -> Result<()>;
}

/// Event dispatcher
pub struct EventDispatcher {
    bus: EventBus,
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventDispatcher {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            handlers: vec![],
        }
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Start listening (spawn background task)
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut receiver = self.bus.subscribe();

            while let Ok(envelope) = receiver.recv().await {
                for handler in &self.handlers {
                    if handler.handles(&envelope.event) {
                        if let Err(e) = handler.handle(&envelope).await {
                            tracing::error!(
                                "Event handler error: {:?}, event: {:?}",
                                e,
                                envelope.event
                            );
                        }
                    }
                }
            }
        })
    }
}
```

---

## 7. INDEX MODULE (CQRS)

### 7.1 Index Configuration

```rust
// crates/rustok-index/src/config.rs

pub struct IndexConfig {
    /// Reindex batch size
    pub batch_size: usize,

    /// Parallel workers for reindexing
    pub workers: usize,

    /// Enable real-time sync via events
    pub realtime_sync: bool,

    /// Full reindex schedule (cron)
    pub reindex_schedule: Option<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            workers: 4,
            realtime_sync: true,
            reindex_schedule: Some("0 3 * * *".to_string()), // 3 AM daily
        }
    }
}
```

### 7.2 Product Indexer

```rust
// crates/rustok-index/src/indexers/product_indexer.rs

use async_trait::async_trait;

pub struct ProductIndexer {
    db: DatabaseConnection,
}

impl ProductIndexer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Index single product
    pub async fn index_product(&self, product_id: Uuid) -> Result<()> {
        // Fetch product with all relations
        let product = commerce_products::Entity::find_by_id(product_id)
            .one(&self.db)
            .await?
            .ok_or(RusToKError::not_found::<commerce_products::Entity>(product_id))?;

        // Fetch variants with prices
        let variants = commerce_variants::Entity::find()
            .filter(commerce_variants::Column::ProductId.eq(product_id))
            .all(&self.db)
            .await?;

        let prices: Vec<i64> = commerce_prices::Entity::find()
            .filter(commerce_prices::Column::VariantId.is_in(
                variants.iter().map(|v| v.id).collect::<Vec<_>>()
            ))
            .all(&self.db)
            .await?
            .iter()
            .map(|p| p.amount)
            .collect();

        // Fetch categories
        let categories = commerce_product_categories::Entity::find()
            .filter(commerce_product_categories::Column::ProductId.eq(product_id))
            .find_also_related(commerce_categories::Entity)
            .all(&self.db)
            .await?;

        // Fetch tags
        let tags = taggables::Entity::find()
            .filter(taggables::Column::TargetType.eq("product"))
            .filter(taggables::Column::TargetId.eq(product_id))
            .find_also_related(tags::Entity)
            .all(&self.db)
            .await?;

        // Fetch meta
        let meta = meta::Entity::find()
            .filter(meta::Column::TargetType.eq("product"))
            .filter(meta::Column::TargetId.eq(product_id))
            .one(&self.db)
            .await?;

        // Calculate stock
        let total_stock: i32 = /* sum inventory levels */;

        // Build search vector
        let search_text = format!(
            "{} {} {}",
            product.title,
            product.subtitle.unwrap_or_default(),
            product.description.unwrap_or_default()
        );

        // Upsert index record
        let index_record = index_products::ActiveModel {
            id: Set(generate_id()),
            tenant_id: Set(product.tenant_id),
            product_id: Set(product_id),

            title: Set(product.title),
            subtitle: Set(product.subtitle),
            handle: Set(product.handle),
            description: Set(product.description),
            status: Set(product.status),

            min_price: Set(prices.iter().min().copied()),
            max_price: Set(prices.iter().max().copied()),
            currencies: Set(/* unique currencies */),
            total_stock: Set(total_stock),
            has_stock: Set(total_stock > 0),

            categories: Set(json!(categories)),
            tags: Set(tags.iter().map(|t| t.name.clone()).collect()),

            meta_title: Set(meta.as_ref().and_then(|m| m.title.clone())),
            meta_description: Set(meta.as_ref().and_then(|m| m.description.clone())),

            search_vector: Set(/* tsvector */),
            indexed_at: Set(Utc::now().into()),
        };

        index_products::Entity::insert(index_record)
            .on_conflict(
                OnConflict::column(index_products::Column::ProductId)
                    .update_columns([
                        index_products::Column::Title,
                        index_products::Column::MinPrice,
                        // ... all fields
                        index_products::Column::IndexedAt,
                    ])
                    .to_owned()
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Full reindex for tenant
    pub async fn reindex_tenant(&self, tenant_id: Uuid) -> Result<IndexStats> {
        let mut stats = IndexStats::default();

        let products = commerce_products::Entity::find()
            .filter(commerce_products::Column::TenantId.eq(tenant_id))
            .all(&self.db)
            .await?;

        for product in products {
            match self.index_product(product.id).await {
                Ok(_) => stats.success += 1,
                Err(e) => {
                    tracing::error!("Failed to index product {}: {:?}", product.id, e);
                    stats.failed += 1;
                }
            }
        }

        stats.total = stats.success + stats.failed;
        Ok(stats)
    }
}

#[async_trait]
impl EventHandler for ProductIndexer {
    fn handles(&self, event: &DomainEvent) -> bool {
        matches!(
            event,
            DomainEvent::ProductCreated { .. }
                | DomainEvent::ProductUpdated { .. }
                | DomainEvent::ProductPublished { .. }
                | DomainEvent::VariantUpdated { .. }
                | DomainEvent::InventoryUpdated { .. }
        )
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let product_id = match &envelope.event {
            DomainEvent::ProductCreated { product_id } => *product_id,
            DomainEvent::ProductUpdated { product_id } => *product_id,
            DomainEvent::ProductPublished { product_id } => *product_id,
            DomainEvent::VariantUpdated { product_id, .. } => *product_id,
            DomainEvent::InventoryUpdated { variant_id, .. } => {
                // Lookup product_id from variant
                self.get_product_id_by_variant(*variant_id).await?
            }
            _ => return Ok(()),
        };

        self.index_product(product_id).await
    }
}
```

### 7.3 Content Indexer

```rust
// crates/rustok-index/src/indexers/content_indexer.rs

pub struct ContentIndexer {
    db: DatabaseConnection,
}

impl ContentIndexer {
    pub async fn index_node(&self, node_id: Uuid) -> Result<()> {
        let node = nodes::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or(RusToKError::not_found::<nodes::Entity>(node_id))?;

        // Fetch body
        let body = bodies::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?;

        // Fetch category
        let category = if let Some(cat_id) = node.category_id {
            categories::Entity::find_by_id(cat_id).one(&self.db).await?
        } else {
            None
        };

        // Fetch author
        let author = if let Some(author_id) = node.author_id {
            users::Entity::find_by_id(author_id).one(&self.db).await?
        } else {
            None
        };

        // Fetch tags
        let tags = taggables::Entity::find()
            .filter(taggables::Column::TargetType.eq("node"))
            .filter(taggables::Column::TargetId.eq(node_id))
            .find_also_related(tags::Entity)
            .all(&self.db)
            .await?;

        // Fetch meta
        let meta = meta::Entity::find()
            .filter(meta::Column::TargetType.eq("node"))
            .filter(meta::Column::TargetId.eq(node_id))
            .one(&self.db)
            .await?;

        // Build index record
        let index_record = index_content::ActiveModel {
            id: Set(generate_id()),
            tenant_id: Set(node.tenant_id),
            node_id: Set(node_id),

            kind: Set(node.kind),
            title: Set(node.title),
            slug: Set(node.slug),
            excerpt: Set(node.excerpt),
            body_preview: Set(body.as_ref().map(|b| truncate(&b.body, 500))),
            status: Set(node.status),

            author_id: Set(node.author_id),
            author_name: Set(author.map(|a| a.email)), // или name если есть

            category_id: Set(node.category_id),
            category_name: Set(category.as_ref().map(|c| c.name.clone())),
            category_slug: Set(category.as_ref().map(|c| c.slug.clone())),

            tags: Set(tags.iter().filter_map(|(_, t)| t.as_ref().map(|t| t.name.clone())).collect()),
            parent_id: Set(node.parent_id),
            reply_count: Set(node.reply_count),

            meta_title: Set(meta.as_ref().and_then(|m| m.title.clone())),
            meta_description: Set(meta.as_ref().and_then(|m| m.description.clone())),

            search_vector: Set(/* tsvector */),
            published_at: Set(node.published_at),
            indexed_at: Set(Utc::now().into()),
        };

        // Upsert
        index_content::Entity::insert(index_record)
            .on_conflict(/* ... */)
            .exec(&self.db)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for ContentIndexer {
    fn handles(&self, event: &DomainEvent) -> bool {
        matches!(
            event,
            DomainEvent::NodeCreated { .. }
                | DomainEvent::NodeUpdated { .. }
                | DomainEvent::NodePublished { .. }
        )
    }

    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        let node_id = match &envelope.event {
            DomainEvent::NodeCreated { node_id, .. } => *node_id,
            DomainEvent::NodeUpdated { node_id } => *node_id,
            DomainEvent::NodePublished { node_id, .. } => *node_id,
            _ => return Ok(()),
        };

        self.index_node(node_id).await
    }
}
```

---

## 8. DEPLOYMENT ARCHITECTURE

### 8.1 Monolith (Default)

```yaml
# docker-compose.yml
services:
  rustok:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db:5432/rustok
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis

  db:
    image: postgres:16
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=rustok
      - POSTGRES_PASSWORD=rustok
      - POSTGRES_DB=rustok

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### 8.2 Microservices (Scale)

```yaml
# docker-compose.scale.yml
services:
  # API Gateway
  api:
    build:
      context: .
      dockerfile: apps/server/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db-primary:5432/rustok
      - INDEX_SERVICE_URL=http://index:3001
    deploy:
      replicas: 3

  # Index Service (отдельный)
  index:
    build:
      context: .
      dockerfile: crates/rustok-index/Dockerfile
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db-replica:5432/rustok
      - MEILISEARCH_URL=http://meilisearch:7700
    deploy:
      replicas: 2

  # Primary DB (writes)
  db-primary:
    image: postgres:16
    environment:
      - POSTGRES_USER=rustok
      - POSTGRES_PASSWORD=rustok

  # Replica DB (reads for index)
  db-replica:
    image: postgres:16
    environment:
      - POSTGRES_USER=rustok
      - POSTGRES_PASSWORD=rustok
    # Настроить streaming replication

  # Full-text search
  meilisearch:
    image: getmeili/meilisearch:v1.6
    volumes:
      - meilisearch_data:/meili_data

volumes:
  meilisearch_data:
```

### 8.3 Architecture Diagram

```text
                         ┌─────────────────┐
                         │   Load Balancer │
                         └────────┬────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
       ┌────────────┐      ┌────────────┐      ┌────────────┐
       │  API Pod 1 │      │  API Pod 2 │      │  API Pod 3 │
       └─────┬──────┘      └─────┬──────┘      └─────┬──────┘
             │                   │                   │
             └───────────────────┼───────────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
              ▼                  ▼                  ▼
       ┌────────────┐     ┌────────────┐    ┌─────────────┐
       │ PostgreSQL │     │   Redis    │    │ Event Bus   │
       │  Primary   │     │  (Cache)   │    │ (In-memory) │
       └─────┬──────┘     └────────────┘    └──────┬──────┘
             │                                     │
             │ Replication                         │ Events
             ▼                                     ▼
       ┌────────────┐                      ┌─────────────┐
       │ PostgreSQL │◄─────────────────────│Index Service│
       │  Replica   │                      └──────┬──────┘
       └────────────┘                             │
                                                  ▼
                                          ┌─────────────┐
                                          │ Meilisearch │
                                          └─────────────┘
```

---

## 9. MODULE REGISTRATION (Updated)

```rust
// apps/server/src/app.rs

use rustok_core::{events::EventBus, RusToKModule};
use rustok_commerce::CommerceModule;
use rustok_community::CommunityModule;
use rustok_index::IndexModule;

pub struct App {
    event_bus: EventBus,
    modules: Vec<Box<dyn RusToKModule>>,
}

impl App {
    pub fn new() -> Self {
        let event_bus = EventBus::new(1024);

        Self {
            event_bus: event_bus.clone(),
            modules: vec![
                Box::new(CommerceModule::new(event_bus.clone())),
                Box::new(CommunityModule::new(event_bus.clone())),
                Box::new(IndexModule::new(event_bus.clone())), // CQRS sync
            ],
        }
    }
}

#[async_trait]
impl Hooks for App {
    // ... Loco hooks

    async fn after_context(ctx: &AppContext) -> Result<()> {
        // Start event dispatcher
        let mut dispatcher = EventDispatcher::new(ctx.event_bus.clone());

        // Register index handlers
        dispatcher.register(Arc::new(ProductIndexer::new(ctx.db.clone())));
        dispatcher.register(Arc::new(ContentIndexer::new(ctx.db.clone())));

        // Start background task
        dispatcher.start();

        Ok(())
    }
}
```

---

## 10. SUMMARY: What Lives Where

| Layer | Tables/Entities | Purpose |
|-------|----------------|---------|
| **Core** | users, tenants, nodes, bodies, categories, tags, taggables, meta, media, tenant_modules | Universal foundation |
| **Commerce** | products, variants, options, prices, inventory, orders, commerce_categories | E-commerce domain |
| **Community** | reactions, reputation, follows | Social features (extends nodes) |
| **Index** | index_products, index_content | CQRS read models |

---

## 11. DATA FLOW

```text
┌──────────────────────────────────────────────────────────────────┐
│                         WRITE PATH                               │
│                                                                  │
│  User Request                                                    │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌──────────────┐   │
│  │ GraphQL │───▶│ Service │───▶│   ORM   │───▶│ PostgreSQL   │   │
│  │  API    │    │  Layer  │    │(SeaORM) │    │ (normalized) │   │
│  └─────────┘    └────┬────┘    └─────────┘    └──────────────┘   │
│                      │                                           │
│                      ▼                                           │
│                 ┌─────────┐                                      │
│                 │  Event  │                                      │
│                 │   Bus   │                                      │
│                 └────┬────┘                                      │
└──────────────────────┼───────────────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│                         READ PATH                                │
│                                                                  │
│                 ┌─────────────┐                                  │
│                 │   Index     │                                  │
│                 │  Handlers   │                                  │
│                 └──────┬──────┘                                  │
│                        │                                         │
│                        ▼                                         │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                   INDEX TABLES                             │  │
│  │  ┌─────────────────┐    ┌─────────────────┐               │  │
│  │  │ index_products  │    │  index_content  │               │  │
│  │  │ (denormalized)  │    │ (denormalized)  │               │  │
│  │  └─────────────────┘    └─────────────────┘               │  │
│  └────────────────────────────────────────────────────────────┘  │
│                        │                                         │
│                        ▼                                         │
│                 ┌─────────────┐                                  │
│                 │   Search    │    (Optional: Meilisearch)       │
│                 │   Queries   │                                  │
│                 └─────────────┘                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 12. CHECKLIST (Updated)

Before implementing any feature, verify:

- Uses `Uuid` for all IDs (generated from ULID).
- Includes `tenant_id` for multi-tenant entities.
- Implements proper error handling with `RusToKError`.
- Has SeaORM entity with relations.
- Has service layer (not direct DB access in handlers).
- Publishes events for state changes.
- GraphQL resolvers check tenant context.
- Admin resource registered with proper permissions.
- Index updated via event handler (if searchable).
- Core tables used for universal features (tags, meta).
- Module-specific tables for domain logic.

---

**END OF MANIFEST v4.0**
