FROM lukemathwalker/cargo-chef:latest-rust-1.55.0-alpine as chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# install system dependencies
RUN apk add --no-cache opus-dev autoconf automake
ARG RUSTFLAGS='-C target-feature=-crt-static'

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --offline --bin sunny_flowers

FROM alpine:edge AS runtime
WORKDIR /app
RUN apk add --no-cache ffmpeg youtube-dl
COPY --from=builder /app/target/release/sunny_flowers /usr/local/bin
CMD ["/usr/local/bin/sunny_flowers"]
