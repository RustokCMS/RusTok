# Health endpoints (`apps/server`)

Документ описывает поведение health endpoints в `apps/server/src/controllers/health.rs`.

## Endpoints

- `GET /health` — базовый статус процесса и версия приложения.
- `GET /health/live` — liveness probe (процесс жив).
- `GET /health/ready` — readiness probe с агрегированным статусом зависимостей и модулей.
- `GET /health/modules` — health только по зарегистрированным модулям.

## Readiness модель

`/health/ready` возвращает общий статус и детальные проверки:

- `status`: `ok | degraded | unhealthy`
- `checks`: инфраструктурные проверки
- `modules`: проверки health для модулей из `ModuleRegistry`
- `degraded_reasons`: список причин деградации

### Поля проверки

Каждая запись в `checks` и `modules` содержит:

- `name`: имя проверки (например, `database`, `search_backend`, `module:content`)
- `kind`: `dependency` или `module`
- `criticality`: `critical` или `non_critical`
- `status`: `ok | degraded | unhealthy`
- `latency_ms`: время выполнения проверки
- `reason`: причина деградации/ошибки (опционально)

## Агрегация статуса

- Если есть `critical` проверка со статусом `unhealthy` → общий `status = unhealthy`.
- Если `unhealthy` для critical нет, но есть не-`ok` проверки → общий `status = degraded`.
- Если все проверки `ok` → общий `status = ok`.

## Надёжность проверок

Для каждой readiness-проверки используются защитные механизмы:

- timeout на выполнение проверки,
- in-process circuit breaker (порог ошибок + cooldown),
- fail-fast поведение при открытом circuit.

Это предотвращает зависание `/health/ready` при проблемной зависимости.
