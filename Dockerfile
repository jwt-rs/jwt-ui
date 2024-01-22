# -----------------------------
# Build jwt-ui base image
# -----------------------------

FROM clux/muslrust:stable AS builder
WORKDIR /usr/src

# Download and compile Rust dependencies in an empty project and cache as a separate Docker layer
RUN USER=root cargo new --bin jwtui-temp
WORKDIR /usr/src/jwtui-temp
COPY Cargo.* .
RUN cargo build --release --target x86_64-unknown-linux-musl
# remove src from empty project
RUN rm -r src

# Copy actual source files and Build the app binary
COPY src ./src
# remove previous deps
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/jwtui*

RUN --mount=type=cache,target=/volume/target \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl --bin jwtui
RUN mv target/x86_64-unknown-linux-musl/release/jwtui .

# -----------------------------
# build final jwtui image
# -----------------------------

FROM debian:stable-slim

# Copy the compiled binary from the builder container
COPY --from=builder --chown=nonroot:nonroot /usr/src/jwtui-temp/jwtui /usr/local/bin

# Install dependencies
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y -qq libxcb1 && \
    apt-get autoremove && apt-get autoclean

RUN /usr/local/bin/jwtui -h

ENTRYPOINT [ "/usr/local/bin/jwtui" ]
