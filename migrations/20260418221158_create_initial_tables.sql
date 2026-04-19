CREATE TABLE IF NOT EXISTS packets (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    captured_at TEXT    NOT NULL,
    src_ip      TEXT    NOT NULL,
    dst_ip      TEXT    NOT NULL,
    protocol    TEXT    NOT NULL,
    length      INTEGER NOT NULL
);
