use leptos_graphql::{execute as execute_graphql, GraphqlHttpError, GraphqlRequest};
use serde::{Deserialize, Serialize};

use crate::model::StorefrontBlogData;

pub type ApiError = GraphqlHttpError;

const STOREFRONT_BLOG_QUERY: &str = "query StorefrontBlog($postSlug: String!, $filter: PostsFilter, $locale: String) { selectedPost: postBySlug(slug: $postSlug, locale: $locale) { id effectiveLocale title slug excerpt body bodyFormat status publishedAt tags featuredImageUrl } posts(filter: $filter) { total items { id title effectiveLocale slug excerpt status publishedAt } } }";

#[derive(Debug, Deserialize)]
struct StorefrontBlogResponse {
    #[serde(rename = "selectedPost")]
    selected_post: Option<crate::model::BlogPostDetail>,
    posts: crate::model::BlogPostList,
}

#[derive(Debug, Serialize)]
struct StorefrontBlogVariables {
    #[serde(rename = "postSlug")]
    post_slug: String,
    filter: PostsFilter,
    locale: Option<String>,
}

#[derive(Debug, Serialize)]
struct PostsFilter {
    status: Option<String>,
    locale: Option<String>,
    page: u64,
    #[serde(rename = "perPage")]
    per_page: u64,
}

fn configured_tenant_slug() -> Option<String> {
    [
        "RUSTOK_TENANT_SLUG",
        "NEXT_PUBLIC_TENANT_SLUG",
        "NEXT_PUBLIC_DEFAULT_TENANT_SLUG",
    ]
    .into_iter()
    .find_map(|key| {
        std::env::var(key).ok().and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
    })
}

fn graphql_url() -> String {
    if let Some(url) = option_env!("RUSTOK_GRAPHQL_URL") {
        return url.to_string();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let origin = web_sys::window()
            .and_then(|window| window.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string());
        format!("{origin}/api/graphql")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base =
            std::env::var("RUSTOK_API_URL").unwrap_or_else(|_| "http://localhost:5150".to_string());
        format!("{base}/api/graphql")
    }
}

async fn request<V, T>(query: &str, variables: V) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &graphql_url(),
        GraphqlRequest::new(query, Some(variables)),
        None,
        configured_tenant_slug(),
        None,
    )
    .await
}

pub async fn fetch_storefront_blog(
    post_slug: String,
    locale: Option<String>,
) -> Result<StorefrontBlogData, ApiError> {
    let response: StorefrontBlogResponse = request(
        STOREFRONT_BLOG_QUERY,
        StorefrontBlogVariables {
            post_slug,
            filter: PostsFilter {
                status: Some("PUBLISHED".to_string()),
                locale: locale.clone(),
                page: 1,
                per_page: 6,
            },
            locale,
        },
    )
    .await?;

    Ok(StorefrontBlogData {
        selected_post: response.selected_post,
        posts: response.posts,
    })
}
