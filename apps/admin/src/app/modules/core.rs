use leptos::prelude::*;

use super::{register_component, AdminComponentRegistration, AdminSlot};

pub fn register_components() {
    register_component(AdminComponentRegistration {
        id: "core-dashboard-module-status",
        slot: AdminSlot::DashboardSection,
        order: 10,
        render: module_status_card,
    });
}

fn module_status_card() -> AnyView {
    view! {
        <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
            <h4 class="text-lg font-semibold text-slate-900">"Modules ready"</h4>
            <p class="mt-2 text-sm text-slate-600">
                "Frontend packages can register dashboard widgets dynamically."
            </p>
            <div class="mt-4 flex flex-wrap gap-2 text-xs">
                <span class="rounded-full bg-indigo-50 px-3 py-1 font-semibold text-indigo-600">
                    "Leptos"
                </span>
                <span class="rounded-full bg-emerald-50 px-3 py-1 font-semibold text-emerald-600">
                    "Registry"
                </span>
                <span class="rounded-full bg-slate-100 px-3 py-1 font-semibold text-slate-600">
                    "Dynamic"
                </span>
            </div>
        </div>
    }
    .into_any()
}
