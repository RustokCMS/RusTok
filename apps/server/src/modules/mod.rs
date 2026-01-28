use rustok_core::ModuleRegistry;
use rustok_blog::BlogModule;
use rustok_commerce::CommerceModule;

pub fn build_registry() -> ModuleRegistry {
    ModuleRegistry::new()
        .register(CommerceModule)
        .register(BlogModule)
}
