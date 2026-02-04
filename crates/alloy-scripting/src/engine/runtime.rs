use parking_lot::RwLock;
use rhai::{Dynamic, Engine, EvalAltResult, RhaiNativeFunc, Variant, AST};
use std::collections::HashMap;
use std::sync::Arc;

use crate::context::ExecutionContext;
use crate::error::{ScriptError, ScriptResult};

use super::config::EngineConfig;

struct CompiledScript {
    ast: AST,
    name: String,
}

pub struct ScriptEngine {
    engine: Engine,
    config: EngineConfig,
    cache: RwLock<HashMap<String, Arc<CompiledScript>>>,
}

impl ScriptEngine {
    pub fn new(config: EngineConfig) -> Self {
        let mut engine = Engine::new();

        engine.set_max_operations(config.max_operations);
        engine.set_max_call_levels(config.max_call_depth);
        engine.set_max_string_size(config.max_string_size);
        engine.set_max_array_size(config.max_array_size);
        engine.set_max_map_size(config.max_array_size);

        engine.set_allow_looping(true);
        engine.set_allow_shadowing(true);
        engine.set_strict_variables(true);

        Self {
            engine,
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Register a native function with Rhai using the required trait bounds.
    pub fn register_fn<A, const N: usize, const X: bool, R, const F: bool>(
        &mut self,
        name: &str,
        func: impl RhaiNativeFunc<A, N, X, R, F> + Send + Sync + 'static,
    ) where
        R: Variant + Clone,
    {
        self.engine.register_fn(name, func);
    }

    pub fn register_type<T: Clone + Send + Sync + 'static>(&mut self, name: &str) {
        self.engine.register_type_with_name::<T>(name);
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn compile(&self, name: &str, source: &str) -> ScriptResult<Arc<CompiledScript>> {
        {
            let cache = self.cache.read();
            if let Some(compiled) = cache.get(name) {
                return Ok(Arc::clone(compiled));
            }
        }

        let ast = self
            .engine
            .compile(source)
            .map_err(|e| ScriptError::Compilation(e.to_string()))?;

        let compiled = Arc::new(CompiledScript {
            ast,
            name: name.to_string(),
        });

        let mut cache = self.cache.write();
        cache.insert(name.to_string(), Arc::clone(&compiled));

        Ok(compiled)
    }

    pub fn invalidate(&self, name: &str) {
        let mut cache = self.cache.write();
        cache.remove(name);
    }

    pub fn execute(
        &self,
        name: &str,
        source: &str,
        ctx: &ExecutionContext,
    ) -> ScriptResult<Dynamic> {
        let compiled = self.compile(name, source)?;
        self.execute_compiled(&compiled, ctx)
    }

    pub fn execute_compiled(
        &self,
        compiled: &CompiledScript,
        ctx: &ExecutionContext,
    ) -> ScriptResult<Dynamic> {
        let mut scope = ctx.to_scope();

        let result = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, &compiled.ast)
            .map_err(|e| Self::convert_error(*e, self.config.max_operations))?;

        Ok(result)
    }

    fn convert_error(err: EvalAltResult, op_limit: u64) -> ScriptError {
        match err {
            EvalAltResult::ErrorTerminated(reason, _) => ScriptError::Aborted(reason.to_string()),
            EvalAltResult::ErrorTooManyOperations(_) => {
                ScriptError::OperationLimit { limit: op_limit }
            }
            EvalAltResult::ErrorRuntime(msg, _) => {
                let msg_str = msg.to_string();
                if msg_str.starts_with("ABORT:") {
                    ScriptError::Aborted(msg_str.trim_start_matches("ABORT:").trim().to_string())
                } else {
                    ScriptError::Runtime(msg_str)
                }
            }
            other => ScriptError::Runtime(other.to_string()),
        }
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}
