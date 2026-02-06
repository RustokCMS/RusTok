use std::collections::HashMap;
use std::sync::Arc;

use crate::migrations::ModuleMigration;
use crate::module::{EventListener, RusToKModule};

#[derive(Clone, Default)]
pub struct ModuleRegistry {
    modules: Arc<HashMap<String, Arc<dyn RusToKModule>>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(HashMap::new()),
        }
    }

    pub fn register<M: RusToKModule + 'static>(mut self, module: M) -> Self {
        let modules = Arc::make_mut(&mut self.modules);
        modules.insert(module.slug().to_string(), Arc::new(module));
        self
    }

    pub fn get(&self, slug: &str) -> Option<&dyn RusToKModule> {
        self.modules.get(slug).map(|module| module.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn RusToKModule> {
        let mut modules: Vec<&dyn RusToKModule> = self
            .modules
            .values()
            .map(|module| module.as_ref())
            .collect();
        modules.sort_by_key(|module| module.slug());
        modules
    }

    /// Returns an iterator over all registered modules
    pub fn modules(&self) -> impl Iterator<Item = &Arc<dyn RusToKModule>> {
        self.modules.values()
    }

    pub fn migrations(&self) -> Vec<ModuleMigration> {
        self.list()
            .into_iter()
            .map(|module| ModuleMigration {
                module_slug: module.slug(),
                migrations: module.migrations(),
            })
            .collect()
    }

    pub fn event_listeners(&self) -> Vec<Box<dyn EventListener>> {
        self.list()
            .into_iter()
            .flat_map(|module| module.event_listeners())
            .collect()
    }

    pub fn contains(&self, slug: &str) -> bool {
        self.modules.contains_key(slug)
    }
}
