# Stage 1: Build the Rust binary
FROM rust:latest AS builder

# Create a new empty shell project
WORKDIR /usr/src/myapp

# Copy Cargo files and download dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY LICENSE ./LICENSE
RUN cargo fetch

# Copy the source code

# Build the release binary
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

# Install any runtime dependencies if needed (e.g. SSL certs)
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/myapp/target/release/timetable /usr/local/bin/timetable

# Set the startup command
CMD ["timetable"]
