use crate::entities::oauth_app::model::OAuthApp;
use crate::shared::ui::Button;
use leptos::prelude::*;

#[component]
pub fn RotateSecretDialog(app: OAuthApp, on_success: Callback<String>, on_cancel: Callback<()>) -> impl IntoView {
    let app_name = app.name.clone();
    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium">"Rotate Secret"</h3>
            <p class="text-sm text-slate-500">"Generate a new client secret for "<span class="font-semibold">{app_name}</span></p>
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=move |_| on_success.run("sk_live_rotated_mock_secret_67890".to_string())>"Rotate"</Button>
                <Button on_click=move |_| on_cancel.run(())>"Cancel"</Button>
            </div>
        </div>
    }
}
