FROM lukemathwalker/cargo-chef:latest-rust-1.59.0-slim as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.59.0-slim as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.59.0 as builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release --bin bfstats

FROM debian:bullseye-slim as runtime

WORKDIR /app
COPY --from=builder /app/target/release/bfstats /usr/local/bin

WORKDIR /usr/local/bin/
COPY /templates ./templates

ENTRYPOINT ["./bfstats"]