use leptos::*;
use crate::types::{ButtonVariant, Size};

#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = Size::Md)] size: Size,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
    #[prop(optional)] r#type: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let size_classes = match size {
        Size::Sm => "px-3 py-1.5 text-sm",
        Size::Md => "px-4 py-2 text-base",
        Size::Lg => "px-6 py-3 text-lg",
    };

    let base_classes = "inline-flex items-center justify-center font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";
    
    let variant_classes = variant.classes();
    
    let additional_classes = class.unwrap_or("");
    
    let full_class = format!(
        "{} {} {} {}",
        base_classes, size_classes, variant_classes, additional_classes
    );

    let is_disabled = disabled || loading;

    let button_type = r#type.unwrap_or("button");

    view! {
        <button
            type=button_type
            class=full_class
            disabled=is_disabled
            on:click=move |_| {
                if let Some(ref handler) = on_click {
                    handler();
                }
            }
        >
            {move || if loading {
                view! {
                    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                }.into_view()
            } else {
                ().into_view()
            }}
            {children()}
        </button>
    }
}
