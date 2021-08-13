FROM rust as planner
WORKDIR /app
# We only pay the installation cost once, 
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning 
# the cargo-chef version with `--version X.X.X`
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN curl -o /usr/lib/libwkhtmltox.so \
    --location \
    https://github.com/rdvojmoc/DinkToPdf/blob/master/v0.12.4/64%20bit/libwkhtmltox.so?raw=true

RUN cargo build --release --bin bfstats

FROM debian:buster-slim as runtime

WORKDIR /app
COPY --from=builder /app/target/release/bfstats /usr/local/bin

RUN apt-get update && apt-get install -y \
  curl \
  && curl -o /usr/lib/libwkhtmltox.so \
    --location \
    https://github.com/rdvojmoc/DinkToPdf/blob/master/v0.12.4/64%20bit/libwkhtmltox.so?raw=true

RUN apt-get update && apt-get install -y \
  wkhtmltopdf \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin/
COPY /templates ./templates

ENTRYPOINT ["./bfstats"]