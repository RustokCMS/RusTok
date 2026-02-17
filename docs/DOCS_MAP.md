# DOCS_MAP — Реестр документации RusToK

Этот файл — единственный канонический индекс всей документации в репозитории. Он отвечает на три вопроса:

1. Где что лежит?
2. Зачем это нужно?
3. Кто обязан обновлять и когда?

## Правила актуальности

- Любые изменения **архитектуры, API, событий, модулей, tenancy, маршрутизации, UI контрактов, observability** требуют:
  1) обновить профильный документ,
  2) обновить запись в этом реестре (Purpose/Update triggers/Status).
- Не создавайте новый документ, если подходящий уже существует — расширяйте существующий.
- Устаревшие документы помечайте как `deprecated` или `archived` и указывайте замену в связях.

Легенда:
- Status: `stable | draft | deprecated | archived`
- Audience: `dev | ops | product | ai`

## Где смотреть в первую очередь

- [README.md](../README.md) — общий обзор платформы.
- [QUICKSTART.md](../QUICKSTART.md) — быстрый запуск окружения.
- [docs/architecture.md](architecture.md) — базовая архитектура.
- [docs/ARCHITECTURE_GUIDE.md](ARCHITECTURE_GUIDE.md) — расширенные детали архитектуры.
- [docs/modules/modules.md](modules/modules.md) — модульная карта.
- [docs/UI/README.md](UI/README.md) — точка входа в UI-документацию.

## Golden paths (типовые сценарии)

- **Добавить новый модуль** → `docs/modules/modules.md`, `docs/modules/module-manifest.md`, обновить `DOCS_MAP.md`.
- **Добавить событие** → `docs/transactional_event_publishing.md`, `docs/ERROR_HANDLING_POLICY.md`, обновить `DOCS_MAP.md`.
- **Добавить API/endpoint** → `docs/api-architecture.md`, `docs/input-validation.md`, `docs/rate-limiting.md`, обновить `DOCS_MAP.md`.
- **Изменить маршрутизацию/tenancy** → `docs/TENANT_RESOLVER_V2_MIGRATION.md` (если релевантно), `docs/modules/module-registry.md`, обновить `DOCS_MAP.md`.
- **Изменить UI контракт** → `docs/UI/GRAPHQL_ARCHITECTURE.md`, `docs/UI/README.md`, обновить `DOCS_MAP.md`.

---

## Корень репозитория (Governance)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| README.md | Project Overview | Высокоуровневое описание платформы | dev/product/ai | platform | изменение позиционирования/архитектуры | stable |
| README.ru.md | Project Overview (RU) | Русская версия обзора | dev/product/ai | platform | изменения в README.md | stable |
| QUICKSTART.md | Quick Start | Быстрый запуск окружения | dev/ops | platform | изменение dev-setup | stable |
| OBSERVABILITY_QUICKSTART.md | Observability Quickstart | Быстрый старт наблюдаемости | ops/dev | ops | изменения telemetry/metrics | stable |
| PLATFORM_INFO_RU.md | Platform Info (RU) | Русская справка о платформе | dev/product | platform | изменения позиционирования | stable |
| RUSTOK_MANIFEST.md | System Manifest | Каноничный обзор модулей и инвариантов | dev/ai | platform | новый модуль/инвариант | stable |
| TECHNICAL_ARTICLE.md | Technical Article | Технический обзор архитектуры | dev/product | platform | архитектурные изменения | stable |
| CHANGELOG.md | Changelog | История изменений и релизов | dev/product | platform | релизы/изменения | stable |
| CONTRIBUTING.md | Contributing Guide | Руководство для контрибьюторов | dev/ai | platform | изменение процессов | stable |
| .github/PULL_REQUEST_TEMPLATE.md | PR Template (upper) | Шаблон PR для GitHub | dev/ai | platform | изменения процесса PR | stable |
| .github/pull_request_template.md | PR Template (lower) | Дублирующий шаблон PR | dev/ai | platform | изменения процесса PR | stable |
| LICENSE | License | Лицензия проекта | dev/product | platform | юридические изменения | stable |
| docs/DOCS_MAP.md | Documentation Registry | Реестр всей документации | dev/ai | platform | новые/изменённые доки | stable |
| AGENTS.md | AI Agent Rules | Правила работы AI-агентов | ai/dev | platform | изменения workflow | stable |

---

