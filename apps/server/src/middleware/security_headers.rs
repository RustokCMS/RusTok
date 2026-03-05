/// Security Headers Middleware
///
/// Adds OWASP-recommended security response headers to every HTTP response:
/// - `Content-Security-Policy` — restricts resource loading
/// - `X-Content-Type-Options: nosniff` — prevents MIME sniffing
/// - `X-Frame-Options: DENY` — prevents clickjacking
/// - `X-XSS-Protection: 0` — disables legacy XSS filter (modern browsers use CSP)
/// - `Referrer-Policy: strict-origin-when-cross-origin`
/// - `Permissions-Policy` — disables unused browser features
/// - `Strict-Transport-Security` — enforces HTTPS (only in production)
///
/// Mounted globally in `app.rs::after_routes()` via `axum::middleware::from_fn`.
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

/// Default CSP for API server: no HTML rendering, only JSON/GraphQL responses.
/// Allows same-origin fetch only; blocks everything else.
const CSP: &str = "default-src 'none'; frame-ancestors 'none'";

/// HSTS: 1 year, include subdomains.
/// Only injected when `RUSTOK_HTTPS=true` env var is set to avoid breaking local dev.
const HSTS: &str = "max-age=31536000; includeSubDomains";

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Content-Security-Policy
    headers.insert("content-security-policy", HeaderValue::from_static(CSP));

    // X-Content-Type-Options
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

    // X-XSS-Protection — disabled per OWASP recommendation (CSP is the modern replacement)
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));

    // Referrer-Policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy — disable all unused browser features
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), \
             magnetometer=(), microphone=(), payment=(), usb=()",
        ),
    );

    // Strict-Transport-Security — only in production (HTTPS)
    if std::env::var("RUSTOK_HTTPS").as_deref() == Ok("true") {
        headers.insert("strict-transport-security", HeaderValue::from_static(HSTS));
    }

    response
}
