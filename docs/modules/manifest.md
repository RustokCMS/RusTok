# Module Manifest (WordPress/NodeBB-style rebuilds)

Этот документ описывает **манифест модулей** — файл, который фиксирует состав
модулей для сборки RusToK и позволяет **устанавливать/удалять модули через
пересборку** (в стиле WordPress/NodeBB: админка → rebuild → новый бинарник).

Манифест — единый источник правды о том, какие модули входят в конкретную
сборку. Изменение манифеста = новая сборка и деплой.

## Зачем нужен манифест

- **Динамичность состава модулей**: можно добавлять/удалять модули без
  долгосрочной “жесткой сборки”.
- **Сборка = набор модулей**: каждый артефакт соответствует списку модулей.
- **Админка как в NodeBB**: установка/удаление модуля инициирует rebuild.
- **План внедрения**: подробная дорожная карта находится в
  `docs/modules/module-rebuild-plan.md`.

## Где используется

1. **Админка** изменяет манифест (install/uninstall).
2. **Build-service** читает манифест, собирает новый бинарник.
3. **Deploy** обновляет приложение.
4. **Registry** в рантайме уже содержит только модули из новой сборки.

## Формат манифеста (TOML)

Рекомендуемый формат — TOML, чтобы удобно использовать в CI и в Rust-скриптах.

```toml
schema = 1
app = "rustok-server"

[build]
target = "x86_64-unknown-linux-gnu"
profile = "release"
deployment_profile = "monolith" # monolith | headless

[modules]
# slug = { crate = "...", source = "...", version = "...", features = [...] }
content = { crate = "rustok-content", source = "crates-io", version = "0.1" }
commerce = { crate = "rustok-commerce", source = "git", git = "ssh://git/commerce.git", rev = "abc123" }
blog = { crate = "rustok-blog", source = "path", path = "../modules/rustok-blog" }
forum = { crate = "rustok-forum", source = "crates-io", version = "0.1", features = ["comments"] }
pages = { crate = "rustok-pages", source = "path", path = "../modules/rustok-pages" }

[settings]
default_enabled = ["content", "commerce", "pages"]
```

### Поля

| Поле | Тип | Обязательное | Описание |
| --- | --- | --- | --- |
| `schema` | int | да | Версия формата манифеста. |
| `app` | string | да | Целевое приложение/бинарник. |
| `build.target` | string | нет | Целевой triple сборки. |
| `build.profile` | string | нет | Профиль сборки (`release`/`debug`). |
| `build.deployment_profile` | string | нет | Режим деплоя: `monolith` (единый релиз server+admin+storefront) или `headless` (раздельные сервисы). |
| `modules` | table | да | Карта `slug -> module spec`. |
| `settings.default_enabled` | array | нет | Какие модули включать по умолчанию после сборки. |

### Module spec

| Поле | Тип | Обязательное | Описание |
| --- | --- | --- | --- |
| `crate` | string | да | Имя crate модуля. |
| `source` | string | да | `crates-io` \| `git` \| `path`. |
| `version` | string | нет | Версия для `crates-io`. |
| `git` | string | нет | Git URL. |
| `rev` | string | нет | Commit SHA/таг. |
| `path` | string | нет | Локальный путь (monorepo или vendor). |
| `features` | array | нет | Фичи для конкретного модуля. |

> Сами метаданные модуля (slug/name/description/version/deps) всё равно берутся
> из `RusToKModule` во время сборки и регистрации в `ModuleRegistry`.


## Режимы деплоя (monolith/headless)

### `monolith`
- один release-id и один оркестрируемый pipeline для `apps/server`, `apps/admin`, `apps/storefront`;
- откат — на единый release.

### `headless`
- backend и UI деплоятся на разных серверах/сервисах;
- rebuild от одного `modules.toml` запускает раздельные pipeline-ветки;
- перед деплоем UI обязателен preflight compatibility-check с целевой версией backend.

## Жизненный цикл install/uninstall

### Установка
1. Админка добавляет модуль в манифест.
2. Build-service запускает сборку.
3. Деплой выкатывает новый бинарник.
4. Registry содержит новый модуль, а `tenant_modules` управляет его включением.

### Удаление
1. Админка удаляет модуль из манифеста.
2. Build-service пересобирает приложение без модуля.
3. Новый бинарник больше не содержит код модуля.

## Админка в стиле NodeBB

UI шаги:
1. Выбрать модуль из каталога (или указать URL/путь).
2. Нажать **Install / Uninstall**.
3. Админка показывает статус сборки (очередь → build → deploy).
4. После деплоя модуль доступен для включения на уровне tenant.

## Минимальные гарантии

- **Консистентность**: если сборка прошла, модуль гарантированно присутствует.
- **Безопасность**: нет runtime-подгрузки нативного кода.
- **Воспроизводимость**: манифест фиксирует точный состав и версии.

## Blueprint: API для admin rebuild

Ниже — минимальная схема API, которую можно подключить к админке, чтобы запускать
пересборки и показывать прогресс.

### Endpoint: создать сборку

`POST /admin/builds`

```json
{
  "manifest_ref": "main",
  "requested_by": "admin@rustok",
  "reason": "install module: pages",
  "modules": {
    "content": { "source": "crates-io", "crate": "rustok-content", "version": "0.1" },
    "forum": { "source": "git", "crate": "rustok-forum", "git": "ssh://git/forum.git", "rev": "abc123" },
    "pages": { "source": "path", "crate": "rustok-pages", "path": "../modules/rustok-pages" }
  }
}
```

**Ответ:**
```json
{ "build_id": "bld_01H...", "status": "queued" }
```

### Endpoint: статус сборки

`GET /admin/builds/{build_id}`

```json
{
  "build_id": "bld_01H...",
  "status": "running",
  "stage": "build",
  "progress": 62,
  "logs_url": "https://builds/rustok/bld_01H.../logs"
}
```

### Endpoint: деплой/активация

`POST /admin/builds/{build_id}/deploy`

```json
{ "environment": "prod" }
```

### Endpoint: rollback

`POST /admin/builds/{build_id}/rollback`

```json
{ "target_release": "rel_2025_01_10_001" }
```

## Blueprint: build pipeline (Docker/K8s)

Пример пайплайна:

1. **Checkout + deps**: забрать repo + загрузить зависимости.
2. **Render manifest**: зафиксировать `modules.toml` в workspace.
3. **Cargo build**: `cargo build -p rustok-server --release`.
4. **Docker image**: собрать образ с готовым бинарником.
5. **Push**: загрузить в registry.
6. **Deploy**: обновить deployment (K8s) или контейнер (docker-compose).
7. **Smoke**: проверить `/health` и `/health/modules`.

## Blueprint: rollback

1. **Хранить релизы**: у каждого деплоя есть `release_id` и образ.
2. **Откат**: переключить deployment на предыдущий `release_id`.
3. **Проверка**: повторить smoke-check.
4. **Фиксация**: записать rollback в журнал событий админки.

## Blueprint: что сохраняет админка

- `build_id`, `release_id`, `status`, `started_at`, `finished_at`
- `manifest_hash`, `modules_delta`, `requested_by`, `reason`

> **Статус документа:** Актуальный. Формат манифеста (`modules.toml`) и жизненный цикл rebuild могут уточняться — фиксируйте изменения здесь и в `docs/index.md`.
