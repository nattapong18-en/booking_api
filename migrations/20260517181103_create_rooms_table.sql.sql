-- Add migration script here
CREATE TABLE IF NOT EXISTS rooms (
    room_id INTEGER PRIMARY KEY AUTOINCREMENT
);

INSERT OR IGNORE INTO rooms (room_id) VALUES (1);
INSERT OR IGNORE INTO rooms (room_id) VALUES (2);
INSERT OR IGNORE INTO rooms (room_id) VALUES (3);