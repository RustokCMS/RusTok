use leptos::prelude::*;

use crate::types::Size;

#[component]
pub fn Input(
    #[prop(default = "text")] r#type: &'static str,
    #[prop(default = Size::Md)] size: Size,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] invalid: bool,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] value: Option<ReadSignal<String>>,
    #[prop(optional)] set_value: Option<WriteSignal<String>>,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] name: String,
) -> impl IntoView {
    let size_cls = match size {
        Size::Sm => "h-8 text-xs px-2",
        Size::Md => "h-9 text-sm px-3",
        Size::Lg | Size::Icon => "h-10 text-base px-4",
    };

    let state_cls = if invalid {
        "border-[hsl(var(--iu-danger))] focus-visible:ring-[hsl(var(--iu-danger))]"
    } else {
        "border-[hsl(var(--iu-border))] focus-visible:ring-[hsl(var(--iu-primary))]"
    };

    view! {
        <input
            type=r#type
            class=format!(
                "w-full rounded-[var(--iu-radius-md)] border bg-[hsl(var(--iu-bg))] \
                 text-[hsl(var(--iu-fg))] transition-colors \
                 focus-visible:outline-none focus-visible:ring-2 \
                 disabled:cursor-not-allowed disabled:opacity-50 {} {} {}",
                size_cls, state_cls, class
            )
            disabled=disabled
            aria-invalid=invalid
            placeholder=placeholder
            name=name
            prop:value=move || value.map(|v| v.get()).unwrap_or_default()
            on:input=move |ev| {
                if let Some(set) = set_value {
                    set.set(event_target_value(&ev));
                }
            }
        />
    }
}
