use leptos::*;

#[component]
pub fn Separator(
    #[prop(default = "horizontal")] orientation: &'static str,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let orientation_classes = match orientation {
        "vertical" => "h-full w-px",
        _ => "w-full h-px", // horizontal (default)
    };
    
    let base_classes = "bg-gray-200";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {} {}", base_classes, orientation_classes, additional_classes);

    view! {
        <div class=full_class role="separator" />
    }
}
