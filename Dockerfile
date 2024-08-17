FROM rust:latest

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        libclang-dev

WORKDIR /usr/src/dexscreener-pricebot-v2

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

COPY .env ./.env

COPY README.md ./README.md

RUN cargo install --features "database" --path .

CMD ["dexscreener-pricebot-v2"]