use reqwest::Client;
use std::error::Error;

use super::BenchmarkConfig;

pub fn build_client(config: &BenchmarkConfig) -> Result<Client, Box<dyn Error>> {
    let client_builder = Client::builder();
    let client_builder = if let Some(timeout) = config.request.timeout {
        client_builder.timeout(std::time::Duration::from_secs(timeout))
    } else {
        client_builder.timeout(std::time::Duration::from_secs(5)) // デフォルトのタイムアウトを5秒に設定
    };
    let client = client_builder.build().map_err(|e| {
        eprintln!("Failed to build client: {}", e);
        e
    })?;
    Ok(client)
}
