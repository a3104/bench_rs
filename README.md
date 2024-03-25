# HTTP Benchmarking Tool

This is a Rust project that includes a benchmarking tool for HTTP requests. The tool sends multiple requests to a
specified URL and collects various statistics about the responses.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing
purposes.

### Installing

Clone the repository to your local machine:

```bash
git clone https://github.com/a3104/bench_rs.git
cd bench_rs
cargo build --release

if needed, install the required dependencies
(apt install pkg-config libssl-dev)
```

Navigate to the project directory:

## Usage

The program takes four arguments:

1. `url`: The URL to which the requests will be sent.
    1. {C} : repladed Sequentially with the count of the request threadsId_count
2. `threads`: The number of threads to use for sending requests.
3. `count per threads`: The number of requests each thread will send.
4. `is_logging`: A boolean value indicating whether to log errors. set true / false

Here is an example of how to run the program:

```bash
cargo run "http://example.com" 10 100 false
```

This will start the program with 10 threads, each sending 100 requests to "http://example.com", and errors will be
logged.

# Console output  example

```
--- Request Timing Statistics ---
Total Requests: 10000
Total Transfer: 1530000 Bytes
Total Time: 155 ms
Requests Per Second: 37037.03703703704
Bandwidth: 78967.74193548386 Mbps
Average Time: 0
Minimum Time: 0
Maximum Time: 10
Error Count: 0
-----------------------------
Status 200 Count: 0 (0%)
Status 4xx Count: 10000 (100%)
Status 5xx Count: 0 (0%)
-----------------------------
under 10ms count 9997 99.97 %
between 10 to 100ms count 3 0.03 %
between 100 to 200ms count 0 0 %
between 200 to 500ms count 0 0 %
between 500 to 1000ms count 0 0 %
between 1000 to 10000ms count 0 0 %
over 10000ms count 0 0 %
```


## License

This project is licensed under the MIT License 

## Acknowledgments

* Thanks to the Rust community for the comprehensive documentation and resources.
