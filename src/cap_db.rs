use crate::IpCapture;
use anyhow::bail;
use chrono::{DateTime, Local};
use sqlx::{Pool, QueryBuilder, Sqlite, SqlitePool, migrate::MigrateDatabase, query_builder};
use std::path::{Path, PathBuf};
use tokio::{sync::mpsc, time::Duration, time::interval};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct DbInfo {
    pub path: PathBuf,
    pub file_size: u64,
    pub last_modified: DateTime<Local>,
    pub tables: Vec<TableInfo>,
}

/// Per-table summary.
#[derive(Debug)]
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,
    pub columns: Vec<ColumnInfo>,
}

/// One column from PRAGMA table_info.
#[derive(Debug)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub col_type: String,
    pub not_null: bool,
    pub primary_key: bool,
}

fn has_sqlite_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("db" | "sqlite" | "sqlite3")
    )
}

/* Every valid SQLite database file begins with the following 16 bytes (in hex):
53 51 4c 69 74 65 20 66 6f 72 6d 61 74 20 33 00.
This byte sequence corresponds to the UTF-8 string "SQLite format 3"
including the null terminator character at the end.*/
fn is_sqlite_magic(p: &Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(p) else {
        return false;
    };
    let mut magic = [0u8; 16];
    matches!(f.read_exact(&mut magic), Ok(())) && magic == *b"SQLite format 3\0"
}

// Why does this take 1.14 seconds?
// Fuck it, just load this at startup in the background.
pub fn get_databases(rootpath: &Path) -> Vec<PathBuf> {
    WalkDir::new(rootpath)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| has_sqlite_extension(e.path()) || is_sqlite_magic(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn list_databases(rootpath: &Path) {
    println!("---DATABASES---");
    let db_list = get_databases(rootpath);
    if db_list.is_empty() {
        println!("No databases found.");
    } else {
        let mut count: u8 = 0;
        for db in db_list {
            count += 1;
            println!("{}: {}", count, db.to_str().unwrap_or("Invalid. Seek help"));
            //TODO: handle to_str() failure better... eventually... maybe
        }
    }
}

pub async fn create_db(path: &str) -> Result<Pool<Sqlite>, anyhow::Error> {
    //This check SHOULD be redundant since ideally the caller checks if a db exists
    //before calling create_db... But this makes it idiot proof
    if Sqlite::database_exists(path).await.unwrap_or(false) {
        bail!("Database already exists");
    }
    Sqlite::create_database(path).await?;
    let pool = SqlitePool::connect(path)
        .await
        .expect("could not connect to the database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn create_sqlite_pool(path: &str) -> Result<Pool<Sqlite>, anyhow::Error> {
    if !Sqlite::database_exists(path).await.unwrap_or(false) {
        create_db(path).await
    } else {
        Ok(SqlitePool::connect(path).await?)
    }
}

//Write bulk data to db
async fn flush(pool: &SqlitePool, buffer: &mut Vec<IpCapture>) {
    let mut qb = QueryBuilder::<Sqlite>::new(
        "INSERT INTO packet_capture (timestamp, src_ip, dst_ip, protocol, length)",
    );
    //TODO: URGENT: Implement the right casts for these datatypes. to_string is a bad idea
    qb.push_values(buffer.iter(), |mut row, packet| {
        row.push_bind(packet.timestamp.to_string())
            .push_bind(packet.source.to_string())
            .push_bind(packet.ethernet_frame_type.to_string())
            .push_bind(packet.transport_protocol.to_string())
            .push_bind(packet.length);
    });
}

pub async fn write_captures_to_db(mut rx: mpsc::Receiver<IpCapture>, pool: SqlitePool) {
    let mut buffer: Vec<IpCapture> = Vec::with_capacity(256); //Capacity should a settings variable
    let mut flush_timer = interval(Duration::from_millis(100));
    flush_timer.tick().await;
    loop {
        tokio::select! {
            packet = rx.recv() => {
                match packet {
                    Some(p) => {
                        buffer.push(p);
                        if buffer.len() > 256 {
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
