use std::sync::{OnceLock, RwLock};

use leptos::prelude::View;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorefrontSlot {
    HomeAfterHero,
}

#[derive(Clone)]
pub struct StorefrontComponentRegistration {
    pub id: &'static str,
    pub slot: StorefrontSlot,
    pub order: usize,
    pub render: fn() -> View,
}

static REGISTRY: OnceLock<RwLock<Vec<StorefrontComponentRegistration>>> = OnceLock::new();

fn registry() -> &'static RwLock<Vec<StorefrontComponentRegistration>> {
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn register_component(component: StorefrontComponentRegistration) {
    let mut components = registry().write().expect("storefront module registry lock");
    components.push(component);
}

pub fn components_for_slot(slot: StorefrontSlot) -> Vec<StorefrontComponentRegistration> {
    let components = registry()
        .read()
        .expect("storefront module registry lock")
        .iter()
        .filter(|component| component.slot == slot)
        .cloned()
        .collect::<Vec<_>>();

    let mut sorted = components;
    sorted.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.id.cmp(right.id))
    });
    sorted
}