## Архитектура и платформа (docs/)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/architecture.md | Architecture Overview | Каноничный обзор архитектуры платформы | dev/ai | platform | архитектурные изменения | stable |
| docs/QUICK_START.md | Docs Quick Start | Быстрый старт по документации | dev/ai | platform | изменения onboarding | stable |
| docs/ARCHITECTURE_GUIDE.md | Architecture Guide | Исторический гайд (см. architecture.md) | dev/ai | platform | изменения архитектуры | archived |
| docs/ARCHITECTURE_DIAGRAM.md | Architecture Diagram | Диаграммы архитектуры (дополнение) | dev/ai | platform | изменения архитектуры | stable |
| docs/ARCHITECTURE_IMPROVEMENTS_VISUAL.md | Architecture Improvements (Visual) | Исторические визуальные улучшения | dev/ai | platform | изменения roadmap/архитектуры | archived |
| docs/ARCHITECTURE_RECOMMENDATIONS.md | Architecture Recommendations | Исторические рекомендации | dev/ai | platform | пересмотр архитектуры | archived |
| docs/ARCHITECTURE_RECOMMENDATIONS_EXTENDED.md | Architecture Recommendations (Extended) | Исторические расширенные рекомендации | dev/ai | platform | пересмотр архитектуры | archived |
| docs/ARCHITECTURE_REVIEW_2026-02-12.md | Architecture Review | Архитектурный обзор на дату | dev/ai | platform | новые ревью | archived |
| docs/ARCHITECTURE_REVIEW_SUMMARY.md | Architecture Review Summary | Сводка обзора архитектуры | dev/ai | platform | новые ревью | archived |
| docs/DATABASE_SCHEMA.md | Database Schema | Текущее описание схемы БД | dev/ai | data | изменения схемы | stable |
| docs/I18N_ARCHITECTURE.md | I18N Architecture | Локализация и i18n подход | dev/ai | platform | изменения i18n | stable |
| docs/MANIFEST_ADDENDUM.md | Manifest Addendum | Историческое дополнение (перенесено в RUSTOK_MANIFEST) | dev/ai | platform | изменения манифеста | archived |
| docs/STATE_MACHINE_GUIDE.md | State Machine Guide | Правила state machine | dev/ai | platform | изменения state machine | stable |
| docs/transactional_event_publishing.md | Transactional Event Publishing | Публикация событий/transactional outbox | dev/ai | platform | изменения event/outbox | stable |
| docs/EVENTBUS_CONSISTENCY_AUDIT.md | EventBus Consistency Audit | Исторический аудит согласованности событий | dev/ai | platform | новые аудиты | archived |
| docs/TENANT_CACHE_V2_MIGRATION.md | Tenant Cache v2 Migration | Миграция tenant cache | dev/ops | platform | миграции cache | archived |
| docs/TENANT_RESOLVER_V2_MIGRATION.md | Tenant Resolver v2 Migration | Миграция tenant resolver | dev/ops | platform | миграции tenancy | archived |
| docs/CORE_STABILIZATION_RECOMMENDATIONS.md | Core Stabilization | Рекомендации по стабилизации ядра | dev/ai | platform | пересмотр ядра | draft |
| docs/MODULE_IMPROVEMENTS.md | Module Improvements | Улучшения модульной системы | dev/ai | platform | изменения модулей | draft |
| docs/REFACTORING_ROADMAP.md | Refactoring Roadmap | Дорожная карта рефакторинга | dev/ai | platform | изменения roadmap | draft |
| docs/ROADMAP.md | Roadmap | План работ по платформе | dev/product | platform | планирование | draft |
| docs/IMMEDIATE_ACTIONS.md | Immediate Actions | Срочные действия/приоритеты | dev/ops | platform | пересмотр приоритетов | draft |
| docs/IMPLEMENTATION_PROGRESS.md | Implementation Progress | Прогресс реализации | dev/product | platform | изменение статуса | draft |
| docs/REVIEW_ACTION_CHECKLIST.md | Review Action Checklist | Чеклист ревью | dev/ai | platform | изменения стандартов | stable |
| docs/REVIEW_SUMMARY.md | Review Summary | Итоговый обзор ревью | dev/product | platform | итоговые ревью | archived |
| docs/SPRINT_1_COMPLETION.md | Sprint 1 Completion | Итоги спринта | dev/product | platform | итог спринта | archived |
| docs/CRITICAL_FIXES_CHECKLIST.md | Critical Fixes Checklist | Критические фиксы и чеклист | dev/ops | platform | фиксы/постмортемы | draft |

---

