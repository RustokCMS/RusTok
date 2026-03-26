#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelTargetType {
    WebDomain,
    MobileApp,
    ApiClient,
    Embedded,
    External,
}

impl ChannelTargetType {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "web_domain" => Some(Self::WebDomain),
            "mobile_app" => Some(Self::MobileApp),
            "api_client" => Some(Self::ApiClient),
            "embedded" => Some(Self::Embedded),
            "external" => Some(Self::External),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WebDomain => "web_domain",
            Self::MobileApp => "mobile_app",
            Self::ApiClient => "api_client",
            Self::Embedded => "embedded",
            Self::External => "external",
        }
    }

    pub fn supports_host_resolution(&self) -> bool {
        matches!(self, Self::WebDomain)
    }
}
