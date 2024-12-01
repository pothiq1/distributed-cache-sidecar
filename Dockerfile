# Use official Rust image as the build environment
FROM rust:1.72 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# Use a minimal image for the runtime
FROM debian:buster-slim

COPY --from=builder /usr/src/app/target/release/distributed-cache-sidecar /usr/local/bin/distributed-cache-sidecar

CMD ["distributed-cache-sidecar"]
