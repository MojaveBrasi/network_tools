# Network Traffic Capture Server

## About
* CLI for Capturing and analyzing traffic in a LAN
* Web dashboard eventually I guess

## Dual Embedded Database
* Packet captures are stored in columnar fashion with DuckDB for timeseries analysis
* Other metrics stored in SQLite relational database

## Async Runtime 
* The rust pnet library, which uses libpcap, is single-threaded. Packets move through MPSC channel
* Multiple concurrent database writers which can be added/removed at runtime

