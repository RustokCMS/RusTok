use leptos::prelude::*;

use crate::entities::module::{MarketplaceModule, ModuleInfo};

pub fn module_card(
    module: ModuleInfo,
    _catalog_module: Option<MarketplaceModule>,
    _tenant_loading: Signal<bool>,
    _platform_loading: Signal<bool>,
    _platform_installed: Signal<bool>,
    _platform_busy: Signal<bool>,
    _platform_version: Signal<Option<String>>,
    _recommended_version: Signal<Option<String>>,
    _on_toggle: Option<Callback<(String, bool)>>,
    _on_install: Option<Callback<(String, String)>>,
    _on_inspect: Option<Callback<String>>,
    _on_uninstall: Option<Callback<String>>,
) -> impl IntoView {
    let name = module.name;
    let description = module.description;
    view! {
        <div class="rounded-xl border border-border bg-card p-4">
            <h3 class="text-sm font-semibold">{name}</h3>
            <p class="text-sm text-muted-foreground">{description}</p>
        </div>
    }
}
