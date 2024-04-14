# Use the official Rust image as the builder.
FROM rust:latest as builder

WORKDIR /usr/src/azure_rust_api

# Copy the Rust project files into the container
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Create a dummy file to allow dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Fetch dependencies.
RUN cargo fetch  # Ensures all dependencies are downloaded

# Install Diesel CLI
RUN cargo install diesel_cli --no-default-features --features "mysql"

# Copy the actual source code and migrations of the Rust application.
COPY ./src ./src
COPY ./migrations ./migrations

# Build the Rust application for release.
RUN cargo build --release

# The final stage uses Ubuntu as the runtime environment
FROM ubuntu:latest
ARG APP=/usr/src/app

# Install necessary packages, including MariaDB client libraries, OpenSSL, and MySQL client
RUN apt-get update && \
    apt-get install -y libc6 ca-certificates libmariadb3 openssl libssl-dev mysql-client && \
    rm -rf /var/lib/apt/lists/*

# Set up the application directory
RUN mkdir -p $APP

# Copy the compiled binary from the builder stage to the application directory
COPY --from=builder /usr/src/azure_rust_api/target/release/azure_rust_api $APP/azure_rust_api

# Copy the Diesel CLI executable and migration files
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /usr/src/azure_rust_api/migrations $APP/migrations

WORKDIR $APP

# Create a startup script to run Diesel migrations and then start the application
RUN echo "#!/bin/bash\n\
echo \"Using DATABASE_URL: \$DATABASE_URL\"\n\
while ! mysqladmin ping -h\"db\" --silent; do\n\
    echo 'Waiting for database to come up...';\n\
    sleep 1;\n\
done\n\
diesel migration run --database-url=\$DATABASE_URL\n\
./azure_rust_api" > start.sh \
&& chmod +x start.sh


# Define the container's entry point
ENTRYPOINT ["./start.sh"]