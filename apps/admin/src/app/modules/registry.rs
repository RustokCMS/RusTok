use std::cell::RefCell;

use leptos::prelude::AnyView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdminSlot {
    DashboardSection,
}

#[derive(Clone)]
pub struct AdminComponentRegistration {
    pub id: &'static str,
    pub slot: AdminSlot,
    pub order: usize,
    pub render: fn() -> AnyView,
}

thread_local! {
    static REGISTRY: RefCell<Vec<AdminComponentRegistration>> = const { RefCell::new(Vec::new()) };
}

pub fn register_component(component: AdminComponentRegistration) {
    REGISTRY.with(|registry| {
        registry.borrow_mut().push(component);
    });
}

pub fn components_for_slot(slot: AdminSlot) -> Vec<AdminComponentRegistration> {
    REGISTRY.with(|registry| {
        let components = registry
            .borrow()
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
    })
}
