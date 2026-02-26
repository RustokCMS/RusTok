# План миграции RBAC на relation-модель (source of truth)

- Дата: 2026-02-26
- Статус: In progress (execution in phases)
- Область: `apps/server`, `crates/rustok-core`, `crates/rustok-rbac`, миграции `apps/server/migration`
- Цель: перейти на модель, где права доступа вычисляются **только** через таблицы связей (`roles`, `permissions`, `user_roles`, `role_permissions`), и убрать зависимость авторизации от денормализованного `users.role`.

---


## 0. Статус выполнения (progress tracker)

> Обновляется по мере мерджей в `apps/server`/`crates/*`.

- [x] **Фаза 0 — Архитектурное решение и подготовка (частично):**
  - План и архитектурные ссылки зафиксированы в `docs/architecture/*` и `docs/index.md`.
  - ADR про RBAC source-of-truth в relation-модели — **в работе** (не закрыт этим шагом).
- [x] **Фаза 1 — Быстрые исправления консистентности (базовые пункты):**
  - user creation flows для `register/sign_up/create_user` уже заведены через назначение relation RBAC (`assign_role_permissions`).
  - parity reset-password/session invalidation ведётся отдельным remediation-потоком и ADR (см. cross-link ниже).
- [~] **Фаза 2 — Единый Permission Resolver (начато):**
  - В `AuthService` добавлены tenant-aware методы `get_user_permissions / has_permission / has_any_permission / has_all_permissions`.
  - Реализованы tenant-scoping и deduplication; сохранена семантика `resource:manage` как wildcard.
  - Переведена часть GraphQL-checks (users CRUD/read/list, alloy, content mutation/query) и RBAC extractors на relation-проверки.
  - `GraphQL update_user` теперь синхронно обновляет relation-модель (`user_roles`) через `replace_user_role`, чтобы legacy-role и relation RBAC не расходились.
  - Назначение relation-ролей/пермишенов переведено на conflict-safe idempotent upsert (`ON CONFLICT DO NOTHING`) для устойчивости к конкурентным операциям.
  - Поле `User.can` в GraphQL переведено с role-based (`users.role`) на tenant-aware relation-проверку через `AuthService::has_permission`.
- [~] **Фаза 3 — AuthContext и токены (начато):**
  - `CurrentUser.permissions` теперь резолвятся из relation-модели, а не из `users.role`.
  - Полный отказ от role-claim в policy-решениях ещё не завершён (есть оставшиеся role-based места).
- [ ] **Фаза 4 — Миграция данных и защитные инварианты:** не начато.
- [ ] **Фаза 5 — Dual-read и cutover:** не начато.
- [ ] **Фаза 6 — Cleanup legacy-модели:** не начато.

### Что осталось приоритетно на ближайший шаг

1. Добрать оставшиеся runtime-проверки, где `users.role` всё ещё влияет на policy scope в доменных сервисах.
2. Добавить observability для resolver-решений (latency/cache/deny reasons).
3. Подготовить и согласовать ADR по final cutover (`relation-only`).

---

## 1. Контекст и проблема

Сейчас в коде одновременно существуют две модели:

1. Ролевая проверка по `users.role` (например, через `Rbac::has_permission(&auth.role, ...)`).
2. Relation RBAC через таблицы `user_roles` + `role_permissions` + `permissions`.

Это создаёт риск рассинхронизации: пользователь может иметь роль в `users`, но не иметь корректных связей в `user_roles`.

### 1.1 Симптомы в текущей реализации

- В `register`/`sign_up` вызывается `AuthService::assign_role_permissions`, а в части других user-flow это не гарантировано единообразно.
- Проверки прав в GraphQL сейчас ориентируются на `auth.role`, а не на relation-права.
- В `CurrentUser` permissions формируются от роли, а не через relation-вычисление.

---

## 2. Целевая архитектура

## 2.1 Source of truth

Единственный источник прав:

- `roles` — роли в рамках tenant.
- `permissions` — атомарные разрешения (`resource`, `action`) в рамках tenant.
- `user_roles` — связи пользователь ↔ роль.
- `role_permissions` — связи роль ↔ разрешение.

## 2.2 Роль поля `users.role`

На переходный период `users.role` сохраняется как legacy/денормализованное поле (для совместимости API, UI и отчётности), но **не участвует в принятии решения доступа**.

## 2.3 Принцип принятия решения

Все runtime-checks авторизации должны идти через единый сервис резолва прав:

