FROM rust:slim as builder

WORKDIR /usr/src/app
COPY . .

# Build the release binary
RUN cargo build --release

# Production image
FROM debian:bookworm-slim
WORKDIR /app

# Install sqlite and ca-certificates
RUN apt-get update && apt-get install -y libsqlite3-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/api /app/api

EXPOSE 9000
ENV PORT=9000
ENV HOST="0.0.0.0"

CMD ["/app/api"]
