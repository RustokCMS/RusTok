# Final cutover gate для перехода RBAC в relation-only

- Date: 2026-03-05
- Status: Accepted

## Context

После выполнения фаз 1–5 из `docs/architecture/rbac-relation-migration-plan.md` платформа уже использует relation-based resolver как авторитативный источник permission decision в runtime, а legacy role-path работает в dual-read как shadow-сравнение.

Для безопасного включения `relation-only` в production нужно формально зафиксировать:

1. Обязательные Go/No-Go критерии перед переключением.
2. Явный rollback-gate и порядок действий при деградации.
3. Связь RBAC cutover с уже принятым auth remediation gate (`scripts/auth_release_gate.sh --require-all-gates`) из ADR `2026-02-26-auth-lifecycle-unification-session-invalidation.md`.

Без отдельного ADR финального cutover критерии остаются только в migration-плане, что повышает риск неоднозначной интерпретации release-ready статуса.

## Decision

1. **Relation-only switch разрешён только при полном выполнении pre-cutover gate:**
   - Подтверждён staging rehearsal (dry-run/backfill/rollback) с инвариантами `users_without_roles_total == 0`, `orphan_user_roles_total == 0`, `orphan_role_permissions_total == 0` в финальном пост-шага отчёте.
   - Production dual-read baseline (через `scripts/rbac_cutover_baseline.sh`) показывает:
     - `rbac_decision_mismatch_total` delta = 0,
     - `rbac_shadow_compare_failures_total` delta = 0,
     - decision volume >= `min-decision-delta`.
   - Auth parity gate закрыт (`scripts/auth_release_gate.sh --require-all-gates`) и приложены актуальные артефакты к release-пакету.

2. **Rollback-gate обязателен и определяется заранее до включения relation-only:**
   - Технический rollback: немедленный возврат к предыдущему `RUSTOK_RBAC_AUTHZ_MODE` (dual-read режим) без схемных/DDL изменений.
   - Операционный trigger rollback:
     - аномальный рост 401/403 относительно baseline,
     - рост deny-rate без ожидаемого бизнес-контекста,
     - деградация latency authorization checks выше согласованного SLO,
     - рост `rbac_shadow_compare_failures_total`/индикаторов ошибок authz-path после switch.
   - Максимальное время решения о rollback в окне инцидента: 15 минут с момента подтверждённой деградации.

3. **Release evidence (обязательный пакет артефактов) фиксируется как часть change management:**
   - `artifacts/rbac-staging/*` — отчёты rehearsal-инвариантов.
   - `artifacts/rbac-cutover/*` — baseline markdown/json + raw metric samples.
   - Auth release-gate отчёт (`auth_release_gate`) из актуального релизного цикла.
   - Gate decision summary (`Go`/`No-Go`) с указанием ответственных ролей (backend, SRE/on-call, release manager).

4. **`users.role` после cutover остаётся transitional display/debug полем до завершения Phase 6 cleanup** и не возвращается как источник policy decision.

## Consequences

### Положительные

- Финальный cutover получает однозначный, аудируемый и повторяемый механизм принятия решения.
- Уменьшается риск «тихого» включения relation-only без проверенных baseline/gate.
- Явно синхронизируются RBAC и auth readiness критерии в одном release процессе.

### Компромиссы

- Релиз relation-only зависит от дисциплины ведения артефактов и соблюдения операционного runbook.
- Добавляется формальный overhead на подготовку gate decision summary перед релизом.

### Follow-up actions

1. Обновить progress tracker в `docs/architecture/rbac-relation-migration-plan.md` с ссылкой на этот ADR.
2. Приложить реальные staging/baseline артефакты к ближайшему cutover окну и зафиксировать решение Go/No-Go.
3. После стабилизации relation-only перейти к Phase 6 cleanup (удаление legacy fallback-path и связанных feature flags).
