use leptos::*;

#[component]
pub fn Button(
    #[prop(into)] on_click: Callback<web_sys::MouseEvent>,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional, into)] class: String,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    view! {
        <button
            class=format!("primary-button {}", class)
            on:click=move |ev| on_click.call(ev)
            disabled=disabled
        >
            {children.map(|c| c())}
        </button>
    }
}

#[component]
pub fn Input(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
    #[prop(into)] placeholder: String,
    #[prop(default = "text")] type_: &'static str,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    view! {
        <div class="input-group">
            {move || (!label.is_empty()).then(|| view! { <label>{label.clone()}</label> })}
            <input
                type=type_
                placeholder=placeholder
                prop:value=value
                on:input=move |ev| set_value.set(event_target_value(&ev))
            />
        </div>
    }
}
