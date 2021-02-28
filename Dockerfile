FROM rust:1.50-buster AS builder
WORKDIR /app

# Create a layer with pre-compiled dependencies, no source code
COPY Cargo.toml Cargo.lock /app
RUN mkdir /app/src
RUN echo "fn main() {}" > /app/src/main.rs
RUN cargo build --release

# Actually build
COPY src /app/src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:buster-slim AS runtime
RUN apt-get update
RUN apt-get install -y ca-certificates openssl
COPY --from=builder /app/target/release/crawler /usr/local/bin
ENV RUST_LOG=crawler=debug,actix=debug
ENTRYPOINT ["/usr/local/bin/crawler"]
