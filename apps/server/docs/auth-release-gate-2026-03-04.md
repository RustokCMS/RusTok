# Auth consistency release gate report (2026-03-04)

Документ фиксирует финальные gate-артефакты Phase D для плана
`docs/architecture/user-auth-consistency-remediation-plan.md`.

## 1) Integration report

Команды:

- `cargo test -p rustok-server auth_lifecycle`
- `cargo test -p rustok-server auth`

Результат:

- `auth_lifecycle`: **23/23 passed**.
- `auth` (расширенный auth-срез, включая transport mappings + service invariants): **43/43 passed**.

Вывод: обязательный инвариантный минимум из раздела 6 закрыт и расширен дополнительными
service/transport проверками.

## 2) REST/GraphQL parity report

Проверки parity подтверждены тестовым покрытием в одном shared use-case слое
`AuthLifecycleService` и transport-level mapping tests:

- `create_user`: единый use-case и стабильный error/status contract;
- `confirm_reset`/`reset_password`: единая revoke-policy + единый invalid token contract;
- `change_password`: единая revoke-policy (кроме текущей сессии) и общий lifecycle-path.

Результат parity-проверок: **Pass (test-backed parity evidence)**.

## 3) Security checklist sign-off

Checklist:

- [x] `password reset => revoke all active sessions` (unit/service tests passed).
- [x] Inactive user login bypass отсутствует (`UserInactive` + metric increment check).
- [x] Duplicate-email race нормализуется в `EmailAlreadyExists` без transport drift.
- [x] Ошибки `InvalidResetToken` и `UserInactive` маппятся консистентно между transport-адаптерами.

Итог: **Pass**.

## 4) Gate decision

- Integration: ✅ Closed
- REST/GraphQL parity: ✅ Closed
- Security review: ✅ Closed

Phase D считается закрытой для текущего remediation-пакета.
