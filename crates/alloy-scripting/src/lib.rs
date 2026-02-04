pub mod bridge;
pub mod context;
pub mod engine;
pub mod error;
pub mod model;
pub mod storage;

pub use bridge::Bridge;
pub use context::{ExecutionContext, ExecutionPhase};
pub use engine::{EngineConfig, ScriptEngine};
pub use error::{ScriptError, ScriptResult};
pub use model::{
    register_entity_proxy, EntityProxy, EventType, HttpMethod, Script, ScriptId, ScriptStatus,
    ScriptTrigger,
};
pub use storage::{InMemoryStorage, ScriptQuery, ScriptRegistry};

pub fn create_default_engine() -> ScriptEngine {
    let config = EngineConfig::default();
    let mut engine = ScriptEngine::new(config);

    bridge::register_utils(engine.engine_mut());

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Map;

    #[test]
    fn test_simple_script() {
        let engine = create_default_engine();
        let ctx = ExecutionContext::new(ExecutionPhase::Manual);

        let result = engine
            .execute(
                "test_hello",
                r#"
                log("Hello from script!");
                let x = 10 + 20;
                x
            "#,
                &ctx,
            )
            .unwrap();

        assert_eq!(result.as_int().unwrap(), 30);
    }

    #[test]
    fn test_abort() {
        let engine = create_default_engine();
        let ctx = ExecutionContext::new(ExecutionPhase::Before);

        let result = engine.execute("test_abort", r#"abort("Deal amount too small")"#, &ctx);

        assert!(matches!(result, Err(ScriptError::Aborted(_))));
    }

    #[test]
    fn test_entity_access() {
        let engine = create_default_engine();

        let mut deal = Map::new();
        deal.insert("amount".into(), (50000_i64).into());
        deal.insert("name".into(), "Big Deal".into());

        let ctx = ExecutionContext::new(ExecutionPhase::Before).with_entity(deal.into());

        let result = engine
            .execute(
                "test_entity",
                r#"
                if entity.amount > 10000 {
                    log("Big deal detected: " + entity.name);
                }
                entity.amount
            "#,
                &ctx,
            )
            .unwrap();

        assert_eq!(result.as_int().unwrap(), 50000);
    }

    #[test]
    fn test_operation_limit() {
        let config = EngineConfig {
            max_operations: 100,
            ..Default::default()
        };
        let mut engine = ScriptEngine::new(config);
        bridge::register_utils(engine.engine_mut());

        let ctx = ExecutionContext::new(ExecutionPhase::Manual);

        let result = engine.execute(
            "test_infinite",
            r#"
                let i = 0;
                while i < 1000000 {
                    i += 1;
                }
                i
            "#,
            &ctx,
        );

        assert!(matches!(result, Err(ScriptError::OperationLimit { .. })));
    }
}
