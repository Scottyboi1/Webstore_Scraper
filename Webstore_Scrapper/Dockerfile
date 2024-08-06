# Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory to /app/Webstor_Scrapper
WORKDIR /app/Webstore_Scrapper

# Copy Cargo.toml and Cargo.lock
COPY Webstor_Scrappere/Cargo.toml Webstore_Scrapper/Cargo.lock ./

# Build dependencies
RUN cargo fetch

# Copy the source code
COPY Webstore_Scrapper ./

# Build the Rust application
RUN cargo build --release

# Run the application
CMD ["./target/release/Goodwill_Web_Scrapper"]
