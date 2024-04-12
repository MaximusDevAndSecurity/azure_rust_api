# Use the official Rust image as the builder.
FROM rust:latest as builder

# Create a workspace and move into it
WORKDIR /usr/src/azure_rust_api

# Copy the Rust project files into the container
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Before fetching dependencies, ensure that at least a dummy main.rs exists
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Fetch dependencies.
RUN cargo fetch  # Ensures all dependencies are downloaded

# Copy the actual source code of the Rust application.
COPY ./src ./src

# Build the Rust application for release.
RUN cargo build --release

# The final stage uses Ubuntu as the runtime environment
FROM ubuntu:latest
ARG APP=/usr/src/app

# Install necessary packages, including MariaDB client libraries and potentially OpenSSL
RUN apt-get update && apt-get install -y libc6 ca-certificates libmariadb3 openssl libssl-dev && rm -rf /var/lib/apt/lists/*

# Set up the application directory
RUN mkdir -p $APP

# Copy the compiled binary from the builder stage to the application directory
COPY --from=builder /usr/src/azure_rust_api/target/release/azure_rust_api $APP/azure_rust_api

# Set the working directory
WORKDIR $APP

# Define the container's entry point
ENTRYPOINT ["./azure_rust_api"]
