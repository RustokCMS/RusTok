# rustok-workflow - Public API

## Module registration

```rust
use rustok_workflow::WorkflowModule;

// In apps/server module registry:
registry.register(Box::new(WorkflowModule));
```

`WorkflowModule` implements `RusToKModule`:
- `slug()` -> `"workflow"`
- `name()` -> `"Workflow"`
- `kind()` -> `ModuleKind::Optional`
- `dependencies()` -> `[]`
- `migrations()` -> `WorkflowsMigration`, `WorkflowPhase4Migration`
- `permissions()` -> `Workflows::{Create,Read,Update,Delete,List,Execute,Manage}`, `WorkflowExecutions::{Read,List}`

## Public modules

`controllers`, `dto`, `entities`, `error`, `graphql`, `services`, `steps`, `templates`.

## Services

### `WorkflowService`

```rust
pub struct WorkflowService { /* db: DatabaseConnection */ }

impl WorkflowService {
    pub fn new(db: DatabaseConnection) -> Self;

    // Workflows
    pub async fn create(&self, tenant_id: Uuid, actor_id: Option<Uuid>, input: CreateWorkflowInput) -> WorkflowResult<Uuid>;
    pub async fn get(&self, tenant_id: Uuid, id: Uuid) -> WorkflowResult<WorkflowResponse>;
    pub async fn list(&self, tenant_id: Uuid) -> WorkflowResult<Vec<WorkflowSummary>>;
    pub async fn update(&self, tenant_id: Uuid, id: Uuid, actor_id: Option<Uuid>, input: UpdateWorkflowInput) -> WorkflowResult<()>;
    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> WorkflowResult<()>;

    // Steps
    pub async fn add_step(&self, tenant_id: Uuid, workflow_id: Uuid, input: CreateWorkflowStepInput) -> WorkflowResult<Uuid>;
    pub async fn update_step(&self, tenant_id: Uuid, workflow_id: Uuid, step_id: Uuid, input: UpdateWorkflowStepInput) -> WorkflowResult<()>;
    pub async fn delete_step(&self, tenant_id: Uuid, workflow_id: Uuid, step_id: Uuid) -> WorkflowResult<()>;

    // Executions
    pub async fn trigger_manual(&self, tenant_id: Uuid, workflow_id: Uuid, actor_id: Option<Uuid>, payload: Value, force: bool) -> WorkflowResult<Uuid>;
    pub async fn list_executions(&self, tenant_id: Uuid, workflow_id: Uuid) -> WorkflowResult<Vec<WorkflowExecutionResponse>>;
    pub async fn get_execution(&self, tenant_id: Uuid, execution_id: Uuid) -> WorkflowResult<WorkflowExecutionResponse>;
    pub async fn trigger_by_webhook(&self, tenant_id: Uuid, webhook_slug: &str, payload: Value) -> WorkflowResult<Vec<Uuid>>;
    pub async fn list_versions(&self, tenant_id: Uuid, workflow_id: Uuid) -> WorkflowResult<Vec<WorkflowVersionSummary>>;
    pub async fn get_version(&self, tenant_id: Uuid, workflow_id: Uuid, version: i32) -> WorkflowResult<WorkflowVersionDetail>;
    pub async fn restore_version(&self, tenant_id: Uuid, workflow_id: Uuid, version: i32, actor_id: Option<Uuid>) -> WorkflowResult<()>;
    pub async fn create_from_template(&self, tenant_id: Uuid, actor_id: Option<Uuid>, template_id: &str, name: String) -> WorkflowResult<Uuid>;
}
```

### `WorkflowEngine`

```rust
pub struct WorkflowEngine { /* db, step registry */ }

impl WorkflowEngine {
    pub fn new(db: DatabaseConnection) -> Self;
    pub fn with_step(mut self, step_type: impl Into<String>, step: Arc<dyn WorkflowStep>) -> Self;
    pub async fn execute(&self, workflow_id: Uuid, trigger_event_id: Uuid, context: Value) -> WorkflowResult<Uuid>;
}
```

### `WorkflowTriggerHandler`

```rust
pub struct WorkflowTriggerHandler { /* db, engine */ }

impl WorkflowTriggerHandler {
    pub fn new(db: DatabaseConnection, engine: Arc<WorkflowEngine>) -> Self;
    // Implements EventHandler - call from EventBus subscriber loop
    pub async fn handle(&self, event: &EventEnvelope) -> WorkflowResult<()>;
}
```

### `WorkflowCronScheduler`

```rust
pub struct WorkflowCronScheduler { /* db, engine */ }

impl WorkflowCronScheduler {
    pub fn new(db: DatabaseConnection, engine: Arc<WorkflowEngine>) -> Self;
    pub async fn tick(&self) -> WorkflowResult<()>;
}
```

## Step extension trait

```rust
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, config: &Value, ctx: &mut StepContext) -> WorkflowResult<()>;
}
```

Modules can register custom step types via `WorkflowEngine::with_step(...)`.

## Templates

```rust
pub static BUILTIN_TEMPLATES: &[WorkflowTemplate];

pub struct WorkflowTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    // ...
}
```

## Transport entry points

- `graphql::WorkflowQuery`
- `graphql::WorkflowMutation`
- `controllers::routes()`
- `controllers::webhook_routes()`

## Error type

```rust
pub enum WorkflowError {
    NotFound(Uuid),
    Unauthorized,
    InvalidConfig(String),
    StepFailed { step_id: Uuid, reason: String },
    Database(DbErr),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
```

## DTOs

Key request/response types (all in `rustok_workflow::dto`):

- `CreateWorkflowInput` / `UpdateWorkflowInput` / `WorkflowResponse` / `WorkflowSummary`
- `CreateWorkflowStepInput` / `UpdateWorkflowStepInput` / `WorkflowStepResponse`
- `WorkflowExecutionResponse` / `WorkflowStepExecutionResponse`
- `WorkflowVersionSummary` / `WorkflowVersionDetail`
