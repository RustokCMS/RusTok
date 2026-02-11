# Решение проблемы с parcel_css и tailwind-rs

## Проблема (была)

Приложения `apps/admin` и `apps/storefront` не компилировались из-за проблем с зависимостью `tailwind-rs`:
- Использовала устаревший `parcel_css` v1.0.0-alpha.32
- Ошибки компиляции из-за несовместимости API
- Блокировала всю фронтенд-разработку

## Решение (применено)

**Мигрировали с `tailwind-rs` на официальный Tailwind CSS CLI**

### Преимущества нового решения

| Критерий | tailwind-rs (было) | Tailwind CLI (теперь) |
|----------|-------------------|----------------------|
| Компиляция | ❌ Не работает | ✅ Работает |
| Поддержка | ❌ Заброшен | ✅ Активная (Tailwind Labs) |
| Версия TW | ❌ Устаревшая | ✅ Последняя (v4 готов) |
| Скорость | ⚠️ Медленнее | ✅ Оптимизирован |
| Плагины | ❌ Ограничены | ✅ Все доступны |

## Что изменилось

### 1. Зависимости

✅ **Удалено:** `tailwind-rs` из всех `Cargo.toml`  
✅ **Добавлено:** `tailwindcss` через npm

### 2. Файлы конфигурации

Созданы:
- `apps/admin/tailwind.config.js`
- `apps/storefront/tailwind.config.js`
- `package.json` с npm скриптами

Обновлены:
- `apps/admin/Trunk.toml` - использует `npx tailwindcss`
- `Cargo.toml` - фронтенд приложения снова включены
- `.gitignore` - игнорирует сгенерированные CSS файлы

## Как работать теперь

### Для разработки админки (Leptos CSR + Trunk)

```bash
cd apps/admin
trunk serve
```

Trunk автоматически запускает Tailwind CLI через hooks.

### Для разработки витрины (Leptos SSR)

```bash
cd apps/storefront
cargo leptos watch
```

### Только пересборка CSS (если нужно)

```bash
# Watch-режим (автоматически пересобирает при изменениях)
npm run css:admin
npm run css:storefront

# Production сборка (минифицированная)
npm run css:admin:build
npm run css:storefront:build
```

## Использование Tailwind классов

Всё как раньше - просто используйте классы в Leptos компонентах:

```rust
use leptos::*;

#[component]
pub fn Button() -> impl IntoView {
    view! {
        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
            "Нажми меня"
        </button>
    }
}
```

Tailwind автоматически:
1. Сканирует ваши `.rs` файлы
2. Находит классы
3. Генерирует только используемый CSS (tree-shaking)

## Настройка кастомных стилей

Отредактируйте `tailwind.config.js` в нужном приложении:

```javascript
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          500: '#ваш-цвет',
        },
      },
    },
  },
  plugins: [],
}
```

## Проверка работоспособности

```bash
# 1. Проверить что workspace компилируется
cargo check

# 2. Проверить что фронтенд приложения компилируются (теперь должно работать!)
cargo check --package rustok-admin
cargo check --package rustok-storefront

# 3. Собрать админку для production
cd apps/admin && trunk build --release

# 4. Собрать витрину для production  
cd apps/storefront && cargo leptos build --release
```

## Требования

- ✅ Node.js v18+ (для Tailwind CLI)
- ✅ npm или npx
- ✅ Rust 1.80+ (как и раньше)

## CI/CD

Добавьте шаг установки Node.js и сборки CSS:

```yaml
- name: Setup Node.js
  uses: actions/setup-node@v3
  with:
    node-version: '20'

- name: Install npm dependencies
  run: npm install

- name: Build Tailwind CSS
  run: |
    npm run css:admin:build
    npm run css:storefront:build

- name: Build Rust
  run: cargo build --release
```

## Альтернатива для CI (без Node.js)

Если не хотите Node.js в CI:

1. Локально соберите CSS: `npm run css:admin:build && npm run css:storefront:build`
2. Закоммитьте сгенерированные файлы в git (удалив их из `.gitignore`)
3. CI просто собирает Rust (используя предварительно скомпилированный CSS)

## Документация

- `TAILWIND_CSS_SETUP.md` - Подробная инструкция по использованию (EN)
- `TAILWIND_MIGRATION_SUMMARY.md` - Детали миграции (EN)
- `CODE_AUDIT_REPORT_2026-02-11.md` - Обновлённый отчёт аудита

## Устранение проблем

### CSS не обновляется?

```bash
# Вручную пересоберите CSS
npm run css:admin:build

# Очистите кэш Trunk
rm -rf apps/admin/dist
trunk serve
```

### Классы не генерируются?

1. Проверьте что ваши `.rs` файлы включены в `content` в `tailwind.config.js`
2. Перезапустите dev-сервер
3. Убедитесь что Tailwind CLI запущен

### Node.js не найден?

Установите Node.js:
```bash
# Проверить версию
node --version  # Должна быть v18+

# Или установить через nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

## Результат

✅ **Фронтенд приложения снова компилируются!**  
✅ **Используем современный, поддерживаемый инструментарий**  
✅ **Доступны все фичи последней версии Tailwind**  
✅ **Быстрее работает и меньше багов**  

## Вопросы?

См. `TAILWIND_CSS_SETUP.md` для детальных инструкций.
