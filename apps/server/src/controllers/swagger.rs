use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RusTok API",
        version = "1.0.0",
        description = "Unified API for RusTok CMS & Commerce"
    ),
    paths(
        // Auth
        crate::controllers::auth::login,
        crate::controllers::auth::register,
        crate::controllers::auth::refresh,
        crate::controllers::auth::logout,
        crate::controllers::auth::me,
        // Content
        crate::controllers::content::nodes::list_nodes,
        crate::controllers::content::nodes::get_node,
        crate::controllers::content::nodes::create_node,
        crate::controllers::content::nodes::update_node,
        crate::controllers::content::nodes::delete_node,
        // Blog
        crate::controllers::blog::posts::list_posts,
        crate::controllers::blog::posts::get_post,
        crate::controllers::blog::posts::create_post,
        crate::controllers::blog::posts::update_post,
        crate::controllers::blog::posts::delete_post,
    ),
    components(
        schemas(
            crate::controllers::auth::LoginParams,
            crate::controllers::auth::RegisterParams,
            crate::controllers::auth::RefreshRequest,
            crate::controllers::auth::UserResponse,
            crate::controllers::auth::AuthResponse,
            crate::controllers::auth::UserInfo,
            crate::controllers::auth::LogoutResponse,
            
            // Content
            rustok_content::dto::NodeListItem,
            rustok_content::dto::NodeResponse,
            rustok_content::dto::CreateNodeInput,
            rustok_content::dto::UpdateNodeInput,
            rustok_content::dto::NodeTranslationInput,
            rustok_content::dto::BodyInput,
            rustok_content::entities::node::ContentStatus,

            // Blog
            rustok_blog::services::post::CreatePostInput,
        )
    ),
    modifiers(&crate::controllers::swagger::SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "content", description = "Content Management endpoints"),
        (name = "blog", description = "Blog endpoints")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
