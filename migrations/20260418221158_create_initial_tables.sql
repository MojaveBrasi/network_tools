CREATE TABLE IF NOT EXISTS packet_capture (
    timestamp   INTEGER NOT NULL,
    src_ip      BLOB    NOT NULL,
    dst_ip      BLOB    NOT NULL,
    protocol    TEXT    NOT NULL,
    length      INTEGER NOT NULL
);
