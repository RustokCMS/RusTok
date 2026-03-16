# rustok-storage

Абстрактный слой хранилища для платформы RusTok.
Предоставляет единый `StorageBackend` trait для всех доменных модулей, работающих с файлами, — независимо от конкретного бэкенда (локальная файловая система, S3, MinIO и т.д.).

## Назначение

- Изолирует доменные модули (`rustok-media` и др.) от деталей конкретного хранилища.
- Позволяет переключать бэкенды через конфигурацию без изменения бизнес-логики.
- Обеспечивает единообразный API: `store / delete / public_url`.

## Структура

```
crates/rustok-storage/
├── src/
│   ├── lib.rs          # pub-реэкспорт всего публичного API
│   ├── backend.rs      # StorageBackend trait + UploadedObject
│   ├── error.rs        # StorageError, Result<T>
│   ├── local.rs        # LocalStorage + LocalStorageConfig
│   └── service.rs      # StorageService (высокоуровневая обёртка)
└── Cargo.toml
```

## Публичный API

### `StorageBackend` trait

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store(&self, path: &str, data: Bytes, content_type: &str)
        -> Result<UploadedObject>;
    async fn delete(&self, path: &str) -> Result<()>;
    fn public_url(&self, path: &str) -> String;
}
```

`delete` идемпотентна: отсутствующий объект → `Ok(())`.

### `UploadedObject`

```rust
pub struct UploadedObject {
    pub path: String,       // Relative storage path
    pub public_url: String, // URL for clients
    pub size: u64,
}
```

### `StorageService`

Высокоуровневая обёртка над `StorageBackend`:

```rust
// Создание из конфигурации
let service = StorageService::from_config(&config);

// Генерация tenant-scoped пути: <tenant_id>/<year>/<month>/<uuid>.<ext>
let path = StorageService::generate_path(tenant_id, "photo.jpg");

// Сохранение объекта
let obj = service.store(&path, data, "image/jpeg").await?;

// Получение публичного URL
let url = service.public_url(&path);

// Удаление
service.delete(&path).await?;

// Имя активного бэкенда (для метрик/логов)
let driver = service.backend_name(); // "local"
```

## Конфигурация

В файле `config/development.yaml` (секция `rustok`):

```yaml
rustok:
  storage:
    driver: local      # "local" | будущие: "s3", "gcs"
    local:
      base_dir: storage/media   # Директория на диске
      base_url: /media          # URL-префикс для публичных ссылок
```

`StorageConfig` сериализуется/десериализуется через serde; все поля имеют дефолты (local бэкенд).

## Бэкенды

### `LocalStorage`

- Хранит файлы в `base_dir/<path>`.
- Автоматически создаёт промежуточные директории.
- Защита от path traversal: пути с `..` отклоняются с `StorageError::InvalidPath`.
- Публичные URL: `<base_url>/<path>`.

Рекомендуется использовать вместе со статическим сервером (nginx, `tower-http::fs::ServeDir`) для раздачи файлов.

### S3-совместимый бэкенд (planned)

Планируется в виде опциональной Cargo-фичи `s3` (deps: `aws-sdk-s3`, `aws-config`).
Скаффолдинг уже в `Cargo.toml` (`[features] s3 = [...]`).

## Интеграция в сервер

`StorageService` регистрируется в `ctx.shared_store` при старте сервера:

```rust
// apps/server/src/services/app_runtime.rs
#[cfg(feature = "mod-media")]
fn init_storage(ctx: &AppContext, settings: &RustokSettings) {
    let service = StorageService::from_config(&settings.storage);
    ctx.shared_store.insert(service);
}
```

Из контроллеров или GraphQL-резолверов:

```rust
let storage = ctx.shared_store
    .get::<StorageService>()
    .ok_or(Error::InternalServerError)?;
```

Фича-флаг: `mod-media` (включён по умолчанию).

## Ошибки

```rust
pub enum StorageError {
    Io(std::io::Error),              // Файловые операции
    InvalidPath(String),             // Path traversal / несуществующий путь
    NotFound(String),                // Объект не найден (зарезервировано для S3)
    Backend(String),                 // Прочие ошибки бэкенда
}
```

## Health check

В `GET /health/ready` добавлена проверка хранилища (NonCritical):

```
storage → ok | degraded
```

Результат обновляет метрику `rustok_media_storage_health{driver="local"}`.

## Prometheus-метрики

| Метрика | Тип | Описание |
|---------|-----|----------|
| `rustok_media_storage_health{driver}` | Gauge | 1=healthy, 0=unhealthy |

Дополнительные метрики (счётчики загрузок и байтов) определены в `rustok-telemetry` и инкрементируются на уровне контроллера `rustok-media`.

## Тестирование

Для тестов используйте `tempfile::TempDir` + `LocalStorage`:

```rust
use rustok_storage::{LocalStorage, StorageBackend};
use tempfile::TempDir;

let dir = TempDir::new().unwrap();
let storage = LocalStorage::new(dir.path(), "/media");

let obj = storage.store("test.txt", b"hello".into(), "text/plain").await.unwrap();
assert_eq!(obj.public_url, "/media/test.txt");
```

## Зависимости

| Crate | Зачем |
|-------|-------|
| `async-trait` | Async trait objects |
| `bytes` | Zero-copy буферы данных |
| `chrono` | Год/месяц в generate_path |
| `uuid` | ID в generate_path |
| `tokio` | `fs::write`, `fs::remove_file`, `fs::create_dir_all` |
| `serde` | StorageConfig |
| `thiserror` | StorageError |
| `tracing` | Инструментирование store/delete |
