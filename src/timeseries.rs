use crate::IpCapture;
use crate::database::dbfmt;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tokio::time::{Interval, interval};

use duckdb::{Connection, Result};

#[derive(Debug)]
pub struct TimeSeriesWriter {
    capture_reciever: Option<mpsc::Receiver<IpCapture>>,
    connection: Option<Connection>,
    path: Option<String>,
    buffer: Vec<IpCapture>,
    flush_timer: Interval,
}

impl TimeSeriesWriter {
    pub fn new() -> Self {
        Self { 
            capture_reciever: None, 
            connection: None, 
            path: None,
            buffer: Vec::with_capacity(4096), 
            flush_timer: interval(Duration::from_mins(1)) }
    }
    pub fn at(mut self, path: &str) -> Result<Self, std::io::Error> {
        let rawpath = String::from(path);
        let newpath = dbfmt(&rawpath);
        if let Some(parent) = Path::new(&newpath).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        self.path = Some(newpath);
        Ok(self)
    }
    pub fn assign_rx(mut self, rx: mpsc::Receiver<IpCapture>) -> Self {
        self.capture_reciever = Some(rx);
        self
    }
    pub fn connect(mut self) -> Result<Self, duckdb::Error> {
        if let Some(path) = &self.path {
        let conn = Connection::open(&path)?;
        self.connection = Some(conn);
        }
        Ok(self)
    }
    pub fn create_tables(&self) -> Result<(), duckdb::Error> {
        if let Some(conn) = &self.connection {
            conn.execute_batch("BEGIN; \
                CREATE TABLE foo(x INTEGER) \
                COMMIT;")?;
        }
        Ok(())
    }
}
