use leptos::children::Children;
use leptos::prelude::*;

#[component]
pub fn Card(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!(
            "rounded-xl border bg-card text-card-foreground shadow {}",
            class
        )>
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("flex flex-col space-y-1.5 p-6 {}", class)>
            {children()}
        </div>
    }
}

#[component]
pub fn CardContent(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("p-6 pt-0 {}", class)>
            {children()}
        </div>
    }
}

#[component]
pub fn CardFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    view! {
        <div class=format!("flex items-center p-6 pt-0 {}", class)>
            {children()}
        </div>
    }
}
