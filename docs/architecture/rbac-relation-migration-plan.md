# План миграции RBAC на relation/Casbin runtime

- Дата: 2026-03-18
- Статус: Финализирован
- Область: `apps/server`, `crates/rustok-rbac`, `crates/rustok-core`, `apps/server/migration`
- Цель: зафиксировать финальное состояние модульной RBAC-схемы, где relation-данные остаются каноническим permission graph, а runtime authorization выполняется только через `casbin_only`.

---

## 1. Текущая модель

### 1.1 Source of truth

- Канонические RBAC-связи хранятся только в `roles`, `permissions`, `user_roles`, `role_permissions`.
- Публичный server-side фасад для RBAC: `apps/server/src/services/rbac_service.rs`.
- Policy/use-case ядро и Casbin runtime живут в `crates/rustok-rbac`.
- Runtime и schema используют только relation graph; effective role полностью выводится из permission relations.

### 1.2 Runtime модель

- Живой runtime decision path один: `casbin_only`.
- Relation graph остаётся источником разрешений, но не отдельным runtime engine.

### 1.3 Канонический control plane

- Канонический runtime contract фиксирован на `casbin_only`.
- Rollout-mode env switch удалён из live contract вместе с compatibility aliases.
- Любые будущие изменения authorization engine требуют нового ADR, а не возврата к mode-switch модели.

---

## 2. Что уже завершено (актуализировано на 2026-03-18)

### 2.1 Domain и runtime extraction

- `PermissionResolver`, `permission_policy`, `permission_evaluator`, `permission_authorizer` вынесены в `rustok-rbac`.
- `RuntimePermissionResolver` и relation/cache adapters используются как модульный runtime contract.
- `RbacService`, `rbac_runtime`, `rbac_persistence` отделены в `apps/server`.
- Legacy server shim `services::auth` удалён.
- Core naming приведён к новой схеме `Identity*`.

### 2.2 Observability и runtime safety

- Permission cache, decision, denied-reason и latency metrics публикуются из server runtime.
- `/metrics` публикует canonical Casbin runtime counters:
  - `rustok_rbac_engine_decisions_casbin_total`
  - `rustok_rbac_engine_eval_duration_ms_total`
  - `rustok_rbac_engine_eval_duration_samples`
- Relation-vs-Casbin parity telemetry и cutover-specific gates удалены из live runtime вместе с rollout branches.

### 2.3 Cleanup уже выполненного legacy слоя

- Удалены transitional legacy runtime paths и связанный cache/load слой.
- Удалены obsolete mismatch signals прошлой migration-схемы.
- Shadow/parity runtime слой удалён из live code path.
- Актуальные server/library callsites используют `RbacService`.
- Старые staging/cutover helper scripts удалены из live repo как завершённый migration-only tooling.

---

## 3. Итог по фазам

### Фаза A. Relation baseline

Статус: исторически завершена.

- Relation-resolver был исходным authoritative path на этапе миграции.
- После финального cutover relation graph остался только source-of-truth для permission data.

### Фаза B. Casbin parity

Статус: исторически завершена.

- Runtime больше не использует relation-authoritative или shadow-authoritative path.
- Historical parity telemetry и cutover artifacts относятся к завершённому migration phase и не определяют текущий decision path.

### Фаза C. Casbin cutover

Статус: закрыта.

- Рабочий runtime path: `casbin_only`.
- Runtime больше не зависит от `RUSTOK_RBAC_AUTHZ_MODE`.

### Фаза D. Post-cutover cleanup

Статус: завершена.

- rollout-mode branches удалены из runtime contract;
- server-side shadow telemetry удалена из живого decision path;
- compatibility env aliases удалены вместе с mode-switch surface.

---

## 4. Остаточный scope

### 4.1 Активный backlog по этому плану

- Активного backlog по migration contract больше нет.
- Исторические ADR и decision records сохраняются как evidence завершённого cutover и не требуют переписывания под текущий runtime contract.

---

## 5. Артефакты и источники истины

### 5.1 Основные документы

- ADR runtime source-of-truth: `DECISIONS/2026-02-26-rbac-relation-source-of-truth-cutover.md`
- ADR final cutover gate: `DECISIONS/2026-03-05-rbac-relation-only-final-cutover-gate.md`
- Module docs: `crates/rustok-rbac/docs/README.md`
- Server module docs: `apps/server/docs/README.md`

### 5.2 Операционные артефакты

- relation graph в БД (`roles`, `permissions`, `user_roles`, `role_permissions`)
- module runtime в `crates/rustok-rbac`
- server adapter/runtime wiring в `apps/server`

### 5.3 Проверенные расхождения относительно старого плана

При актуализации плана 2026-03-18 подтверждено следующее:

- Rollout-mode enum/env surface удалён; live code path больше не поддерживает переключение authorization engine.
- Server runtime публикует только single-engine Casbin counters, без relation-vs-casbin parity telemetry.
- Старые migration helper scripts и parity artifacts не считаются частью текущего runtime contract.
- ADR по cutover остаются в `DECISIONS/` и не дублируются в live runtime docs.

---

## 6. Критерии закрытия плана

План закрыт; на дату актуализации выполнено всё ниже:

1. Runtime выполняет authorization только через `casbin_only`.
2. Legacy rollout branches отсутствуют в живом code path.
3. Документация и verification планы синхронизированы с single-engine схемой.
