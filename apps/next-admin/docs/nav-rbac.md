# Контракт навигации и RBAC

## Назначение

Этот документ фиксирует текущий контракт sidebar/navigation в `apps/next-admin`.
Навигация является UX-фильтром, а не security boundary: server pages, API routes,
GraphQL и server actions обязаны выполнять собственные проверки доступа.

## Активные файлы

- `src/shared/config/nav-config.ts` задаёт host-owned core navigation.
- `src/modules/index.ts` импортирует module-owned entrypoints и собирает registry.
- `src/modules/registry.ts` отдаёт зарегистрированные module-owned пункты.
- `src/shared/hooks/use-nav.ts` фильтрует пункты по session role и enabled modules.
- `src/widgets/app-shell/app-sidebar.tsx` группирует отфильтрованные пункты в shell.
- `src/widgets/command-palette/index.tsx` рекурсивно индексирует те же отфильтрованные пункты.

## Источники данных

- Пользовательская роль приходит из `next-auth` session: `session.user.role`.
- Enabled modules читаются через host hook `useEnabledModules()`.
- Module-owned navigation обязана приходить из module/package entrypoint, а не из
  host-owned feature folder.
- Starter-only routes вроде `billing`, `exclusive`, `workspaces` и `workspaces/team`
  не должны попадать в public navigation RusTok.

## Фильтрация

`useFilteredNavItems()` применяет два синхронных UX-фильтра:

1. Если у пункта есть `moduleSlug`, он показывается только когда этот slug есть в
   списке enabled modules.
2. Если у пункта есть `access.role`, роль пользователя должна быть не ниже заданной
   в иерархии `customer < manager < admin < super_admin`.

`access.requireOrg` сейчас считается legacy starter-полем и скрывает пункт. Новые
пункты не должны использовать `requireOrg`, `permission`, `plan` или `feature` без
обновления реального runtime contract.

Фильтрация применяется рекурсивно: сначала проверяются сами пункты и дети, затем
пустые container-пункты с `url: '#'` скрываются. Это не даёт показывать пустые
collapsible-разделы, если все дочерние routes недоступны роли или отключены по
`moduleSlug`.

## Sidebar grouping

Shell группирует уже отфильтрованные пункты через поле `group` с fallback на
`moduleSlug` для module-owned пунктов:

- `Overview` — `Dashboard`.
- `Management` — collapsible `Access`, `Platform`, `Operations`.
- `Module Plugins` — collapsible module-owned containers `Blog`, `Forum`, `Catalog`, `Workflows`.
- `Account` — `Profile`; `Sign Out` остаётся в footer user menu.

Core Next Admin navigation использует `i18nKey` для всех host-owned labels.
Sidebar и command palette берут localized labels из `messages/en.json` и
`messages/ru.json`. Module-owned пункты могут передать `i18nKey`, но не должны
вводить собственную locale fallback-chain поверх host/runtime locale.

Active state считается рекурсивно по текущему pathname, поэтому detail routes вроде
`/dashboard/product/:id` подсвечивают родительский пункт `/dashboard/product`.

## Добавление пункта

Host-owned platform screen добавляется в `coreNavItems` только если это действительно
обязанность host shell. Module-owned screen должен регистрироваться из package/module
entrypoint через `registerAdminModule()`.

Пример host-owned пункта:

```ts
{
  title: 'Access',
  url: '#',
  i18nKey: 'access',
  group: 'management',
  icon: 'users',
  items: [
    {
      title: 'Users',
      url: '/dashboard/users',
      i18nKey: 'users',
      access: { role: 'manager' }
    }
  ]
}
```

Пример module-owned пункта должен жить в пакете модуля:

```ts
registerAdminModule({
  slug: 'product',
  navItems: [
    {
      title: 'Catalog',
      url: '#',
      i18nKey: 'catalog',
      group: 'modulePlugins',
      icon: 'product',
      moduleSlug: 'product',
      items: [
        {
          title: 'Products',
          url: '/dashboard/product',
          i18nKey: 'products'
        }
      ]
    }
  ]
});
```

## Verification

После изменений навигации нужно прогнать:

- `npm run typecheck` в `apps/next-admin`;
- визуальную проверку sidebar на active state и группировку;
- проверку, что отключённые module slug не показывают module-owned пункты.
- проверку command palette: дочерние routes индексируются рекурсивно и названы
  на текущем языке.
