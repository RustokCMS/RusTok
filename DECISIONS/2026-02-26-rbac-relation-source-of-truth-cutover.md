# RBAC source of truth: relation-модель и staged cutover

- Date: 2026-02-26
- Status: Accepted

## Context

В платформе исторически сосуществуют две модели авторизации:

1. Legacy-проверки от денормализованного `users.role`.
2. Relation RBAC (`roles`, `permissions`, `user_roles`, `role_permissions`).

Параллельное существование этих подходов приводит к риску рассинхронизации и дублированию policy-логики. По состоянию migration-фаз 1–5 безопасный rollout уже опирается на relation-resolver, но часть operational и legacy-path контуров всё ещё требуется контролируемо снять без регресса доступа.

Отдельное ограничение: быстрый rollout ранее сосредоточил часть runtime-оркестрации в `apps/server::AuthService`, поэтому решение должно зафиксировать целевую границу module-first (`crates/rustok-rbac` как policy-host, `apps/server` как adapter слой).

## Decision

1. **Единственный runtime source of truth для RBAC — relation-модель.**
   - Доступ вычисляется через `roles/permissions/user_roles/role_permissions`.
   - `users.role` переводится в transitional display/debug атрибут и не участвует в policy decision.

2. **Cutover выполняется staged-стратегией:** `dual-read -> relation-only -> cleanup`.
   - `dual-read`: relation-decision авторитативен, legacy-path работает только как shadow-сравнение и источник mismatch-метрик.
   - `relation-only`: shadow и fallback выключены, решения принимаются только relation-resolver.
   - `cleanup`: удаляются legacy decision paths, временные флаги и fallback-ветки.

3. **Module-first boundary фиксируется как обязательный инвариант.**
   - `crates/rustok-rbac` хранит policy/use-case API и decision semantics.
   - `apps/server` ограничивается transport/infra адаптерами (DB/cache/wiring/metrics/logging).

4. **Rollout-gates для production cutover обязательны.**
   - Data invariants подтверждены (`users_without_roles == 0` вне согласованного whitelist).
   - `rbac_decision_mismatch_total` стабильно равен 0 в окне наблюдения перед relation-only.
   - Post-cutover наблюдение по 401/403 rate и latency не показывает регрессий.

## Consequences

### Положительные

- Снижается риск рассинхронизации прав между таблицами и `users.role`.
- Появляется единый policy-source и предсказуемая эволюция RBAC через модуль `rustok-rbac`.
- Rollout становится наблюдаемым и обратимым на этапе `dual-read`.

### Компромиссы

- На переходном этапе растёт операционная сложность (feature flags, shadow-metrics, staging rehearsal).
- Требуются дисциплина и freeze API-контрактов на время cutover-окна.

### Follow-up actions

1. Завершить фазу 4 (staging dry-run/backfill/rollback report) и приложить артефакты в migration issue.
2. Зафиксировать релизное окно для включения relation-only режима.
3. После стабилизации выполнить фазу 6 cleanup: удалить legacy/fallback и обновить runbooks.
