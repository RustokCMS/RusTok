use leptos::prelude::*;

use crate::types::{BadgeVariant, Size};

#[component]
pub fn Badge(
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(default = Size::Md)] size: Size,
    #[prop(default = false)] dismissible: bool,
    #[prop(optional)] on_dismiss: Option<Box<dyn Fn() + 'static>>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let size_cls = match size {
        Size::Sm => "px-1.5 py-0 text-[10px]",
        _ => "px-2 py-0.5 text-xs",
    };

    let variant_cls = match variant {
        BadgeVariant::Default => "bg-[hsl(var(--iu-primary))] text-[hsl(var(--iu-primary-fg))]",
        BadgeVariant::Secondary => "bg-[hsl(var(--iu-muted))] text-[hsl(var(--iu-muted-fg))]",
        BadgeVariant::Success => "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400",
        BadgeVariant::Warning => "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
        BadgeVariant::Danger => "bg-[hsl(var(--iu-danger)/0.15)] text-[hsl(var(--iu-danger))]",
    };

    view! {
        <span
            class=format!(
                "inline-flex items-center gap-1 rounded-full font-medium {} {} {}",
                size_cls, variant_cls, class
            )
        >
            {children()}
            {dismissible.then(|| {
                let handler = on_dismiss.map(|f| Callback::new(move |_| f()));
                view! {
                    <button
                        type="button"
                        class="ml-0.5 rounded-full opacity-70 hover:opacity-100 focus:outline-none"
                        aria-label="Dismiss"
                        on:click=move |_| {
                            if let Some(cb) = handler {
                                cb.run(());
                            }
                        }
                    >
                        "×"
                    </button>
                }
            })}
        </span>
    }
}
