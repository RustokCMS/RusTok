# Периодический план верификации ядра RusToK

**Дата создания:** 2026-03-12
**Частота проверки:** при каждом существенном PR / по расписанию

> Этот документ — чеклист для периодической проверки ядра платформы.
> Цель: убедиться, что ядро сохраняет целостность архитектуры, AI-агенты
> не внедрили дублирующий самопис, и все контракты работают корректно.

---

## 1. Core Agnosticism — ядро не знает о доменных модулях

> [!CAUTION]
> Самая частая ошибка агентов — добавить domain-specific код прямо в server.

### 1.1 Hard-coded imports в ядре

```bash
# В apps/server/src/ НЕ ДОЛЖНО быть прямых use из доменных модулей
# (кроме ModuleRegistry / trait imports)
grep -rn "use rustok_content" apps/server/src/ --include="*.rs"
grep -rn "use rustok_commerce" apps/server/src/ --include="*.rs"
grep -rn "use rustok_blog" apps/server/src/ --include="*.rs"
grep -rn "use rustok_forum" apps/server/src/ --include="*.rs"
grep -rn "use rustok_pages" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Нет результатов, кроме:
- `graphql/schema.rs` — KNOWN ISSUE до Фазы 4 (dynamic registration).
- `graphql/{content,blog,commerce,forum,pages}/` — допустимо ТОЛЬКО если модуль-scope GraphQL.
- `app.rs` — регистрация модулей через `ModuleRegistry::register()`.

### 1.2 `schema.rs` — проверка на domain coupling

```bash
grep -n "Query\|Mutation" apps/server/src/graphql/schema.rs
```

**Ожидаемый результат (текущий/known):** `ContentQuery`, `BlogQuery`, и т.д. — помечены в интеграционном плане. После Фазы 4 должны исчезнуть.

### 1.3 `rustok-core` — не содержит domain logic

```bash
# core НЕ ДОЛЖЕН знать о конкретных модулях
grep -rn "content\|commerce\|blog\|forum\|pages" crates/rustok-core/src/ --include="*.rs"
```

**Ожидаемый результат:** Нет совпадений (допустимы имена в doc comments / module examples).

---

## 2. Кеширование — собственная реализация, не Loco Cache

### 2.1 Никто не добавил `loco_rs::cache`

```bash
grep -rn "loco_rs::cache\|loco_rs::prelude::cache\|CacheDriver" apps/server/src/ crates/ --include="*.rs"
```

**Ожидаемый результат:** Нет совпадений. Используется ТОЛЬКО `rustok_core::CacheBackend`.

### 2.2 CacheBackend trait не изменён без ADR

Проверить файл `crates/rustok-core/src/context.rs` — trait `CacheBackend` должен содержать:
- `health()`, `get()`, `set()`, `set_with_ttl()`, `invalidate()`, `stats()`

### 2.3 FallbackCacheBackend жив

```bash
grep -rn "FallbackCacheBackend" crates/rustok-core/src/cache.rs
```

**Ожидаемый результат:** Struct + impl блок существуют.

### 2.4 Circuit breaker на Redis backend

```bash
grep -rn "CircuitBreaker" crates/rustok-core/src/cache.rs
```

**Ожидаемый результат:** Используется в `RedisCacheBackend`.

### 2.5 Anti-stampede coalescing работает

```bash
grep -rn "in_flight\|get_or_load_with_coalescing" apps/server/src/middleware/tenant.rs
```

**Ожидаемый результат:** Оба присутствуют в `TenantCacheInfrastructure`.

---

## 3. Event Bus — Outbox, не Loco Queue

### 3.1 Никто не добавил Loco Queue для событий

```bash
grep -rn "loco_rs::bgworker\|loco_rs::queue\|QueueProvider" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Нет совпадений (кроме Hooks::connect_workers signature).

### 3.2 Outbox relay жив и настроен

```bash
grep -rn "spawn_outbox_relay_worker\|OutboxRelay" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Вызывается в `app.rs` / `connect_workers`.

### 3.3 EventTransport trait не изменён

Файл `crates/rustok-core/src/events.rs` — trait `EventTransport`.

### 3.4 Transactional event bus работает через outbox

```bash
grep -rn "TransactionalEventBus" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Используется в GraphQL schema и сервисных слоях.

