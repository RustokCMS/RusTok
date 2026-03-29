# rustok-workflow вЂ” РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ РјРѕРґСѓР»СЏ

Р’РёР·СѓР°Р»СЊРЅР°СЏ Р°РІС‚РѕРјР°С‚РёР·Р°С†РёСЏ РЅР° РїР»Р°С‚С„РѕСЂРјРµРЅРЅРѕР№ РѕС‡РµСЂРµРґРё.

РљР°РЅРѕРЅРёС‡РµСЃРєР°СЏ РјРѕРґСѓР»СЊРЅР°СЏ РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ workflow Р¶РёРІС‘С‚ РІ СЌС‚РѕРј С„Р°Р№Р»Рµ.

## РќР°Р·РЅР°С‡РµРЅРёРµ

`rustok-workflow` РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚ РІРёР·СѓР°Р»СЊРЅС‹Р№ РєРѕРЅСЃС‚СЂСѓРєС‚РѕСЂ Р°РІС‚РѕРјР°С‚РёР·Р°С†РёР№ (Р°РЅР°Р»РѕРі n8n / Directus Flows),
РІСЃС‚СЂРѕРµРЅРЅС‹Р№ РІ СЃРѕР±С‹С‚РёР№РЅСѓСЋ РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂСѓ РїР»Р°С‚С„РѕСЂРјС‹. РњРѕРґСѓР»СЊ РѕСЂРєРµСЃС‚СЂРёСЂСѓРµС‚ РІР·Р°РёРјРѕРґРµР№СЃС‚РІРёРµ РјРµР¶РґСѓ
РґРѕРјРµРЅРЅС‹РјРё РјРѕРґСѓР»СЏРјРё С‡РµСЂРµР· СЃРѕР±С‹С‚РёСЏ, РЅРµ СЃРѕР·РґР°РІР°СЏ СЃРѕР±СЃС‚РІРµРЅРЅС‹Р№ event loop.

## РђСЂС…РёС‚РµРєС‚СѓСЂР°

```
DomainEvent (blog.published, order.paid, ...)
       в†“
  EventBus (outbox в†’ EventTransport)
       в†“
  WorkflowTriggerHandler     в†ђ РїРѕРґРїРёСЃР°РЅ РЅР° СЃРѕР±С‹С‚РёСЏ РїР»Р°С‚С„РѕСЂРјС‹
       в†“
  WorkflowEngine             в†ђ РЅР°С…РѕРґРёС‚ matching workflows РїРѕ tenant + trigger
       в†“
  Step 1 в†’ Step 2 в†’ Step 3  в†ђ Р»РёРЅРµР№РЅР°СЏ С†РµРїРѕС‡РєР° С€Р°РіРѕРІ
       в†“         в†“
  РєР°Р¶РґС‹Р№ С€Р°Рі РјРѕР¶РµС‚ РїСѓР±Р»РёРєРѕРІР°С‚СЊ DomainEvent РѕР±СЂР°С‚РЅРѕ РІ outbox
```

Workflow **РЅРµ РІР»Р°РґРµРµС‚** С‚СЂР°РЅСЃРїРѕСЂС‚РѕРј СЃРѕР±С‹С‚РёР№ вЂ” РѕРЅ СЂР°Р±РѕС‚Р°РµС‚ С‡РµСЂРµР· Р°Р±СЃС‚СЂР°РєС†РёРё
`EventBus` / `EventTransport` РёР· `rustok-core`. РљРѕРЅРєСЂРµС‚РЅС‹Р№ С‚СЂР°РЅСЃРїРѕСЂС‚ (Iggy, RabbitMQ,
Р±Р°Р·РѕРІС‹Р№ Outbox) РЅРµ РёРјРµРµС‚ Р·РЅР°С‡РµРЅРёСЏ РґР»СЏ РјРѕРґСѓР»СЏ.

## РњРѕРґРµР»СЊ РґР°РЅРЅС‹С…

