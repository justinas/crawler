# Crawler

## Run

Tested with Rust 1.45.2. Run with:

    $ RUST_LOG=crawler=debug,actix=debug cargo run

Test with:

    $ curl -H 'Content-Type: application/json' --data '{"domain": "http://example.com"}' \
        http://127.0.0.1:8080/crawl
    $ curl http://127.0.0.1:8080/domains
    ["example.com"]
    $ curl http://127.0.0.1:8080/domains/example.com
    {"count":1,"urls":["https://www.iana.org/domains/example"]}

## Docker image

Build with:

    $ docker build .
