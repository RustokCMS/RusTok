use leptos::prelude::*;

use crate::spinner::Spinner;
use crate::types::{ButtonVariant, Size};

#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = Size::Md)] size: Size,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
    #[prop(default = "button")] r#type: &'static str,
    children: Children,
) -> impl IntoView {
    let size_cls = match size {
        Size::Sm => "h-8 px-3 text-xs gap-1.5",
        Size::Md => "h-9 px-4 text-sm gap-2",
        Size::Lg => "h-10 px-6 text-base gap-2",
        Size::Icon => "h-9 w-9",
    };

    let variant_cls = match variant {
        ButtonVariant::Primary => {
            "bg-[hsl(var(--iu-primary))] text-[hsl(var(--iu-primary-fg))] hover:opacity-90"
        }
        ButtonVariant::Secondary => {
            "bg-[hsl(var(--iu-muted))] text-[hsl(var(--iu-muted-fg))] hover:opacity-80"
        }
        ButtonVariant::Ghost => {
            "bg-transparent text-[hsl(var(--iu-fg))] hover:bg-[hsl(var(--iu-accent))]"
        }
        ButtonVariant::Outline => {
            "border border-[hsl(var(--iu-border))] bg-transparent \
             text-[hsl(var(--iu-fg))] hover:bg-[hsl(var(--iu-accent))]"
        }
        ButtonVariant::Destructive => {
            "bg-[hsl(var(--iu-danger))] text-[hsl(var(--iu-danger-fg))] hover:opacity-90"
        }
    };

    let full_class = format!(
        "inline-flex items-center justify-center font-medium \
         rounded-[var(--iu-radius-md)] transition-all \
         focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 \
         disabled:pointer-events-none disabled:opacity-50 {} {} {}",
        size_cls, variant_cls, class
    );

    let is_disabled = disabled || loading;

    view! {
        <button
            type=r#type
            class=full_class
            disabled=is_disabled
            on:click=move |_| {
                if !is_disabled {
                    if let Some(ref handler) = on_click {
                        handler();
                    }
                }
            }
        >
            {move || {
                if loading {
                    Some(view! { <Spinner size=Size::Sm /> })
                } else {
                    None
                }
            }}
            {children()}
        </button>
    }
}
