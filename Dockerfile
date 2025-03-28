FROM rust:1.82 as builder
WORKDIR /app
COPY ./ ./
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/user-service /usr/local/bin/
EXPOSE 9001
CMD ["user-service"]