---

## 4. Email — текущее состояние и миграция

### 4.1 Email service существует

```bash
ls apps/server/src/services/email.rs
```

### 4.2 После Фазы 2: Loco Mailer используется

```bash
grep -rn "loco_rs::mailer\|Mailer" apps/server/src/services/email.rs
```

**Ожидаемый результат (до Фазы 2):** Нет совпадений (используется `lettre`).
**Ожидаемый результат (после Фазы 2):** Loco Mailer adapter присутствует.

### 4.3 Нет inline HTML в email (после Фазы 2)

```bash
grep -rn "<p>\|<a href\|<html" apps/server/src/services/email.rs
```

**Ожидаемый результат (после Фазы 2):** Нет — все через шаблоны.

---

## 5. Settings — YAML vs DB

### 5.1 `RustokSettings` загружается с DB fallback (после Фазы 1)

```bash
grep -rn "SettingsService\|platform_settings" apps/server/src/ --include="*.rs"
```

### 5.2 YAML не дублирует DB

Проверить `config/development.yaml` — должен содержать только bootstrap defaults.

### 5.3 Module settings не пустые (после Фазы 4)

```bash
# GraphQL query tenantModules должен возвращать settings != {}
curl -s http://localhost:5150/graphql -H 'Content-Type: application/json' \
  -d '{"query":"{ tenantModules { moduleslug settings } }"}' | jq '.data.tenantModules[] | select(.settings == "{}")'
```

**Ожидаемый результат (после Фазы 4):** Пустой ответ (нет пустых settings).

---

## 6. i18n — многоязычность по умолчанию (после Фазы 0)

### 6.1 Locale resolution middleware существует

```bash
grep -rn "locale\|Accept-Language\|i18n" apps/server/src/middleware/ --include="*.rs"
```

### 6.2 API ошибки локализованы

```bash
# Проверить что FieldError сообщения не hardcoded
grep -rn "FieldError::new(\"" apps/server/src/graphql/ --include="*.rs" | head -20
```

**Ожидаемый результат (после Фазы 0):** Все через i18n keys, не строковые литералы.

### 6.3 Модули предоставляют translation bundles

```bash
grep -rn "fn translations" crates/rustok-*/src/ --include="*.rs"
```

---

## 7. RBAC — собственная реализация

### 7.1 RBAC engine из rustok-rbac

```bash
grep -rn "AuthService::has_permission\|AuthService::has_any_permission" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Все проверки через `AuthService`, не custom middleware.

### 7.2 Нет дублирующих auth middleware

```bash
grep -rn "fn check_permission\|fn verify_role" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Нет adhoc проверок мимо `AuthService`.

---

## 8. Tenant Resolution — инфраструктура

### 8.1 Tenant middleware работает

```bash
grep -rn "TenantCacheInfrastructure\|init_tenant_cache_infrastructure" apps/server/src/ --include="*.rs"
```

### 8.2 Redis pub/sub invalidation listener

```bash
grep -rn "TENANT_INVALIDATION_CHANNEL\|spawn_invalidation_listener" apps/server/src/ --include="*.rs"
```

### 8.3 Negative cache существует

```bash
grep -rn "negative_cache\|set_negative\|check_negative" apps/server/src/middleware/tenant.rs
```

---

## 9. Loco Integration — правильное использование Framework

### 9.1 Все маршруты через Hooks::routes / after_routes

```bash
# Не должно быть standalone Router::new() без интеграции в Loco
grep -rn "axum::Router::new()" apps/server/src/ --include="*.rs" | grep -v "test"
```

**Ожидаемый результат:** Нет (или только в модульных sub-routers, подключённых через Loco).

### 9.2 AppContext через State, не глобальные переменные

