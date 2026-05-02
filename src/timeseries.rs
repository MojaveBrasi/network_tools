use crate::IpCapture;
use std::sync::mpsc;

use duckdb::{Connection, Result};

pub struct TimeSeriesWriter<'a> {
    capture_reciever: &'a mpsc::Receiver<IpCapture>,
    connection: Connection,
    path: String,
}

pub fn open_conn() -> Result<(), duckdb::Error> {
    let path = "./testpath.db";
    let conn = Connection::open(&path)?;
    println!("{}", conn.is_autocommit());
    Ok(())
}
