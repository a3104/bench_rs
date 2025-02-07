use rand::distributions::Uniform;
use rand::{distributions::Alphanumeric, Rng};
use uuid::Uuid;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use tokio::task::JoinHandle;

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

                let request_builder =
                    build_request(&client_clone, &config_clone, &request_url, cnt);

                let response = request_builder.send().await.map_err(|e| {
                    eprintln!("Request failed: {}", e);
                    e
                });

                handle_response(response, start_time, &timings_clone).await;
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
        print_statistics(timings_data); // 統計情報を表示
    } else {
        println!("No timing data available.");
    }

    Ok(())
}

fn build_client(config: &BenchmarkConfig) -> Result<reqwest::Client, Box<dyn Error>> {
    let client_builder = reqwest::Client::builder();
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

fn build_request(
    client: &reqwest::Client,
    config: &Arc<BenchmarkConfig>,
    url: &str,
    counter: usize,
) -> reqwest::RequestBuilder {
    let mut request_builder = match config.request.method.as_deref() {
        Some("POST") => client.post(url),
        _ => client.get(url),
    };

    if let Some(body) = &config.request.body {
        let replaced_body = replace_special_strings(body, counter);
        request_builder = request_builder.body(replaced_body);
    }

    if let Some(headers) = &config.request.headers {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers.iter() {
            let replaced_value = replace_special_strings(value, counter);
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(&replaced_value),
            ) {
                header_map.insert(header_name, header_value); // ヘッダーを設定
            }
        }
        request_builder = request_builder.headers(header_map);
    }

    request_builder
}

async fn handle_response(
    response: Result<reqwest::Response, reqwest::Error>,
    start_time: Instant,
    timings: &Arc<Mutex<Vec<BenchResult>>>,
) {
    match response {
        Ok(res) => {
            let elapsed = start_time.elapsed().as_millis();
            let mut timings = timings.lock().unwrap();
            timings.push(BenchResult {
                start_time: start_time.clone(),
                status_code: Some(res.status().as_u16()), // ステータスコードを保存
                elapsed_time: elapsed,
                total_transfer: res.content_length().unwrap_or(0), // コンテンツ長を保存
                is_error: false,
            });
        }
        Err(e) => {
            let elapsed = start_time.elapsed().as_millis();
            let mut timings = timings.lock().unwrap();
            timings.push(BenchResult {
                start_time: start_time.clone(),
                status_code: e.status().map(|x| x.as_u16()), // エラーステータスコードを保存
                elapsed_time: elapsed,
                total_transfer: 0,
                is_error: true,
            });
        } // <-- 追加: Err ブロックの閉じ括弧
    }
}

pub fn generate_random_hex_string(length: usize, rng: &mut impl Rng) -> String {
    let hex_chars = Uniform::new_inclusive(0, 15);
    (0..length)
        .map(|_| format!("{:X}", rng.sample(&hex_chars)))
        .collect()
}

