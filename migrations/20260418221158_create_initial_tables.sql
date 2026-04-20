CREATE TABLE IF NOT EXISTS packet_captures (
    timestamp   TEXT    NOT NULL,
    ether_type  TEXT    NOT NULL,
    src_ip      TEXT    NOT NULL,
    dst_ip      TEXT    NOT NULL,
    protocol    TEXT    NOT NULL,
    length      INTEGER NOT NULL
);
