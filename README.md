# Booking API

A room bookings API built with Rust and Axum.

## Setup

```bash
cp .env.example .env

cargo run

curl "http://localhost:10000/rooms?date_from=2026-06-01&date_to=2026-06-03"

