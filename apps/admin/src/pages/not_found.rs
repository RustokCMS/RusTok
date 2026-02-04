use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="not-found">
            <div class="not-found-card">
                <h1>"404"</h1>
                <p>"Страница не найдена."</p>
                <a class="primary-button" href="/dashboard">"Вернуться в дашборд"</a>
            </div>
        </section>
    }
}
