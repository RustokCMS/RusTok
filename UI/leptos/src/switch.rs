use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SwitchSize {
    Sm,
    #[default]
    Md,
}

#[component]
pub fn Switch(
    #[prop(optional)] checked: Option<ReadSignal<bool>>,
    #[prop(optional)] set_checked: Option<WriteSignal<bool>>,
    #[prop(default = false)] disabled: bool,
    #[prop(default = SwitchSize::Md)] size: SwitchSize,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let (track_cls, thumb_cls) = match size {
        SwitchSize::Sm => (
            "w-7 h-4",
            "h-3 w-3 data-[state=checked]:translate-x-3",
        ),
        SwitchSize::Md => (
            "w-9 h-5",
            "h-4 w-4 data-[state=checked]:translate-x-4",
        ),
    };

    let is_checked = move || checked.map(|c| c.get()).unwrap_or(false);

    view! {
        <button
            id=id
            type="button"
            role="switch"
            aria-checked=move || is_checked().to_string()
            data-state=move || if is_checked() { "checked" } else { "unchecked" }
            disabled=disabled
            class=format!(
                "relative inline-flex shrink-0 cursor-pointer items-center \
                 rounded-full border-2 border-transparent transition-colors \
                 focus-visible:outline-none focus-visible:ring-2 \
                 focus-visible:ring-[hsl(var(--iu-primary))] focus-visible:ring-offset-2 \
                 disabled:cursor-not-allowed disabled:opacity-50 {} {}",
                track_cls, class
            )
            style=move || {
                if is_checked() {
                    "background-color: hsl(var(--iu-primary))"
                } else {
                    "background-color: hsl(var(--iu-muted))"
                }
            }
            on:click=move |_| {
                if !disabled {
                    if let Some(set) = set_checked {
                        let current = checked.map(|c| c.get()).unwrap_or(false);
                        set.set(!current);
                    }
                }
            }
        >
            <span
                class=format!(
                    "pointer-events-none block rounded-full bg-white shadow-lg \
                     ring-0 transition-transform data-[state=unchecked]:translate-x-0 {}",
                    thumb_cls
                )
                data-state=move || if is_checked() { "checked" } else { "unchecked" }
            />
        </button>
    }
}
