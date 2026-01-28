use std::collections::HashMap;
use std::sync::Arc;

use crate::module::RusToKModule;

#[derive(Clone, Default)]
pub struct ModuleRegistry {
    modules: Arc<HashMap<String, Box<dyn RusToKModule>>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(HashMap::new()),
        }
    }

    pub fn register<M: RusToKModule + 'static>(mut self, module: M) -> Self {
        let mut modules = (*self.modules).clone();
        modules.insert(module.slug().to_string(), Box::new(module));
        self.modules = Arc::new(modules);
        self
    }

    pub fn get(&self, slug: &str) -> Option<&dyn RusToKModule> {
        self.modules.get(slug).map(|module| module.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn RusToKModule> {
        self.modules
            .values()
            .map(|module| module.as_ref())
            .collect()
    }

    pub fn contains(&self, slug: &str) -> bool {
        self.modules.contains_key(slug)
    }
}
