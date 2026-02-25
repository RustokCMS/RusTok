use leptos::prelude::*;
use leptos_router::components::Outlet;

use super::header::Header;
use super::sidebar::Sidebar;

#[component]
pub fn AppLayout() -> impl IntoView {
    view! {
        <div class="flex h-screen bg-slate-50 overflow-hidden">
            <Sidebar />
            <div class="flex flex-1 flex-col overflow-hidden">
                <Header />
                <main class="flex-1 overflow-y-auto">
                    <Outlet />
                </main>
            </div>
        </div>
    }
}
