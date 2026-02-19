# Module Contract Template

> **Шаблон для документирования модулей RusToK**  
> Скопируйте этот файл в `docs/modules/<module-name>.md`

---

## 1. Purpose & Scope

### 1.1 Overview

**Название модуля:** `rustok-<name>`  
**Slug:** `<slug>`  
**Тип:** Core Component / Domain Module / Wrapper Module / Infrastructure  

### 1.2 Description

_Краткое описание назначения модуля (2-3 предложения)_

### 1.3 Bounded Context

_Какую бизнес-область покрывает модуль? С какими другими контекстами граничит?_

### 1.4 Dependencies

| Module | Type | Description |
|--------|------|-------------|
| `rustok-core` | Required | Events, traits, errors |
| _..._ | _..._ | _..._ |

---

## 2. Data Model

### 2.1 Tables

```sql
-- Основная таблица
CREATE TABLE <module>_<entity> (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    -- fields...
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 2.2 Indexes

```sql
CREATE INDEX idx_<table>_<field> ON <table> (<field>);
```

### 2.3 Relations

_Описание связей между таблицами модуля_

---

## 3. Domain Events

### 3.1 Emitted Events

| Event | Payload | When |
|-------|---------|------|
| `<Entity>Created` | `{ id: Uuid, ... }` | После создания |
| `<Entity>Updated` | `{ id: Uuid, ... }` | После обновления |
| `<Entity>Deleted` | `{ id: Uuid }` | После удаления |

### 3.2 Consumed Events

| Event | Source | Action |
|-------|--------|--------|
| _..._ | _..._ | _..._ |

### 3.3 Event Payload Contracts

```rust
// В rustok-core/src/events/types.rs

<Entity>Created { 
    <field>: Type,
    // ...
},
```

---

## 4. APIs

### 4.1 REST Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | `/api/v1/<entities>` | List all | Required |
| POST | `/api/v1/<entities>` | Create | Required |
| GET | `/api/v1/<entities>/:id` | Get by ID | Required |
| PUT | `/api/v1/<entities>/:id` | Update | Required |
| DELETE | `/api/v1/<entities>/:id` | Delete | Required |

### 4.2 GraphQL Schema

```graphql
type <Entity> {
  id: ID!
  # fields...
}

type Query {
  <entities>: [<Entity>!]!
  <entity>(id: ID!): <Entity>
}

type Mutation {
  create<Entity>(input: Create<Entity>Input!): <Entity>!
  update<Entity>(id: ID!, input: Update<Entity>Input!): <Entity>!
  delete<Entity>(id: ID!): Boolean!
}
```

---

## 5. RBAC Permissions

| Permission | Description |
|------------|-------------|
| `<module>.<entity>.read` | Просмотр |
| `<module>.<entity>.write` | Создание/редактирование |
| `<module>.<entity>.delete` | Удаление |

```rust
fn permissions(&self) -> &[Permission] {
    &[
        Permission::new("<module>.<entity>.read"),
        Permission::new("<module>.<entity>.write"),
        Permission::new("<module>.<entity>.delete"),
    ]
}
```

---

## 6. Indexing Strategy

### 6.1 Read Model Table

```sql
CREATE TABLE index_<entities> (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    -- denormalized fields...
    search_vector   TSVECTOR,
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 6.2 Indexer Handler

```rust
impl EventHandler for <Entity>Indexer {
    fn handles(&self, event: &DomainEvent) -> bool {
        matches!(event, 
            DomainEvent::<Entity>Created { .. } |
            DomainEvent::<Entity>Updated { .. } |
            DomainEvent::<Entity>Deleted { .. }
        )
    }
    
    async fn handle(&self, envelope: &EventEnvelope) -> Result<()> {
        // Index logic
    }
}
```

---

## 7. Failure Modes & Idempotency

### 7.1 Error Cases

| Error | HTTP Code | Recovery |
|-------|-----------|----------|
| NotFound | 404 | - |
| ValidationError | 400 | Fix input |
| Conflict | 409 | Retry with fresh data |

### 7.2 Idempotency

_Как обеспечивается идемпотентность операций?_

---

## 8. Integration Tests

### 8.1 Scenarios

| # | Scenario | Status |
|---|----------|--------|
| 1 | Create → Read → Indexed | ⬜ TODO |
| 2 | Update → Event emitted → Index updated | ⬜ TODO |
| 3 | Delete → Removed from index | ⬜ TODO |
| 4 | RBAC: unauthorized access blocked | ⬜ TODO |

### 8.2 Cross-Module Flows

_Описание сценариев взаимодействия с другими модулями_

---

## 9. Implementation Checklist

| # | Task | Status |
|---|------|--------|
| 1 | Create crate `rustok-<name>` | ⬜ TODO |
| 2 | Migrations | ⬜ TODO |
| 3 | SeaORM entities | ⬜ TODO |
| 4 | DTOs (Request/Response) | ⬜ TODO |
| 5 | Services | ⬜ TODO |
| 6 | Events integration | ⬜ TODO |
| 7 | REST controllers | ⬜ TODO |
| 8 | GraphQL resolvers | ⬜ TODO |
| 9 | Indexer handler | ⬜ TODO |
| 10 | RBAC permissions | ⬜ TODO |
| 11 | Tests | ⬜ TODO |
| 12 | Documentation | ⬜ TODO |

---

## 10. Notes / Open Questions

_Любые открытые вопросы или заметки о модуле_

