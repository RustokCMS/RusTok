use leptos::prelude::*;
use leptos_auth::hooks::use_current_user;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::{t_string, use_i18n};

#[component]
pub fn Sidebar() -> impl IntoView {
    let i18n = use_i18n();
    let current_user = use_current_user();

    view! {
        <aside class="w-64 bg-sidebar border-r border-sidebar-border h-screen flex flex-col shrink-0">
            <div class="p-5 border-b border-sidebar-border">
                <A href="/dashboard" attr:class="flex items-center gap-2">
                    <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
                        <span class="text-primary-foreground font-bold text-sm">"R"</span>
                    </div>
                    <span class="text-sm font-semibold text-sidebar-foreground">{t_string!(i18n, app.brand.title)}</span>
                </A>
            </div>

            <nav class="flex-1 px-3 py-4 overflow-y-auto">
                <NavGroupLabel label=move || t_string!(i18n, app.nav.group.overview).to_string() />
                <NavLink href="/dashboard" icon="grid">
                    {move || t_string!(i18n, app.nav.dashboard)}
                </NavLink>

                {move || {
                    let role = current_user.get()
                        .map(|u| u.role.to_uppercase())
                        .unwrap_or_default();
                    let is_admin = role == "ADMIN" || role == "SUPER_ADMIN";
                    if is_admin {
                        view! {
                            <div class="pt-3">
                                <NavGroupLabel label=move || t_string!(i18n, app.nav.group.management).to_string() />
                                <NavLink href="/users" icon="users">
                                    {move || t_string!(i18n, app.nav.users)}
                                </NavLink>
                                <NavLink href="/modules" icon="package">
                                    {move || t_string!(i18n, app.nav.modules)}
                                </NavLink>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <div class="pt-3">
                    <NavGroupLabel label=move || t_string!(i18n, app.nav.group.account).to_string() />
                    <NavLink href="/profile" icon="user">
                        {move || t_string!(i18n, app.nav.profile)}
                    </NavLink>
                    <NavLink href="/security" icon="lock">
                        {move || t_string!(i18n, app.nav.security)}
                    </NavLink>
                </div>
            </nav>

            <div class="border-t border-border p-4">
                <div class="flex items-center gap-3 px-2">
                    <div class="flex h-9 w-9 items-center justify-center rounded-full bg-primary/10 text-sm font-bold text-primary">
                        {move || current_user.get().and_then(|u| u.name.as_ref().and_then(|n| n.chars().next())).unwrap_or('?')}
                    </div>
                    <div class="grid flex-1 min-w-0 text-left text-sm leading-tight">
                        <span class="truncate font-semibold text-sidebar-foreground text-xs">
                            {move || current_user.get().and_then(|u| u.name.clone()).unwrap_or_else(|| t_string!(i18n, app.menu.defaultUser).to_string())}
                        </span>
                        <span class="truncate text-xs text-sidebar-foreground/60">
                            {move || current_user.get().map(|u| u.email.clone()).unwrap_or_default()}
                        </span>
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
