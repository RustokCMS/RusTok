use crate::entities::oauth_app::model::{AppType, OAuthApp};
use crate::shared::ui::{Button, Input, Textarea};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOAuthAppResult {
    pub app: OAuthApp,
    pub client_secret: String,
}

#[component]
pub fn CreateAppForm(
    on_success: Callback<CreateOAuthAppResult>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (description, set_description) = signal(String::new());

    let submit = move |_| {
        let mock_app = OAuthApp {
            id: uuid::Uuid::new_v4(),
            name: name.get(),
            slug: slug.get(),
            description: Some(description.get()),
            app_type: AppType::ThirdParty,
            client_id: uuid::Uuid::new_v4(),
            redirect_uris: vec![],
            scopes: vec![],
            grant_types: vec!["authorization_code".into()],
            manifest_ref: None,
            auto_created: false,
            is_active: true,
            active_token_count: 0,
            last_used_at: None,
            created_at: chrono::Utc::now(),
        };
        on_success.run(CreateOAuthAppResult { app: mock_app, client_secret: "sk_live_mock_secret_12345".into() });
    };

    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium">"Create New Connected App"</h3>
            <Input value=name set_value=set_name placeholder="App name" />
            <Input value=slug set_value=set_slug placeholder="Slug/Identifier" />
            <Textarea prop:value=description on:input=move |ev| set_description.set(event_target_value(&ev)) />
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=submit>"Create App"</Button>
                <Button on_click=move |_| on_cancel.run(())>"Cancel"</Button>
            </div>
        </div>
    }
}
