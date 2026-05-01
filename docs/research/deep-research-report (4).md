# Аудит архитектуры и кода RusTokRs/RusTok

## Исполнительное резюме

Репозиторий RusTokRs/RusTok уже выглядит как архитектурно амбициозная модульная платформа: `apps/server` выступает composition root, состав модулей задаётся через `modules.toml`, для optional-модулей генерируется wiring через `build.rs`, tenant-aware runtime и `tenant_modules` уже присутствуют, а в документации явно зафиксированы границы между core, optional и capability-слоями. Есть зрелые признаки инженерной дисциплины: manifest-контракты, `xtask`, миграции, отдельные runtime workers, широкий CI и production-пример конфигурации с Redis/outbox. fileciteturn81file0L1-L1 fileciteturn79file0L1-L1 fileciteturn77file0L1-L1 fileciteturn78file0L1-L1 fileciteturn56file0L1-L1 fileciteturn88file0L1-L1 fileciteturn93file0L1-L1

Однако до состояния «идеальной платформы» кодовой базе сейчас мешают не локальные баги, а несколько системных архитектурных дефектов. Главный из них — управление составом платформы через изменяемый локальный `modules.toml` во время работы приложения: серверные GraphQL-мутации читают и записывают manifest с файловой системы, тогда как production-образ сервера этот файл вообще не копирует. В режиме контейнеров и горизонтального масштабирования это ломает единый source of truth, создаёт гонки и делает build/deploy path неидемпотентным. fileciteturn80file0L1-L1 fileciteturn90file0L1-L1 fileciteturn71file0L1-L1 fileciteturn103file0L1-L1

Второй класс проблем связан с рассогласованием декларативных контрактов и фактического runtime-поведения. `modules.toml` и документация описывают `settings.default_enabled` как платформенный контракт по умолчанию, но реальный runtime включённые модули определяет через явные строки `tenant_modules`, админский bootstrap трактует отсутствующие optional-модули как disabled, в коде есть собственный жёстко прошитый список default-модулей, а seed-данные включают ещё один, третий набор. Это означает, что «включено по умолчанию» сейчас не имеет одного канонического вычисления. fileciteturn41file0L1-L1 fileciteturn79file0L1-L1 fileciteturn98file0L1-L1 fileciteturn40file0L1-L1 fileciteturn43file0L1-L1 fileciteturn47file0L1-L1 fileciteturn102file0L1-L1

Третий класс — консистентность жизненного цикла модулей и миграций. `ModuleLifecycleService` сначала фиксирует состояние модуля в БД, а потом вызывает `on_enable`/`on_disable`; при ошибке он откатывает только флаг в `tenant_modules`, но не способен откатить уже случившиеся внешние побочные эффекты. В миграциях есть хрупкая глобальная сортировка «по имени плюс один special-case», а i18n-миграция для расширения locale-колонок прямо противоречит собственной архитектурной документации: документ запрещает сужать такие колонки обратно, а `down()` это делает. fileciteturn96file0L1-L1 fileciteturn58file0L1-L1 fileciteturn95file0L1-L1 fileciteturn94file0L1-L1 fileciteturn61file0L1-L1

Итоговый вывод: RusTok уже силён как «архитектурный скелет» modular monolith, но пока не дотягивает до production-grade control plane. Если кратко, платформа нуждается не в очередном наборе модулей, а в стабилизации composition state, унификации module-policy, исправлении консистентности жизненного цикла, ужесточении CI-гейтов и переводе deployment path на truly stateless и revisioned модель. fileciteturn81file0L1-L1 fileciteturn80file0L1-L1 fileciteturn88file0L1-L1

## Фактическое состояние платформы

По текущему коду RusTok — это модульный монолит с очень сильной декларативной частью. `modules.toml` задаёт core/optional-модули, build-профиль и `default_enabled`; `apps/server/build.rs` читает этот manifest и генерирует код для registry, GraphQL и route wiring; `apps/server/src/modules/mod.rs` собирает core-модули вручную и подмешивает optional-модули через codegen. Документация прямо закрепляет, что composition root — это `modules.toml` плюс `apps/server`, а tenant-level enablement лежит поверх уже собранной platform composition. fileciteturn79file0L1-L1 fileciteturn77file0L1-L1 fileciteturn78file0L1-L1 fileciteturn80file0L1-L1