```bash
grep -rn "lazy_static\|static.*OnceCell\|static.*Mutex" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Нет runtime state мимо `AppContext.shared_store`.

### 9.3 Ошибки через loco_rs::Result

```bash
# Handlers должны возвращать loco Result, не axum IntoResponse напрямую
grep -rn "impl IntoResponse" apps/server/src/controllers/ --include="*.rs"
```

**Ожидаемый результат:** Нет custom IntoResponse в controllers.

---

## 10. Модульная система — целостность

### 10.1 ModuleRegistry содержит все зарегистрированные модули

```bash
grep -rn "ModuleRegistry::new()\|\.register(" apps/server/src/app.rs
```

### 10.2 Module lifecycle hooks вызываются

```bash
grep -rn "on_enable\|on_disable" apps/server/src/services/module_lifecycle.rs
```

### 10.3 `modules.toml` манифест валиден

```bash
# Проверить что все модули из modules.toml имеют соответствующие crate
cat modules.toml
```

---

## 11. Storage — двухслойная архитектура (после Фазы 3)

> Loco Storage = транспортный слой. `StorageAdapter` = CMS-логика поверх.

### 11.1 StorageAdapter существует и используется

```bash
grep -rn "StorageAdapter" apps/server/src/ --include="*.rs"
```

**Ожидаемый результат:** Trait + impl в `services/storage/`.

### 11.2 Никто не вызывает Loco Storage напрямую (мимо adapter)

```bash
# Прямой вызов loco storage мимо StorageAdapter — антипаттерн
grep -rn "storage\.upload\|storage\.download" apps/server/src/controllers/ --include="*.rs"
grep -rn "loco_rs::storage" apps/server/src/controllers/ --include="*.rs"
```

**Ожидаемый результат:** Нет — контроллеры используют только `StorageAdapter`.

### 11.3 Файлы организованы по дате и tenant

```bash
# Проверить что пути содержат tenant_id/YYYY/MM/ pattern
grep -rn "tenant_id.*YYYY\|Utc::now.*format\|chrono.*format.*%Y.*%m" apps/server/src/services/storage/ --include="*.rs"
```

### 11.4 media_assets таблица существует

```bash
grep -rn "media_assets" apps/server/src/models/ migration/ --include="*.rs"
```

### 11.5 Нет ad-hoc upload мимо StorageAdapter

```bash
grep -rn "multipart\|tokio::fs::write\|std::fs::write" apps/server/src/controllers/ --include="*.rs"
```

**Ожидаемый результат:** Нет — все через `StorageAdapter`.

---

## 12. Observability — telemetry и health

### 12.1 Telemetry initializer

```bash
ls apps/server/src/initializers/telemetry.rs
```

### 12.2 Health endpoint работает

```bash
curl -s http://localhost:5150/api/_health | jq .
```

### 12.3 Metrics endpoint

```bash
curl -s http://localhost:5150/api/_metrics | head -20
```

---

## 13. Антипаттерны — что НЕ должно появиться

| Антипаттерн | Как обнаружить | Серьёзность |
|---|---|---|
| Loco Cache вместо CacheBackend | `grep "loco_rs::cache"` | 🔴 Критичная |
| Loco Queue вместо Outbox | `grep "loco_rs::bgworker" \| grep "QueueProvider"` | 🔴 Критичная |
| Domain imports в core crate | `grep "content\|commerce" crates/rustok-core/` | 🔴 Критичная |
| Static globals мимо AppContext | `grep "lazy_static\|OnceCell"` | 🟡 Высокая |
| Inline SQL вместо SeaORM | `grep "raw_sql\|execute_unprepared"` в новом коде | 🟡 Высокая |
| Новый HTTP client вместо Loco fetch | `grep "reqwest::Client::new()"` в ядре | 🟢 Средняя |
| Custom auth middleware мимо AuthService | Ручной `fn check_role` | 🟡 Высокая |
| Hard-coded tenant ID в бизнес-логике | `grep "00000000-0000-0000-0000"` в не-config файлах | 🟡 Высокая |

---

## Как проводить верификацию

1. **Автоматическая:** Добавить проверки из §§1–11 в CI как lint step (grep-based). Fail on match.
2. **Ручная (периодическая):** Пройти чеклист вручную раз в спринт / при крупном PR.
3. **Agent pre-commit:** AI-агенты обязаны сверяться с этим документом перед любым изменением в `apps/server/` или `crates/rustok-core/`.

> **Правило для агентов:** Если вы собираетесь добавить новую зависимость, middleware, или
> инфраструктурный сервис в `apps/server/` — **сначала** проверьте по этому документу,
> не дублирует ли это уже существующее решение.
