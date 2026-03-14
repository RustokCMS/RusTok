# Field Schema & Custom Fields — Implementation Plan

## Scope and objective

Система кастомных полей в RusToK состоит из **двух уровней**:

1. **`rustok-core/src/field_schema.rs`** — типы, валидация, trait. Часть ядра платформы,
   аналогично `i18n.rs` или `types.rs`. Не модуль, не крейт — просто ещё один контракт в core.
2. **`apps/server`** — таблица `custom_field_definitions`, сервис CRUD, интеграция с users.
   Расширение существующих сущностей, не отдельный модуль.

### Связь с Flex

`rustok-flex` (docs/modules/flex.md) — это **прокачанная версия** того же механизма:
- Flex использует **те же `FieldType` / `ValidationRule`** из `rustok-core`
- Flex добавляет **своё хранилище** (`flex_schemas`, `flex_entries`) для standalone сущностей
- Flex — опциональный модуль; field_schema в core — обязательная часть платформы

```
rustok-core (контракт платформы)
│
├── src/field_schema.rs          ← FieldType, ValidationRule, FieldDefinition,
│                                   HasCustomFields trait, CustomFieldsSchema,
│                                   validate(), apply_defaults()
│
├── src/i18n.rs                  ← (аналогия: тоже "просто типы в core")
├── src/types.rs                 ← (аналогия: UserRole, UserStatus)
│
│   ┌─────────────────────────────────────────────────────┐
│   │  ПОТРЕБИТЕЛИ field_schema (зависят только от core)  │
│   └─────────────────────────────────────────────────────┘
│
├── apps/server                      (users живут тут)
│   ├── migration/
│   │   └── m20260315_create_custom_field_definitions.rs
│   ├── models/_entities/
│   │   └── custom_field_definitions.rs   ← SeaORM entity
│   ├── services/
│   │   └── custom_field_service.rs       ← CRUD для определений
│   ├── graphql/
│   │   └── custom_fields.rs             ← Admin API
│   └── models/users.rs                  ← impl HasCustomFields for User
│
├── rustok-commerce                  (зависит от core)
│   └── products.metadata            ← impl HasCustomFields for Product
│
├── rustok-content                   (зависит от core)
│   └── nodes.metadata               ← impl HasCustomFields for Node
│
└── rustok-flex                      (опциональный модуль, зависит от core)
    ├── flex_schemas.fields_config   ← переиспользует FieldType/ValidationRule
    ├── flex_entries.data            ← своё хранилище (standalone + attached)
    └── validate_entry()             ← вызывает core::field_schema::validate()
```

### Принципы

- **Типы в core, данные в модулях** — field_schema не знает о конкретных сущностях.
- **Flex Hard Laws соблюдены** — стандартные модули не зависят от Flex, только от core.
- **Schema-first** — админ тенанта определяет схему полей, данные валидируются при записи.
- **Tenant isolation** — схемы полей привязаны к тенанту.
- **JSONB storage** — значения в существующих `metadata` колонках, без EAV.
- **Backward compatible** — существующие `metadata` без схемы продолжают работать.

### Удаление крейта `rustok-custom-fields`

Отдельный крейт `rustok-custom-fields` **НЕ создаётся**. Вместо него:
- Типы и валидация → `rustok-core/src/field_schema.rs`
- Таблица и сервис → `apps/server/`
- Этот документ остаётся в `crates/rustok-custom-fields/docs/` как архитектурное решение,
  а при реализации переносится в `docs/architecture/field-schema.md`.

---

## Part 1: `rustok-core/src/field_schema.rs`

### 1.1 FieldType enum

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported field types for custom fields and Flex schemas.
/// Shared contract: used by custom_field_definitions, flex_schemas,
/// and any module that needs runtime-defined field types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Textarea,
    Integer,
    Decimal,
    Boolean,
    Date,
    DateTime,
    Url,
    Email,
    Phone,
    Select,
    MultiSelect,
    Color,
    Json,
}
```

### 1.2 ValidationRule

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SelectOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    /// Localized labels: {"en": "Male", "ru": "Мужской"}
    pub label: HashMap<String, String>,
}
```

### 1.3 FieldDefinition (DTO)

