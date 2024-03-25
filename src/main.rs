use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::OpenOptions;
use std::io::Write;

// コマンドライン引数を取得するために使用
use std::env;

// reqwestクレートを使用してHTTPリクエストを送る
use reqwest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let is_logging: bool = args.get(3).unwrap_or(&String::from("false")) == "true";

    if args.len() != 4 {
        panic!("Usage: program <url> <threads> <logging>");
    }

    let url = &args[1];
    let thread_num: usize = args[2].parse().expect("Invalid number of threads");

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..thread_num {
        let counter_clone = Arc::clone(&counter);
        let url = url.clone();
        let unixtime_ns = chrono::Utc::now().timestamp_nanos();

        let handle = thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .build()
                .expect("Failed to build client.");

            for c in 0..2000 {
                let url = url.replace("{C}", &(unixtime_ns.to_string()+&c.to_string()));
                println!("{}",&url);

                let res = client.get(&url).send();

                match res {
                    Ok(mut response) => {
                        if let Ok(body) = response.text() {
                            let mut num = counter_clone.lock().unwrap();
                            *num += 1;

                            print!("\r{}", num);

                            if !is_logging {
                                continue;
                            }
                            let mut file = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .append(true)
                                .open("log.txt")
                                .expect("Failed to open log.txt");

                            if let Err(e) = writeln!(file, "{}", body) {
                                eprintln!("Couldn't write to file: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        // 以下のステートメントによりループを続ける
                        continue;
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