| РўР°Р±Р»РёС†Р° | РќР°Р·РЅР°С‡РµРЅРёРµ |
|---------|-----------|
| `workflows` | РћРїСЂРµРґРµР»РµРЅРёРµ workflow: С‚СЂРёРіРіРµСЂ, СЃС‚Р°С‚СѓСЃ, tenant |
| `workflow_versions` | Р’РµСЂСЃРёРѕРЅРёСЂРѕРІР°РЅРёРµ: СЃРЅСЌРїС€РѕС‚ steps + config РєР°Р¶РґРѕР№ РІРµСЂСЃРёРё |
| `workflow_steps` | РЁР°РіРё workflow: С‚РёРї, РєРѕРЅС„РёРі, РїРѕСЂСЏРґРѕРє, РѕР±СЂР°Р±РѕС‚РєР° РѕС€РёР±РѕРє |
| `workflow_executions` | Р–СѓСЂРЅР°Р» Р·Р°РїСѓСЃРєРѕРІ: СЃС‚Р°С‚СѓСЃ, РєРѕРЅС‚РµРєСЃС‚, РѕС€РёР±РєР° |
| `workflow_step_executions` | Р–СѓСЂРЅР°Р» РІС‹РїРѕР»РЅРµРЅРёСЏ РєР°Р¶РґРѕРіРѕ С€Р°РіР° РІ СЂР°РјРєР°С… Р·Р°РїСѓСЃРєР° |

## РўРёРїС‹ С‚СЂРёРіРіРµСЂРѕРІ

| РўРёРї | РСЃС‚РѕС‡РЅРёРє |
|-----|---------|
| `event` | `DomainEvent` С‡РµСЂРµР· `EventBus` |
| `cron` | Р Р°СЃРїРёСЃР°РЅРёРµ (cron-РІС‹СЂР°Р¶РµРЅРёРµ), С‚РёРє С‡РµСЂРµР· `WorkflowCronScheduler` |
| `webhook` | Р’С…РѕРґСЏС‰РёР№ HTTP-Р·Р°РїСЂРѕСЃ РЅР° РїР»Р°С‚С„РѕСЂРјРµРЅРЅС‹Р№ СЌРЅРґРїРѕРёРЅС‚ |
| `manual` | РљРЅРѕРїРєР° РІ Р°РґРјРёРЅРєРµ / API-РІС‹Р·РѕРІ |

## РўРёРїС‹ С€Р°РіРѕРІ

| РўРёРї | Р§С‚Рѕ РґРµР»Р°РµС‚ |
|-----|-----------|
| `action` | Р’С‹Р·С‹РІР°РµС‚ РґРµР№СЃС‚РІРёРµ РїР»Р°С‚С„РѕСЂРјРµРЅРЅРѕРіРѕ СЃРµСЂРІРёСЃР° |
| `emit_event` | РџСѓР±Р»РёРєСѓРµС‚ `DomainEvent` РІ outbox |
| `condition` | Р’РµС‚РІР»РµРЅРёРµ РїРѕ Р·РЅР°С‡РµРЅРёСЋ РІ JSON-РєРѕРЅС‚РµРєСЃС‚Рµ |
| `delay` | РћС‚Р»РѕР¶РµРЅРЅРѕРµ РІС‹РїРѕР»РЅРµРЅРёРµ С‡РµСЂРµР· scheduled event |
| `http` | Р’РЅРµС€РЅРёР№ HTTP-Р·Р°РїСЂРѕСЃ (webhook out) |
| `alloy_script` | Р—Р°РїСѓСЃРєР°РµС‚ Rhai-СЃРєСЂРёРїС‚ С‡РµСЂРµР· `alloy` |
| `notify` | РЈРІРµРґРѕРјР»РµРЅРёРµ (email, Slack, Telegram) |

## РЎРІСЏР·СЊ СЃ Alloy

