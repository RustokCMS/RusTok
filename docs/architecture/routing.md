# API Architecture: boundaries between GraphQL and REST

Этот документ фиксирует границы использования API-стилей, чтобы команды не смешивали ответственность между UI-слоем, интеграциями и служебными потоками.

## Policy

- **GraphQL endpoint** используется **только** фронтами и админками (UI-клиентами).
- **REST endpoint** используется для:
  - внешних интеграций;
  - webhook-коллбеков;
  - служебных сценариев;
  - сценариев совместимости с существующими и сторонними клиентами.

## Use-case → API style matrix

Матрица ниже обязательна как decision table для новых команд: если use-case попадает в строку, выбираем соответствующий API style и не смешиваем границы.

| Use-case | API style | Почему |
|---|---|---|
| UI queries/mutations (storefront, admin) | **GraphQL** | UI нужны гибкие выборки, композиция полей и единый контракт для экранов. |
| External callbacks (webhook receive/ack) | **REST** | Для callback-потоков важны простые HTTP-контракты, статусы и идемпотентность. |
| Partner ingestion (интеграция с партнёрами) | **REST** | Внешним системам проще интегрироваться через стабильные REST endpoint'ы и версии. |
| Batch jobs / service automation | **REST** | Служебные и фоновые задачи требуют предсказуемых endpoint'ов, удобных для скриптов и scheduler'ов. |

## Boundary rule

Если сценарий относится к интеграциям, webhook-потокам, служебной автоматизации или совместимости, по умолчанию выбирается **REST**. GraphQL не используется как универсальный интеграционный слой для внешних и служебных клиентов.

## Runtime hardening additions

- Tenant resolution now runs in strict mode when `settings.rustok.tenant.resolution=header` and `settings.rustok.tenant.fallback_mode=disabled`: missing tenant header returns `400` instead of silently falling back to the default tenant.
- `settings.rustok.tenant.resolution=subdomain` is now distinct from `domain`: the host must match one of `settings.rustok.tenant.base_domains`, and only a single left-most label is treated as the tenant slug.
- HTTP routing no longer trusts `Forwarded` / `X-Forwarded-*` by default. All host, client IP, and proto extraction goes through `settings.rustok.runtime.request_trust`; forwarded headers are used only in `trusted_only` mode for requests coming from configured proxy CIDR ranges.
- Disabled tenants are rejected inside tenant middleware with `403`, before auth, channel resolution, or handler execution.
- Embedded UI routes and API routes now have different security-header profiles: API/operator routes keep a strict CSP, while embedded `/admin` and storefront routes use a UI-compatible CSP and static asset caching.