## Качество, тестирование и инженерные стандарты

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/CODE_QUALITY_STANDARDS.md | Code Quality Standards | Стандарты качества кода | dev/ai | platform | изменения стандартов | stable |
| docs/testing-guidelines.md | Testing Guidelines | Общие правила тестирования | dev/ai | platform | изменения тест-политики | stable |
| docs/INTEGRATION_TESTS_GUIDE.md | Integration Tests Guide | Руководство по интеграционным тестам | dev/ai | platform | изменение тестов | stable |
| docs/PROPERTY_BASED_TESTS.md | Property-Based Tests | Практики property-based тестов | dev/ai | platform | обновление практик | draft |
| docs/PROPERTY_BASED_TESTS_GUIDE.md | Property-Based Tests Guide | Руководство по property-based тестам | dev/ai | platform | обновление практик | draft |
| docs/ERROR_HANDLING_GUIDE.md | Error Handling Guide | Практики обработки ошибок | dev/ai | platform | изменения error-handling | stable |
| docs/ERROR_HANDLING_POLICY.md | Error Handling Policy | Политики ошибок | dev/ai | platform | изменения политики ошибок | stable |
| docs/input-validation.md | Input Validation | Валидация входных данных | dev/ai | platform | изменения API/валидации | stable |
| docs/rate-limiting.md | Rate Limiting | Политики лимитирования | dev/ops | platform | изменения лимитов | stable |
| docs/rbac-enforcement.md | RBAC Enforcement | Политики доступа | dev/ai | platform | изменения RBAC | stable |
| docs/SECURITY_AUDIT_GUIDE.md | Security Audit Guide | Руководство по security аудиту | ops/dev | security | изменения процедур | stable |
| docs/lockfile-troubleshooting.md | Lockfile Troubleshooting | Решение проблем lockfile | dev | platform | изменения tooling | stable |

---

## Observability и инфраструктура

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/DISTRIBUTED_TRACING_GUIDE.md | Distributed Tracing Guide | Инструкции по tracing | ops/dev | ops | изменения tracing | stable |
| docs/INSTRUMENTATION_EXAMPLES.md | Instrumentation Examples | Примеры инструментирования | dev/ops | ops | изменения telemetry | stable |
| docs/OPENTELEMETRY_INTEGRATION.md | OpenTelemetry Integration | Интеграция OTel | ops/dev | ops | изменения OTel | stable |
| docs/structured-logging.md | Structured Logging | Структурные логи | ops/dev | ops | изменения логирования | stable |
| docs/METRICS_DASHBOARD_GUIDE.md | Metrics Dashboard Guide | Настройка дашбордов | ops/dev | ops | изменения метрик | stable |
| docs/module-metrics.md | Module Metrics | Метрики модулей | dev/ops | platform | новые метрики | stable |
| docs/grafana-setup.md | Grafana Setup | Настройка Grafana | ops | ops | изменения инфраструктуры | stable |
| docs/grafana-dashboard-example.json | Grafana Dashboard Example | Пример дашборда | ops | ops | изменения метрик | stable |
| docs/BENCHMARKS_GUIDE.md | Benchmarks Guide | Руководство по бенчмаркам | dev/ops | platform | изменения бенчмарков | stable |
| docs/CIRCUIT_BREAKER_GUIDE.md | Circuit Breaker Guide | Рекомендации по circuit breaker | dev/ops | platform | изменения resiliency | stable |
| docs/REDIS_CIRCUIT_BREAKER.md | Redis Circuit Breaker | Настройка circuit breaker для Redis | ops/dev | platform | изменения Redis | stable |
| docs/tenant-cache-stampede-protection.md | Tenant Cache Stampede Protection | Защита от cache stampede | dev/ops | platform | изменения cache | stable |

---

## API, данные и интеграции

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/api-architecture.md | API Architecture | Архитектура API/GraphQL/REST | dev/ai | platform | изменения API | stable |
| docs/dataloader-implementation.md | DataLoader Implementation | Реализация DataLoader | dev/ai | platform | изменения data access | stable |

---

## Модули и пакеты (docs/modules)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/modules/modules.md | Modules Overview | Карта модулей и назначений | dev/ai | platform | изменения модулей | stable |
| docs/modules/module-registry.md | Module Registry | Реестр модулей и их подключения | dev/ai | platform | изменения реестра | stable |
| docs/modules/module-manifest.md | Module Manifest | Формат манифеста модуля | dev/ai | platform | изменения манифеста | stable |
| docs/modules/MODULE_MATRIX.md | Module Matrix | Матрица модулей | dev/ai | platform | изменения модулей | stable |
| docs/modules/module-rebuild-plan.md | Module Rebuild Plan | План пересборки модулей | dev/ai | platform | изменения планов | draft |
| docs/modules/INSTALLATION_IMPLEMENTATION.md | Installation Implementation | План/подход установки | dev/ops | platform | изменения установки | draft |
| docs/modules/ALLOY_MANIFEST.md | Alloy Manifest | Манифест Alloy | dev/ai | platform | изменения Alloy | stable |
| docs/modules/flex.md | Flex Module | Док по модулю Flex | dev/ai | platform | изменения Flex | draft |
| docs/modules/UI_PACKAGES_INDEX.md | UI Packages Index | Индекс UI-пакетов | dev/ui | ui | изменения UI пакетов | stable |
| docs/modules/UI_PACKAGES_QUICKSTART.md | UI Packages Quickstart | Быстрый старт UI-пакетов | dev/ui | ui | изменения UI пакетов | stable |
| docs/modules/MODULE_UI_PACKAGES_INSTALLATION.md | UI Packages Installation | Установка UI-пакетов | dev/ui | ui | изменения установки UI пакетов | stable |

