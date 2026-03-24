#![recursion_limit = "256"]

pub mod app;
pub mod entities;
pub mod modules;
pub mod pages;
pub mod shared;
pub mod widgets;

use axum::{extract::Path, routing::get, Router};
use futures::StreamExt;
use leptos::prelude::RenderHtml;
use leptos::view;

use crate::app::{StorefrontModulePage, StorefrontShell};
use crate::shared::context::enabled_modules::fetch_enabled_modules;

fn render_document(locale: &str, title: &str, app_html: String) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="{locale}">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <link rel="stylesheet" href="/assets/app.css" />
</head>
<body>
  <div id="app">{app_html}</div>
</body>
</html>"#,
        locale = locale,
        title = title,
        app_html = app_html
    )
}

async fn enabled_modules_or_empty() -> Vec<String> {
    match fetch_enabled_modules().await {
        Ok(modules) => modules,
        Err(err) => {
            eprintln!("failed to fetch enabled modules for storefront SSR: {err}");
            Vec::new()
        }
    }
}

pub async fn render_shell(
    locale: &str,
    query_params: std::collections::HashMap<String, String>,
) -> String {
    let locale_owned = locale.to_string();
    let enabled_modules = enabled_modules_or_empty().await;

    let app_html = {
        let locale = locale_owned.clone();
        view! {
            <StorefrontShell
                locale=locale
                enabled_modules=enabled_modules
                query_params=query_params
            />
        }
        .to_html_stream_in_order()
        .collect::<String>()
        .await
    };
    render_document(locale, "RusToK Storefront", app_html)
}

pub async fn render_module_page(
    locale: &str,
    route_segment: &str,
    query_params: std::collections::HashMap<String, String>,
) -> String {
    let locale_owned = locale.to_string();
    let route_segment_owned = route_segment.to_string();
    let enabled_modules = enabled_modules_or_empty().await;

    let app_html = {
        let locale = locale_owned.clone();
        let route_segment = route_segment_owned.clone();
        view! {
            <StorefrontModulePage
                locale=locale
                enabled_modules=enabled_modules
                route_segment=route_segment
                query_params=query_params
            />
        }
        .to_html_stream_in_order()
        .collect::<String>()
        .await
    };
    render_document(locale, "RusToK Module Storefront", app_html)
}

pub fn router() -> Router {
    Router::new()
        .route(
            "/",
            get(
                |axum::extract::Query(params): axum::extract::Query<
                    std::collections::HashMap<String, String>,
                >| async move {
                    let locale = params
                        .get("lang")
                        .map(|value| value.to_lowercase())
                        .unwrap_or_else(|| "en".to_string());
                    render_shell(locale.as_str(), params).await
                },
            ),
        )
        .route(
            "/modules/:route_segment",
            get(
                |Path(route_segment): Path<String>,
                 axum::extract::Query(params): axum::extract::Query<
                    std::collections::HashMap<String, String>,
                >| async move {
                    let locale = params
                        .get("lang")
                        .map(|value| value.to_lowercase())
                        .unwrap_or_else(|| "en".to_string());
                    render_module_page(locale.as_str(), route_segment.as_str(), params).await
                },
            ),
        )
}
