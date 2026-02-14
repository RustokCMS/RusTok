use leptos::*;

#[component]
pub fn Input(
    #[prop(default = "text")] r#type: &'static str,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] value: Option<ReadSignal<String>>,
    #[prop(optional)] on_input: Option<Box<dyn Fn(ev::Event) + 'static>>,
    #[prop(optional)] error: Option<&'static str>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] name: Option<&'static str>,
) -> impl IntoView {
    let base_classes = "block w-full rounded-md border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";
    
    let state_classes = if error.is_some() {
        "border-red-300 focus:border-red-500 focus:ring-red-500"
    } else {
        "border-gray-300 focus:border-blue-500 focus:ring-blue-500"
    };
    
    let additional_classes = class.unwrap_or("");
    
    let full_class = format!("{} {} {}", base_classes, state_classes, additional_classes);

    view! {
        <div class="space-y-1">
            <input
                type=r#type
                placeholder=placeholder.unwrap_or("")
                class=full_class
                disabled=disabled
                name=name.unwrap_or("")
                value=move || value.map(|v| v.get()).unwrap_or_default()
                on:input=move |ev| {
                    if let Some(ref handler) = on_input {
                        handler(ev);
                    }
                }
            />
            {move || error.map(|err| view! {
                <p class="text-xs text-red-600">{err}</p>
            })}
        </div>
    }
}