---

## UI документация (docs/UI)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/UI/README.md | UI Docs Entry | Точка входа в UI-документацию | dev/ai | ui | изменения UI структуры | stable |
| docs/UI/GRAPHQL_ARCHITECTURE.md | GraphQL Architecture (UI) | Архитектура GraphQL для UI | dev/ui | ui | изменения UI GraphQL | stable |
| docs/UI/GRAPHQL_ONLY_DECISION.md | GraphQL Only Decision | Решение по GraphQL-only | dev/ui | ui | изменения политики API | stable |
| docs/UI/DASHBOARD_GRAPHQL_QUERIES.md | Dashboard GraphQL Queries | Запросы для Dashboard | dev/ui | ui | изменения UI query | draft |
| docs/UI/LEPTOS_AUTH_IMPLEMENTATION.md | Leptos Auth Implementation | Реализация auth в Leptos | dev/ui | ui | изменения auth | draft |
| docs/UI/LEPTOS_AUTH_INTEGRATION.md | Leptos Auth Integration | Интеграция auth в UI | dev/ui | ui | изменения auth | draft |
| docs/UI/LEPTOS_GRAPHQL_ENHANCEMENT.md | Leptos GraphQL Enhancement | Улучшения GraphQL в Leptos | dev/ui | ui | изменения UI GraphQL | draft |
| docs/UI/admin-server-connection-quickstart.md | Admin Server Connection | Быстрый старт подключения админки | dev/ui | ui | изменения подключения | stable |
| docs/UI/admin-template-migration.md | Admin Template Migration | Миграция шаблона админки | dev/ui | ui | изменения админки | draft |
| docs/admin-migration-checklist.md | Admin Migration Checklist | Чеклист миграции админки | dev/ui | ui | изменения админки | draft |
| docs/UI/admin-libraries-parity.md | Admin Libraries Parity | Сравнение библиотек | dev/ui | ui | изменения библиотек | draft |
| docs/UI/admin-reuse-matrix.md | Admin Reuse Matrix | Матрица переиспользования | dev/ui | ui | изменения компонентов | draft |
| docs/UI/ui-parity.md | UI Parity | Сравнение UI функционала | dev/ui | ui | изменения UI | draft |
| docs/UI/tech-parity.md | Tech Parity | Техническое сравнение | dev/ui | ui | изменения UI | draft |
| docs/UI/CUSTOM_LIBRARIES_STATUS.md | Custom Libraries Status | Статус кастомных библиотек | dev/ui | ui | изменения библиотек | draft |
| docs/UI/LIBRARIES_IMPLEMENTATION_SUMMARY.md | Libraries Implementation Summary | Итог по библиотекам | dev/ui | ui | итоговые обновления | archived |
| docs/UI/DESIGN_SYSTEM_ANALYSIS.md | Design System Analysis | Анализ дизайн-системы | dev/ui | ui | изменения дизайн-системы | draft |
| docs/UI/DESIGN_SYSTEM_DECISION.md | Design System Decision | Решение по дизайн-системе | dev/ui | ui | изменения решения | stable |
| docs/UI/ADMIN_DEVELOPMENT_PROGRESS.md | Admin Development Progress | Прогресс разработки админки | dev/product | ui | отчёты прогресса | archived |
| docs/UI/PHASE_0_COMPLETE.md | Phase 0 Complete | Итоги фазы 0 | dev/product | ui | итог фазы | archived |
| docs/UI/PHASE_1_IMPLEMENTATION_GUIDE.md | Phase 1 Implementation Guide | Гайд по фазе 1 | dev/ui | ui | изменения фазы | archived |
| docs/UI/PHASE_1_PROGRESS.md | Phase 1 Progress | Прогресс фазы 1 | dev/product | ui | отчёт прогресса | archived |
| docs/UI/SPRINT_2_PROGRESS.md | Sprint 2 Progress | Прогресс спринта 2 | dev/product | ui | отчёт спринта | archived |
| docs/UI/SPRINT_3_PROGRESS.md | Sprint 3 Progress | Прогресс спринта 3 | dev/product | ui | отчёт спринта | archived |
| docs/UI/README_SPRINT_3.md | Sprint 3 README | Сводка спринта 3 | dev/product | ui | итог спринта | archived |
| docs/UI/FINAL_SPRINT_3_SUMMARY.md | Final Sprint 3 Summary | Итог спринта 3 | dev/product | ui | итог спринта | archived |
| docs/UI/FINAL_STATUS.md | Final Status | Итоговый статус UI | dev/product | ui | итог проекта | archived |
| docs/UI/TASK_COMPLETE_SUMMARY.md | Task Complete Summary | Итоги задач UI | dev/product | ui | итог проекта | archived |
| docs/UI/IMPLEMENTATION_SUMMARY.md | Implementation Summary | Сводка реализации UI | dev/product | ui | итог проекта | archived |
| docs/UI/MASTER_IMPLEMENTATION_PLAN.md | Master Implementation Plan | План реализации UI | dev/ui | ui | изменения плана | draft |
| docs/UI/PARALLEL_DEVELOPMENT_WORKFLOW.md | Parallel Development Workflow | Процесс параллельной разработки | dev/ui | ui | изменения процесса | stable |
| docs/UI/CRITICAL_WARNINGS.md | Critical Warnings | Важные предупреждения | dev/ui | ui | изменения рисков | draft |
| docs/UI/SWITCHING_TO_NEW_APP.md | Switching to New App | Переход на новое UI приложение | dev/ui | ui | изменения миграции | draft |
| docs/UI/developer-storefront-plan.md | Developer Storefront Plan | План storefront | dev/ui | ui | изменения storefront | draft |
| docs/UI/storefront.md | Storefront | Документация по storefront | dev/ui | ui | изменения storefront | draft |
| docs/UI/mini-kits.md | Mini Kits | Набор мини-китов UI | dev/ui | ui | изменения kits | draft |
| docs/UI/rust-ui-component-catalog.md | Rust UI Component Catalog | Каталог UI компонентов | dev/ui | ui | изменения компонентов | stable |
| docs/UI/rust-ui-nav-snapshot.txt | Rust UI Nav Snapshot | Снимок навигации UI | dev/ui | ui | изменения навигации | archived |
| docs/UI/agent-execution-guide.md | Agent Execution Guide | Руководство для агентов (UI) | ai/dev | ui | изменения agent workflow | stable |

