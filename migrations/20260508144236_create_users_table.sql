-- Add migration script here
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE rooms (
    room_id INTEGER PRIMARY KEY,
    available INTEGER NOT NULL DEFAULT 0
);