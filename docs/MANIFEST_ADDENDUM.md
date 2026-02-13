# RusToK Manifest — Addendum

> **Дополнения к манифесту на основе архитектурного обсуждения**  
> **Дата:** 2026-02-06  
> **Добавить в:** `RUSTOK_MANIFEST.md` как новые секции

---

## 26. HYBRID CONTENT STRATEGY

### 26.1 Principle

RusToK использует **гибридный подход** к контенту:

| Слой | Описание | Примеры |
|------|----------|---------|
| **Core Logic (Rust)** | Критические данные в строгих структурах | Products, Orders, Users |
| **Marketing Logic (Flex)** | Маркетинговый контент через конструктор | Лендинги, формы, баннеры |
| **Integration** | Flex индексируется в общий Index module | Единый поиск |

### 26.2 Decision

- **Основной упор:** стандартные схемы и модули (нормализованные таблицы)
- **Flex:** подключается только для edge-cases
- **Не плодим зависимости:** стандартные модули не зависят от Flex

---

## 27. FLEX MODULE PRINCIPLE

> **Новый модуль, появившийся из архитектурного обсуждения**

### 27.1 Definition

**Flex (Generic Content Builder)** — опциональный вспомогательный модуль-конструктор данных для ситуаций, когда стандартных модулей недостаточно.

### 27.2 Hard Rules

| # | Rule | Status |
|---|------|--------|
| 1 | Flex is **OPTIONAL** | ✅ Approved |
| 2 | Standard modules NEVER depend on Flex | ✅ Approved |
| 3 | Flex depends only on rustok-core | ✅ Approved |
| 4 | **Removal-safe:** платформа работает без Flex | ✅ Approved |
| 5 | Integration via events/index, not JOIN | ✅ Approved |

### 27.3 Guardrails

| Constraint | Value | Status |
|------------|-------|--------|
| Max fields per schema | 50 | ⬜ TODO |
| Max nesting depth | 2 | ⬜ TODO |
| Max relation depth | 1 | ⬜ TODO |
| Mandatory pagination | Yes | ⬜ TODO |
| Strict validation on write | Yes | ⬜ TODO |

### 27.4 Decision Tree

```
Нужны кастомные данные?
    ↓
Закрывается стандартным модулем?
    → Да → Используй стандартный модуль
    → Нет → Оправдано создание нового модуля?
        → Да → Создай доменный модуль
        → Нет → Используй Flex
```

---

## 28. MODULE CONTRACTS FIRST

### 28.1 Decision

Перед реализацией бизнес-логики модулей — определить контракты для **всех** планируемых модулей.

### 28.2 Contract Contents

Для каждого модуля определить:

| Артефакт | Описание |
|----------|----------|
| Tables/Migrations | SQL-схемы с `tenant_id` |
| Events | Emit/consume + payload contracts |
| Index schemas | Read model таблицы |
| Permissions | RBAC permissions list |
| API stubs | GraphQL-стабы для UI + REST-стабы для integrations/service flows |
| Integration tests | Cross-module scenarios |

### 28.3 Implementation

- ⬜ TODO: Создать `docs/modules/<module>.md` для каждого модуля
- ⬜ TODO: Использовать шаблон `docs/templates/module_contract.md`

---

## 29. REFERENCE SYSTEMS POLICY

### 29.1 Decision

Внешние системы (VirtoCommerce, phpFox, etc.) используются как **design/architecture references**, не как code dependencies.

### 29.2 Rules

| # | Rule |
|---|------|
| 1 | Copy **WHAT** (entities, fields, scenarios), not **HOW** (code) |
| 2 | `references/` directory in `.gitignore` |
| 3 | Only derived docs (module-map, events, db-notes) go to git |
| 4 | No committing proprietary sources |
| 5 | Rust 1:1 port impossible and not needed |

### 29.3 Reference Sources

| System | Use For |
|--------|---------|
| VirtoCommerce | Commerce module decomposition |
| phpFox | Social graph, activity feed |
| Medusa/Discourse | Feature parity, module design |

---

## 30. CONTENT ↔ COMMERCE STRATEGY

### 30.1 Decision

Commerce **владеет** своими данными (SEO, rich description). Indexer собирает композитную картину.

### 30.2 Rejected Approach

```
❌ Product.node_id → Content.nodes
```

Причина: создаёт скрытую связь между bounded contexts.

### 30.3 Approved Approach

```
✅ Commerce: owns SEO fields + rich description (JSONB)
✅ Index: builds composite read model from events
```

---

## 31. MIGRATIONS CONVENTION

### 31.1 Naming Format

```
mYYYYMMDD_<module>_<nnn>_<description>.rs
```

### 31.2 Examples

```
m20250201_content_001_create_nodes.rs
m20250201_content_002_create_bodies.rs
m20250201_commerce_001_create_products.rs
m20250201_commerce_002_create_variants.rs
```

### 31.3 Rules

| # | Rule | Status |
|---|------|--------|
| 1 | Module prefix prevents collisions | ⬜ TODO |
| 2 | One migration = one goal | ⬜ TODO |
| 3 | Coordinate via module prefix | ⬜ TODO |

---

## 32. DEVELOPMENT STRATEGY

### 32.1 Philosophy

> "Или идеально, или говно"

Архитектурные контракты должны быть корректными на уровне компилятора.

### 32.2 Three Phases

| Phase | Name | Goal |
|-------|------|------|
| 1 | **The Forge** | Core + Admin stability |
| 2 | **House Blueprint** | Module contracts & skeletons |
| 3 | **Construction** | Business logic by priority |

### 32.3 Key Principle

**Stabilize before scale:** пока ядро и админка не стабильны — не добавляем новые фичи, только минимальные реализации.

---

## 33. ADMIN AS ARCHITECTURE TESTER

### 33.1 Principle

Админка — не UI-проект, а **архитектурный тестер**.

### 33.2 MVP Focus

| Priority | Description |
|----------|-------------|
| High | API/contracts working correctly |
| Low | UI polish (later) |

### 33.3 Checklist

Админка должна уметь:

- ⬜ Tenant CRUD
- ⬜ Enable/disable модули
- ⬜ Module config editing
- ⬜ CRUD базовых сущностей
- ⬜ View events/index status
- ⬜ RBAC management

---

## Implementation Status

| Section | Status |
|---------|--------|
| 26. Hybrid Content Strategy | ✅ Documented |
| 27. Flex Module Principle | ⬜ TODO: Implement |
| 28. Module Contracts First | ⬜ TODO: Create docs |
| 29. Reference Systems Policy | ⬜ TODO: Create references/ |
| 30. Content ↔ Commerce | ⬜ TODO: Verify implementation |
| 31. Migrations Convention | ⬜ TODO: Apply to existing |
| 32. Development Strategy | ✅ Active |
| 33. Admin as Tester | ⬜ TODO: MVP checklist |

---

## См. также

- [ROADMAP.md](./ROADMAP.md)
- [ARCHITECTURE_GUIDE.md](./ARCHITECTURE_GUIDE.md)  
- [modules/flex.md](./modules/flex.md)

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
