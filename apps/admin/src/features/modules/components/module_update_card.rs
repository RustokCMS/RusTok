use leptos::prelude::*;

use crate::entities::module::{InstalledModule, MarketplaceModule};

pub fn module_update_card(
    module: MarketplaceModule,
    _installed_module: InstalledModule,
    _platform_loading: Signal<bool>,
    _platform_busy: Signal<bool>,
    _on_inspect: Option<Callback<String>>,
    _on_upgrade: Callback<(String, String)>,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-border bg-card p-4">
            <h3 class="text-sm font-semibold">{module.name}</h3>
            <p class="text-sm text-muted-foreground">{module.latest_version}</p>
        </div>
    }
}