### UI Deprecated (docs/UI/deprecated)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/UI/deprecated/README.md | Deprecated UI Docs | Индекс устаревших UI-доков | dev/ui | ui | архивирование | deprecated |
| docs/UI/deprecated/ADMIN_IMPLEMENTATION_PLAN.md | Admin Implementation Plan | Устаревший план админки | dev/ui | ui | заменён новым планом | deprecated |
| docs/UI/deprecated/FINAL_SUMMARY.md | Final Summary | Устаревшая финальная сводка | dev/product | ui | заменено новыми отчётами | deprecated |
| docs/UI/deprecated/FRONTEND_DEVELOPMENT_LOG.md | Frontend Development Log | Лог разработки | dev/ui | ui | исторический лог | deprecated |
| docs/UI/deprecated/PHASE_1_START.md | Phase 1 Start | Устаревший старт фазы | dev/ui | ui | заменено новыми планами | deprecated |
| docs/UI/deprecated/PROGRESS_SUMMARY.md | Progress Summary | Устаревшая сводка прогресса | dev/product | ui | заменено новыми отчётами | deprecated |
| docs/UI/deprecated/SESSION_SUMMARY.md | Session Summary | Устаревшая сессия | dev/product | ui | заменено новыми отчётами | deprecated |
| docs/UI/deprecated/UPDATE_SUMMARY.md | Update Summary | Устаревший summary | dev/product | ui | заменено новыми отчётами | deprecated |

### UI Tools (docs/UI/tools)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/UI/tools/rust_ui_catalog_parser.py | Rust UI Catalog Parser | Утилита генерации каталога | dev/ui | ui | изменения формата каталога | stable |

---

## IU / UI Kits (IU/*)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| IU/README.md | IU Overview | Обзор IU набора | dev/ui | ui | изменения IU | stable |
| IU/docs/README.md | IU Docs Entry | Точка входа IU документации | dev/ui | ui | изменения IU | stable |
| IU/docs/admin-skeleton.md | Admin Skeleton | Скелет админки IU | dev/ui | ui | изменения UI каркаса | stable |
| IU/docs/api-contracts.md | API Contracts | Контракты API для IU | dev/ui | ui | изменения API контрактов | stable |
| IU/leptos/README.md | IU Leptos | IU компоненты для Leptos | dev/ui | ui | изменения Leptos UI | stable |
| IU/leptos/components/README.md | IU Leptos Components | Индекс компонентов Leptos | dev/ui | ui | изменения компонентов | stable |
| IU/next/README.md | IU Next.js | IU компоненты для Next.js | dev/ui | ui | изменения Next UI | stable |
| IU/next/components/README.md | IU Next Components | Индекс компонентов Next | dev/ui | ui | изменения компонентов | stable |
| IU/tokens/README.md | IU Tokens | Токены дизайн-системы | dev/ui | ui | изменения токенов | stable |

