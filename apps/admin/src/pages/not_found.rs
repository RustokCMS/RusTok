use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="flex min-h-screen items-center justify-center bg-background">
            <div class="grid gap-4 rounded-xl border border-border bg-card p-10 text-center shadow-md">
                <h1 class="text-5xl font-semibold text-card-foreground">"404"</h1>
                <p class="text-muted-foreground">"Страница не найдена."</p>
                <a
                    class="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90"
                    href="/dashboard"
                >
                    "Вернуться в дашборд"
                </a>
            </div>
        </section>
    }
}