`rustok-workflow` РёСЃРїРѕР»СЊР·СѓРµС‚ Alloy РєР°Рє capability РґР»СЏ РѕС‚РґРµР»СЊРЅС‹С… С€Р°РіРѕРІ, РЅРѕ Р±РѕР»СЊС€Рµ РЅРµ РѕР±СЉСЏРІР»СЏРµС‚
runtime-Р·Р°РІРёСЃРёРјРѕСЃС‚СЊ `workflow -> alloy` РІ module registry.

Workflow РѕСЂРєРµСЃС‚СЂРёСЂСѓРµС‚ вЂ” Alloy РёСЃРїРѕР»РЅСЏРµС‚. Alloy РјРѕР¶РµС‚ Р±С‹С‚СЊ С€Р°РіРѕРј РІРЅСѓС‚СЂРё workflow:

```
Trigger: order.paid
  в†’ Step 1: alloy_script "СЃРіРµРЅРµСЂРёСЂСѓР№ invoice PDF"
  в†’ Step 2: notify вЂ” РѕС‚РїСЂР°РІСЊ email РєР»РёРµРЅС‚Сѓ
  в†’ Step 3: http вЂ” СѓРІРµРґРѕРјРёС‚СЊ CRM
```

Р’ РїРµСЂСЃРїРµРєС‚РёРІРµ Alloy РјРѕР¶РµС‚ РїРѕСЂРѕР¶РґР°С‚СЊ workflow РёР· РѕРїРёСЃР°РЅРёСЏ РЅР° РЅР°С‚СѓСЂР°Р»СЊРЅРѕРј СЏР·С‹РєРµ.

## RBAC

Р РµСЃСѓСЂСЃ `Workflows`: `Create`, `Read`, `Update`, `Delete`, `List`, `Execute`, `Manage`.
Р РµСЃСѓСЂСЃ `WorkflowExecutions`: `Read`, `List`.

Р’СЃРµ С‚Р°Р±Р»РёС†С‹ СЃРѕРґРµСЂР¶Р°С‚ `tenant_id` вЂ” РїРѕР»РЅР°СЏ РёР·РѕР»СЏС†РёСЏ РјРµР¶РґСѓ С‚РµРЅР°РЅС‚Р°РјРё.

## Admin UI

- Publishable Leptos root page package: `crates/rustok-workflow/admin`.
- `rustok-module.toml [provides.admin_ui]` С‚РµРїРµСЂСЊ РѕР±СЉСЏРІР»СЏРµС‚ `rustok-workflow-admin`, `route_segment = "workflow"`, `nav_label = "Workflow"` Рё nested subpage `templates` С‡РµСЂРµР· `[[provides.admin_ui.pages]]`.
- `apps/admin` РјРѕРЅС‚РёСЂСѓРµС‚ СЌС‚РѕС‚ РїР°РєРµС‚ С‡РµСЂРµР· generic routes `/modules/workflow` Рё `/modules/workflow/templates`, РЅРµ Р·РЅР°СЏ Рѕ РјРѕРґСѓР»Рµ РїРѕ РёРјРµРЅРё РІ router РёР»Рё sidebar.
- Root page СѓР¶Рµ Р¶РёРІС‘С‚ РІ РјРѕРґСѓР»СЊРЅРѕРј crate Рё РїРѕРєСЂС‹РІР°РµС‚ overview/list, Р° templates С‚РµРїРµСЂСЊ РѕС‚РєСЂС‹РІР°СЋС‚СЃСЏ РєР°Рє module-owned nested route РїРѕРІРµСЂС… generic host contract.
- Legacy detail/edit flow РІСЃС‘ РµС‰С‘ РѕСЃС‚Р°С‘С‚СЃСЏ РЅР° РјР°СЂС€СЂСѓС‚Р°С… `/workflows/*` РІРЅСѓС‚СЂРё `apps/admin`; РЅРѕРІС‹Р№ nested contract РїРѕРєР° Р·Р°РєСЂС‹РІР°РµС‚ package-owned overview/templates СЃР»РѕР№, Р° РЅРµ РІРµСЃСЊ legacy workflow editor.

## РЎРІСЏР·Р°РЅРЅС‹Рµ РґРѕРєСѓРјРµРЅС‚С‹