Мультитенантность реализована существенно лучше среднего уровня для ранней платформы. Есть tenant-aware middleware, выделенный `TenantContext`, кэш tenant-resolution с negative cache, invalidation и защитой от stampede, `tenant_modules`, `tenant_locales`, а в схемах и документации `tenant_id` описан как главный boundary изоляции данных. При этом shared-DB / shared-schema модель остаётся базовой: в просмотренных материалах не указано, что на уровне БД действует row-level security; изоляция обеспечивается в основном на уровне application/runtime и схемы данных. fileciteturn56file0L1-L1 fileciteturn59file0L1-L1 fileciteturn60file0L1-L1 fileciteturn95file0L1-L1

I18n-контракт в документации описан очень правильно: effective locale должен определяться host/runtime-слоем, локализованные данные должны жить в parallel records, а locale-хранилище — иметь безопасную ширину `VARCHAR(32)`. Это сильная сторона архитектуры. Проблема не в замысле, а в том, что часть runtime и migration-логики пока не доведена до того же уровня строгости. fileciteturn94file0L1-L1 fileciteturn95file0L1-L1

По эксплуатационной части база тоже уже есть: `EventRuntime` поддерживает `memory` / `outbox` / `iggy`, фоновые workers поднимаются централизованно, есть release backend с `record_only`, `filesystem`, `http`, `container`, а production-пример конфигурации сразу рекомендует Redis rate limiting и outbox→iggy relay без fallback. Но текущая реализация release/control-plane остаётся локально-файловой и shell-oriented, что делает её слабее, чем остальной runtime. fileciteturn51file0L1-L1 fileciteturn49file0L1-L1 fileciteturn87file0L1-L1 fileciteturn93file0L1-L1

Ниже — упрощённая схема того, как система выглядит сейчас по коду и документам.

```mermaid
flowchart LR
    A[Admin / Storefront / Next hosts] --> B[apps/server]
    B --> C[ModuleRegistry + generated wiring]
    C --> D[core modules]
    C --> E[optional modules]
    B --> F[(PostgreSQL)]
    F --> G[tenant_modules]
    F --> H[tenant_locales]
    F --> I[sys_events / builds / releases]
    B --> J[local modules.toml]
    J --> K[build.rs / runtime manifest loading]
    B --> L[Build / Release services]
    L --> M[filesystem / HTTP / container deploy backends]
```

С точки зрения «идеальной платформы» красная зона здесь одна: `local modules.toml` находится прямо в runtime-петле управления платформой. Именно вокруг этого узла и концентрируются самые опасные дефекты. fileciteturn79file0L1-L1 fileciteturn77file0L1-L1 fileciteturn71file0L1-L1 fileciteturn103file0L1-L1

## Критические дефекты и риски

