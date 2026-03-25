# Архитектурные принципы

> Статус: current-state guidance

Документ фиксирует принципы, по которым сейчас держится архитектура RusToK.

## 1. Modular monolith

RusToK — это modular monolith, а не набор независимых сервисов.

Следствия:

- общий composition root находится в `apps/server`;
- platform modules компонуются в один runtime;
- границы между модулями проходят по контрактам, а не по процессам.

## 2. Только две категории platform modules

Для platform modules допустимы только:

- `Core`
- `Optional`

`Core` modules всегда активны.
`Optional` modules участвуют в build/runtime flow и затем могут переключаться на tenant level.

Не допускается скрытая третья категория “почти модуль”, “обязательная capability, но не модуль” и т.п.

## 3. Не смешивать роль, wiring и упаковку

Нужно всегда различать три оси:

- архитектурная роль: module / shared library / capability crate
- модульный статус: `Core` / `Optional`
- техническая упаковка: `crate`

Именно это убирает путаницу вида:

- `crate != автоматически module`
- `ModuleRegistry != классификатор архитектурных ролей`
- `bootstrap wiring != отдельный тип модуля`

## 4. Source of truth по модулям

Состав platform modules задаётся через `modules.toml`.

Дальше он обязан быть согласован с:

- `build_registry()`
- manifest validation
- build/codegen wiring
- verification docs

## 5. Server как host, а не второй доменный слой

`apps/server` владеет:

- transport
- runtime wiring
- auth/session integration
- RBAC enforcement path
- operational endpoints

`apps/server` не должен становиться местом для накапливания domain logic, если для неё уже есть модульный crate.

## 6. Write-side correctness важнее convenience

Write-side операции должны оставаться:

- транзакционными
- tenant-safe
- RBAC-aware
- совместимыми с event contract

События публикуются через transactional path там, где нужна атомарность write + event persistence.

## 7. Read-side отделён от write-side

RusToK держит:

- write-side в нормализованных доменных данных;
- read-side в индексах, проекциях и fast-query surfaces.

Это позволяет:

- не тащить тяжёлые JOIN в storefront/read paths;
- держать доменные инварианты в сервисах;
- эволюционировать consumers и projections независимо.

## 8. Capability crates не подменяют module taxonomy

Capability/support crates вроде:

- `alloy`
- `alloy-scripting`
- `rustok-mcp`
- `rustok-telemetry`

не должны описываться как обычные tenant-toggle modules, если они такими не являются.

Но и обратное тоже верно: если компонент объявлен как platform module, он обязан жить в taxonomy `Core/Optional`.

## 9. Документация должна отражать код

Если код и docs расходятся, приоритет у текущего кода, а docs должны быть обновлены.

Особенно это касается:

- module taxonomy
- event flow
- API surface
- host wiring
- tenant/RBAC boundaries

## 10. Любое boundary-change требует синхронного обновления

При изменении архитектурных границ нужно обновлять одновременно:

1. локальные docs компонента
2. central docs в `docs/`
3. `docs/index.md`
4. verification plans
5. ADR, если изменение нетривиально
