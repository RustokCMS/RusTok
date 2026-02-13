# RusToK Module Matrix

> Полная карта модулей и модульных crate'ов в текущем репозитории.
> 
> Последняя верификация: по коду `apps/server/src/modules/mod.rs`, `Cargo.toml` workspace и crate-манифестам.

---

## 1) Что реально зарегистрировано в `rustok-server`

Источник истины для runtime-реестра: `apps/server/src/modules/mod.rs`.

| Порядок | Slug | Crate | Статус в сервере | Кратко |
|---|---|---|---|---|
| 1 | `content` | `rustok-content` | ✅ зарегистрирован | Базовый контентный модуль |
| 2 | `commerce` | `rustok-commerce` | ✅ зарегистрирован | Каталог, цены, заказы, склад |
| 3 | `blog` | `rustok-blog` | ✅ зарегистрирован | Блоговая надстройка |
| 4 | `forum` | `rustok-forum` | ✅ зарегистрирован | Форум: категории, темы, ответы |
| 5 | `pages` | `rustok-pages` | ✅ зарегистрирован | Страницы и меню |

> Важно: `tenant`, `rbac`, `index` имеют `RusToKModule`-реализации, но **в текущей серверной сборке не регистрируются** в `build_registry()`.

---

## 2) Модульные crates с `RusToKModule`

| Slug | Crate | Роль | Зарегистрирован в сервере |
|---|---|---|---|
| `content` | `rustok-content` | CMS foundation | ✅ да |
| `commerce` | `rustok-commerce` | Commerce domain | ✅ да |
| `blog` | `rustok-blog` | Blog wrapper | ✅ да |
| `forum` | `rustok-forum` | Forum wrapper | ✅ да |
| `pages` | `rustok-pages` | Pages/menu wrapper | ✅ да |
| `tenant` | `rustok-tenant` | Tenant metadata/helpers | ❌ нет |
| `rbac` | `rustok-rbac` | Access control helpers | ❌ нет |
| `index` | `rustok-index` | Read model / indexing | ❌ нет |

---

## 3) Инфраструктурные crates (не registry-модули)

| Crate | Роль |
|---|---|
| `rustok-core` | Контракты модулей, registry, события, базовые типы |
| `rustok-outbox` | Надёжная публикация событий (outbox) |
| `rustok-iggy` | L2 transport/replay через Iggy |
| `rustok-iggy-connector` | Connector-слой для Iggy runtime |
| `rustok-telemetry` | Метрики, tracing, observability |
| `rustok-mcp` | MCP toolkit/integration crate |
| `alloy-scripting` | Скриптовый движок/оркестрация скриптов |

---

## 4) Приложения и модульный контекст

| App | Package | Роль |
|---|---|---|
| `apps/server` | `rustok-server` | API-сервер, держит `ModuleRegistry` |
| `apps/admin` | `rustok-admin` | Админ UI |
| `apps/storefront` | `rustok-storefront` | Витрина (Leptos SSR) |
| `apps/mcp` | `rustok-mcp-server` | MCP stdio-сервер, использует `rustok-mcp` |

---

## 5) Актуальный фрагмент регистрации модулей

```rust
pub fn build_registry() -> ModuleRegistry {
    ModuleRegistry::new()
        .register(ContentModule)
        .register(CommerceModule)
        .register(BlogModule)
        .register(ForumModule)
        .register(PagesModule)
}
```

---

## 6) Практические замечания

- Если нужна модульная функциональность `tenant`/`rbac`/`index` в runtime-реестре, их нужно явно добавить в `apps/server/src/modules/mod.rs`.
- Для разделения понятий в документации полезно использовать два статуса:
  - **"реализован как module crate"** (`impl RusToKModule` есть);
  - **"зарегистрирован в runtime"** (добавлен в `build_registry()`).

---

## См. также

- `docs/modules/modules.md` — обзорная документация по модульной структуре.
- `docs/modules/module-registry.md` — lifecycle/guards/toggle-логика.
- `docs/modules/module-manifest.md` — манифест и rebuild-подход.