| Дефект / риск | Местоположение в коде | Влияние на систему | Приоритет | Рекомендации и пример патча / псевдокод | Оценка трудоёмкости | Возможные побочные эффекты |
|---|---|---|---|---|---|---|
| **Composition state хранится как mutable local file**. Runtime-операции установки/удаления/обновления модулей читают и пишут `modules.toml` с локальной ФС, хотя документально это composition root. Production-образ сервера сам `modules.toml` не копирует. | `xtask/README.md`, `docs/architecture/modules.md`, `apps/server/src/graphql/mutations.rs`, `apps/admin/src/features/modules/api.rs`, `apps/server/Dockerfile` fileciteturn90file0L1-L1 fileciteturn80file0L1-L1 fileciteturn71file0L1-L1 fileciteturn43file0L1-L1 fileciteturn103file0L1-L1 | Невозможность надёжного multi-instance control plane; риск расхождения между pod’ами/контейнерами; проблемы после рестарта; build/deploy path зависит от локального состояния контейнера. | **Высокий** | Вынести composition state в БД: `platform_state(id, revision, manifest_json, active_release_id, updated_by, updated_at)`. `modules.toml` оставить only-read для dev/bootstrap. Псевдокод: `BEGIN; row = SELECT ... FOR UPDATE; validate(diff); INSERT build_request(expected_revision=row.revision+1,...); UPDATE platform_state SET manifest_json=?, revision=revision+1; COMMIT;` | 4–6 чд | Потребуется миграция control-plane, адаптация admin UI и release flow, возможно разовый импорт текущего manifest. |
| **Гонка lost update и опасный rollback**. Путь `load -> mutate -> save -> request_build`, а при ошибке build выполняется восстановление исходного manifest без CAS/версии. При двух параллельных операциях поздний rollback может затереть чужое успешное изменение. | `apps/server/src/graphql/mutations.rs` fileciteturn71file0L1-L1 | Неконсистентный platform state, «пропадающие» изменения, трудновоспроизводимые баги в module install/uninstall/upgrade. | **Высокий** | Добавить optimistic locking по `revision`/`manifest_hash`; делать commit revision до постановки build job и запрещать blind-rollback. Псевдокод: `if current_revision != expected_revision { return 409 }`. Build job должен ссылаться на immutable `manifest_revision`, а не на «текущий файл». | 2–3 чд | Придётся менять API админки и build-service contract. |
| **`default_enabled` не является единым источником истины**. Документация и `modules.toml` описывают declarative default graph, но runtime `find_enabled()` смотрит только на `tenant_modules`, admin bootstrap трактует отсутствующие optional modules как disabled, в admin API есть свой hardcoded список, dev seed включает ещё один набор модулей. | `docs/modules/manifest.md`, `modules.toml`, `apps/server/src/models/tenant_modules.rs`, `crates/rustok-tenant/admin/src/api.rs`, `apps/admin/src/shared/context/enabled_modules.rs`, `apps/admin/src/features/modules/api.rs`, `apps/server/src/seeds/mod.rs` fileciteturn41file0L1-L1 fileciteturn79file0L1-L1 fileciteturn98file0L1-L1 fileciteturn40file0L1-L1 fileciteturn47file0L1-L1 fileciteturn43file0L1-L1 fileciteturn102file0L1-L1 | Непредсказуемое поведение новых tenant’ов, рассинхрон UI и backend, ошибки в support/операциях и невозможность доказать, что платформа реально соблюдает manifest contract. | **Высокий** | Ввести один сервис `EffectiveModulePolicyService`, который считает `effective_enabled = core ∪ manifest.default_enabled ∪ tenant_overrides(+/-)`. Удалить hardcoded lists из admin API/seed’ов. Псевдокод: `enabled(slug) = core || tenant_override.unwrap_or(default_enabled.contains(slug))`. | 3–5 чд | Понадобится миграция UI и возможное переосмысление существующих `tenant_modules` записей. |
| **Enable/disable hooks не транзакционны относительно side effects**. Состояние модуля пишется в БД внутри транзакции, но `on_enable`/`on_disable` вызываются после commit; при ошибке откатывается только флаг в `tenant_modules`. | `apps/server/src/services/module_lifecycle.rs` fileciteturn96file0L1-L1 | Частичные включения/выключения: внешние ресурсы, события, кэши или подготовительные операции могут уже измениться, а DB-флаг окажется откатан. | **Высокий** | Перейти на двухфазную модель: `pending_enable/pending_disable` + outbox/journal + компенсирующие операции. Минимум: писать `module_operations` и делать hooks идемпотентными. Псевдокод: `INSERT module_operation(status=pending)` → worker executes hook → `status=done` / `status=failed`. | 4–7 чд | Модульным crate придётся адаптировать `on_enable`/`on_disable` к идемпотентному контракту. |
| **Оркестрация миграций хрупкая**. Серверный `Migrator` просто сливает module-owned migrations и сортирует их по имени; для одной пары миграций уже пришлось добавить ad-hoc override `migration_sort_key`. | `apps/server/migration/src/lib.rs` fileciteturn58file0L1-L1 | С ростом числа модулей и FK-зависимостей возрастает риск latent-order bugs; ошибка проявится на prod/test миграции, а не на compile time. | **Высокий** | Заменить сортировку по имени на явный dependency DAG для миграций или хотя бы на декларацию `depends_on_migration = [...]`. Псевдокод: `toposort(migrations)` вместо `sort_by(name)`. | 3–4 чд | Придётся добавить metadata к module-owned migrations и написать конвертер/validator. |
| **I18n-rollback противоречит собственному контракту**. Архитектурные документы требуют считать widening locale columns до `VARCHAR(32)` безопасной forward migration и прямо запрещают rollback со сужением, но `down()` возвращает `VARCHAR(5)`. | `docs/architecture/database.md`, `docs/architecture/i18n.md`, `apps/server/migration/src/m20260405_000001_expand_locale_storage_columns.rs` fileciteturn95file0L1-L1 fileciteturn94file0L1-L1 fileciteturn61file0L1-L1 | Потеря валидных locale tags и повреждение данных при rollback. Это особенно критично для BCP47-like тегов и будущих локалей. | **Высокий** | Сделать migration irreversible: `down()` должен быть no-op или выдавать controlled error/manual step. Если rollback нужен, то только в ручный script с explicit precheck отсутствия значений >5. | 0.5–1 чд | Невозможность «удобного» отката без отдельной операционной процедуры; это нормальная цена за отсутствие data loss. |
| **CI не покрывает критичные contract-gates**. `xtask` объявлен основным инструментом проверки `modules.toml`, module docs/wiring и scoped validation, но в CI нет шагов `cargo xtask validate-manifest` и `cargo xtask module validate`. Coverage лишь генерирует `lcov.info`, но не публикует артефакт и не проверяет порог. | `xtask/README.md`, `.github/workflows/ci.yml` fileciteturn90file0L1-L1 fileciteturn88file0L1-L1 | Документированный platform contract может ломаться незаметно; coverage не становится управляющим сигналом; риск drift между docs/manifests/server wiring. | **Высокий** | В CI добавить `cargo xtask validate-manifest`, `cargo xtask module validate`, targeted `cargo xtask module test` для изменённых модулей, upload coverage artifact и threshold check. | 1–2 чд | Временно увеличится время CI; часть старых несоответствий всплывёт сразу. |
| **Дублирование module lifecycle logic и возможность обхода правил**. `crates/rustok-tenant::TenantService.toggle_module()` напрямую пишет флаг модуля, не используя dependency/core-проверки `ModuleLifecycleService`; admin bootstrap также сам собирает effective state. | `crates/rustok-tenant/src/services/tenant_service.rs`, `apps/server/src/services/module_lifecycle.rs`, `crates/rustok-tenant/admin/src/api.rs` fileciteturn37file0L1-L1 fileciteturn96file0L1-L1 fileciteturn40file0L1-L1 | Размытие единственного источника правил для module lifecycle; риск будущего API-bypass, если сервис будет переиспользован снаружи. | **Средний** | Свести все операции включения/выключения к одному application service. В `TenantService` оставить только чтение tenant metadata, а toggle вынести/делегировать в `ModuleLifecycleService`. | 1–2 чд | Небольшой refactor public API crate `rustok-tenant`; возможны мелкие правки admin-side server functions. |
| **Ослабленная изоляция модулей на compile-time**. Default feature set сервера компилирует почти все optional-модули, хотя `modules.toml.default_enabled` включает только подмножество. Это расширяет бинарь и attack surface и снижает реальную модульность. | `apps/server/Cargo.toml`, `modules.toml` fileciteturn72file0L1-L1 fileciteturn79file0L1-L1 | Увеличенные build times, больше зависимостей в runtime, сложнее доказать минимальный состав production-артефакта. | **Средний** | Генерировать feature-профили из manifest или ввести явные профили: `minimal`, `commerce`, `content`, `full`. Для production-build использовать manifest-derived features, а не «почти всё». | 2–4 чд | Потребуются новые build matrix и возможно разделение dev/full profile. |
| **Dependency hygiene дрейфует**. Dependabot настроен широко, но содержит путь `/apps/mcp`, который в просмотренной выборке репозитория больше не подтверждается; явная repository-level license policy (`deny.toml`) в просмотренных файлах не указана. | `.github/dependabot.yml` fileciteturn64file0L1-L1 | Часть обновлений может silently не обрабатываться; лицензионный контроль в `cargo deny` остаётся менее прозрачным, чем мог бы быть. | **Низкий / средний** | Удалить/проверить stale directories, добавить явный `deny.toml` с license allowlist, bans и advisories policy, публиковать SBOM на релиз. | 0.5–1.5 чд | После введения `deny.toml` могут всплыть текущие несовместимости лицензий или bans. |

