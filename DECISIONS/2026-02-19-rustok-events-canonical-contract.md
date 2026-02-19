# Canonical event contract in `rustok-events` (Phase 2/3)

- Date: 2026-02-19
- Status: Proposed

## Context

Сейчас каноническое определение `DomainEvent` остаётся в `rustok-core`, а `rustok-events` работает как совместимый re-export слой. Это оставляет жёсткую связанность: добавление или изменение payload доменного события требует правки core crate, что нарушает Open/Closed и увеличивает blast radius для платформенных релизов.

Одновременно есть уже принятый шаг Phase 1: точка импорта событийных контрактов выровнена на `rustok-events`, поэтому можно закрыть миграцию до конца без одномоментного big-bang.

## Decision

1. Сделать `rustok-events` каноническим источником `DomainEvent` и схем payload (Phase 2).
2. Оставить в `rustok-core` временный compatibility-layer (re-export + deprecation note) до следующего release train.
3. В следующей breaking-фазе удалить legacy re-export в `rustok-core` и перевести все импорты на `rustok-events` (Phase 3).
4. Зафиксировать migration checklist:
   - обновление импортов во всех `rustok-*` модулях;
   - обновление event schema snapshots/кодогенерации;
   - проверка обратной совместимости сериализации (`event_type`, `schema_version`).

## Consequences

**Плюсы**
- Новые доменные события можно эволюционировать без изменения `rustok-core`.
- Снижается связность между platform foundation и доменными crate'ами.
- Появляется явная граница ответственности для event contracts.

**Риски и минусы**
- Требуется координированная миграция импортов и тестов по всем модулям.
- Возможен breaking impact для внешних интеграций, импортирующих события из `rustok-core`.

**Follow-up**
- Подготовить отдельный PR на Phase 2 (canonical move + совместимость).
- Подготовить отдельный PR на Phase 3 (удаление legacy слоя) после коммуникации breaking change.
