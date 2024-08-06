# Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build dependencies only
RUN cargo fetch

# Copy the rest of the application code
COPY . .

# Build the Rust application
RUN cargo build --release

# Run the application
CMD ["./target/release/Goodwill_Web_Scrapper"]
