// Header Component
use leptos::prelude::*;

use crate::components::features::auth::user_menu::UserMenu;
use crate::components::ui::LanguageToggle;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="h-14 bg-white border-b border-slate-200 flex items-center justify-between px-6 shrink-0">
            // Left: breadcrumb placeholder
            <div class="flex items-center gap-2 text-sm text-slate-500">
                <span class="font-medium text-slate-900">"RusTok"</span>
                <span>"/"</span>
                <span>"Admin"</span>
            </div>

            // Right: language toggle + user menu
            <div class="flex items-center gap-3">
                <LanguageToggle />
                <UserMenu />
            </div>
        </header>
    }
}
