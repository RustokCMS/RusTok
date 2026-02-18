use leptos::prelude::*;

use super::{register_component, StorefrontComponentRegistration, StorefrontSlot};

pub fn register_components() {
    register_component(StorefrontComponentRegistration {
        id: "storefront-module-spotlight",
        slot: StorefrontSlot::HomeAfterHero,
        order: 10,
        render: module_spotlight,
    });
}

fn module_spotlight() -> AnyView {
    view! {
        <section class="container-app">
            <div class="rounded-3xl bg-base-100 p-6 shadow">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                    <div>
                        <h3 class="text-2xl font-bold">"Composable storefront modules"</h3>
                        <p class="mt-2 text-sm opacity-70">
                            "Ship curated sections from optional packages without touching core."
                        </p>
                    </div>
                    <div class="flex flex-wrap gap-2 text-xs">
                        <span class="badge badge-primary">"Leptos"</span>
                        <span class="badge badge-secondary">"Registry"</span>
                        <span class="badge badge-outline">"Extensions"</span>
                    </div>
                </div>
            </div>
        </section>
    }
    .into_any()
}
