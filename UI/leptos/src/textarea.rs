use leptos::prelude::*;

use crate::types::Size;

#[component]
pub fn Textarea(
    #[prop(default = Size::Md)] size: Size,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] invalid: bool,
    #[prop(default = 3u32)] rows: u32,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] value: Option<ReadSignal<String>>,
    #[prop(optional)] set_value: Option<WriteSignal<String>>,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] name: String,
) -> impl IntoView {
    let size_cls = match size {
        Size::Sm => "text-xs px-2 py-1.5",
        Size::Md => "text-sm px-3 py-2",
        Size::Lg | Size::Icon => "text-base px-4 py-3",
    };

    let state_cls = if invalid {
        "border-[hsl(var(--iu-danger))] focus-visible:ring-[hsl(var(--iu-danger))]"
    } else {
        "border-[hsl(var(--iu-border))] focus-visible:ring-[hsl(var(--iu-primary))]"
    };

    view! {
        <textarea
            rows=rows
            class=format!(
                "w-full rounded-[var(--iu-radius-md)] border bg-[hsl(var(--iu-bg))] \
                 text-[hsl(var(--iu-fg))] resize-y transition-colors \
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