```rust
/// Runtime field definition. Not a DB entity — a portable DTO
/// that both custom_field_definitions rows and flex_schemas.fields_config
/// elements can be converted into.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub field_key: String,
    pub field_type: FieldType,
    pub label: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<HashMap<String, String>>,
    #[serde(default)]
    pub is_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationRule>,
    #[serde(default)]
    pub position: i32,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool { true }
```

### 1.4 HasCustomFields trait

```rust
/// Trait for entities that support custom fields via `metadata` JSONB column.
/// Each module implements this for its own entities.
pub trait HasCustomFields {
    /// Entity type key for looking up field definitions.
    /// Examples: "user", "product", "node", "topic".
    fn entity_type() -> &'static str;

    /// Returns current metadata as JSON.
    fn metadata(&self) -> &serde_json::Value;

    /// Sets metadata.
    fn set_metadata(&mut self, value: serde_json::Value);
}
```

### 1.5 CustomFieldsSchema (валидатор)

```rust
/// Schema-based validator for custom field values.
/// Constructed from a list of FieldDefinitions (loaded from DB or config).
pub struct CustomFieldsSchema {
    definitions: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldValidationError {
    pub field_key: String,
    pub message: String,
    pub error_code: FieldErrorCode,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldErrorCode {
    Required,
    InvalidType,
    TooShort,
    TooLong,
    BelowMinimum,
    AboveMaximum,
    PatternMismatch,
    InvalidOption,
    InvalidFormat,
}

impl CustomFieldsSchema {
    pub fn new(definitions: Vec<FieldDefinition>) -> Self;

    /// Validate metadata against schema. Returns errors (empty = valid).
    pub fn validate(&self, metadata: &serde_json::Value) -> Vec<FieldValidationError>;

    /// Fill in default values for missing fields.
    pub fn apply_defaults(&self, metadata: &mut serde_json::Value);

    /// Remove fields not defined in schema.
    pub fn strip_unknown(&self, metadata: &mut serde_json::Value);

    /// Active definitions only.
    pub fn active_definitions(&self) -> Vec<&FieldDefinition>;
}
```

### 1.6 validate_field_value (внутренняя функция)

```rust
/// Validate a single field value against its type and rules.
fn validate_field_value(
    key: &str,
    value: &serde_json::Value,
    field_type: FieldType,
    validation: Option<&ValidationRule>,
) -> Vec<FieldValidationError>;
```

Правила валидации по типу:

