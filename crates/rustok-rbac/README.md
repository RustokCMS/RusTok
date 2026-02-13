# rustok-rbac

Role and permission helpers for RusToK.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Взаимодействие
- crates/rustok-core
- apps/server
- доменные модули требующие ACL

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Модуль ролей и прав доступа (ACL/RBAC) для всех доменов.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-core
  - apps/server
  - доменные модули с проверкой прав
- **Точки входа:**
  - `crates/rustok-rbac/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

