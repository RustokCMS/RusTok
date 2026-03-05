use crate::shared::providers::locale::translate;
use crate::shared::ui::ui_button;
use leptos_router::components::A;

#[component]
pub fn not_found() -> impl IntoView {
    view! {
        <section class="flex min-h-screen items-center justify-center bg-background">
            <div class="grid gap-4 rounded-xl border border-border bg-card p-10 text-center shadow-md">
                <h1 class="text-5xl font-semibold text-card-foreground">"404"</h1>
                <p class="text-muted-foreground">{move || translate("app.not_found.text")}</p>
                <div class="flex justify-center">
                    <A href="/dashboard">
                        <ui_button>
                            {move || translate("app.not_found.back")}
                        </ui_button>
                    </A>
                </div>
            </div>
        </section>
    }
}
