# Bookings API

A room bookings API built with RUST and AXUM

## Setup

cp .env.example .env
cargo run

# Get available rooms
curl "http://localhost:10000/rooms?date_from=2026-06-01&date_to=2026-06-03"