---

## Документация приложений (apps/*)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| apps/server/README.md | Server App README | Обзор backend приложения | dev/ai | platform | изменения сервера | stable |
| apps/server/docs/README.md | Server Docs Entry | Точка входа в docs сервера | dev/ai | platform | изменения сервера | stable |
| apps/server/docs/health.md | Health Endpoints | Документация health эндпоинтов | dev/ops | platform | изменения health | stable |
| apps/server/docs/event-transport.md | Event Transport | Транспорт событий сервера | dev/ai | platform | изменения событий | stable |
| apps/server/docs/library-stack.md | Library Stack | Библиотечный стек сервера | dev/ai | platform | изменения стека | stable |
| apps/server/docs/CACHE_STAMPEDE_PROTECTION.md | Cache Stampede Protection | Защита от cache stampede | dev/ops | platform | изменения cache | stable |
| apps/server/docs/LOCO_FEATURE_SUPPORT.md | Loco Feature Support | Поддержка фич Loco | dev/ai | platform | изменения Loco | stable |
| apps/server/docs/loco/README.md | Loco Docs | Документация по Loco | dev/ai | platform | изменения Loco | stable |
| apps/server/docs/loco/changes.md | Loco Changes | Изменения в Loco | dev/ai | platform | обновления Loco | stable |
| apps/server/docs/loco/upstream/README.md | Loco Upstream README | Upstream документация Loco | dev/ai | platform | обновления upstream | stable |
| apps/server/docs/upstream-libraries/README.md | Upstream Libraries | Документация upstream библиотек | dev/ai | platform | обновления upstream | stable |
| apps/admin/README.md | Admin App README | Обзор Leptos админки | dev/ui | ui | изменения админки | stable |
| apps/admin/docs/README.md | Admin Docs Entry | Точка входа в docs админки | dev/ui | ui | изменения админки | stable |
| apps/storefront/README.md | Storefront README | Обзор Leptos storefront | dev/ui | ui | изменения storefront | stable |
| apps/next-frontend/README.md | Next Frontend README | Обзор Next.js storefront | dev/ui | ui | изменения storefront | stable |
| apps/next-frontend/docs/README.md | Next Frontend Docs | Документация Next.js storefront | dev/ui | ui | изменения storefront | stable |
| apps/next-admin/README.md | Next Admin README | Обзор Next.js admin | dev/ui | ui | изменения admin | stable |
| apps/next-admin/docs/clerk_setup.md | Clerk Setup | Настройка Clerk | dev/ui | ui | изменения auth | stable |
| apps/next-admin/docs/nav-rbac.md | Nav RBAC | RBAC для навигации | dev/ui | ui | изменения RBAC | stable |
| apps/next-admin/docs/themes.md | Themes | Темы интерфейса | dev/ui | ui | изменения тем | stable |
| apps/next-admin/__CLEANUP__/cleanup.md | Cleanup Notes | Исторические заметки | dev/ui | ui | архивирование | archived |
| apps/mcp/README.md | MCP App README | Обзор MCP адаптера | dev/ai | platform | изменения MCP | stable |
| apps/mcp/docs/README.md | MCP Docs Entry | Документация MCP адаптера | dev/ai | platform | изменения MCP | stable |

---

## Концепты, исследования и шаблоны

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/concepts/social-modules.md | Social Modules | Концепции соц. модулей | dev/product | platform | изменения концепта | draft |
| docs/concepts/social-matrix.md | Social Matrix | Матрица соц. функций | dev/product | platform | изменения концепта | draft |
| docs/research/ADR-xxxx-grpc-adoption.md | ADR: gRPC Adoption | Черновик ADR по gRPC | dev/ai | platform | принятие ADR | draft |
| docs/templates/module_contract.md | Module Contract Template | Шаблон контракта модуля | dev/ai | platform | изменения шаблонов | stable |

---