- `get_user_permissions(tenant_id, user_id)`
- `has_permission(tenant_id, user_id, permission)`
- `has_any_permission(...)`
- `has_all_permissions(...)`

---

## 3. План внедрения по фазам

## Фаза 0 — Архитектурное решение и подготовка

**Цель:** зафиксировать единый контракт до изменений кода.

Шаги:
1. Оформить ADR в `DECISIONS/`: "RBAC source of truth = relation model".
2. Зафиксировать rollout-стратегию: `dual-read -> relation-only -> cleanup`.
3. Обновить документацию:
   - `docs/architecture/rbac.md` — целевая модель enforcement.
   - `docs/index.md` — ссылка на этот план.

Критерии завершения:
- ADR принят.
- Архитектурные документы не противоречат коду/плану.

---

## Фаза 1 — Быстрые исправления консистентности

**Цель:** убрать текущие функциональные дыры до большой миграции.

Шаги:
1. Гарантировать назначение relation-RBAC при любом создании пользователя:
   - `create_user` (GraphQL),
   - `register` (REST),
   - `sign_up` (GraphQL),
   - invite-flow и сервисные сценарии seed/sync.
2. Привести reset-password flow к единому правилу инвалидирования сессий во всех API-каналах.
3. Добавить smoke-тесты на обязательное наличие `user_roles` после создания user.

Критерии завершения:
- Ни один публичный flow не создаёт пользователя без `user_roles`.
- После reset-password активные сессии отзываются одинаково во всех каналах.

---

## Фаза 2 — Единый Permission Resolver

**Цель:** отделить authorization policy от transport-слоёв (REST/GraphQL/extractors).

Шаги:
1. Ввести общий сервис/компонент `PermissionResolver` (или расширить `AuthService`) с tenant-aware API.
2. Реализовать в нём:
   - чтение прав через relation-таблицы,
   - дедупликацию разрешений,
   - корректную обработку невалидных resource/action,
   - observability (latency, cache hit/miss, denied reason).
3. Перевести GraphQL mutations/queries с ролевых проверок на resolver.
4. Перевести REST permission checks/extractors на resolver.

Критерии завершения:
- В runtime-коде нет прямых `has_permission(&auth.role, ...)` в business-критичных точках.
- Все основные permission checks используют единый resolver.

---

## Фаза 3 — AuthContext и токены

**Цель:** убрать зависимость принятия решения от claim-роли.

Шаги:
1. В `CurrentUser` / `AuthContext` перестать вычислять permissions из `users.role`.
2. Использовать resolver для фактических permissions пользователя в tenant.
3. Оставить роль в JWT как display/debug claim (опционально), но не как источник truth.
4. Ввести инвалидацию permission-кэша при изменении связей ролей/прав.

Критерии завершения:
- Решение о доступе не зависит от role-claim в JWT.
- Изменение роли/прав начинает действовать в пределах целевого SLA (например, ≤ 60 сек с кэшем).

---

## Фаза 4 — Миграция данных и защитные инварианты

**Цель:** гарантировать полноту данных relation RBAC во всех tenant.

Шаги:
1. Backfill-процедура:
   - найти users без `user_roles`,
   - создать tenant-роль по `users.role` (если отсутствует),
   - создать связь `user_roles`,
   - добавить отсутствующие `role_permissions` для системных ролей.
2. Добавить проверки целостности в maintenance/job:
   - orphan users без ролей,
   - orphan `user_roles` без `roles`,
   - orphan `role_permissions` без `permissions`.
3. Подготовить rollback-скрипт (логически обратимый шаг).

Критерии завершения:
- `0` пользователей без ролей (кроме явно разрешённых service-аккаунтов, если такие есть).
- Консистентность подтверждена отчётом/метриками.

---

## Фаза 5 — Dual-read и cutover

**Цель:** безопасно переключиться в production без деградации доступа.

Шаги:
1. Включить dual-read режим (короткий период):
   - decision по relation,
   - параллельный shadow-check по legacy роли,
   - метрика расхождений `rbac_decision_mismatch_total`.
2. Снять baseline по mismatch и устранить источники.
3. Включить relation-only режим feature-флагом.
4. Наблюдать SLO (401/403 rate, latency, error budget).

Критерии завершения:
- mismatch стабильно равен 0 в целевом окне (например, 7 дней).
- relation-only работает без роста инцидентов.

---

## Фаза 6 — Cleanup legacy-модели

**Цель:** убрать технический долг после стабилизации.

