# Унификация auth lifecycle и policy инвалидирования сессий

- Date: 2026-02-26
- Status: Accepted

## Context

В `apps/server` исторически существовали параллельные реализации auth use-cases в REST и GraphQL. Это привело к расхождениям:

- разный side-effect при создании пользователя в отдельных entrypoint;
- разная семантика reset/change password относительно сессий;
- дублирование бизнес-веток и error-mapping в транспортах.

При этом миграция RBAC source-of-truth в relation-модель уже описана отдельно и не должна смешиваться с задачей консистентности auth lifecycle.

## Decision

1. Зафиксировать `AuthLifecycleService` как единый application service для auth use-cases (`register`, `login/sign_in`, `refresh`, `request/confirm reset`, `change_password`, `update_profile`, `create_user`).
2. Оставить REST handlers и GraphQL mutations тонкими adapter-слоями (I/O parsing, transport mapping).
3. Применять единую policy инвалидирования сессий во всех каналах:
   - `confirm_password_reset`/`reset_password`: soft-revoke всех активных сессий пользователя через `sessions.revoked_at`;
   - `change_password`: soft-revoke всех остальных активных сессий пользователя (кроме текущей);
   - `sign_out`: soft-revoke только текущей сессии.
4. Явно разделить ответственность документов:
   - auth lifecycle consistency и transport parity — в `docs/architecture/user-auth-consistency-remediation-plan.md`;
   - RBAC relation migration и source-of-truth cutover — в `docs/architecture/rbac-relation-migration-plan.md`.

## Consequences

- Снижается вероятность drift между REST и GraphQL по auth-поведению.
- Критичные security-сценарии (reset/change password) становятся предсказуемыми и проверяемыми инвариантными тестами.
- Документация и rollout gate могут проверять консистентность независимо от RBAC cutover.
- Следующим шагом остаётся поддерживать parity тестами и не возвращать бизнес-ветки в transport-слой.
