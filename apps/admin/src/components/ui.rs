use leptos::prelude::*;
use leptos::web_sys;

use crate::providers::locale::{use_locale, Locale};

#[component]
pub fn Button(
    #[prop(into)] on_click: Callback<web_sys::MouseEvent>,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional, into)] class: String,
    #[prop(default = Signal::derive(|| false))] disabled: Signal<bool>,
) -> impl IntoView {
    let base_class = "inline-flex items-center justify-center rounded-xl bg-blue-600 px-4 py-3 text-sm font-semibold text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60";
    let merged_class = move || {
        if class.is_empty() {
            base_class.to_string()
        } else {
            format!("{base_class} {class}")
        }
    };

    view! {
        <button
            class=merged_class
            on:click=move |ev| on_click.run(ev)
            disabled=move || disabled.get()
        >
            {children.map(|c| c())}
        </button>
    }
}

#[component]
pub fn Input(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
    #[prop(into)] placeholder: TextProp,
    #[prop(default = "text")] type_: &'static str,
    #[prop(default = String::new().into(), into)] label: TextProp,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            {move || {
                let label_value = label.get();
                (!label_value.is_empty()).then(|| {
                    view! { <label class="text-sm text-slate-600">{label_value}</label> }
                })
            }}
            <input
                type=type_
                placeholder=placeholder
                prop:value=value
                on:input=move |ev| set_value.set(event_target_value(&ev))
                class="rounded-xl border border-slate-200 px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
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
        <div class="flex gap-2">
            <button
                type="button"
                class=move || {
                    let is_active = locale.locale.get() == Locale::Ru;
                    if is_active {
                        "rounded-lg border border-blue-600 bg-blue-600 px-3 py-1 text-sm font-semibold text-white"
                    } else {
                        "rounded-lg border border-slate-200 px-3 py-1 text-sm font-semibold text-blue-600"
                    }
                }
                on:click=move |_| set_locale(Locale::Ru)
            >
                "RU"
            </button>
            <button
                type="button"
                class=move || {
                    let is_active = locale.locale.get() == Locale::En;
                    if is_active {
                        "rounded-lg border border-blue-600 bg-blue-600 px-3 py-1 text-sm font-semibold text-white"
                    } else {
                        "rounded-lg border border-slate-200 px-3 py-1 text-sm font-semibold text-blue-600"
                    }
                }
                on:click=move |_| set_locale(Locale::En)
            >
                "EN"
            </button>
        </div>
    }
}
