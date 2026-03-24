use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiRouteContext {
    pub locale: Option<String>,
    pub route_segment: Option<String>,
    pub subpath: Option<String>,
    pub query: BTreeMap<String, String>,
}

impl UiRouteContext {
    pub fn query_value(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    pub fn subpath(&self) -> Option<&str> {
        self.subpath.as_deref()
    }

    pub fn subpath_matches(&self, prefix: &str) -> bool {
        self.subpath()
            .map(|subpath| subpath == prefix || subpath.starts_with(&format!("{prefix}/")))
            .unwrap_or(false)
    }
}
