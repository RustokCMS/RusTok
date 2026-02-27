use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use crate::error::{ScriptError, ScriptResult};
use crate::model::{EventType, HttpMethod, Script, ScriptId, ScriptStatus, ScriptTrigger};
use crate::storage::{ScriptPage, ScriptQuery, ScriptRegistry};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scripts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub trigger_type: String,
    pub trigger_config: Json,
    pub status: String,
    pub version: i32,
    pub run_as_system: bool,
    pub permissions: Json,
    pub author_id: Option<String>,
    pub error_count: i32,
    pub last_error_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct SeaOrmStorage {
    db: DatabaseConnection,
    tenant_id: Option<Uuid>,
}

impl SeaOrmStorage {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            tenant_id: None,
        }
    }

    pub fn with_tenant(db: DatabaseConnection, tenant_id: Uuid) -> Self {
        Self {
            db,
            tenant_id: Some(tenant_id),
        }
    }

    pub fn for_tenant(&self, tenant_id: Uuid) -> Self {
        Self {
            db: self.db.clone(),
            tenant_id: Some(tenant_id),
        }
    }

    fn trigger_to_parts(trigger: &ScriptTrigger) -> (String, serde_json::Value) {
        match trigger {
            ScriptTrigger::Event { entity_type, event } => (
                "event".to_string(),
                serde_json::json!({
                    "entity_type": entity_type,
                    "event": event.as_str(),
                }),
            ),
            ScriptTrigger::Cron { expression } => (
                "cron".to_string(),
                serde_json::json!({ "expression": expression }),
            ),
            ScriptTrigger::Manual => ("manual".to_string(), serde_json::json!({})),
            ScriptTrigger::Api { path, method } => (
                "api".to_string(),
                serde_json::json!({
                    "path": path,
                    "method": method.as_str(),
                }),
            ),
        }
    }

    fn trigger_from_parts(
        trigger_type: &str,
        trigger_config: &serde_json::Value,
    ) -> ScriptResult<ScriptTrigger> {
        match trigger_type {
            "event" => {
                let entity_type = trigger_config
                    .get("entity_type")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let event_str = trigger_config
                    .get("event")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                let event = EventType::parse(event_str)
                    .ok_or_else(|| ScriptError::InvalidTrigger(format!("event: {event_str}")))?;
                Ok(ScriptTrigger::Event { entity_type, event })
            }
            "cron" => {
                let expression = trigger_config
                    .get("expression")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                Ok(ScriptTrigger::Cron { expression })
            }
            "manual" => Ok(ScriptTrigger::Manual),
            "api" => {
                let path = trigger_config
                    .get("path")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let method_str = trigger_config
                    .get("method")
                    .and_then(|value| value.as_str())
                    .unwrap_or("GET");
                let method = HttpMethod::parse(method_str)
                    .ok_or_else(|| ScriptError::InvalidTrigger(format!("method: {method_str}")))?;
                Ok(ScriptTrigger::Api { path, method })
            }
            _ => Err(ScriptError::InvalidTrigger(trigger_type.to_string())),
        }
    }

    fn status_from_str(value: &str) -> ScriptResult<ScriptStatus> {
        match value {
            "draft" => Ok(ScriptStatus::Draft),
            "active" => Ok(ScriptStatus::Active),
            "paused" => Ok(ScriptStatus::Paused),
            "disabled" => Ok(ScriptStatus::Disabled),
            "archived" => Ok(ScriptStatus::Archived),
            _ => Err(ScriptError::InvalidStatus(value.to_string())),
        }
    }

    fn model_to_script(model: Model) -> ScriptResult<Script> {
        let trigger = Self::trigger_from_parts(&model.trigger_type, &model.trigger_config)?;
        let status = Self::status_from_str(&model.status)?;
        let permissions = model
            .permissions
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Script {
            id: model.id,
            tenant_id: model.tenant_id,
            name: model.name,
            description: model.description,
            code: model.code,
            trigger,
            status,
            version: model.version.max(1) as u32,
            run_as_system: model.run_as_system,
            permissions,
            author_id: model.author_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
            error_count: model.error_count.max(0) as u32,
            last_error_at: model.last_error_at,
        })
    }

    fn permissions_to_json(permissions: &[String]) -> serde_json::Value {
        serde_json::Value::Array(
            permissions
                .iter()
                .map(|value| serde_json::Value::String(value.clone()))
                .collect(),
        )
    }

    fn apply_query(
        select: sea_orm::Select<Entity>,
        query: ScriptQuery,
        tenant_id: Option<Uuid>,
    ) -> sea_orm::Select<Entity> {
        let select = match query {
            ScriptQuery::ById(id) => select.filter(Column::Id.eq(id)),
            ScriptQuery::ByName(name) => select.filter(Column::Name.eq(name)),
            ScriptQuery::ByEvent { entity_type, event } => select
                .filter(Column::TriggerType.eq("event"))
                .filter(Column::Status.eq(ScriptStatus::Active.as_str()))
                .filter(Expr::cust_with_values(
                    "trigger_config->>'entity_type' = $1",
                    [entity_type],
                ))
                .filter(Expr::cust_with_values(
                    "trigger_config->>'event' = $1",
                    [event.as_str()],
                )),
            ScriptQuery::ByApiPath(path) => select
                .filter(Column::TriggerType.eq("api"))
                .filter(Column::Status.eq(ScriptStatus::Active.as_str()))
                .filter(Expr::cust_with_values(
                    "trigger_config->>'path' = $1",
                    [path],
                )),
            ScriptQuery::Scheduled => select
                .filter(Column::TriggerType.eq("cron"))
                .filter(Column::Status.eq(ScriptStatus::Active.as_str())),
            ScriptQuery::ByStatus(status) => select.filter(Column::Status.eq(status.as_str())),
            ScriptQuery::All => select,
        };

        if let Some(tid) = tenant_id {
            select.filter(Column::TenantId.eq(tid))
        } else {
            select
        }
    }
}

