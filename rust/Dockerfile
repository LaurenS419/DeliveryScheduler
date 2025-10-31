# -------- Build stage --------
FROM rust:latest AS builder

WORKDIR /usr/src/
COPY . .
RUN cargo build --release

# -------- Runtime stage --------
FROM ubuntu:22.04

# Install needed libraries
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/target/release/drone_scheduler /usr/local/bin/drone_scheduler

# Default command
ENTRYPOINT ["/usr/local/bin/drone_scheduler"]
