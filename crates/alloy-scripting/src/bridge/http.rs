use rhai::{Dynamic, Engine, Map};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::utils::json_to_dynamic;

pub fn register_http(engine: &mut Engine) {
    engine.register_fn("http_get", http_get);
    engine.register_fn("http_get", http_get_with_headers);
    engine.register_fn("http_post", http_post_json);
    engine.register_fn("http_post", http_post_json_with_headers);
    engine.register_fn("http_request", http_request);
}

fn run_blocking<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(f))
}

fn build_http_response(status: u16, body: serde_json::Value) -> Map {
    let mut result = Map::new();
    result.insert("status".into(), Dynamic::from(status as i64));
    result.insert("ok".into(), Dynamic::from(status >= 200 && status < 300));
    result.insert("body".into(), json_to_dynamic(body));
    result
}

fn http_get(url: &str) -> Map {
    http_get_with_headers(url, Map::new())
}

fn http_get_with_headers(url: &str, headers: Map) -> Map {
    let url = url.to_string();
    let headers = extract_headers(headers);

    debug!(target: "alloy::script", "HTTP GET {}", url);

    let result = run_blocking(async move {
        let client = reqwest::Client::new();
        let mut request = client.get(&url);

        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let body = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or(serde_json::Value::Null);
                Ok((status, body))
            }
            Err(err) => Err(err.to_string()),
        }
    });

    match result {
        Ok((status, body)) => build_http_response(status, body),
        Err(err) => {
            warn!(target: "alloy::script", "HTTP GET {} failed: {}", url, err);
            let mut map = Map::new();
            map.insert("status".into(), Dynamic::from(0_i64));
            map.insert("ok".into(), Dynamic::from(false));
            map.insert("error".into(), Dynamic::from(err));
            map
        }
    }
}

fn http_post_json(url: &str, body: Dynamic) -> Map {
    http_post_json_with_headers(url, body, Map::new())
}

fn http_post_json_with_headers(url: &str, body: Dynamic, headers: Map) -> Map {
    let url = url.to_string();
    let headers = extract_headers(headers);
    let body_json = dynamic_to_json_value(body);

    debug!(target: "alloy::script", "HTTP POST {}", url);

    let result = run_blocking(async move {
        let client = reqwest::Client::new();
        let mut request = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body_json);

        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let resp_body = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or(serde_json::Value::Null);
                Ok((status, resp_body))
            }
            Err(err) => Err(err.to_string()),
        }
    });

    match result {
        Ok((status, resp_body)) => build_http_response(status, resp_body),
        Err(err) => {
            warn!(target: "alloy::script", "HTTP POST {} failed: {}", url, err);
            let mut map = Map::new();
            map.insert("status".into(), Dynamic::from(0_i64));
            map.insert("ok".into(), Dynamic::from(false));
            map.insert("error".into(), Dynamic::from(err));
            map
        }
    }
}

fn http_request(method: &str, url: &str, body: Dynamic, headers: Map) -> Map {
    let method = method.to_uppercase();
    let url = url.to_string();
    let headers = extract_headers(headers);
    let body_json = dynamic_to_json_value(body);

    debug!(target: "alloy::script", "HTTP {} {}", method, url);

    let result = run_blocking(async move {
        let client = reqwest::Client::new();
        let req_method =
            reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);

        let mut request = client
            .request(req_method, &url)
            .header("Content-Type", "application/json")
            .json(&body_json);

        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let resp_body = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or(serde_json::Value::Null);
                Ok((status, resp_body))
            }
            Err(err) => Err(err.to_string()),
        }
    });

    match result {
        Ok((status, resp_body)) => build_http_response(status, resp_body),
        Err(err) => {
            warn!(target: "alloy::script", "HTTP {} {} failed: {}", method, url, err);
            let mut map = Map::new();
            map.insert("status".into(), Dynamic::from(0_i64));
            map.insert("ok".into(), Dynamic::from(false));
            map.insert("error".into(), Dynamic::from(err));
            map
        }
    }
}

fn extract_headers(headers: Map) -> HashMap<String, String> {
    headers
        .into_iter()
        .filter_map(|(key, value)| value.try_cast::<String>().map(|v| (key.to_string(), v)))
        .collect()
}

fn dynamic_to_json_value(d: Dynamic) -> serde_json::Value {
    use crate::utils::dynamic_to_json;
    dynamic_to_json(d)
}