#[async_trait::async_trait]
impl ScriptRegistry for SeaOrmStorage {
    async fn find(&self, query: ScriptQuery) -> ScriptResult<Vec<Script>> {
        let select = Self::apply_query(Entity::find(), query, self.tenant_id);
        let models = select
            .order_by_asc(Column::Name)
            .all(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        models.into_iter().map(Self::model_to_script).collect()
    }

    async fn find_paginated(
        &self,
        query: ScriptQuery,
        offset: u64,
        limit: u64,
    ) -> ScriptResult<ScriptPage> {
        let total = Self::apply_query(Entity::find(), query.clone(), self.tenant_id)
            .order_by_asc(Column::Name)
            .count(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        let models = Self::apply_query(Entity::find(), query, self.tenant_id)
            .order_by_asc(Column::Name)
            .offset(offset)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        let items: ScriptResult<Vec<Script>> =
            models.into_iter().map(Self::model_to_script).collect();

        Ok(ScriptPage {
            items: items?,
            total,
        })
    }

    async fn get(&self, id: ScriptId) -> ScriptResult<Script> {
        let model = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?
            .ok_or_else(|| ScriptError::NotFound {
                name: id.to_string(),
            })?;

        Self::model_to_script(model)
    }

    async fn get_by_name(&self, name: &str) -> ScriptResult<Script> {
        let mut query = Entity::find().filter(Column::Name.eq(name));
        if let Some(tid) = self.tenant_id {
            query = query.filter(Column::TenantId.eq(tid));
        }
        let model = query
            .one(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?
            .ok_or_else(|| ScriptError::NotFound {
                name: name.to_string(),
            })?;

        Self::model_to_script(model)
    }

    async fn save(&self, mut script: Script) -> ScriptResult<Script> {
        let now = Utc::now();
        let (trigger_type, trigger_config) = Self::trigger_to_parts(&script.trigger);
        let permissions_json = Self::permissions_to_json(&script.permissions);

        if let Some(existing) = Entity::find_by_id(script.id)
            .one(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?
        {
            script.version = script.version.max(1) + 1;
            script.updated_at = now;

            let mut active: ActiveModel = existing.into();
            active.name = ActiveValue::Set(script.name.clone());
            active.description = ActiveValue::Set(script.description.clone());
            active.code = ActiveValue::Set(script.code.clone());
            active.trigger_type = ActiveValue::Set(trigger_type);
            active.trigger_config = ActiveValue::Set(trigger_config);
            active.status = ActiveValue::Set(script.status.as_str().to_string());
            active.version = ActiveValue::Set(script.version as i32);
            active.run_as_system = ActiveValue::Set(script.run_as_system);
            active.permissions = ActiveValue::Set(permissions_json);
            active.author_id = ActiveValue::Set(script.author_id.clone());
            active.error_count = ActiveValue::Set(script.error_count as i32);
            active.last_error_at = ActiveValue::Set(script.last_error_at);
            active.updated_at = ActiveValue::Set(script.updated_at);

            let updated = active
                .update(&self.db)
                .await
                .map_err(|err| ScriptError::Storage(err.to_string()))?;
            return Self::model_to_script(updated);
        }

        script.version = script.version.max(1);
        script.created_at = now;
        script.updated_at = now;

        let model = ActiveModel {
            id: ActiveValue::Set(script.id),
            tenant_id: ActiveValue::Set(script.tenant_id),
            name: ActiveValue::Set(script.name.clone()),
            description: ActiveValue::Set(script.description.clone()),
            code: ActiveValue::Set(script.code.clone()),
            trigger_type: ActiveValue::Set(trigger_type),
            trigger_config: ActiveValue::Set(trigger_config),
            status: ActiveValue::Set(script.status.as_str().to_string()),
            version: ActiveValue::Set(script.version as i32),
            run_as_system: ActiveValue::Set(script.run_as_system),
            permissions: ActiveValue::Set(permissions_json),
            author_id: ActiveValue::Set(script.author_id.clone()),
            error_count: ActiveValue::Set(script.error_count as i32),
            last_error_at: ActiveValue::Set(script.last_error_at),
            created_at: ActiveValue::Set(script.created_at),
            updated_at: ActiveValue::Set(script.updated_at),
        };

        let inserted = model
            .insert(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        Self::model_to_script(inserted)
    }

    async fn delete(&self, id: ScriptId) -> ScriptResult<()> {
        let result = Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        if result.rows_affected == 0 {
            return Err(ScriptError::NotFound {
                name: id.to_string(),
            });
        }

        Ok(())
    }

    async fn set_status(&self, id: ScriptId, status: ScriptStatus) -> ScriptResult<()> {
        let result = Entity::update_many()
            .col_expr(Column::Status, Expr::value(status.as_str()))
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        if result.rows_affected == 0 {
            return Err(ScriptError::NotFound {
                name: id.to_string(),
            });
        }

        Ok(())
    }

    async fn record_error(&self, id: ScriptId) -> ScriptResult<bool> {
        let model = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?
            .ok_or_else(|| ScriptError::NotFound {
                name: id.to_string(),
            })?;

        let mut script = Self::model_to_script(model)?;
        let should_disable = script.register_error();
        let status = if should_disable {
            ScriptStatus::Disabled
        } else {
            script.status
        };

        Entity::update_many()
            .col_expr(Column::ErrorCount, Expr::value(script.error_count as i32))
            .col_expr(Column::LastErrorAt, Expr::value(script.last_error_at))
            .col_expr(Column::Status, Expr::value(status.as_str()))
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        Ok(should_disable)
    }

    async fn reset_errors(&self, id: ScriptId) -> ScriptResult<()> {
        let result = Entity::update_many()
            .col_expr(Column::ErrorCount, Expr::value(0))
            .col_expr(
                Column::LastErrorAt,
                Expr::value(Option::<DateTime<Utc>>::None),
            )
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|err| ScriptError::Storage(err.to_string()))?;

        if result.rows_affected == 0 {
            return Err(ScriptError::NotFound {
                name: id.to_string(),
            });
        }

        Ok(())
    }
}
