use leptos::prelude::*;
use leptos_router::components::A;

use crate::shared::ui::Button;
use crate::{t_string, use_i18n};

#[component]
pub fn NotFound() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <section class="flex min-h-screen items-center justify-center bg-background">
            <div class="grid gap-4 rounded-xl border border-border bg-card p-10 text-center shadow-md">
                <h1 class="text-5xl font-semibold text-card-foreground">"404"</h1>
                <p class="text-muted-foreground">{move || t_string!(i18n, app.notFound.text)}</p>
                <div class="flex justify-center">
                    <A href="/dashboard">
                        <Button on_click=move |_| {}>
                            {move || t_string!(i18n, app.notFound.back)}
                        </Button>
                    </A>
                </div>
            </div>
        </section>
    }
}
