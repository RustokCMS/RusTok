mod mutation;
mod query;
mod types;

use std::sync::Arc;

use alloy_scripting::{ScriptEngine, ScriptOrchestrator, SeaOrmStorage};
use async_graphql::{Context, FieldError, Result};
use rhai::Dynamic;

use crate::context::AuthContext;
use crate::graphql::errors::GraphQLError;

pub use mutation::AlloyMutation;
pub use query::AlloyQuery;
pub use types::*;

#[derive(Clone)]
pub struct AlloyState {
    pub engine: Arc<ScriptEngine>,
    pub storage: Arc<SeaOrmStorage>,
    pub orchestrator: Arc<ScriptOrchestrator<SeaOrmStorage>>,
}

impl AlloyState {
    pub fn new(
        engine: Arc<ScriptEngine>,
        storage: Arc<SeaOrmStorage>,
        orchestrator: Arc<ScriptOrchestrator<SeaOrmStorage>>,
    ) -> Self {
        Self {
            engine,
            storage,
            orchestrator,
        }
    }
}

pub(crate) fn require_admin(ctx: &Context<'_>) -> Result<AuthContext> {
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

    if !matches!(
        auth.role,
        rustok_core::UserRole::Admin | rustok_core::UserRole::SuperAdmin
    ) {
        return Err(<FieldError as GraphQLError>::permission_denied("Forbidden"));
    }

    Ok(auth.clone())
}

fn json_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s),
        serde_json::Value::Array(arr) => {
            let vec: Vec<Dynamic> = arr.into_iter().map(json_to_dynamic).collect();
            Dynamic::from(vec)
        }
        serde_json::Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (key, value) in map {
                rhai_map.insert(key.into(), json_to_dynamic(value));
            }
            Dynamic::from(rhai_map)
        }
    }
}

fn dynamic_to_json(value: Dynamic) -> serde_json::Value {
    if value.is::<()>() {
        return serde_json::Value::Null;
    }

    if let Ok(v) = value.clone().try_cast::<bool>() {
        return serde_json::Value::Bool(v);
    }

    if let Ok(v) = value.clone().try_cast::<i64>() {
        return serde_json::Value::Number(v.into());
    }

    if let Ok(v) = value.clone().try_cast::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(v) {
            return serde_json::Value::Number(num);
        }
    }

    if let Ok(v) = value.clone().try_cast::<String>() {
        return serde_json::Value::String(v);
    }

    if let Ok(v) = value.clone().try_cast::<Vec<Dynamic>>() {
        return serde_json::Value::Array(v.into_iter().map(dynamic_to_json).collect());
    }

    if let Ok(v) = value.try_cast::<rhai::Map>() {
        let mut json_map = serde_json::Map::new();
        for (key, value) in v {
            json_map.insert(key.to_string(), dynamic_to_json(value));
        }
        return serde_json::Value::Object(json_map);
    }

    serde_json::Value::Null
}
