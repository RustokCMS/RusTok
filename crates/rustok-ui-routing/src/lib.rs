use std::collections::BTreeMap;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate, use_query_map};
use leptos_router::NavigateOptions;
use rustok_api::{sanitize_admin_route_query, AdminQueryKey, UiRouteContext};

#[derive(Clone)]
pub struct AdminQueryWriter {
    apply: Rc<dyn Fn(Vec<(AdminQueryKey, Option<String>)>, bool)>,
}

impl AdminQueryWriter {
    pub fn update(&self, updates: Vec<(AdminQueryKey, Option<String>)>, replace: bool) {
        (self.apply)(updates, replace);
    }

    pub fn push_value(&self, key: AdminQueryKey, value: impl Into<String>) {
        self.update(vec![(key, Some(value.into()))], false);
    }

    pub fn replace_value(&self, key: AdminQueryKey, value: impl Into<String>) {
        self.update(vec![(key, Some(value.into()))], true);
    }

    pub fn clear_key(&self, key: AdminQueryKey) {
        self.update(vec![(key, None)], true);
    }
}

pub fn read_admin_query_value(route_context: &UiRouteContext, key: AdminQueryKey) -> Option<String> {
    route_context.query_value(key.as_str()).map(str::to_owned)
}

pub fn use_admin_query_value(key: AdminQueryKey) -> Signal<Option<String>> {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let raw_query = use_query_map();
    let route_segment = route_context.route_segment.clone();
    let subpath = route_context.subpath.clone();

    Signal::derive(move || {
        let query = raw_query
            .get()
            .latest_values()
            .map(|(query_key, query_value)| (query_key.to_string(), query_value.to_string()))
            .collect::<BTreeMap<_, _>>();
        sanitize_admin_route_query(route_segment.as_deref(), subpath.as_deref(), &query)
            .get(key.as_str())
            .cloned()
    })
}

pub fn use_admin_query_writer() -> AdminQueryWriter {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let raw_query = use_query_map();
    let location = use_location();
    let navigate = use_navigate();
    let route_segment = route_context.route_segment.clone();
    let subpath = route_context.subpath.clone();

    AdminQueryWriter {
        apply: Rc::new(move |updates, replace| {
            let mut next_query = raw_query
                .get_untracked()
                .latest_values()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<BTreeMap<_, _>>();

            for (key, value) in updates {
                match value
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                {
                    Some(value) => {
                        next_query.insert(key.as_str().to_string(), value);
                    }
                    None => {
                        next_query.remove(key.as_str());
                    }
                }
            }

            let next_query = sanitize_admin_route_query(
                route_segment.as_deref(),
                subpath.as_deref(),
                &next_query,
            );
            let pathname = location.pathname.get_untracked();
            let href = if next_query.is_empty() {
                pathname
            } else {
                let query = serde_urlencoded::to_string(next_query)
                    .unwrap_or_default();
                format!("{pathname}?{query}")
            };

            navigate(
                &href,
                NavigateOptions {
                    replace,
                    ..NavigateOptions::default()
                },
            );
        }),
    }
}
