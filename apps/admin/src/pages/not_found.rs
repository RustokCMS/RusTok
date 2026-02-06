use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="flex min-h-screen items-center justify-center bg-slate-50">
            <div class="grid gap-3 rounded-3xl bg-white p-10 text-center shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                <h1 class="text-5xl font-semibold">"404"</h1>
                <p class="text-slate-600">"Страница не найдена."</p>
                <a
                    class="inline-flex items-center justify-center rounded-xl bg-blue-600 px-4 py-3 text-sm font-semibold text-white transition hover:bg-blue-700"
                    href="/dashboard"
                >
                    "Вернуться в дашборд"
                </a>
            </div>
        </section>
    }
}
