mod core;
mod registry;

use std::cell::Cell;

pub use registry::{
    components_for_slot, register_component, AdminComponentRegistration, AdminSlot,
};

thread_local! {
    static INIT: Cell<bool> = Cell::new(false);
}

pub fn init_modules() {
    INIT.with(|flag| {
        if flag.get() {
            return;
        }
        flag.set(true);
        core::register_components();
    });
}
