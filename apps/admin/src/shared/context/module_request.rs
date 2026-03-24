use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_router::params::ParamsMap;
use rustok_api::UiRouteContext;

#[component]
pub fn ModuleRequestProvider(
    route_segment: Option<String>,
    subpath: Option<String>,
    query_params: ParamsMap,
    children: Children,
) -> impl IntoView {
    let query = query_params
        .latest_values()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<BTreeMap<_, _>>();

    provide_context(UiRouteContext {
        locale: None,
        route_segment,
        subpath,
        query,
    });

    children()
}
