use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::RequestBuilder;

use super::utils::generate_luid;
use super::utils::generate_random_hex_string;
use super::utils::generate_random_number_string;
use super::utils::generate_random_string;
use super::utils::replace_random_strings;
use super::BenchmarkConfig;

pub fn build_request(client: &Client, config: &BenchmarkConfig, url: &str, cnt: usize) -> (RequestBuilder, Option<String>) {
    let mut request_builder = match config.request.method.as_deref() {
        Some("POST") => client.post(url),
        _ => client.get(url),
    };
    let body = config.request.body.clone().unwrap_or_else(|| "".to_string());
    let replaced_body = replace_special_strings(body.as_str(), cnt);

    if let Some(headers) = &config.request.headers {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers.iter() {
            let replaced_value = replace_special_strings(value, cnt);
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(&replaced_value),
            ) {
                header_map.insert(header_name, header_value); // ヘッダーを設定
            }
        }
        request_builder = request_builder.headers(header_map);
    }

    (request_builder, Some(replaced_body.clone()))
}

pub fn replace_special_strings(url: &str, cnt: usize) -> String {
    let luid = generate_luid();
    let mut replaced_url = url.replace("$CNT", &cnt.to_string()); // $CNTをカウンター値に置換

    let mut rng = rand::thread_rng();
    replaced_url =
        replace_random_strings(&replaced_url, &mut rng, "$RND(", &generate_random_string);
    replaced_url = replace_random_strings(
        &replaced_url,
        &mut rng,
        "$NRND(",
        &generate_random_number_string,
    );
    replaced_url = replace_random_strings(
        &replaced_url,
        &mut rng,
        "$RNDB64(",
        &generate_random_hex_string,
    );

    replaced_url = replaced_url.replace("$LUID", &luid); // $LUIDを生成して置換
    replaced_url
}
