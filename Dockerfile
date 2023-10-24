# Rust as the base image
FROM rust:1.67 as build

# Create a new empty shell project
RUN USER=root cargo new --bin kmr
WORKDIR /kmr

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/kmr*
RUN cargo build --release

# The final base image
FROM debian:bullseye

RUN apt-get update && \
    apt-get install iproute2 iputils-ping -y
# Copy from the previous build
COPY --from=build /kmr/target/release/kmr /usr/src/kmr
COPY ./config ./config
# COPY --from=build /holodeck/target/release/holodeck/target/x86_64-unknown-linux-musl/release/holodeck .

# Run the binary
CMD ["/usr/src/kmr"]