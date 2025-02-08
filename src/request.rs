use reqwest::Client;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::error::Error;

pub fn build_request(client: &Client, config: &BenchmarkConfig, url: &str, cnt: usize) -> (RequestBuilder, Option<String>) {
    let mut request_builder = client.get(url);
    if let Some(headers) = &config.request.headers {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers {
            header_map.insert(HeaderName::from_bytes(key.as_bytes()).unwrap(), HeaderValue::from_str(value).unwrap());
        }
        request_builder = request_builder.headers(header_map);
    }
    if let Some(timeout) = config.request.timeout {
        request_builder = request_builder.timeout(std::time::Duration::from_secs(timeout));
    }
    if let Some(body) = &config.request.body {
        request_builder = request_builder.body(body.clone());
        return (request_builder, Some(body.clone()));
    }
    (request_builder, None)
}

pub fn replace_special_strings(url: &str, cnt: usize) -> String {
    url.replace("{C}", &cnt.to_string())
}
