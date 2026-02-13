# RusToK — Чеклист критических исправлений

> **Для немедленного исполнения**

---

## 🔴 P0: Критические (безопасность/надежность) - ✅ ВЫПОЛНЕНО

### 1. TransactionalEventBus во всех модулях - ✅

- [x] **rustok-commerce/src/services/catalog.rs** - TransactionalEventBus
- [x] **rustok-commerce/src/services/inventory.rs** - TransactionalEventBus
- [x] **rustok-commerce/src/services/pricing.rs** - TransactionalEventBus
- [x] **rustok-forum/src/services/*.rs** - Все сервисы используют TransactionalEventBus
- [x] **rustok-blog/src/services/*.rs** - TransactionalEventBus
- [x] **rustok-pages/src/services/*.rs** - Все сервисы используют TransactionalEventBus
- [x] **rustok-content/src/services/*.rs** - TransactionalEventBus

### 2. Убрать `let _ =` для событий - ✅

- [x] **crates/rustok-commerce/src/services/catalog.rs** - Все события через `publish_in_tx(...).await?`
- [x] **crates/rustok-commerce/src/services/inventory.rs** - Все события через `publish_in_tx(...).await?`
- [x] **crates/rustok-commerce/src/services/pricing.rs** - Все события через `publish_in_tx(...).await?`

---

## 🟡 P1: Важные (стабильность/производительность) - ✅ ВЫПОЛНЕНО

### 3. Добавить rate limiting в EventDispatcher - ✅

- [x] **crates/rustok-core/src/events/handler.rs**
  ```rust
  pub struct DispatcherConfig {
      pub fail_fast: bool,
      pub max_concurrent: usize,
      pub retry_count: usize,
      pub retry_delay_ms: u64,
      pub max_queue_depth: usize,  // ✅ Добавлено (default: 10000)
  }
  ```

### 4. Graceful shutdown - ✅

- [x] **apps/server/src/app.rs**
  ```rust
  impl Hooks for App {
      async fn shutdown(&self, ctx: &AppContext) {
          // ✅ Остановка outbox relay worker
          // ✅ Закрытие database connections
          // ✅ Логирование shutdown sequence
      }
  }
  ```

### 5. Упрощение tenant cache - 📋 BACKLOG

- [ ] **apps/server/src/middleware/tenant.rs**
  - Рассмотреть переход на `moka::future::Cache` (не критично)

---

## 🟢 P2: Качество кода - ✅ ВЫПОЛНЕНО

### 6. Стандартизация slugify - ✅

- [x] **crates/rustok-commerce/src/services/catalog.rs**
  - Unicode NFC normalization (защита от homograph attacks)
  - ASCII-only alphanumeric фильтрация
  - Защита reserved names (admin, api, etc.)
  - Максимальная длина 255 символов

### 7. Валидация событий - ✅

- [x] **crates/rustok-core/src/events/types.rs**
  ```rust
  impl DomainEvent {
      pub fn validate(&self) -> Result<(), String> {
          // ✅ Валидация inventory событий
          // ✅ Валидация price событий
          // ✅ Валидация order событий
          // ✅ Валидация user событий
          // ✅ Валидация media событий
          // ✅ Валидация locale событий
      }
  }
  ```

---

## 📋 Порядок исправления - ✅ ЗАВЕРШЕНО

```
✅ День 1-2: P0 (TransactionalEventBus) - ВЫПОЛНЕНО
✅ День 3:   P1 (Graceful shutdown) - ВЫПОЛНЕНО
✅ День 4-5: P1 (Rate limiting) - ВЫПОЛНЕНО
✅ День 6+:  P2 (Качество кода) - ВЫПОЛНЕНО
```

---

## ✅ Проверка после исправлений

```bash
# Сборка
cargo build --release

# Тесты
cargo test --workspace

# Проверка безопасности
cargo audit

# Форматирование и линт
cargo fmt --check
cargo clippy -- -D warnings
```

---

## 🎯 Результат

Все P0 и P1 критические исправления выполнены. Платформа готова к production использованию с:
- Атомарной публикацией событий (TransactionalEventBus)
- Graceful shutdown
- Валидацией данных событий
- Защищенной генерацией slug
- Rate limiting конфигурацией
