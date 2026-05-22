# Booking API

[![CI](https://github.com/nattapong18-en/booking_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nattapong18-en/booking_api/actions/workflows/ci.yml)

A production-ready room booking API built with **Rust**, **Axum**, **PostgreSQL**, and **Redis**.

## 🚀 Live Demo

| Service | URL |
|---------|-----|
| **API** | [https://booking-api-8ux7.onrender.com](https://booking-api-8ux7.onrender.com) |
| **Frontend** | [https://booking-frontend-omega-three.vercel.app](https://booking-frontend-omega-three.vercel.app) |

## 📦 Tech Stack

- **Rust** — Systems programming language
- **Axum** — Web framework
- **SQLx** — Async SQL toolkit
- **PostgreSQL** — Primary database
- **Redis** — Caching layer
- **JWT** — Authentication
- **Docker** — Containerization
- **Render** — Cloud deployment (API)
- **Vercel** — Cloud deployment (Frontend)
- **GitHub Actions** — CI/CD

## ✨ Features

- ✅ User registration & login (JWT + bcrypt)
- ✅ Room availability check with date range
- ✅ Create booking (overlap protection)
- ✅ Cancel booking
- ✅ View my bookings
- ✅ Input validation
- ✅ Redis caching (Cache-Aside pattern)
- ✅ Cache invalidation on booking/cancellation
- ✅ Foreign key constraints
- ✅ CORS support
- ✅ Unit tests
- ✅ CI/CD pipeline (auto-deploy after tests pass)

## 🔧 Setup

### Prerequisites

- Rust & Cargo ([rustup.rs](https://rustup.rs))
- PostgreSQL (local or cloud)
- Redis (local or Upstash)

### Installation

```bash
# Clone the repo
git clone https://github.com/nattapong18-en/booking_api.git
cd booking_api

# Setup environment
cp .env.example .env
# Edit .env with your DATABASE_URL, REDIS_URL, and JWT_SECRET

# Run migrations
cargo install sqlx-cli
sqlx migrate run

# Build & run
cargo run