### Наиболее опасная связка дефектов

Самая неприятная комбинация сейчас состоит из трёх элементов: локально изменяемый `modules.toml`, отсутствие versioned control plane и runtime/UI-дрейф по `default_enabled`. По отдельности это «архитектурные шероховатости», а вместе — уже production-блокер для multi-instance и CI/CD-автоматизации: одна нода может принять изменение manifest, другая продолжить работать со старым state, а UI при этом покажет третью интерпретацию «включённых по умолчанию» модулей. fileciteturn71file0L1-L1 fileciteturn43file0L1-L1 fileciteturn47file0L1-L1 fileciteturn103file0L1-L1

## Оценка по ключевым атрибутам

| Атрибут | Наблюдение | Вердикт |
|---|---|---|
| **Многоязычность (i18n/l10n)** | Контракт сильный: host/runtime-owned locale selection, translations tables, `VARCHAR(32)` как целевая ширина. Но реализация подрывается опасным `down()` для locale-колонок и общим дрейфом between docs and runtime. fileciteturn94file0L1-L1 fileciteturn95file0L1-L1 fileciteturn61file0L1-L1 | **Выше среднего по дизайну, средне по надёжности реализации** |
| **Мульти-тенантность** | Tenant-aware middleware, cache/invalidation, `tenant_modules`, `tenant_locales` и `tenant_id` как boundary реализованы убедительно. DB-level RLS в просмотренных материалах **не указано**. fileciteturn56file0L1-L1 fileciteturn59file0L1-L1 fileciteturn60file0L1-L1 fileciteturn95file0L1-L1 | **Хорошо для application-layer isolation, но не «идеально» без DB policy layer** |
| **Изоляция модулей / слоёв** | Документация и build.rs задают внятные модульные границы, но compile-time default features перегружают сервер optional-модулями, а lifecycle-правила размазаны по нескольким сервисам. fileciteturn80file0L1-L1 fileciteturn77file0L1-L1 fileciteturn72file0L1-L1 fileciteturn37file0L1-L1 fileciteturn96file0L1-L1 | **Средне** |
| **Масштабируемость горизонтальная / вертикальная** | Вертикально платформа выглядит перспективно; горизонтально мешают local-file control plane, per-node compile/runtime coupling и partially local deployment path. Production-пример для Redis/outbox есть, но реальная stateless control plane модель не доведена. fileciteturn87file0L1-L1 fileciteturn93file0L1-L1 fileciteturn103file0L1-L1 | **Средне-низко для идеальной multi-instance платформы** |
| **Отказоустойчивость** | Есть supervision для relay/background workers и restart-loop; нет транзакционной гарантии для module hooks, а build/release path частично зависит от локального состояния. fileciteturn49file0L1-L1 fileciteturn51file0L1-L1 fileciteturn96file0L1-L1 | **Средне** |
| **Безопасность** | RBAC, tenant validation, request trust settings, rate limiting и production example присутствуют. Но явная repository-level license/SBOM/provenance policy в просмотренных файлах **не указана**, а rollout/path и mutable manifest делают control plane менее безопасным операционно. fileciteturn54file0L1-L1 fileciteturn56file0L1-L1 fileciteturn88file0L1-L1 fileciteturn93file0L1-L1 | **Средне** |
| **CI/CD** | CI широкая и качественная по breadth, но отсутствуют ключевые `xtask`-гейты, coverage не становится release-signal, полноценный CD workflow в репозитории **не указан**. fileciteturn88file0L1-L1 fileciteturn90file0L1-L1 | **Средне-низко** |
| **Тестируемость** | Много unit/integration tests, миграционные тесты, проверка idempotency workers. Browser E2E для UI в просмотренной выборке **не указаны**. Самые опасные гонки control plane пока не покрыты. fileciteturn49file0L1-L1 fileciteturn58file0L1-L1 fileciteturn96file0L1-L1 | **Средне** |
| **Производительность** | В README приведены только **симулированные** benchmark-цифры; данных о реальном perf budget/SLO/load tests в просмотренных файлах нет. fileciteturn81file0L1-L1 | **Не указано достоверно** |
| **Управление конфигурацией** | Typed settings и production example сильные; но platform composition управляется mutable local file, а не revisioned control-plane state. fileciteturn54file0L1-L1 fileciteturn93file0L1-L1 fileciteturn71file0L1-L1 | **Средне-низко** |
| **Миграции БД** | Есть единый migrator, module-owned migrations и тесты. Но ordering model хрупкая, а locale migration имеет unsafe rollback. fileciteturn58file0L1-L1 fileciteturn61file0L1-L1 | **Средне** |
| **Схемы данных** | Схемный контракт продуман: base rows + translations/bodies + `tenant_id`. Но shared-schema модель усиливает требования к runtime-изоляции и тестам. fileciteturn95file0L1-L1 | **Выше среднего** |
| **API-совместимость** | Hybrid API model явно заявлена, но формальная policy по back-compat/versioning в просмотренных корневых документах и CI **не указана**. fileciteturn81file0L1-L1 fileciteturn88file0L1-L1 | **Не указано** |
| **Зависимости и лицензии** | MIT license есть; CI включает `cargo audit`, `cargo deny`, Dependabot. Явная `deny.toml` policy и SBOM/release provenance в просмотренной выборке **не указаны**; есть признак stale automation path. fileciteturn66file0L1-L1 fileciteturn88file0L1-L1 fileciteturn64file0L1-L1 | **Средне** |

