# Booking API

[![CI](https://github.com/nattapong18-en/booking_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nattapong18-en/booking_api/actions/workflows/ci.yml)

A room booking API built with **Rust** and **Axum**.

## 🚀 Live Demo

| Service | URL |
|---------|-----|
| **API** | [https://booking-api-8ux7.onrender.com](https://booking-api-8ux7.onrender.com) |
| **Frontend** | [https://booking-frontend-omega-three.vercel.app](https://booking-frontend-omega-three.vercel.app) |

## 📦 Tech Stack

- **Rust** — Systems programming language
- **Axum** — Web framework
- **SQLx** — Async SQL toolkit
- **SQLite** — Database
- **JWT** — Authentication
- **Docker** — Containerization
- **Render** — Cloud deployment

## ✨ Features

- ✅ User registration & login (JWT)
- ✅ Room availability check
- ✅ Create booking (overlap protection)
- ✅ Cancel booking
- ✅ View my bookings
- ✅ Input validation
- ✅ Error handling
- ✅ CORS support
- ✅ Unit tests

## 🔧 Setup

### Prerequisites

- Rust & Cargo ([rustup.rs](https://rustup.rs))
- SQLite3

### Installation

```bash
# Clone the repo
git clone https://github.com/nattapong18-en/booking_api.git
cd booking_api

# Setup environment
cp .env.example .env
# Edit .env with your values

# Run migrations
cargo install sqlx-cli
sqlx migrate run

# Build & run
cargo run
