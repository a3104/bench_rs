mod client;
mod request;
mod response;
mod statistics;
mod utils;

use client::build_client;
use request::{build_request, replace_special_strings};
use response::handle_response;
use statistics::print_statistics;

use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Debug)]
struct RequestConfig {
    url: String,
    headers: Option<HashMap<String, String>>, // ヘッダーはオプションで指定可能
    timeout: Option<u64>,                     // タイムアウトをオプションで指定可能
    method: Option<String>,                   // HTTPメソッドをオプションで指定可能
    body: Option<String>,                     // POSTリクエストのボディをオプションで指定可能
}

#[derive(Deserialize, Debug)]
struct BenchmarkConfig {
    total_requests: usize,    // 総リクエスト数
    concurrent_access: usize, // 同時アクセス数
    logging: Option<String>,  // ログ出力先
    request: RequestConfig,   // リクエストの設定
}

pub async fn run_json_benchmark(config_json: &str) -> Result<(), Box<dyn Error>> {
    let config: BenchmarkConfig = serde_json::from_str(config_json).map_err(|e| {
        eprintln!("Failed to parse config: {}", e);
        e
    })?;
    println!("Benchmark Config: {:?}", config);

    let timings = Arc::new(Mutex::new(Vec::<BenchResult>::new())); // ベンチマーク結果を保存するための共有ベクター
    let counter = Arc::new(Mutex::new(0)); // リクエストカウンター

    let client = build_client(&config)?;

    let mut handles: Vec<JoinHandle<()>> = vec![];
    let config_arc = Arc::new(config);
    for _ in 0..config_arc.concurrent_access {
        let timings_clone = Arc::clone(&timings);
        let config_clone = Arc::clone(&config_arc);
        let client_clone = client.clone();
        let counter_clone = Arc::clone(&counter);

        let handle = tokio::spawn(async move {
            for _ in 0..(config_clone.total_requests / config_clone.concurrent_access) {
                let start_time = Instant::now();
                let request_url = {
                    let mut current_counter = counter_clone.lock().unwrap();
                    *current_counter += 1;
                    replace_special_strings(&config_clone.request.url, *current_counter)
                    // URL内の特殊文字列を置換
                };
                let cnt: usize = *counter_clone.lock().unwrap();

                let (request_builder, body) =
                    build_request(&client_clone, &config_clone, &request_url, cnt);

                let response = request_builder.send().await.map_err(|e| {
                    eprintln!("Request failed: {}", e);
                    e
                });

                handle_response(response, body,start_time, &timings_clone).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.map_err(|e| {
            eprintln!("Task failed: {}", e);
            e
        })?; // 全てのタスクが完了するのを待つ
    }

    let timings_data = timings.lock().unwrap();
    if !timings_data.is_empty() {
        if let Some(logging_path) = &config_arc.logging {
            write_to_csv(logging_path, &timings_data)?;
        }
        print_statistics(timings_data); // 統計情報を表示

    } else {
        println!("No timing data available.");
    }

    Ok(())
}

struct BenchResult {
    url: String,
    post_body: Option<String>,
    start_time: Instant,
    status_code: Option<u16>,
    elapsed_time: u128,
    total_transfer: u64,
    is_error: bool,
}

fn write_to_csv(path: &str, timings_data: &Vec<BenchResult>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    writeln!(file, "URL,Status Code,Is Error,Elapsed Time,Total Transfer")?;
    for timing in timings_data {
        writeln!(
            file,
            "\"{}\",\"{}\",{},{},{},{}",
            timing.url,
            timing.post_body.as_deref().unwrap_or(""),
            timing.status_code.unwrap_or(0),
            timing.is_error,
            timing.elapsed_time,
            timing.total_transfer
        )?;
    }
    Ok(())
}
