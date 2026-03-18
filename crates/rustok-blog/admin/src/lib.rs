use leptos::*;
use leptos_router::*;

#[component]
pub fn BlogAdmin() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold">"Blog Administration"</h1>
            <p>"Management of blog posts and categories."</p>
        </div>
    }
}