## Архив (docs/archive)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| docs/archive/README.md | Archive Index | Индекс архива | dev/ai | platform | архивирование | archived |
| docs/archive/ANALYSIS_COMPLETE.md | Analysis Complete | Итог анализа | dev/product | platform | архив | archived |
| docs/archive/CODE_REVIEW_BADGE.md | Code Review Badge | Материал по ревью | dev | platform | архив | archived |
| docs/archive/CODE_REVIEW_SESSION3.md | Code Review Session 3 | Сессия ревью 3 | dev | platform | архив | archived |
| docs/archive/DATALOADER_IMPLEMENTATION_SUMMARY.md | DataLoader Summary | Сводка DataLoader | dev | platform | архив | archived |
| docs/archive/FINAL_SESSION_SUMMARY.md | Final Session Summary | Итоги финальной сессии | dev/product | platform | архив | archived |
| docs/archive/IMPLEMENTATION_STATUS.md | Implementation Status | Исторический статус | dev/product | platform | архив | archived |
| docs/archive/README_SESSION_COMPLETE.md | Session Complete README | Итоговая сессия | dev/product | platform | архив | archived |
| docs/archive/REVIEW_COMPLETE.md | Review Complete | Итог ревью | dev/product | platform | архив | archived |
| docs/archive/SESSION3_SUMMARY.md | Session 3 Summary | Итоги сессии 3 | dev/product | platform | архив | archived |
| docs/archive/SESSION_SUMMARY_2026-02-11.md | Session Summary 2026-02-11 | Итоги сессии | dev/product | platform | архив | archived |
| docs/archive/WORK_COMPLETED_2026-02-11.md | Work Completed 2026-02-11 | Итоги работы | dev/product | platform | архив | archived |
| docs/archive/WORK_COMPLETED_2026-02-11_SESSION2.md | Work Completed Session 2 | Итоги работы (сессия 2) | dev/product | platform | архив | archived |
| docs/archive/WORK_COMPLETED_2026-02-11_SESSION3.md | Work Completed Session 3 | Итоги работы (сессия 3) | dev/product | platform | архив | archived |

---