## План доработок

Ниже — реалистичный план, который я бы считал минимальным для перевода платформы из «архитектурно интересной» в «операционно надёжную».

| Этап | Содержание | Приоритет | Оценка времени | Ресурсы | Критерии приёмки |
|---|---|---|---|---|---|
| **Стабилизация control plane** | Вынести composition state из локального `modules.toml` в БД; добавить `revision`; убрать blind rollback; сделать build requests revision-aware. | **Высокий** | 8–12 раб. дней | 1 senior Rust/backend, 1 backend, 0.5 DevOps | Две параллельные операции install/upgrade больше не теряют изменения; pods читают один и тот же active manifest; контейнер не зависит от локального `modules.toml`. |
| **Унификация effective module policy** | Реализовать `EffectiveModulePolicyService`; убрать hardcoded default lists и seed drift; синхронизировать admin/bootstrap/backend. | **Высокий** | 4–6 раб. дней | 1 backend, 1 full-stack | Для нового tenant набор enabled-модулей совпадает в БД, GraphQL, Leptos admin и сидировании. |
| **Консистентность lifecycle модулей** | Перевести enable/disable на operation journal + idempotent hooks; запретить прямые toggle-path вне единого сервиса. | **Высокий** | 6–10 раб. дней | 1 senior backend, 1 backend | Ошибка `on_enable` не оставляет частичных side effects без записи в operation journal; нет второго service-path для bypass. |
| **Хардениг миграций и i18n** | Сделать locale-widening irreversible; заменить sort-by-name на dependency-aware orchestration; добавить migration contract tests. | **Высокий** | 4–7 раб. дней | 1 backend | Нет unsafe shrink rollback; тест падает при любом FK-order conflict до merge. |
| **CI/CD как enforceable contract** | Добавить `xtask validate-manifest`, `xtask module validate`, coverage upload + threshold, release SBOM, container build/deploy workflow. | **Высокий** | 3–5 раб. дней | 1 DevOps, 1 backend | CI ломается при drift `modules.toml`/docs/wiring; coverage и SBOM публикуются на каждый релиз. |
| **Изоляция и профили сборки** | Ввести manifest-derived feature profiles (`minimal`, `content`, `commerce`, `full`) и production-only build matrix. | **Средний** | 3–4 раб. дня | 1 backend | Production binary собирается без ненужных optional-модулей; размер бинаря и compile time снижаются. |
| **Операционная зрелость** | Ввести policy для API compatibility, release provenance, license allowlist, chaos-tests для multi-instance control plane. | **Средний** | 4–6 раб. дней | 1 архитектор/tech lead, 1 DevOps, 1 QA | Есть подписанная policy по API/back-compat; chaos-tests проходят; license policy формализована. |

