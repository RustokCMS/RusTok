# RusToK — Roadmap: Strategy & Phases

> **Документ создан на основе архитектурного обсуждения с AI-ассистентами.**  
> **Дата:** 2026-02-06  
> **Статус:** Draft v1

---

## Guiding Principles

| Принцип | Описание |
|---------|----------|
| **Quality Gate** | "Или идеально, или говно" — контракты должны быть корректными на уровне компилятора |
| **Stabilize before Scale** | Пока ядро и админка не стабильны — не добавляем новые фичи |
| **Contracts First** | Перед массовой реализацией — контракты для всех модулей |
| **Flex is Optional** | Стандартные модули никогда не зависят от Flex |

---

## Phase 1 — The Forge (Core + Admin Stability)

**Цель:** Вылизать ядро до состояния, где модульность и event pipeline работают без сюрпризов, а админка честно отражает систему.

### Definition of Done (все должны быть ✅)

| # | Критерий | Статус |
|---|----------|--------|
| 1.1 | Tenant lifecycle (создание → инициализация модулей → конфиги) | ⬜ TODO |
| 1.2 | Module lifecycle: load/init/stop + migrations без паник | ⬜ TODO |
| 1.3 | Event pipeline: emit → outbox → delivery → handler (idempotent) | ⬜ TODO |
| 1.4 | Index pipeline: события → read models без heavy JOINs | ⬜ TODO |
| 1.5 | Error model: типизированные ошибки, запрет `unwrap()` в prod | ⬜ TODO |
| 1.6 | RBAC: roles → permissions per module; проверки в API и админке | ⬜ TODO |
| 1.7 | Admin MVP: enable/disable модулей + CRUD + event/index status | ⬜ TODO |
| 1.8 | Health/Readiness/Liveness (K8s) с агрегацией по модулям | ⬜ TODO |

### Deliverables

- ⬜ TODO: E2E интеграционные сценарии (tenant → enable commerce → create product → indexed → disable module)
- ⬜ TODO: Документация Definition of Done как чеклист

---

## Phase 2 — House Blueprint (Module Contracts & Skeletons)

**Цель:** "Чертёж дома" — добавить каркасы модулей и проверить масштабируемость контрактов ядра.

### Phase 2a — Contracts Pack (no business logic)

| # | Задача | Статус |
|---|--------|--------|
| 2a.1 | Полный список модулей (map) с bounded context | ⬜ TODO |
| 2a.2 | Для каждого: tables/migrations, events, index, permissions, API stubs | ⬜ TODO |
| 2a.3 | Конвенция миграций: `mYYYYMMDD_<module>_<nnn>_...` | ⬜ TODO |

### Phase 2b — Skeleton Workspace

| # | Задача | Статус |
|---|--------|--------|
| 2b.1 | Создать crates для модулей (пустые сервисы `todo!()`) | ⬜ TODO |
| 2b.2 | Все модули реализуют `RusToKModule` | ⬜ TODO |
| 2b.3 | Admin отображает/включает каркасные модули | ⬜ TODO |

### Exit Criteria

- ⬜ Workspace компилируется с большим количеством модулей в разумное время
- ⬜ Все миграции применяются без конфликтов
- ⬜ Модульная регистрация стабильна

---

## Phase 3 — Construction (Business Logic by Priority)

**Цель:** Наполнение модулей реальной логикой после заморозки контрактов.

### Priority Order

| Приоритет | Модули | Статус |
|-----------|--------|--------|
| P1 | Content + Blog (core CMS flow) | ⬜ TODO |
| P2 | Commerce (revenue path) | ⬜ TODO |
| P3 | Forum / Social / Notifications | ⬜ TODO |
| P4 | Flex (optional edge-case) | ⬜ TODO |

---

## Reference Systems Policy

> **Мы берём "ЧТО" (сущности, поля, сценарии), а не "КАК" (код).**

### Источники референсов

| Система | Для чего берём |
|---------|----------------|
| VirtoCommerce | Модульная декомпозиция commerce, API-паттерны |
| phpFox | Социальный граф, activity feed, UX-паттерны |
| Medusa/Discourse | Дизайн модулей, feature parity |

### Правила

1. **`references/` в `.gitignore`** — для локальных исходников/темплейтов
2. **Clean-room процесс** — в git попадают только наши derived docs
3. **Copy WHAT, not HOW** — Rust 1:1 копирование невозможно и не нужно
4. ⬜ TODO: Создать `docs/references/` для module-map, events-catalog, db-notes

---

## Timeline (примерный)

```text
Week 1-4:  Phase 1 — Core + Admin stabilization
Week 5-6:  Phase 2a — Module contracts
Week 7-8:  Phase 2b — Skeleton workspace
Week 9+:   Phase 3 — Business logic implementation
```

---

## См. также

- [ARCHITECTURE_GUIDE.md](./ARCHITECTURE_GUIDE.md) — архитектурные принципы
- [modules/flex.md](./modules/flex.md) — спецификация Flex модуля
- [MANIFEST_ADDENDUM.md](./MANIFEST_ADDENDUM.md) — дополнения к манифесту

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