pub fn replace_special_strings(url: &str, counter: usize) -> String {
    let luid = generate_luid();
    let mut replaced_url = url.replace("$CNT", &counter.to_string()); // $CNTをカウンター値に置換

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

pub fn generate_luid() -> String {
    Uuid::new_v4().to_string()
}

pub fn replace_random_strings<F, R>(url: &str, rng: &mut R, pattern: &str, generator: &F) -> String
where
    F: Fn(usize, &mut R) -> String,
    R: Rng,
{
    let mut replaced_url = url.to_string();
    while let Some(start_index) = replaced_url.find(pattern) {
        if let Some(end_index) = replaced_url[start_index..].find(")") {
            let length_str = &replaced_url[start_index + pattern.len()..start_index + end_index];
            match length_str.parse::<usize>() {
                Ok(length) => {
                    let generated_string = generator(length, rng);
                    replaced_url
                        .replace_range(start_index..start_index + end_index + 1, &generated_string);
                }
                Err(_) => break, // エラー処理
            }
        } else {
            break; // エラー処理
        }
    }
    replaced_url
}

pub fn generate_random_string(length: usize, rng: &mut impl Rng) -> String {
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_random_number_string(length: usize, rng: &mut impl Rng) -> String {
    format!(
        "{:0width$}",
        rng.gen_range(0..10usize.pow(length as u32)),
        width = length
    )
}

struct BenchResult {
    start_time: Instant,
    status_code: Option<u16>,
    elapsed_time: u128,
    total_transfer: u64,
    is_error: bool,
}

struct StatisticsData {
    total_requests: usize,
    total_transfer: u64,
    total_time: u128,
    rps: f64,
    min_time: u128,
    max_time: u128,
    error_count: usize,
    status_200_count: usize,
    status_4xx_count: usize,
    status_5xx_count: usize,
    mean: f64,
    variance: f64,
    std_dev: f64,
    under_10ms_count: usize,
    _10_to_100ms_count: usize,
    _100_to_200ms_count: usize,
    _200_to_500ms_count: usize,
    _500_to_1000ms_count: usize,
    _1000_to_10000ms_count: usize,
    over_10000ms_count: usize,
}
impl<'a> From<MutexGuard<'a, Vec<BenchResult>>> for StatisticsData {
    fn from(timings_data: MutexGuard<Vec<BenchResult>>) -> Self {
        let minmal_instant = timings_data.iter().map(|x| x.start_time).min().unwrap();
        let total_time = minmal_instant.elapsed().as_millis();
        let total_requests = timings_data.len();
        let rps = total_requests as f64 / total_time as f64 * 1000.0; // リクエスト毎秒
        let min_time = timings_data.iter().map(|x| x.elapsed_time).min().unwrap();
        let max_time = timings_data.iter().map(|x| x.elapsed_time).max().unwrap();
        let error_count = timings_data.iter().filter(|x| x.is_error).count();
        let total_transfer: u64 = timings_data.iter().map(|x| x.total_transfer).sum();
        let status_200_count = timings_data
            .iter()
            .filter(|x| x.status_code.is_some())
            .filter(|x| x.status_code.unwrap() == 200)
            .count();
        let status_4xx_count = timings_data
            .iter()
            .filter(|x| x.status_code.is_some())
            .filter(|x| x.status_code.unwrap() >= 400 && x.status_code.unwrap() < 500)
            .count();
        let status_5xx_count = timings_data
            .iter()
            .filter(|x| x.status_code.is_some())
            .filter(|x| x.status_code.unwrap() >= 500)
            .count();

        let elapsed_time: Vec<_> = timings_data.iter().map(|x| x.elapsed_time as f64).collect();
        let mean = elapsed_time.iter().sum::<f64>() / elapsed_time.len() as f64;
        let variance = elapsed_time.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / elapsed_time.len() as f64;
        let std_dev = variance.sqrt();

        let under_10ms_count = timings_data.iter().filter(|x| x.elapsed_time < 10).count();
        let _10_to_100ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 10 && x.elapsed_time < 100)
            .count();

        let _100_to_200ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 100 && x.elapsed_time < 200)
            .count();
        let _200_to_500ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 200 && x.elapsed_time < 500)
            .count();
        let _500_to_1000ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 500 && x.elapsed_time < 1000)
            .count();
        let _1000_to_10000ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 1000 && x.elapsed_time < 10000)
            .count();
        let over_10000ms_count = timings_data
            .iter()
            .filter(|x| x.elapsed_time >= 10000)
            .count();

        StatisticsData {
            total_requests,
            total_transfer,
            total_time,
            rps,
            min_time,
            max_time,
            error_count,
            status_200_count,
            status_4xx_count,
            status_5xx_count,
            mean,
            variance,
            std_dev,
            under_10ms_count,
            _10_to_100ms_count,
            _100_to_200ms_count,
            _200_to_500ms_count,
            _500_to_1000ms_count,
            _1000_to_10000ms_count,
            over_10000ms_count,
        }
    }
}

fn print_statistics(timings_data: MutexGuard<Vec<BenchResult>>) {
    let stats = StatisticsData::from(timings_data);
    println!("\n{:-^48}", " Response Timing Statistics ");
    println!(
        "{:<22} {}",
        "Total Requests:",
        style_text(stats.total_requests)
    );
    println!(
        "{:<22} {}",
        "Total Transfer:",
        style_text(format!("{} Bytes", stats.total_transfer))
    );
    println!(
        "{:<22} {}",
        "Total Time:",
        style_text(format!("{} ms", stats.total_time))
    );
    println!(
        "{:<22} {}",
        "Requests Per Second:",
        style_text(format!("{:.2}", stats.rps))
    );
    println!(
        "{:<22} {}",
        "Bandwidth:",
        style_text(format!(
            "{:.2} Mbps",
            (stats.total_transfer as f64 * 8.0) / (stats.total_time as f64 * 1000000.0)
        ))
    );
    println!(
        "{:<22} {}",
        "Average Time:",
        style_text(format!(
            "{:.2} ms (std dev: {:.2} ms)",
            stats.mean, stats.std_dev
        ))
    );
    println!(
        "{:<22} {}",
        "Minimum Time:",
        style_text(format!("{} ms", stats.min_time))
    );
    println!(
        "{:<22} {}",
        "Maximum Time:",
        style_text(format!("{} ms", stats.max_time))
    );
    println!("{:<22} {}", "Error Count:", style_text(stats.error_count));
    println!("{:-^48}", "-");
    println!(
        "{:<22} {}",
        "Status 200 Count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats.status_200_count,
            stats.status_200_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "Status 4xx Count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats.status_4xx_count,
            stats.status_4xx_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "Status 5xx Count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats.status_5xx_count,
            stats.status_5xx_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!("{:-^48}", "-");
    println!(
        "{:<22} {}",
        "under 10ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats.under_10ms_count,
            stats.under_10ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "10 to 100ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats._10_to_100ms_count,
            stats._10_to_100ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "100 to 200ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats._100_to_200ms_count,
            stats._100_to_200ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "200 to 500ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats._200_to_500ms_count,
            stats._200_to_500ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "500 to 1000ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats._500_to_1000ms_count,
            stats._500_to_1000ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "1000 to 10000ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats._1000_to_10000ms_count,
            stats._1000_to_10000ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
    println!(
        "{:<22} {}",
        "over 10000ms count:",
        style_text(format!(
            "{} ({:.2}%)",
            stats.over_10000ms_count,
            stats.over_10000ms_count as f64 / stats.total_requests as f64 * 100.0
        ))
    );
}

fn style_text<T: std::fmt::Display>(text: T) -> String {
    format!("\x1b[1;32m{}\x1b[0m", text) // 緑色で強調表示
}
