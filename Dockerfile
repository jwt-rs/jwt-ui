# -----------------------------
# Build jwt-ui base image
# -----------------------------

FROM rust as builder
WORKDIR /usr/src

# Download and compile Rust dependencies in an empty project and cache as a separate Docker layer
RUN USER=root cargo new --bin jwtui-temp
WORKDIR /usr/src/jwtui-temp
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release --target x86_64-unknown-linux-gnu
# remove src form empty project
RUN rm -r src

# Copy actual source files and Build the app binary
COPY src ./src
# remove previous deps
RUN rm ./target/x86_64-unknown-linux-gnu/release/deps/jwtui*
# due to cargo bug https://github.com/rust-lang/rust/issues/25289
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y -qq pkg-config libssl-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# RUN RUST_BACKTRACE=1 RUSTFLAGS="-Ctarget-feature=-crt-static" cargo rustc --release --target x86_64-unknown-linux-musl
RUN RUST_BACKTRACE=1 cargo build --release --target x86_64-unknown-linux-gnu

# -----------------------------
# build final jwtui image
# -----------------------------

FROM debian:stable-slim

# Copy the compiled binary from the builder container
COPY --from=builder /usr/src/jwtui-temp/target/x86_64-unknown-linux-gnu/release/jwtui /usr/local/bin

# Install dependencies
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y -qq libxcb1 && \
    apt-get autoremove && apt-get autoclean

RUN /usr/local/bin/jwtui -h

ENTRYPOINT [ "/usr/local/bin/jwtui" ]
