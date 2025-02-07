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
    1. {C} : replaced sequentially with the count of the request threadsId_count
2. `threads`: The number of threads to use for sending requests.
3. `count per threads`: The number of requests each thread will send.
4. `is_logging`: A boolean value indicating whether to log errors. Set to true or false.

Here is an example of how to run the program:

```bash
cargo run "http://example.com" 10 100 false
```

This will start the program with 10 threads, each sending 100 requests to "http://example.com", and errors will be
logged.

# Console output example

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

## JSON Benchmark Configuration

To use `json_bench`, you need a configuration file in JSON format as shown below.

```json
{
  "total_requests": 1000,
  "concurrent_access": 10,
  "request": {
    "url": "http://example.com",
    "headers": {
      "Authorization": "Bearer YOUR_TOKEN"
    },
    "timeout": 5,
    "method": "GET",
    "body": null
  }
}
```

- `total_requests`: The total number of requests to send.
- `concurrent_access`: The number of concurrent threads to use.
- `request.url`: The URL to send the requests to.
- `request.headers`: Headers to include in the request (optional).
- `request.timeout`: Timeout for the request in seconds (optional).
- `request.method`: HTTP method to use (optional, default is GET).
- `request.body`: Body for POST requests (optional).

### Special Variables

You can use special variables in the `url` field to dynamically generate parts of the URL:

- `$CNT`: Replaced with the current request count.
- `$RND(n)`: Replaced with a random alphanumeric string of length `n`.
- `$NRND(n)`: Replaced with a random numeric string of length `n`.
- `$LUID`: Replaced with a unique UUID v4 string.
- `$RNDB64(n)`: Replaced with a random hexadecimal string of length `n`.

Example:

```json
{
  "total_requests": 1000,
  "concurrent_access": 10,
  "request": {
    "url": "http://example.com/$CNT/$RND(5)/$NRND(3)",
    "headers": {
      "Authorization": "Bearer YOUR_TOKEN"
    },
    "timeout": 5,
    "method": "GET",
    "body": null
  }
}
```

After creating the configuration file, run the program as follows:

```bash
bench_rs json #configfile
```

This will execute the benchmark based on the configuration file.

## License

This project is licensed under the MIT License 

## Acknowledgments

* Thanks to the Rust community for the comprehensive documentation and resources.
