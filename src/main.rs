use std::sync::{Arc, Mutex, MutexGuard};
use std::{fmt, thread};
use std::time::Instant;
use std::env;

use reqwest;

struct BenchResult {
    start_time: Instant,
    status_code: Option<u16>,
    elasted_time: u128,
    total_transfer: u64,
    is_error: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        println!("Usage: program <url> <threads> <count per threads>");
        return Ok(());
    }

    let url = &args[1];
    let thread_num: usize = args[2].parse().expect("Invalid number of threads");
    let count_per_thread: usize = args[3].parse().expect("Invalid count per thread");

    let timings = Arc::new(Mutex::new(Vec::<BenchResult>::new()));
    let mut handles = vec![];

    for tc in 0..thread_num {
        let timings_clone = Arc::clone(&timings);
        let url_clone = url.clone();

        let handle = thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to build client.");

            for c in 0..count_per_thread {
                let start_time = Instant::now();

                let replace_str = format!("{}_{}", tc, c);
                let url_with_timestamp = url_clone.replace("{C}", &replace_str);
                let res = client.get(&url_with_timestamp).send();

                match res {
                    Ok(response) => {
                        let elapsed = start_time.elapsed().as_millis();
                        let mut timings = timings_clone.lock().unwrap();
                        // println!("Thread: Time: {}", elapsed);
                        timings.push(BenchResult{
                            start_time: start_time.clone(),
                            status_code: Some(response.status().as_u16()),
                            elasted_time: elapsed,
                            total_transfer: response.content_length().unwrap_or(response.bytes().unwrap().len().try_into().unwrap()),

                            is_error: false,
                        });
                    }
                    Err(e) => {
                        let elapsed = start_time.elapsed().as_millis();
                        let mut timings = timings_clone.lock().unwrap();
                        timings.push(BenchResult{
                            start_time: start_time.clone(),
                            status_code: e.status().map(|x| x.as_u16()),
                            elasted_time: elapsed,
                            total_transfer: 0,
                            is_error: true,
                        });

                    }
                }
            }
        });

        handles.push(handle);
    }

    // Join the threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Calculate statistics
    let timings_data = timings.lock().unwrap();
    if !timings_data.is_empty() {
        print_statistics(timings_data);
    } else {
        println!("No timing data available.");
    }

    Ok(())
}

fn print_statistics(timings_data: MutexGuard<Vec<BenchResult>>) {
    let minmal_instant = timings_data.iter().map(|x| x.start_time).min().unwrap();
    let total_time = minmal_instant.elapsed().as_millis();
    let total_requests = timings_data.len();
    let rps = total_requests as f64 / total_time as f64 * 1000.0;
    let min_time = timings_data.iter().map(|x| x.elasted_time).min().unwrap();
    let max_time = timings_data.iter().map(|x| x.elasted_time).max().unwrap();
    let error_count = timings_data.iter().filter(|x| x.is_error).count();
    let total_transfer: u64 = timings_data.iter().map(|x| x.total_transfer).sum();
    let status_200_count = timings_data.iter().filter(|x| x.status_code.is_some()).filter(|x| x.status_code.unwrap() == 200).count();
    let status_4xx_count = timings_data.iter().filter(|x| x.status_code.is_some()).filter(|x| x.status_code.unwrap() >= 400 && x.status_code.unwrap() < 500).count();
    let status_5xx_count = timings_data.iter().filter(|x| x.status_code.is_some()).filter(|x| x.status_code.unwrap() >= 500).count();

    let elasted_times: Vec<_> = timings_data.iter().map(|x| x.elasted_time as f64).collect();
    let mean = elasted_times.iter().sum::<f64>() / elasted_times.len() as f64;
    let variance = elasted_times.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / elasted_times.len() as f64;
    let std_dev = variance.sqrt();

    println!("\n{:-^48}", " Response Timing Statistics ");
    println!("{:<22} {}", "Total Requests:", style(total_requests));
    println!("{:<22} {}", "Total Transfer:", style(format!("{} Bytes", total_transfer)));
    println!("{:<22} {}", "Total Time:", style(format!("{} ms", total_time)));
    println!("{:<22} {}", "Requests Per Second:", style(format!("{:.2}", rps)));
    println!("{:<22} {}", "Bandwidth:", style(format!("{:.2} Mbps", (total_transfer as f64 * 8.0) / (total_time as f64 * 1000000.0))));
    println!("{:<22} {}", "Average Time:", style(format!("{:.2} ms (std dev: {:.2} ms)", mean, std_dev)));
    println!("{:<22} {}", "Minimum Time:", style(format!("{} ms", min_time)));
    println!("{:<22} {}", "Maximum Time:", style(format!("{} ms", max_time)));
    println!("{:<22} {}", "Error Count:", style(error_count));
    println!("{:-^48}", "-");
    println!("{:<22} {}", "Status 200 Count:", style(format!("{} ({:.2}%)", status_200_count, status_200_count as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "Status 4xx Count:", style(format!("{} ({:.2}%)", status_4xx_count, status_4xx_count as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "Status 5xx Count:", style(format!("{} ({:.2}%)", status_5xx_count, status_5xx_count as f64 / total_requests as f64 * 100.0)));
    println!("{:-^48}", "-");
    println!("{:<22} {}", "under 10ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time < 10).count(), timings_data.iter().filter(|x| x.elasted_time < 10).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "10 to 100ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 10 && x.elasted_time < 100).count(), timings_data.iter().filter(|x| x.elasted_time >= 10 && x.elasted_time < 100).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "100 to 200ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 100 && x.elasted_time < 200).count(), timings_data.iter().filter(|x| x.elasted_time >= 100 && x.elasted_time < 200).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "200 to 500ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 200 && x.elasted_time < 500).count(), timings_data.iter().filter(|x| x.elasted_time >= 200 && x.elasted_time < 500).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "500 to 1000ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 500 && x.elasted_time < 1000).count(), timings_data.iter().filter(|x| x.elasted_time >= 500 && x.elasted_time < 1000).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "1000 to 10000ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 1000 && x.elasted_time < 10000).count(), timings_data.iter().filter(|x| x.elasted_time >= 1000 && x.elasted_time < 10000).count() as f64 / total_requests as f64 * 100.0)));
    println!("{:<22} {}", "over 10000ms count:", style(format!("{} ({:.2}%)", timings_data.iter().filter(|x| x.elasted_time >= 10000).count(), timings_data.iter().filter(|x| x.elasted_time >= 10000).count() as f64 / total_requests as f64 * 100.0)));
}

fn style<T: fmt::Display>(text: T) -> String {
    format!("\x1b[1;32m{}\x1b[0m", text)
}
