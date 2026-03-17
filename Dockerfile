FROM rust:1.94-slim AS builder
WORKDIR /app

# Cache dependencies layer
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build actual source
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
# ca-certificates required for reqwest/rustls to validate Elasticsearch TLS certs
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/github-webhook-ingester /usr/local/bin/

ENV RUST_LOG=info
EXPOSE 3001
CMD ["github-webhook-ingester"]
