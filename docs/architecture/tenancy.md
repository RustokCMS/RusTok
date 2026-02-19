# Tenancy

RusToK является multi-tenant платформой по умолчанию. Изоляция данных и резолюция tenant-а реализованы на уровне платформы и затрагивают каждый запрос.

## Ключевые принципы

- Каждая сущность в БД обязана иметь поле `tenant_id` — это инвариант платформы.
- Запрос без `tenant_id` в WHERE-clause является критической ошибкой безопасности.
- Tenant резолюция происходит в middleware до того, как запрос достигает бизнес-логики.
- Включение/отключение модулей управляется на уровне tenant через таблицу `tenant_modules`.

## Tenant резолюция

Middleware `TenantContext` идентифицирует tenant по одному из трёх ключей:
- `uuid` — прямой идентификатор
- `slug` — человекочитаемый идентификатор
- `host` — hostname запроса (для поддоменов)

**Crate:** `crates/rustok-tenant`  
**Точка входа в сервере:** `apps/server/src/middleware/tenant.rs`

## Tenant Cache v2

Текущая реализация использует `TenantCacheInfrastructure`, хранящийся в `AppContext.shared_store`. Кэш двухслойный:

| Слой | Назначение | TTL | Capacity |
|------|-----------|-----|----------|
| Positive cache | Tenant context по найденным ключам | 5 мин | 1000 |
| Negative cache | Not-found lookups (защита от несуществующих tenant-ов) | 60 сек | 1000 |

**Backend selection:**
- При наличии `RUSTOK_REDIS_URL` (или `REDIS_URL`) — используется `RedisCacheBackend`.
- Иначе — in-memory fallback.

**Версионированные ключи:** формат `v1:<type>:<value>`, плюс отдельные ключи для negative cache. Это позволяет инвалидировать кэш при смене схемы без полного сброса.

## Cross-instance инвалидация

При обновлении tenant или domain данных публикуется сообщение в Redis pub/sub канал `tenant.cache.invalidate`. Каждый инстанс сервера подписан на этот канал и инвалидирует matching positive + negative ключи локально. Это обеспечивает согласованность кэша в multi-instance деплое без race conditions.

## Cache Stampede Protection

Реализован паттерн **Singleflight (request coalescing)**: при cache miss несколько конкурентных запросов одного tenant-а выполняют ровно **один** запрос к БД, остальные ожидают результата.

Подробно: [`docs/tenant-cache-stampede-protection.md`](../tenant-cache-stampede-protection.md)

## Метрики

`/metrics` экспортирует счётчики tenant cache hit/miss и negative-cache. В Redis-режиме счётчики хранятся в Redis, поэтому метрики отражают состояние shared cache (а не только локального процесса).

## Таблицы БД

```sql
-- Арендаторы
CREATE TABLE tenants (
    id         UUID PRIMARY KEY,
    name       VARCHAR(255) NOT NULL,
    slug       VARCHAR(64)  NOT NULL UNIQUE,
    settings   JSONB        NOT NULL DEFAULT '{}',
    is_active  BOOLEAN      NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Включённые модули для каждого tenant-а
CREATE TABLE tenant_modules (
    id          UUID PRIMARY KEY,
    tenant_id   UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module_slug VARCHAR(64) NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT true,
    settings    JSONB       NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, module_slug)
);
```

## Связанные документы

- [Architecture overview](./overview.md)
- [Tenant cache stampede protection](../tenant-cache-stampede-protection.md)
- [Tenant module docs](../../crates/rustok-tenant/docs/README.md)
- [Tenant module implementation plan](../../crates/rustok-tenant/docs/implementation-plan.md)
