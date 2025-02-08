use std::{sync::MutexGuard, time::Instant};

use super::BenchResult;

pub struct StatisticsData {
    total_requests: usize,
    total_transfer: u64,
    total_time: u128,
    rps: f64,
    mean: f64,
    std_dev: f64,
    min_time: u128,
    max_time: u128,
    error_count: usize,
    status_200_count: usize,
    status_4xx_count: usize,
    status_5xx_count: usize,
    under_10ms_count: usize,
    _10_to_100ms_count: usize,
    _100_to_200ms_count: usize,
    _200_to_500ms_count: usize,
    _500_to_1000ms_count: usize,
    _1000_to_10000ms_count: usize,
    over_10000ms_count: usize,
}

impl From<MutexGuard<'_, Vec<BenchResult>>> for StatisticsData {
    fn from(timings: MutexGuard<'_, Vec<BenchResult>>) -> Self {
        let total_requests = timings.len();
        let total_transfer: u64 = timings.iter().map(|t| t.total_transfer).sum();
        let total_elapsed_time: u128 = timings.iter().map(|t| t.elapsed_time).sum();
        let first_time: Instant = timings.iter().map(|t| t.start_time).min().unwrap_or_else(|| Instant::now());
        let total_time: u128 = Instant::now().duration_since(first_time).as_millis();
        let rps = total_requests as f64 / (total_time as f64 / 1000.0);
        let mean = total_elapsed_time as f64 / total_requests as f64;
        let std_dev = (timings.iter().map(|t| (t.elapsed_time as f64 - mean).powi(2)).sum::<f64>() / total_requests as f64).sqrt();
        let min_time = timings.iter().map(|t| t.elapsed_time).min().unwrap_or(0);
        let max_time = timings.iter().map(|t| t.elapsed_time).max().unwrap_or(0);
        let error_count = timings.iter().filter(|t| t.is_error).count();
        let status_200_count = timings.iter().filter(|t| t.status_code == Some(200)).count();
        let status_4xx_count = timings.iter().filter(|t| t.status_code.map_or(false, |code| code >= 400 && code < 500)).count();
        let status_5xx_count = timings.iter().filter(|t| t.status_code.map_or(false, |code| code >= 500)).count();
        let under_10ms_count = timings.iter().filter(|t| t.elapsed_time < 10).count();
        let _10_to_100ms_count = timings.iter().filter(|t| t.elapsed_time >= 10 && t.elapsed_time < 100).count();
        let _100_to_200ms_count = timings.iter().filter(|t| t.elapsed_time >= 100 && t.elapsed_time < 200).count();
        let _200_to_500ms_count = timings.iter().filter(|t| t.elapsed_time >= 200 && t.elapsed_time < 500).count();
        let _500_to_1000ms_count = timings.iter().filter(|t| t.elapsed_time >= 500 && t.elapsed_time < 1000).count();
        let _1000_to_10000ms_count = timings.iter().filter(|t| t.elapsed_time >= 1000 && t.elapsed_time < 10000).count();
        let over_10000ms_count = timings.iter().filter(|t| t.elapsed_time >= 10000).count();
        StatisticsData {
            total_requests,
            total_transfer,
            total_time,
            rps,
            mean,
            std_dev,
            min_time,
            max_time,
            error_count,
            status_200_count,
            status_4xx_count,
            status_5xx_count,
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

fn style_text<T: std::fmt::Display>(text: T) -> String {
    format!("\x1b[1;32m{}\x1b[0m", text)
}

pub fn print_statistics(timings_data: MutexGuard<Vec<BenchResult>>) {
    let min_start_time = timings_data.iter().map(|t| t.start_time).min().unwrap_or_else(|| std::time::Instant::now());
    let total_time = min_start_time.elapsed().as_millis();
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
        style_text(format!(
            "{} Bytes",
            stats.total_transfer.to_string().chars().rev().collect::<Vec<_>>().chunks(3).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join(",").chars().rev().collect::<String>()
        ))
    );
    println!(
        "{:<22} {}",
        "Total Time:",
        style_text(format!("{} ms", total_time))
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
            (stats.total_transfer as f64 * 8.0) / (total_time as f64 * 1000.0)
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
