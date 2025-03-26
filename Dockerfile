FROM rust:1.82

# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

EXPOSE 9001

# Run the binary
CMD ["./target/release/user-service"]
