CREATE TABLE IF NOT EXISTS packet_captures (
    timestamp   INTEGER NOT NULL,
    ether_type  TEXT    NOT NULL,
    src_ip      BLOB    NOT NULL,
    dst_ip      TEXT    NOT NULL,
    protocol    TEXT    NOT NULL,
    length      INTEGER NOT NULL
);