### Рекомендуемые тестовые сценарии для приёмки

| Сценарий | Что проверять | Ожидаемый результат |
|---|---|---|
| **Параллельная установка двух модулей** | Два admin-оператора одновременно меняют composition state | Нет lost update; одна операция получает conflict/409, обе не портят state |
| **Ошибка в `on_enable`** | Hook валится после внешнего side effect | Operation journal фиксирует failure; повтор безопасен; tenant state остаётся консистентным |
| **Новый tenant без overrides** | Tenant без строк в `tenant_modules` | Effective enabled set совпадает с `core + default_enabled` |
| **Containerized runtime** | Сервер запускается без доступа к локальному root manifest | Runtime работает нормально, потому что читает composition state из БД/release snapshot |
| **Rollback i18n migration** | В БД есть locale длиннее 5 символов | Rollback не выполняет destructive shrink; оператор получает безопасный отказ |
| **Manifest drift в PR** | Изменён `modules.toml`, но не обновлены docs/wiring | CI падает на `xtask validate-manifest` |
| **Feature profile build** | Сборка `minimal` и `commerce` | В binaries нет лишних optional-модулей |
| **Scale-out control plane** | Два экземпляра admin/server под shared DB | Оба видят один `active manifest revision`, UI и backend синхронны |

## Целевая архитектура и инженерные шаблоны

