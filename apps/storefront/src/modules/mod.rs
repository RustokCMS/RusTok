mod core;
mod registry;

use std::sync::OnceLock;

pub use registry::{
    components_for_slot, register_component, StorefrontComponentRegistration, StorefrontSlot,
};

pub fn init_modules() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        core::register_components();
    });
}
