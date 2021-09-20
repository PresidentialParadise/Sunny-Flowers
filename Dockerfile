# Building the app
FROM rust:1.55 as build

WORKDIR /sunny-flowers/
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y libopus-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release --locked

# Running the app
FROM debian:bullseye-slim

# Add run deps
RUN apt-get update && apt-get install -y ffmpeg youtube-dl && rm -rf /var/lib/apt/lists/*

# Copy bin from builder
COPY --from=build /sunny-flowers/target/release/sunny_flowers /usr/local/bin/sunny_flowers

CMD ["sunny_flowers"]
