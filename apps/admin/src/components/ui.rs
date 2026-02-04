use leptos::prelude::*;

use crate::providers::locale::{use_locale, Locale};

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
    #[prop(default = MaybeSignal::Static(String::new()), into)] label: MaybeSignal<String>,
) -> impl IntoView {
    view! {
        <div class="input-group">
            {move || {
                let label_value = label.get();
                (!label_value.is_empty()).then(|| view! { <label>{label_value}</label> })
            }}
            <input
                type=type_
                placeholder=placeholder
                prop:value=value
                on:input=move |ev| set_value.set(event_target_value(&ev))
            />
        </div>
    }
}

#[component]
pub fn LanguageToggle() -> impl IntoView {
    let locale = use_locale();

    let set_locale = move |value: Locale| {
        locale.set_locale.set(value);
    };

    view! {
        <div class="locale-toggle">
            <button
                type="button"
                class="ghost-button"
                class:active=move || locale.locale.get() == Locale::Ru
                on:click=move |_| set_locale(Locale::Ru)
            >
                "RU"
            </button>
            <button
                type="button"
                class="ghost-button"
                class:active=move || locale.locale.get() == Locale::En
                on:click=move |_| set_locale(Locale::En)
            >
                "EN"
            </button>
        </div>
    }
}
