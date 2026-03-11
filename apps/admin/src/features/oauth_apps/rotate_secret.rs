use crate::entities::oauth_app::model::OAuthApp;
use crate::shared::ui::Button;
use leptos::prelude::*;

#[component]
pub fn RotateSecretDialog(
    app: OAuthApp,
    on_success: impl Fn(String) + 'static + Clone,
    on_cancel: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let app_name = app.name.clone();
    let rotate_action = Action::new(move |_: &()| {
        let on_success = on_success.clone();
        async move { on_success("sk_live_rotated_mock_secret_67890".to_string()) }
    });

    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium">"Rotate Secret"</h3>
            <p class="text-sm text-slate-500">"Generate a new client secret for "<span class="font-semibold">{app_name}</span></p>
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=move |_| { rotate_action.dispatch(()); }>
                    "Rotate"
                </Button>
                <Button on_click=move |_| on_cancel()>
                    "Cancel"
                </Button>
            </div>
        </div>
    }
}
