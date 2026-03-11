# Loco Mailer и Storage как server-infra слой (без выделения в отдельный модуль)

- Date: 2026-03-11
- Status: Accepted

## Context

В сервере RusToK уже есть anti-duplication матрица Loco vs самопис (`apps/server/docs/LOCO_FEATURE_SUPPORT.md`).
Для двух зон остаётся высокий риск дублирования инфраструктуры:

- Mailer subsystem;
- Storage abstraction.

Параллельно в проекте действует модульная архитектура (`ModuleRegistry`, `ModuleKind::Core/Optional`), где доменные модули `crates/rustok-*` решают предметные задачи и не должны превращаться в слой инфраструктурных обёрток вокруг framework-возможностей.

Нужно зафиксировать границу: должны ли Mailer/Storage становиться отдельными модулями платформы или это часть server infrastructure.

## Decision

1. **Mailer и Storage закрепляются как инфраструктурный слой `apps/server`, построенный на Loco API**:
   - Mailer: Loco Mailer API как основной integration contract;
   - Storage: Loco Storage abstraction как единый upload/assets contract.
2. **Не создавать отдельные платформенные модули `crates/rustok-*` для Mailer/Storage**.
3. Доменные модули используют Mailer/Storage через server-level adapters/policies (единые точки интеграции), без собственной дублирующей infra-реализации.
4. Для отклонений от этого правила требуется отдельный ADR с обоснованием trade-off и migration plan.

## Consequences

### Плюсы

- Убираем риск параллельных реализаций одного infra-слоя.
- Сохраняем чистые границы: доменные модули = domain logic, `apps/server` = runtime/infrastructure.
- Проще сопровождать совместимость с upstream Loco.

### Компромиссы

- Центр тяжести infra-изменений остаётся в `apps/server`.
- Нужна дисциплина API-границ (adapters/policies), чтобы не допустить ad-hoc вызовов из домена.

### Follow-up

1. Мигрировать текущий password-reset delivery на Loco Mailer API.
2. Ввести единый storage adapter/policy для модульных upload/use-cases.
3. Обновлять anti-duplication матрицу и server docs при изменениях Mailer/Storage.