Ниже — та архитектурная форма, к которой, на мой взгляд, стоит привести платформу.

```mermaid
flowchart LR
    A[Admin / Operator UI] --> B[Platform Control Plane API]
    B --> C[(platform_state: manifest_json + revision)]
    B --> D[(tenant_module_overrides)]
    B --> E[(build_requests)]
    E --> F[Build Worker]
    F --> G[Immutable release artifact]
    G --> H[Release registry / deployment backend]
    H --> I[Runtime server pods]
    I --> C
    I --> D
    I --> J[(PostgreSQL domain data)]
    I --> K[(Redis cache/rate-limit)]
    I --> L[Outbox / event runtime]
```

Ключевая идея проста: **runtime не должен редактировать локальный manifest-файл**. Runtime должен читать immutable release snapshot и tenant overrides, а control plane — оперировать revisioned state в БД и порождать build/deploy pipeline как отдельный артефактный процесс.

### Сравнение альтернатив для мульти-тенантности

| Подход | Плюсы | Минусы | Когда выбирать | Рекомендация для RusTok |
|---|---|---|---|---|
| **Shared DB, shared schema, `tenant_id`** | Наименьшая сложность, быстрые cross-tenant migrations, хорошая fit с текущим кодом | Высокие требования к application isolation и тестам | Модульный монолит, умеренное число tenant’ов | **Рекомендую как ближний горизонт**, но усилить RLS для высокорисковых таблиц |
| **Shared DB, schema per tenant** | Лучшая логическая изоляция | Сложные migrations, сложнее cross-tenant operations | Большой B2B SaaS с умеренным числом tenant’ов | Не рекомендую как ближайший шаг: слишком дорогой переход |
| **DB per tenant** | Максимальная изоляция и compliant story | Очень дорогой ops/control plane | Enterprise high-isolation / regulated workloads | Подходит только как future premium tier, не как базовая модель |

### Сравнение альтернатив для кэширования и rate limiting

| Стратегия | Плюсы | Минусы | Рекомендация |
|---|---|---|---|
| **Только in-memory per node** | Дёшево и быстро | Плохо масштабируется горизонтально, слабый global consistency | Только dev/test |
| **Только Redis** | Единое состояние across pods | Повышает latency и зависимость от сети | Подходит для rate limits и distributed invalidation |
| **Гибрид: local cache + Redis invalidation / counters** | Лучший баланс latency и horizontal scale | Нужна аккуратная операционная логика | **Рекомендую**; это наиболее естественное развитие текущей модели |

### Сравнение альтернатив для масштабирования

| Вариант | Сильные стороны | Риски | Рекомендация |
|---|---|---|---|
| **Модульный монолит + scale up** | Простота и сильная консистентность | Быстро упирается в один control plane/state | Годится как baseline |
| **Stateless runtime pods + shared DB/Redis/outbox** | Хороший баланс сложности и масштабирования | Требует revisioned control plane и строгой конфигурации | **Это целевой вариант для RusTok на ближайший горизонт** |
| **Выделение тяжёлых модулей в сервисы** | Лучшая независимость hot paths | Резкий рост операционной сложности | Только после наведения порядка в control plane |