| FieldType   | JSON type        | min/max семантика      | pattern | options |
|-------------|------------------|------------------------|---------|---------|
| Text        | String           | длина строки           | да      | нет     |
| Textarea    | String           | длина строки           | да      | нет     |
| Integer     | Number (i64)     | значение               | нет     | нет     |
| Decimal     | Number (f64)     | значение               | нет     | нет     |
| Boolean     | Boolean          | —                      | нет     | нет     |
| Date        | String (ISO)     | —                      | нет     | нет     |
| DateTime    | String (ISO)     | —                      | нет     | нет     |
| Url         | String           | длина строки           | нет     | нет     |
| Email       | String           | длина строки           | нет     | нет     |
| Phone       | String           | длина строки           | да      | нет     |
| Select      | String           | —                      | нет     | да      |
| MultiSelect | Array of String  | кол-во элементов       | нет     | да      |
| Color       | String (#RRGGBB) | —                      | нет     | нет     |
| Json        | Any              | —                      | нет     | нет     |

### 1.7 Интеграция в lib.rs

```rust
// rustok-core/src/lib.rs — добавить:
pub mod field_schema;

// в pub use:
pub use field_schema::{
    CustomFieldsSchema, FieldDefinition, FieldErrorCode, FieldType,
    FieldValidationError, HasCustomFields, SelectOption, ValidationRule,
};

// в prelude:
pub use crate::field_schema::{FieldType, HasCustomFields, CustomFieldsSchema};
```

### 1.8 Тесты в core

```rust
// rustok-core/tests/ или src/field_schema.rs #[cfg(test)]

#[test] fn validate_required_field_missing() { ... }
#[test] fn validate_text_min_max_length() { ... }
#[test] fn validate_integer_range() { ... }
#[test] fn validate_select_valid_option() { ... }
#[test] fn validate_select_invalid_option() { ... }
#[test] fn validate_multiselect() { ... }
#[test] fn validate_email_format() { ... }
#[test] fn validate_url_format() { ... }
#[test] fn validate_date_iso8601() { ... }
#[test] fn validate_color_hex() { ... }
#[test] fn validate_boolean_type() { ... }
#[test] fn apply_defaults_fills_missing() { ... }
#[test] fn apply_defaults_preserves_existing() { ... }
#[test] fn strip_unknown_removes_extra_keys() { ... }
#[test] fn empty_schema_accepts_anything() { ... }
```

---

## Part 2: `apps/server` — Database & Service Layer

### 2.1 Миграция: `custom_field_definitions`

**Файл:** `apps/server/migration/src/m20260315_000001_create_custom_field_definitions.rs`

| Column          | Type           | Notes                                      |
|-----------------|----------------|--------------------------------------------|
| `id`            | UUID PK        |                                            |
| `tenant_id`     | UUID FK        | → tenants.id, ON DELETE CASCADE            |
| `entity_type`   | String(64)     | "user", "product", "node", "order"         |
| `field_key`     | String(128)    | snake_case, e.g. `phone_number`            |
| `field_type`    | String(32)     | FieldType as string                        |
| `label`         | JSONB NOT NULL | `{"en": "Phone", "ru": "Телефон"}`         |
| `description`   | JSONB nullable | localized description                      |
| `is_required`   | Boolean        | default false                              |
| `default_value` | JSONB nullable | default value                              |
| `validation`    | JSONB nullable | ValidationRule serialized                  |
| `position`      | i32            | display order, default 0                   |
| `is_active`     | Boolean        | default true                               |
| `created_at`    | TimestampTZ    |                                            |
| `updated_at`    | TimestampTZ    |                                            |

**Indexes:**
- `UNIQUE (tenant_id, entity_type, field_key)` — одно определение на ключ
- `idx_cfd_tenant_entity_active (tenant_id, entity_type, is_active)` — lookup

### 2.2 SeaORM Entity

**Файл:** `apps/server/src/models/_entities/custom_field_definitions.rs`

Стандартный SeaORM entity, маппинг 1:1 с таблицей. Relation к `tenants`.

### 2.3 CustomFieldDefinitionService

**Файл:** `apps/server/src/services/custom_field_service.rs`

```rust
pub struct CustomFieldDefinitionService;

impl CustomFieldDefinitionService {
    /// Load all active definitions for entity type → CustomFieldsSchema
    pub async fn get_schema(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
    ) -> Result<CustomFieldsSchema>;

    /// List definitions (including inactive) for admin
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
    ) -> Result<Vec<FieldDefinitionRow>>;

    /// Create new definition
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateFieldDefinitionInput,
    ) -> Result<FieldDefinitionRow>;

    /// Update definition
    pub async fn update(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        input: UpdateFieldDefinitionInput,
    ) -> Result<FieldDefinitionRow>;

    /// Soft-delete (is_active = false)
    pub async fn deactivate(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<()>;

    /// Reorder definitions
    pub async fn reorder(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        ordered_ids: Vec<Uuid>,
    ) -> Result<()>;
}
```

### 2.4 JSONB Query Helpers

**Файл:** `apps/server/src/services/custom_field_query.rs`

```rust
use sea_orm::*;

/// Build a condition to filter entities by a custom field value in metadata.
/// Example: json_eq("metadata", "company", json!("Acme")) →
///   metadata->>'company' = 'Acme'
pub fn json_eq(column: &str, key: &str, value: serde_json::Value) -> Condition;

/// Check if a key exists in metadata JSONB.
pub fn json_has_key(column: &str, key: &str) -> Condition;

/// Extract a value for sorting.
pub fn json_extract(column: &str, key: &str) -> SimpleExpr;
```

---

## Part 3: Users Integration (первый потребитель)

### 3.1 HasCustomFields для User

**Файл:** `apps/server/src/models/users.rs`

```rust
use rustok_core::field_schema::HasCustomFields;

impl HasCustomFields for Model {
    fn entity_type() -> &'static str { "user" }

    fn metadata(&self) -> &serde_json::Value {
        &self.metadata
    }

    fn set_metadata(&mut self, value: serde_json::Value) {
        self.metadata = sea_orm::JsonValue::from(value);
    }
}
```

### 3.2 GraphQL — расширение User inputs

**Файл:** `apps/server/src/graphql/types.rs`

```graphql
# Добавить в существующие inputs:
input CreateUserInput {
    email: String!
    password: String!
    name: String
    role: GqlUserRole
    status: GqlUserStatus
    customFields: JSON            # ← NEW
}

input UpdateUserInput {
    email: String
    password: String
    name: String
    role: GqlUserRole
    status: GqlUserStatus
    customFields: JSON            # ← NEW
}

# Добавить в User type:
type User {
    # ... existing fields ...
    customFields: JSON            # ← NEW: returns metadata
    customFieldDefinitions: [CustomFieldDefinition!]!  # ← NEW: schema for this entity
}
```

### 3.3 Validation flow в мутациях

```rust
// В create_user / update_user mutation:
async fn create_user(ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
    let db = ctx.data::<DatabaseConnection>()?;
    let tenant_id = ctx.data::<RequestContext>()?.tenant_id;

    // 1. Load schema for "user" entity type
    let schema = CustomFieldDefinitionService::get_schema(db, tenant_id, "user").await?;

    // 2. Prepare metadata
    let mut metadata = input.custom_fields.unwrap_or(json!({}));

    // 3. Apply defaults
    schema.apply_defaults(&mut metadata);

    // 4. Validate
    let errors = schema.validate(&metadata);
    if !errors.is_empty() {
        return Err(custom_field_validation_error(errors));
    }

    // 5. Save user with validated metadata
    let user = ActiveModel {
        metadata: Set(metadata),
        // ... other fields
    };
    // ...
}
```

---

## Part 4: Admin API для управления определениями

### 4.1 GraphQL types

```graphql
type CustomFieldDefinition {
    id: UUID!
    entityType: String!
    fieldKey: String!
    fieldType: String!
    label: JSON!
    description: JSON
    isRequired: Boolean!
    defaultValue: JSON
    validation: JSON
    position: Int!
    isActive: Boolean!
    createdAt: String!
    updatedAt: String!
}

input CreateCustomFieldDefinitionInput {
    entityType: String!       # "user", "product", etc.
    fieldKey: String!          # snake_case
    fieldType: String!         # FieldType variant
    label: JSON!               # {"en": "Phone", "ru": "Телефон"}
    description: JSON
    isRequired: Boolean
    defaultValue: JSON
    validation: JSON
    position: Int
}

input UpdateCustomFieldDefinitionInput {
    label: JSON
    description: JSON
    isRequired: Boolean
    defaultValue: JSON
    validation: JSON
    position: Int
    isActive: Boolean
}
```

### 4.2 GraphQL queries & mutations

```graphql
type Query {
    customFieldDefinitions(entityType: String!): [CustomFieldDefinition!]!
    customFieldDefinition(id: UUID!): CustomFieldDefinition
}

type Mutation {
    createCustomFieldDefinition(input: CreateCustomFieldDefinitionInput!): CustomFieldDefinition!
    updateCustomFieldDefinition(id: UUID!, input: UpdateCustomFieldDefinitionInput!): CustomFieldDefinition!
    deleteCustomFieldDefinition(id: UUID!): Boolean!
    reorderCustomFieldDefinitions(entityType: String!, ids: [UUID!]!): [CustomFieldDefinition!]!
}
```

### 4.3 RBAC

- `custom_fields.definitions.read` — Admin, SuperAdmin
- `custom_fields.definitions.write` — Admin, SuperAdmin
- `custom_fields.definitions.delete` — SuperAdmin only
- Заполнение custom fields — по правам на саму entity (кто может edit user, тот может edit custom fields)

### 4.4 Кеширование

```rust
/// In-memory cache: (tenant_id, entity_type) → CustomFieldsSchema
/// Invalidated on create/update/delete definition.
/// TTL: 5 minutes as safety net.
type SchemaCache = DashMap<(Uuid, String), (Instant, CustomFieldsSchema)>;
```

---

## Part 5: Расширение на другие модули

Каждый модуль самостоятельно:

1. Добавляет `rustok-core` в зависимости (уже есть)
2. Реализует `impl HasCustomFields for Product { entity_type() -> "product" }`
3. В своих мутациях вызывает `CustomFieldsSchema::validate()`
4. Данные хранятся в своём `metadata`

**Никакого нового крейта не нужно.** Модуль видит только `rustok-core::field_schema`.

| Модуль           | Entity   | `entity_type`  | metadata column        |
|------------------|----------|----------------|------------------------|
| apps/server      | User     | `"user"`       | `users.metadata`       |
| rustok-commerce  | Product  | `"product"`    | `products.metadata`    |
| rustok-commerce  | Order    | `"order"`      | `orders.metadata`      |
| rustok-content   | Node     | `"node"`       | `nodes.metadata`       |
| rustok-forum     | Topic    | `"topic"`      | `topics.metadata`      |

---

## Part 6: Связь с Flex

Когда `rustok-flex` будет реализован, он:

1. **Импортирует** `FieldType`, `ValidationRule`, `FieldDefinition` из `rustok-core::field_schema`
2. **Хранит** `fields_config` в `flex_schemas` как `Vec<FieldDefinition>` (сериализовано в JSONB)
3. **Валидирует** `flex_entries.data` через `CustomFieldsSchema::validate()`
4. **Не дублирует** логику валидации — вся она в core

```rust
// В rustok-flex:
use rustok_core::field_schema::{FieldDefinition, CustomFieldsSchema};

fn validate_entry(schema: &FlexSchema, data: &serde_json::Value) -> Result<()> {
    let definitions: Vec<FieldDefinition> = serde_json::from_value(schema.fields_config.clone())?;
    let validator = CustomFieldsSchema::new(definitions);
    let errors = validator.validate(data);
    if !errors.is_empty() {
        return Err(FlexError::ValidationFailed(errors));
    }
    Ok(())
}
```

Таким образом:
- Custom fields для users = определения в `custom_field_definitions` + значения в `users.metadata`
- Flex entries = определения в `flex_schemas.fields_config` + значения в `flex_entries.data`
- **Валидация одна и та же** — `CustomFieldsSchema` из core

---

## Delivery phases (summary)

### Phase 0 — Core types & validation
- [ ] Создать `rustok-core/src/field_schema.rs`
- [ ] Реализовать FieldType, ValidationRule, FieldDefinition, HasCustomFields
- [ ] Реализовать CustomFieldsSchema с validate(), apply_defaults(), strip_unknown()
- [ ] Unit-тесты (15+ test cases)
- [ ] Добавить exports в lib.rs и prelude

### Phase 1 — Database layer (apps/server)
- [ ] Миграция `custom_field_definitions`
- [ ] SeaORM entity
- [ ] CustomFieldDefinitionService (CRUD + get_schema)
- [ ] JSONB query helpers

### Phase 2 — Users integration
- [ ] `impl HasCustomFields for User`
- [ ] Расширить GraphQL inputs (customFields)
- [ ] Validation flow в мутациях
- [ ] Тесты

### Phase 3 — Admin API
- [ ] GraphQL queries/mutations для определений
- [ ] RBAC
- [ ] Schema caching с инвалидацией

### Phase 4 — Other modules (по мере необходимости)
- [ ] Commerce: Product, Order
- [ ] Content: Node
- [ ] Forum: Topic

### Phase 5 — Flex alignment
- [ ] Flex переиспользует core::field_schema
- [ ] Убрать дублирование типов в flex_schemas

### Phase 6 — Advanced (future)
- [ ] Conditional fields
- [ ] Computed fields
- [ ] Field groups
- [ ] Import/export schemas
- [ ] Search indexing
- [ ] Audit events

---

## Risks and mitigations

| Risk                                  | Mitigation                                             |
|---------------------------------------|--------------------------------------------------------|
| JSONB performance на больших объёмах  | GIN индекс на `metadata`, лимит 50 полей per entity    |
| Несогласованность схемы и данных      | Валидация при записи; CLI tool для ретро-валидации      |
| Flex дублирует custom fields логику   | Shared types в core, Flex импортирует                   |
| Breaking changes в FieldType          | FieldType extensible через `#[serde(other)]` Unknown   |

## Tracking and updates

When updating field schema architecture:

1. Update this file first.
2. Coordinate with `docs/modules/flex.md` to keep alignment.
3. Ensure Flex Hard Laws remain satisfied (Rule #1: standard modules NEVER depend on Flex).
