FROM lukemathwalker/cargo-chef:latest-rust-slim AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates sqlite3 && \
    apt-get autoremove -y && \
    apt-get clean -y && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/booking_api /app/booking_api

EXPOSE 10000
ENTRYPOINT ["./booking_api"]