### Рекомендуемые паттерны проектирования

Для перехода к целевой архитектуре здесь уместны не «модные» паттерны, а очень приземлённые:

- **Revisioned Configuration Store** для platform composition.
- **Application Service** как единственная точка module lifecycle.
- **Transactional Outbox + Operation Journal** для enable/disable и build orchestration.
- **Idempotent Worker** для build/deploy and module hooks.
- **Policy Object / Resolver** для `effective_enabled`.
- **Immutable Release Snapshot** для runtime consumption.
- **Dependency DAG** для migrations, а не lexical ordering.
- **Profile-driven Build Matrix** вместо единственного «full default».

### Пример CI/CD, который реально закроет текущие дыры

```yaml
name: platform-contract

on:
  pull_request:
  push:
    branches: [main]

jobs:
  contract:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: rustok_test
        ports: ["5432:5432"]
    env:
      DATABASE_URL: postgres://postgres:postgres@localhost:5432/rustok_test
    steps:
      - uses: actions/checkout@v4

      - name: Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt,llvm-tools-preview

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Contract checks
        run: |
          cargo xtask validate-manifest
          cargo xtask module validate
          cargo clippy --workspace --all-targets --all-features -- -D warnings
          cargo nextest run --workspace --all-targets --all-features

      - name: Coverage
        run: |
          cargo install cargo-llvm-cov --locked
          cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

      - name: Enforce minimum coverage
        run: ./scripts/ci/check-coverage.sh lcov.info 75

      - name: Upload coverage artifact
        uses: actions/upload-artifact@v4
        with:
          name: lcov
          path: lcov.info
```

### Пример безопасной миграции для locale columns

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // widen only
        manager.get_connection().execute_unprepared(
            r#"
            ALTER TABLE tenants
              ALTER COLUMN default_locale TYPE VARCHAR(32);
            ALTER TABLE tenant_locales
              ALTER COLUMN locale TYPE VARCHAR(32),
              ALTER COLUMN fallback_locale TYPE VARCHAR(32);
            "#
        ).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // irreversible to avoid truncation of valid locale tags
        Ok(())
    }
}
```

### Пример теста для `effective_enabled`

```rust
#[tokio::test]
async fn effective_enabled_uses_manifest_defaults_and_tenant_overrides() {
    let manifest_defaults = set!["content", "pages"];
    let tenant_overrides = hashmap! {
        "pages".to_string() => false,
        "blog".to_string() => true,
    };

    let effective = EffectiveModulePolicyService::resolve(
        core_modules(),
        &manifest_defaults,
        &tenant_overrides,
    );

    assert!(effective.contains("content")); // from manifest default
    assert!(!effective.contains("pages"));  // disabled override
    assert!(effective.contains("blog"));    // enabled override
}
```

## Открытые вопросы и ограничения

Этот отчёт основан на доступных через подключённый GitHub-коннектор файлах репозитория и на выборочном углублении в критичные кодовые пути. Я намеренно не делал предположений там, где репозиторий сам не даёт подтверждения. В частности:

- реальная production-топология деплоя вне репозитория **не указана**;
- наличие внешнего volume/sidecar для `modules.toml` в боевом окружении **не указано**;
- формальная политика API back-compat/versioning **не указана**;
- наличие DB-level RLS/tenant policies **не указано**;
- наличие отдельного внешнего CD pipeline, SBOM-публикации и signed provenance вне этого репозитория **не указано**. fileciteturn103file0L1-L1 fileciteturn81file0L1-L1 fileciteturn88file0L1-L1

Если ограничиться одним главным тезисом, он такой: **RusTok нужно в первую очередь лечить как control plane, а не как ещё один набор модулей**. После устранения локально-файлового composition state, дрейфа `default_enabled`, нетранзакционных module hooks и слабых CI contract gates платформа сможет использовать уже имеющиеся сильные стороны — модульность, tenant-aware runtime, i18n-contract и event-driven architecture — значительно эффективнее.