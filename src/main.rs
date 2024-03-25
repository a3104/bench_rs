use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::env;

use reqwest;

struct BenchResult {
    start_time: Instant,
    status_code: u16,
    elasted_time: u128,
    total_transfer: u64,
    is_error: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        println!("Usage: program <url> <threads> <count per threads> <is_logging>");
        return Ok(());
    }

    let url = &args[1];
    let thread_num: usize = args[2].parse().expect("Invalid number of threads");
    let count_per_thread: usize = args[3].parse().expect("Invalid count per thread");
    let is_logging: bool = args[4].parse().expect("Invalid logging argument");

    let timings = Arc::new(Mutex::new(Vec::<BenchResult>::new()));
    let mut handles = vec![];

    for tc in 0..thread_num {
        let timings_clone = Arc::clone(&timings);
        let url_clone = url.clone();
        let is_logging = is_logging;

        let handle = thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
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
                            status_code: response.status().as_u16(),
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
                            status_code: e.status().unwrap_or(reqwest::StatusCode::ACCEPTED).as_u16(),
                            elasted_time: elapsed,
                            total_transfer: 0,
                            is_error: true,
                        });

                        if is_logging {
                            eprintln!("Error: {}", e);
                        }
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
        let minmal_instant = timings_data.iter().map(|x| x.start_time).min().unwrap();
        let total_time = minmal_instant.elapsed().as_millis();
        let total_requests = timings_data.len();
        let rps = total_requests as f64 / total_time as f64 * 1000.0;
        let total_time: u128 = timings_data.iter().map(|x| x.elasted_time).sum();
        let average_time = total_time / timings_data.len() as u128;
        let min_time = timings_data.iter().map(|x| x.elasted_time).min().unwrap();
        let max_time = timings_data.iter().map(|x| x.elasted_time).max().unwrap();
        let error_count = timings_data.iter().filter(|x| x.is_error).count();
        let total_transfer: u64 = timings_data.iter().map(|x| x.total_transfer).sum();
        let status_200_count = timings_data.iter().filter(|x| x.status_code == 200).count();
        let status_4xx_count = timings_data.iter().filter(|x| x.status_code >= 400 && x.status_code < 500).count();
        let status_5xx_count = timings_data.iter().filter(|x| x.status_code >= 500).count();


        println!("\n--- Request Timing Statistics ---");
        println!("Total Requests: {}", total_requests);
        println!("Total Transfer: {} Bytes", total_transfer);
        println!("Total Time: {} ms", total_time);
        println!("Requests Per Second: {}", rps);
        println!("Bandwidth: {} Mbps", total_transfer as f64 / total_time as f64 * 8.0);
        println!("Average Time: {}", average_time);
        println!("Minimum Time: {}", min_time);
        println!("Maximum Time: {}", max_time);
        println!("Error Count: {}", error_count);
        println!("-----------------------------");
        println!("Status 200 Count: {} ({}%)", status_200_count, status_200_count as f64 / total_requests as f64 * 100.0);
        println!("Status 4xx Count: {} ({}%)", status_4xx_count, status_4xx_count as f64 / total_requests as f64 * 100.0);
        println!("Status 5xx Count: {} ({}%)", status_5xx_count, status_5xx_count as f64 / total_requests as f64 * 100.0);
        println!("-----------------------------");
        println!("under 10ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time < 10).count(), timings_data.iter().filter(|x| x.elasted_time < 10).count() as f64 / total_requests as f64 * 100.0);
        println!("between 10 to 100ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 10 && x.elasted_time < 100).count(), timings_data.iter().filter(|x| x.elasted_time >= 10 && x.elasted_time < 100).count() as f64 / total_requests as f64 * 100.0);
        println!("between 100 to 200ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 100 && x.elasted_time < 200).count(), timings_data.iter().filter(|x| x.elasted_time >= 100 && x.elasted_time < 200).count() as f64 / total_requests as f64 * 100.0);
        println!("between 200 to 500ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 200 && x.elasted_time < 500).count(), timings_data.iter().filter(|x| x.elasted_time >= 200 && x.elasted_time < 500).count() as f64 / total_requests as f64 * 100.0);

        println!("between 500 to 1000ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 500 && x.elasted_time < 1000).count(), timings_data.iter().filter(|x| x.elasted_time >= 500 && x.elasted_time < 1000).count() as f64 / total_requests as f64 * 100.0);
        println!("between 1000 to 10000ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 1000 && x.elasted_time < 10000).count(), timings_data.iter().filter(|x| x.elasted_time >= 1000 && x.elasted_time < 10000).count() as f64 / total_requests as f64 * 100.0);
        println!("over 10000ms count {} {} %", timings_data.iter().filter(|x| x.elasted_time >= 10000).count(), timings_data.iter().filter(|x| x.elasted_time >= 10000).count() as f64 / total_requests as f64 * 100.0);

    } else {
        println!("No timing data available.");
    }

    Ok(())
}
