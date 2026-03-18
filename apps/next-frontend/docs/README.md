# Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ apps/next-frontend

`apps/next-frontend` вЂ” Next.js storefront РІ РїР°СЂР°Р»Р»РµР»СЊРЅРѕР№ СЃС…РµРјРµ С„СЂРѕРЅС‚РµРЅРґРѕРІ RusToK.

## Р¦РµР»СЊ

РџСЂРёР»РѕР¶РµРЅРёРµ РїРѕРІС‚РѕСЂСЏРµС‚ РєР»СЋС‡РµРІС‹Рµ Р°СЂС…РёС‚РµРєС‚СѓСЂРЅС‹Рµ РїСЂРёРЅС†РёРїС‹ Р°РґРјРёРЅРѕРє:

- FSD-РѕСЂРёРµРЅС‚РёСЂРѕРІР°РЅРЅР°СЏ СЃС‚СЂСѓРєС‚СѓСЂР° СЃР»РѕС‘РІ (`app`, `modules`, `shared`);
- РµРґРёРЅС‹Р№ UI-РєРѕРЅС‚СЂР°РєС‚ С‡РµСЂРµР· internal UI workspace (`UI/next`);
- РїР°СЂРёС‚РµС‚ СЃРµС‚РµРІС‹С… Рё auth-РєРѕРЅС‚СЂР°РєС‚РѕРІ С‡РµСЂРµР· СЃР°РјРѕРїРёСЃРЅС‹Рµ РїР°РєРµС‚С‹ `leptos-*`.

## Р§С‚Рѕ РїРµСЂРµРЅРµСЃРµРЅРѕ РёР· РїРѕРґС…РѕРґР° Р°РґРјРёРЅРѕРє

### 1) РћР±С‰РёРµ frontend-Р±РёР±Р»РёРѕС‚РµРєРё

Р’ storefront РїРѕРґРєР»СЋС‡РµРЅС‹ С‚Рµ Р¶Рµ РІРЅСѓС‚СЂРµРЅРЅРёРµ РїР°РєРµС‚С‹, С‡С‚Рѕ РёСЃРїРѕР»СЊР·СѓСЋС‚СЃСЏ РґР»СЏ РїР°СЂРёС‚РµС‚Р° РІ Р°РґРјРёРЅРєР°С…:

- `leptos-graphql/next` вЂ” РµРґРёРЅС‹Р№ GraphQL РєРѕРЅС‚СЂР°РєС‚ (`/api/graphql`, `Authorization`, `X-Tenant-Slug`);
- `leptos-auth/next` вЂ” РµРґРёРЅС‹Р№ С„РѕСЂРјР°С‚ РєР»РёРµРЅС‚СЃРєРѕР№ auth-СЃРµСЃСЃРёРё Рё С‚РёРїРёР·Р°С†РёСЏ РѕС€РёР±РѕРє;
- `leptos-hook-form`, `leptos-zod`, `leptos-zustand` вЂ” СЃР»РѕР№ СЂР°СЃС€РёСЂРµРЅРёСЏ РґР»СЏ С„РѕСЂРј/РІР°Р»РёРґР°С†РёРё/СЃРѕСЃС‚РѕСЏРЅРёСЏ.

Р”Р»СЏ РїСЂРёРєР»Р°РґРЅРѕРіРѕ РєРѕРґР° РІРёС‚СЂРёРЅС‹ СЃРѕР·РґР°РЅР° FSD-РѕР±С‘СЂС‚РєР° РІ `src/shared/lib/`:

- `src/shared/lib/graphql.ts` вЂ” `storefrontGraphql(...)` + СЂРµСЌРєСЃРїРѕСЂС‚ Р±Р°Р·РѕРІС‹С… GraphQL-С‚РёРїРѕРІ Рё РєРѕРЅСЃС‚Р°РЅС‚;
- `src/shared/lib/auth.ts` вЂ” СЂРµСЌРєСЃРїРѕСЂС‚ auth-С‚РёРїРѕРІ/С…РµР»РїРµСЂРѕРІ (`getClientAuth`, `mapAuthError`, РєР»СЋС‡Рё cookie/token).

4. Р”Р»СЏ РёР·РјРµРЅРµРЅРёР№ UI-РєРѕРЅС‚СЂР°РєС‚РѕРІ СЃРёРЅС…СЂРѕРЅРЅРѕ РѕР±РЅРѕРІР»СЏС‚СЊ:
   - `UI/docs/api-contracts.md`;
   - `docs/UI/storefront.md`;
   - СЌС‚Сѓ СЃС‚СЂР°РЅРёС†Сѓ.

## РЎРІСЏР·Р°РЅРЅС‹Рµ РґРѕРєСѓРјРµРЅС‚С‹

- `/docs/UI/storefront.md`
- `/docs/UI/fsd-restructuring-plan.md`
- `/docs/index.md`

## Module-aware storefront notes

- `app/layout.tsx` now mounts `EnabledModulesProvider` for tenant-aware storefront UI state.
- `src/modules/registry.ts` accepts optional `moduleSlug` and filters registered slot content by enabled modules.
- Self-authored module packages should register storefront entries with `moduleSlug` when the widget belongs to an optional module.
