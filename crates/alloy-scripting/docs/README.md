# alloy-scripting docs

Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ capability-crate `crates/alloy-scripting`.

## РЎРѕРґРµСЂР¶Р°РЅРёРµ

- [Alloy Concept](../../../docs/alloy-concept.md) вЂ” СЃС‚СЂР°С‚РµРіРёС‡РµСЃРєРѕРµ РІРёРґРµРЅРёРµ Alloy: Self-Evolving Integration Runtime
- [implementation-plan.md](./implementation-plan.md) вЂ” Р°СЂС…РёС‚РµРєС‚СѓСЂР°, РєРѕРјРїРѕРЅРµРЅС‚С‹, flow РІС‹РїРѕР»РЅРµРЅРёСЏ Рё future improvements

## РљСЂР°С‚РєРёР№ РѕР±Р·РѕСЂ

`alloy-scripting` вЂ” runtime/engine crate РґР»СЏ Alloy РЅР° Р±Р°Р·Рµ Rhai.

### РћСЃРЅРѕРІРЅС‹Рµ РІРѕР·РјРѕР¶РЅРѕСЃС‚Рё

1. **Event hooks** вЂ” СЃРєСЂРёРїС‚С‹ СЃСЂР°Р±Р°С‚С‹РІР°СЋС‚ РЅР° СЃРѕР±С‹С‚РёСЏ СЃСѓС‰РЅРѕСЃС‚РµР№ (`before_create`, `after_update`, `on_commit`)
2. **Cron scheduler** вЂ” scheduled-РІС‹РїРѕР»РЅРµРЅРёРµ РїРѕ СЂР°СЃРїРёСЃР°РЅРёСЋ
3. **API triggers** вЂ” СЃРєСЂРёРїС‚С‹ РєР°Рє HTTP endpoints
4. **Manual execution** вЂ” СЂСѓС‡РЅРѕР№ Р·Р°РїСѓСЃРє С‡РµСЂРµР· API

### Р‘РµР·РѕРїР°СЃРЅРѕСЃС‚СЊ

- Resource limits (`max_operations`, `timeout`, `call_depth`)
- Auto-disable РїРѕСЃР»Рµ СЃРµСЂРёРё РѕС€РёР±РѕРє
- Sandboxed execution Р±РµР· РїСЂСЏРјРѕРіРѕ FS/network РґРѕСЃС‚СѓРїР° РІРЅРµ СЂР°Р·СЂРµС€С‘РЅРЅС‹С… bridge-СЃР»РѕС‘РІ

### РРЅС‚РµРіСЂР°С†РёСЏ СЃ РїР»Р°С‚С„РѕСЂРјРѕР№

`alloy-scripting` Р±РѕР»СЊС€Рµ РЅРµ СЂРµРіРёСЃС‚СЂРёСЂСѓРµС‚СЃСЏ РєР°Рє tenant-toggle РјРѕРґСѓР»СЊ РІ `ModuleRegistry`.
Р­С‚Рѕ module-agnostic runtime/capability СЃР»РѕР№ Alloy, РєРѕС‚РѕСЂС‹Р№:

- РґР°С‘С‚ РґРІРёР¶РѕРє, storage, execution log Рё migration-СЃР»РѕР№ РґР»СЏ Alloy;
- РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ transport-Р°РґР°РїС‚РµСЂРѕРј `alloy`;
- РёРЅС‚РµРіСЂРёСЂСѓРµС‚СЃСЏ СЃ `rustok-mcp` РєР°Рє СЃ РєР°РЅРѕРЅРёС‡РµСЃРєРѕР№ РІРЅРµС€РЅРµР№ surface-С‚РѕС‡РєРѕР№ Alloy;
- РјРѕР¶РµС‚ РІС‹Р·С‹РІР°С‚СЊСЃСЏ РёР· `rustok-workflow` С‡РµСЂРµР· `alloy_script` Рё `ScriptRunner`, РЅРѕ Р±РµР· runtime-Р·Р°РІРёСЃРёРјРѕСЃС‚Рё `workflow -> alloy`.

РњРѕРґСѓР»СЊ С‚Р°РєР¶Рµ РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚:

- `ScriptableEntity` trait РґР»СЏ РёРЅС‚РµРіСЂР°С†РёРё СЃ РґРѕРјРµРЅРЅС‹РјРё СЃСѓС‰РЅРѕСЃС‚СЏРјРё;
- `HookExecutor` РґР»СЏ СѓРґРѕР±РЅРѕРіРѕ РІС‹Р·РѕРІР° hooks РёР· СЃРµСЂРІРёСЃРѕРІ;
- `ScriptOrchestrator` РґР»СЏ РєРѕРѕСЂРґРёРЅР°С†РёРё РІС‹РїРѕР»РЅРµРЅРёСЏ.

GraphQL Рё transport-shell РґР»СЏ Alloy РІС‹РЅРµСЃРµРЅС‹ РІ `crates/alloy`, С‡С‚РѕР±С‹ РЅРµ Р·Р°РјС‹РєР°С‚СЊ С†РёРєР»
Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№ `alloy-scripting -> rustok-api -> rustok-core -> alloy-scripting`.
`alloy-scripting` РѕСЃС‚Р°С‘С‚СЃСЏ runtime/engine СЃР»РѕРµРј, Р° `apps/server` РїРѕРґРєР»СЋС‡Р°РµС‚ `alloy`
РєР°Рє composition-root shim Р±РµР· СЂРµРіРёСЃС‚СЂР°С†РёРё Alloy РєР°Рє tenant module.

РЎРј. [implementation-plan.md](./implementation-plan.md) РґР»СЏ РґРµС‚Р°Р»РµР№.

