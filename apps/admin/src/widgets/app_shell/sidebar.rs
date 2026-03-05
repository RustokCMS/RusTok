use leptos::prelude::*;
use leptos_auth::hooks::use_current_user;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::shared::i18n::translate;

#[component]
pub fn sidebar() -> impl IntoView {
    let current_user = use_current_user();

    view! {
        <aside class="w-64 bg-sidebar border-r border-sidebar-border h-screen flex flex-col shrink-0">
            <div class="p-5 border-b border-sidebar-border">
                <A href="/dashboard" attr:class="flex items-center gap-2">
                    <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
                        <span class="text-primary-foreground font-bold text-sm">"R"</span>
                    </div>
                    <span class="text-sm font-semibold text-sidebar-foreground">{translate("app.brand.title")}</span>
                </A>
            </div>

            <nav class="flex-1 space-y-6 overflow-y-auto px-4 py-6 custom-scrollbar">
                <div class="space-y-1">
                    {nav_group_label(translate("app.nav.main"))}
                    {nav_link("/", "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6", translate("app.nav.dashboard"))}
                    {nav_link("/users", "M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M9 7a4 4 0 100-8 4 4 0 000 8zm3 5h9a2 2 0 012 2v5a2 2 0 01-2 2h-5", translate("app.nav.users"))}
                    {nav_link("/modules", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10", translate("app.nav.modules"))}
                </div>

                <div class="space-y-1">
                    {nav_group_label(translate("app.nav.account"))}
                    {nav_link("/profile", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z", translate("app.nav.profile"))}
                    {nav_link("/security", "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z", translate("app.nav.security"))}
                </div>
            </nav>

            <div class="border-t border-border p-4">
                <div class="flex items-center gap-3 px-2">
                    <div class="flex h-9 w-9 items-center justify-center rounded-full bg-primary/10 text-sm font-bold text-primary">
                        {move || current_user.get().and_then(|u| u.name.as_ref().and_then(|n| n.chars().next())).unwrap_or('?')}
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-medium text-foreground">
                            {move || current_user.get().and_then(|u| u.name).unwrap_or_else(|| translate("app.nav.guest").to_string())}
                        </p>
                        <p class="truncate text-xs text-muted-foreground">
                            {move || current_user.get().map(|u| u.email).unwrap_or_default()}
                        </p>
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn nav_group_label(label: String) -> impl IntoView {
    view! {
        <p class="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">
            {label}
        </p>
    }
}

#[component]
fn nav_link(href: &'static str, icon: &'static str, label: String) -> impl IntoView {
    let location = use_location();
    let is_active = move || {
        let path = location.pathname.get();
        if href == "/" {
            path == "/" || path == "/dashboard"
        } else {
            path.starts_with(href)
        }
    };

    view! {
        <A
            href=href
            class=move || format!(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-all hover:bg-accent hover:text-accent-foreground {}",
                if is_active() { "bg-accent text-accent-foreground shadow-sm" } else { "text-muted-foreground" }
            )
        >
            {nav_icon(icon)}
            {label}
        </A>
    }
}

#[component]
fn nav_icon(d: &'static str) -> impl IntoView {
    view! {
        <svg class="h-4 w-4 shrink-0 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d=d />
        </svg>
    }
}
