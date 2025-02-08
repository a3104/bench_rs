use reqwest::Response;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub async fn handle_response(response: Result<Response, reqwest::Error>, body: Option<String>, start_time: Instant, timings: &Arc<Mutex<Vec<BenchResult>>>) {
    let elapsed_time = start_time.elapsed().as_millis();
    let mut bench_result = BenchResult {
        url: response.as_ref().map(|r| r.url().to_string()).unwrap_or_default(),
        post_body: body,
        start_time,
        status_code: response.as_ref().map(|r| r.status().as_u16()),
        elapsed_time,
        total_transfer: response.as_ref().map(|r| r.content_length().unwrap_or(0)).unwrap_or(0),
        is_error: response.is_err(),
    };
    if let Ok(resp) = response {
        bench_result.total_transfer += resp.bytes().await.unwrap_or_default().len() as u64;
    }
    let mut timings = timings.lock().unwrap();
    timings.push(bench_result);
}
