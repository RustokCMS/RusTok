use crate::entities::module::{BuildJob, InstalledModule, MarketplaceModule, ModuleInfo, ReleaseInfo};
use leptos::prelude::*;

pub fn modules_list(
    _admin_surface: String,
    modules: Vec<ModuleInfo>,
    _marketplace_modules: Vec<MarketplaceModule>,
    _installed_modules: Vec<InstalledModule>,
    _active_build: Option<BuildJob>,
    _active_release: Option<ReleaseInfo>,
    _build_history: Vec<BuildJob>,
) -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <For
                each=move || modules.clone()
                key=|m| m.module_slug.clone()
                children=move |m| {
                    view! {
                        <div class="rounded-xl border border-border bg-card p-4">
                            <h3 class="text-sm font-semibold">{m.name}</h3>
                            <p class="text-sm text-muted-foreground">{m.description}</p>
                        </div>
                    }
                }
            />
        </div>
    }
}