Шаги:
1. Удалить из runtime-кода все legacy-пути авторизации по `users.role`.
2. Перевести `users.role` в read-only/derived статус на переходный релиз.
3. Подготовить миграцию на удаление `users.role` (или оставить как строго денормализованное отображаемое поле, если это обосновано).
4. Обновить документацию, API-контракты и onboarding.

Критерии завершения:
- В коде отсутствуют decision-ветки по legacy-role.
- Документация описывает только relation RBAC.

---

## 4. Delivery-план (по спринтам)

## Спринт A (foundation)

- Фаза 0 + Фаза 1.
- Частично Фаза 2 (скелет resolver + первые интеграции).
- Результат: консистентные user-flow и отсутствие новых дыр.

## Спринт B (core migration)

- Завершение Фазы 2 + Фаза 3.
- Начало Фазы 4 (backfill на staging).
- Результат: runtime checks централизованы.

## Спринт C (production cutover)

- Завершение Фазы 4 + Фаза 5.
- Подготовка Фазы 6.
- Результат: relation-only в production.

## Спринт D (debt payoff)

- Фаза 6 полностью.
- Результат: закрытие legacy, упрощение поддержки.

---

## 5. Тестовая стратегия

## 5.1 Обязательные интеграционные сценарии

1. User creation из всех входов приводит к валидным `user_roles`.
2. Проверка `users:list/read/update/manage` опирается на relation-права.
3. Изменение роли пользователя отражается в доступе после обновления контекста/кэша.
4. Reset-password во всех API-каналах отзывают сессии единообразно.
5. Tenant isolation: права пользователя tenant A не влияют на tenant B.

## 5.2 Регрессионные проверки

- Негативные проверки denied по всем критичным операциям.
- Проверка, что отсутствие `user_roles` приводит к отказу доступа (fail-closed).
- Нагрузочный тест на resolver + cache.

---

## 6. Наблюдаемость и эксплуатация

Ввести и мониторить:

- `rbac_resolver_latency_ms` (p50/p95/p99).
- `rbac_cache_hit_total`, `rbac_cache_miss_total`.
- `rbac_decision_deny_total` с reason labels.
- `rbac_decision_mismatch_total` (только в dual-read период).
- `users_without_roles_total` (consistency gauge).

Оповещения:

- рост deny/error выше baseline,
- любой `mismatch_total > 0` после cutover,
- рост users_without_roles.

---

## 7. Риски и меры снижения

1. **Риск:** рост латентности из-за join-ов.
   - **Мера:** короткоживущий кэш пермишенов + индексы + батчинг.

2. **Риск:** рассинхрон в период миграции.
   - **Мера:** dual-read, метрика mismatch, staged rollout.

3. **Риск:** частичный backfill.
   - **Мера:** dry-run отчёт + идемпотентные скрипты + post-check job.

4. **Риск:** недокументированное поведение для команд.
   - **Мера:** ADR + обновлённые runbook/архитектурные доки.

---

## 8. Definition of Done (финальный)

- Relation RBAC — единственный runtime source of truth.
- Все публичные user-flow консистентны.
- Legacy role-проверки убраны из decision-path.
- Мониторинг показывает стабильность после cutover.
- Документация в `docs/` отражает фактическое состояние.

---

## 9. Контрольный чеклист «ничего не упустить» (phase-by-phase)

Ниже — операционный чеклист для лидов команды. Каждый пункт должен быть отмечен в трекере задач (Jira/Linear) и привязан к MR/PR.

### 9.1 Фаза 0 — Подготовка

- [ ] ADR создан в `DECISIONS/` и согласован platform/team leads.
- [ ] Определён владелец migration-program (DRI) и резервный DRI.
- [ ] Определён freeze-период для изменения RBAC API-контрактов.
- [ ] Зафиксированы feature flags:
  - [ ] `rbac_relation_dual_read_enabled`
  - [ ] `rbac_relation_enforcement_enabled`
  - [ ] `rbac_legacy_role_fallback_enabled` (временный)
- [ ] Обновлены `docs/architecture/rbac.md` и `docs/index.md`.

### 9.2 Фаза 1 — Консистентность flow

- [ ] Проверены все entrypoints создания пользователя:
  - [ ] REST register
  - [ ] GraphQL sign_up
  - [ ] GraphQL create_user
  - [ ] invite accept
  - [ ] seed / sync / system bootstrap