## Документация крейтов (crates/*)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| crates/alloy-scripting/README.md | alloy-scripting README | Обзор крейта alloy-scripting | dev/ai | module owner | изменения крейта | stable |
| crates/alloy-scripting/docs/README.md | alloy-scripting Docs | Документация крейта alloy-scripting | dev/ai | module owner | изменения крейта | stable |
| crates/leptos-auth/README.md | leptos-auth README | Обзор крейта leptos-auth | dev/ai | ui | изменения крейта | stable |
| crates/leptos-auth/docs/README.md | leptos-auth Docs | Документация крейта leptos-auth | dev/ai | ui | изменения крейта | stable |
| crates/leptos-forms/README.md | leptos-forms README | Обзор крейта leptos-forms | dev/ai | ui | изменения крейта | stable |
| crates/leptos-graphql/README.md | leptos-graphql README | Обзор крейта leptos-graphql | dev/ai | ui | изменения крейта | stable |
| crates/leptos-graphql/docs/README.md | leptos-graphql Docs | Документация крейта leptos-graphql | dev/ai | ui | изменения крейта | stable |
| crates/leptos-hook-form/README.md | leptos-hook-form README | Обзор крейта leptos-hook-form | dev/ai | ui | изменения крейта | stable |
| crates/leptos-hook-form/docs/README.md | leptos-hook-form Docs | Документация крейта leptos-hook-form | dev/ai | ui | изменения крейта | stable |
| crates/leptos-shadcn-pagination/README.md | leptos-shadcn-pagination README | Обзор крейта pagination | dev/ai | ui | изменения крейта | stable |
| crates/leptos-shadcn-pagination/docs/README.md | leptos-shadcn-pagination Docs | Документация крейта pagination | dev/ai | ui | изменения крейта | stable |
| crates/leptos-table/README.md | leptos-table README | Обзор крейта leptos-table | dev/ai | ui | изменения крейта | stable |
| crates/leptos-table/docs/README.md | leptos-table Docs | Документация крейта leptos-table | dev/ai | ui | изменения крейта | stable |
| crates/leptos-ui/README.md | leptos-ui README | Обзор крейта leptos-ui | dev/ai | ui | изменения крейта | stable |
| crates/leptos-zod/README.md | leptos-zod README | Обзор крейта leptos-zod | dev/ai | ui | изменения крейта | stable |
| crates/leptos-zod/docs/README.md | leptos-zod Docs | Документация крейта leptos-zod | dev/ai | ui | изменения крейта | stable |
| crates/leptos-zustand/README.md | leptos-zustand README | Обзор крейта leptos-zustand | dev/ai | ui | изменения крейта | stable |
| crates/leptos-zustand/docs/README.md | leptos-zustand Docs | Документация крейта leptos-zustand | dev/ai | ui | изменения крейта | stable |
| crates/rustok-blog/README.md | rustok-blog README | Обзор модуля rustok-blog | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-blog/docs/README.md | rustok-blog Docs | Документация модуля rustok-blog | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-commerce/README.md | rustok-commerce README | Обзор модуля rustok-commerce | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-commerce/docs/README.md | rustok-commerce Docs | Документация модуля rustok-commerce | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-content/README.md | rustok-content README | Обзор модуля rustok-content | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-content/docs/README.md | rustok-content Docs | Документация модуля rustok-content | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-core/README.md | rustok-core README | Обзор ядра rustok-core | dev/ai | platform | изменения ядра | stable |
| crates/rustok-core/docs/README.md | rustok-core Docs | Документация ядра rustok-core | dev/ai | platform | изменения ядра | stable |
| crates/rustok-core/src/error/README.md | rustok-core Error Module | Документация error-модуля | dev/ai | platform | изменения error-модуля | stable |
| crates/rustok-core/src/resilience/README.md | rustok-core Resilience Module | Документация resilience-модуля | dev/ai | platform | изменения resilience-модуля | stable |
| crates/rustok-core/src/state_machine/README.md | rustok-core State Machine | Документация state machine | dev/ai | platform | изменения state machine | stable |
| crates/rustok-forum/README.md | rustok-forum README | Обзор модуля rustok-forum | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-forum/docs/README.md | rustok-forum Docs | Документация модуля rustok-forum | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-iggy-connector/README.md | rustok-iggy-connector README | Обзор коннектора Iggy | dev/ai | module owner | изменения коннектора | stable |
| crates/rustok-iggy-connector/docs/README.md | rustok-iggy-connector Docs | Документация коннектора Iggy | dev/ai | module owner | изменения коннектора | stable |
| crates/rustok-iggy/README.md | rustok-iggy README | Обзор транспорта Iggy | dev/ai | module owner | изменения транспорта | stable |
| crates/rustok-iggy/docs/README.md | rustok-iggy Docs | Документация транспорта Iggy | dev/ai | module owner | изменения транспорта | stable |
| crates/rustok-index/README.md | rustok-index README | Обзор модуля rustok-index | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-index/docs/README.md | rustok-index Docs | Документация модуля rustok-index | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-mcp/README.md | rustok-mcp README | Обзор MCP адаптера | dev/ai | module owner | изменения MCP | stable |
| crates/rustok-mcp/docs/README.md | rustok-mcp Docs | Документация MCP адаптера | dev/ai | module owner | изменения MCP | stable |
| crates/rustok-outbox/README.md | rustok-outbox README | Обзор outbox модуля | dev/ai | module owner | изменения outbox | stable |
| crates/rustok-outbox/docs/README.md | rustok-outbox Docs | Документация outbox модуля | dev/ai | module owner | изменения outbox | stable |
| crates/rustok-pages/README.md | rustok-pages README | Обзор модуля rustok-pages | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-pages/docs/README.md | rustok-pages Docs | Документация модуля rustok-pages | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-rbac/README.md | rustok-rbac README | Обзор модуля rustok-rbac | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-rbac/docs/README.md | rustok-rbac Docs | Документация модуля rustok-rbac | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-telemetry/README.md | rustok-telemetry README | Обзор модуля rustok-telemetry | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-telemetry/docs/README.md | rustok-telemetry Docs | Документация модуля rustok-telemetry | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-tenant/README.md | rustok-tenant README | Обзор модуля rustok-tenant | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-tenant/docs/README.md | rustok-tenant Docs | Документация модуля rustok-tenant | dev/ai | module owner | изменения модуля | stable |
| crates/rustok-test-utils/README.md | rustok-test-utils README | Обзор тестовых утилит | dev/ai | platform | изменения утилит | stable |
| crates/utoipa-swagger-ui-vendored/README.md | utoipa-swagger-ui-vendored README | Обзор vendored Swagger UI | dev/ai | platform | изменения vendored UI | stable |
| crates/utoipa-swagger-ui-vendored/docs/README.md | utoipa-swagger-ui-vendored Docs | Документация vendored Swagger UI | dev/ai | platform | изменения vendored UI | stable |

---

## Документация пакетов (packages/*)

| Path | Title | Purpose | Audience | Owner | Update triggers | Status |
|------|-------|---------|----------|-------|----------------|--------|
| packages/leptos-graphql/README.md | leptos-graphql README | Документация пакета leptos-graphql | dev/ai | ui | изменения пакета | stable |
| packages/leptos-hook-form/README.md | leptos-hook-form README | Документация пакета leptos-hook-form | dev/ai | ui | изменения пакета | stable |
| packages/leptos-zod/README.md | leptos-zod README | Документация пакета leptos-zod | dev/ai | ui | изменения пакета | stable |
| packages/leptos-zustand/README.md | leptos-zustand README | Документация пакета leptos-zustand | dev/ai | ui | изменения пакета | stable |