- [CRATE_API](../CRATE_API.md)
- [Event flow contract](../../../docs/architecture/event-flow-contract.md)

## Transport adapters

- GraphQL Р°РґР°РїС‚РµСЂС‹ РјРѕРґСѓР»СЏ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-workflow/src/graphql/`.
- REST-РєРѕРЅС‚СЂРѕР»Р»РµСЂС‹ Рё webhook ingress С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-workflow/src/controllers/`.
- `apps/server` РґР»СЏ workflow Р±РѕР»СЊС€Рµ РЅРµ С…СЂР°РЅРёС‚ Р±РёР·РЅРµСЃ-Р»РѕРіРёРєСѓ transport-Р°РґР°РїС‚РµСЂРѕРІ Рё РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ С‚РѕР»СЊРєРѕ РєР°Рє composition root / shim-СЃР»РѕР№.

## Event contracts

РњРѕРґСѓР»СЊ РїСѓР±Р»РёРєСѓРµС‚ СЃРѕР±С‹С‚РёСЏ С‡РµСЂРµР· `emit_event`-С€Р°Рі. РљРѕРЅС‚СЂР°РєС‚ СЃРѕР±С‹С‚РёР№ РѕРїСЂРµРґРµР»СЏРµС‚СЃСЏ
РєРѕРЅС„РёРіСѓСЂР°С†РёРµР№ РєРѕРЅРєСЂРµС‚РЅРѕРіРѕ workflow вЂ” РЅРµ Р·Р°С€РёС‚ РІ РєРѕРґ РјРѕРґСѓР»СЏ.

РЎРёСЃС‚РµРјРЅС‹Рµ СЃРѕР±С‹С‚РёСЏ (backlog):
- `workflow.execution.started`
- `workflow.execution.completed`
- `workflow.execution.failed`

## РЎС‚Р°С‚СѓСЃ СЂРµР°Р»РёР·Р°С†РёРё

Р’СЃРµ С‡РµС‚С‹СЂРµ С„Р°Р·С‹ СЂРµР°Р»РёР·РѕРІР°РЅС‹:

- вњ… Phase 1 вЂ” Foundation: С‚Р°Р±Р»РёС†С‹, entities, `WorkflowService`, `WorkflowEngine`, event trigger, Р±Р°Р·РѕРІС‹Рµ С€Р°РіРё
- вњ… Phase 2 вЂ” Advanced Steps: `alloy_script`, `http`, `delay`, `notify`, cron trigger, manual trigger, error handling
- вњ… Phase 3 вЂ” Admin UI: РіСЂР°С„-СЂРµРґР°РєС‚РѕСЂ РІ Next.js, execution history, Leptos GraphQL API
- вњ… Phase 4 вЂ” Alloy Synergy: webhook trigger, РІРµСЂСЃРёРѕРЅРёСЂРѕРІР°РЅРёРµ, marketplace С€Р°Р±Р»РѕРЅС‹, Alloy-РіРµРЅРµСЂР°С†РёСЏ workflow

### Backlog

- Integration-С‚РµСЃС‚С‹ СЃ СЂРµР°Р»СЊРЅРѕР№ Р‘Р” (sqlite in-memory).
- РџРѕР»РЅР°СЏ СЂРµР°Р»РёР·Р°С†РёСЏ `alloy_script` С€Р°РіР° (СЃРµР№С‡Р°СЃ stub + `ScriptRunner` trait).
- РџРѕР»РЅР°СЏ СЂРµР°Р»РёР·Р°С†РёСЏ `notify` С€Р°РіР° (СЃРµР№С‡Р°СЃ stub + `NotificationSender` trait).
- DAG РІРјРµСЃС‚Рѕ Р»РёРЅРµР№РЅРѕР№ С†РµРїРѕС‡РєРё С€Р°РіРѕРІ.
- РЎРёСЃС‚РµРјРЅС‹Рµ СЃРѕР±С‹С‚РёСЏ `workflow.execution.*` РІ outbox.