- [~] В каждом flow гарантированно формируются `user_roles` (добавлена синхронизация для `GraphQL update_user`; остаются invite/sync edge-cases).
- [ ] В каждом flow роль и tenant валидируются до записи.
- [ ] Reset password в REST и GraphQL имеет одинаковую policy отзыва сессий.
- [ ] Добавлены интеграционные тесты на каждый flow.

### 9.3 Фаза 2 — Resolver

- [ ] Введён единый tenant-aware resolver API.
- [ ] Удалено дублирование permission-логики из handlers/resolvers.
- [ ] Проверки прав на критичных маршрутах переведены на resolver.
- [ ] Включено fail-closed поведение при ошибке резолва.
- [ ] Добавлены метрики latency/hit/miss/denied.

### 9.4 Фаза 3 — Auth context и токены

- [ ] AuthContext получает permissions только из relation-resolver.
- [ ] JWT role-claim не влияет на authorization decision.
- [ ] Внедрена инвалидация permission-cache при изменениях ролей.
- [ ] Проверено поведение long-lived sessions после изменения прав.

### 9.5 Фаза 4 — Data migration

- [ ] Подготовлен idempotent backfill-script.
- [ ] Выполнен dry-run с отчётом расхождений.
- [ ] Выполнен backfill на staging.
- [ ] Выполнен post-check целостности.
- [ ] Подготовлен rollback-план и проверен на staging.

### 9.6 Фаза 5 — Cutover

- [ ] Dual-read включён в production.
- [ ] Mismatch-метрика собирается и алертится.
- [ ] Зафиксирован baseline и окно наблюдения.
- [ ] Переключение на relation-only выполнено под feature flag.
- [ ] Проведён пост-релизный аудит 401/403/latency.

### 9.7 Фаза 6 — Cleanup

- [ ] Удалены legacy decision-paths.
- [ ] `users.role` переведён в read-only/derived (или удалён по ADR).
- [ ] Удалены временные флаги и fallback-код.
- [ ] Обновлены runbooks/onboarding/операционные инструкции.

---

## 10. Матрица ответственности (RACI, укрупнённо)

| Направление | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| ADR и архитектурное решение | Platform architect | Head of Engineering | Security lead, Backend lead | Все команды |
| Runtime migration (server) | Backend team | Backend lead | Platform architect | Frontend teams |
| Data backfill и DB integrity | Backend + DBA | Backend lead | SRE | Platform team |
| Observability и алерты | SRE/Platform | SRE lead | Backend lead | On-call rotation |
| Cutover и release control | Release manager | Head of Engineering | SRE, Backend lead | Product/Support |
| Cleanup legacy и docs finalization | Backend + Platform docs owner | Backend lead | QA | Все команды |

> Примечание: конкретные имена и команды назначаются в трекере проекта перед стартом Фазы 1.

---

## 11. Артефакты на выходе каждой фазы

| Фаза | Обязательные артефакты |
|---|---|
| 0 | ADR, список feature flags, baseline docs update |
| 1 | PR-ы по flow-consistency, integration tests report |
| 2 | Resolver API + migration PR-ы, observability dashboard v1 |
| 3 | AuthContext/JWT policy PR, cache invalidation design note |
| 4 | Backfill script + dry-run report + post-check report |
| 5 | Cutover runbook, mismatch trend report, production validation report |
| 6 | Legacy cleanup PR, final architecture docs, deprecation note |

---

## 12. Гейты релиза и критерии остановки (stop-the-line)

### 12.1 Gate перед production cutover

- [ ] Все P0/P1 дефекты по RBAC закрыты.
- [ ] `users_without_roles_total == 0` (или согласованный whitelist).
- [ ] Mismatch в dual-read не растёт и стремится к 0.
- [ ] QA sign-off по критичным permission-сценариям.
- [ ] On-call команда проинструктирована и runbook актуален.

### 12.2 Stop-the-line условия

Немедленный rollback на предыдущий режим при любом из условий:

1. Рост 403/401 выше согласованного SLO-порога после переключения.
2. Непредвиденный массовый deny для системных ролей (admin/superadmin).
3. Ошибки резолва прав с деградацией business-critical endpoints.
4. Выявленная cross-tenant утечка прав или данных.

### 12.3 Post-cutover окно наблюдения

- Рекомендуемое окно: 7–14 дней.
- Ежедневная сверка:
  - `rbac_decision_mismatch_total`
  - denied/error rate
  - latency p95/p99
  - tenant-specific anomalies
