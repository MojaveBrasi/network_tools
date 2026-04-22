use crate::IpCapture;
use crate::network::ip_to_bytes;
use chrono::{DateTime, Local};
use sqlx::{Pool, QueryBuilder, Row, Sqlite, SqlitePool, migrate::MigrateDatabase};
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
#[derive(Debug, sqlx::FromRow)]
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

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database already exists")]
    AlreadyExists,

    #[error("Database contains no tables")]
    NoTables,

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("SQLx migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
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

// Max depth for search is 2. Don't hide your db
pub fn get_databases(db_path: &Path) -> Vec<PathBuf> {
    WalkDir::new(db_path)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| has_sqlite_extension(e.path()) || is_sqlite_magic(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn list_databases(db_path: &Path) {
    println!("---DATABASES---");
    let db_list = get_databases(db_path);
    if db_list.is_empty() {
        println!("No databases found.");
    } else {
        let mut count: u8 = 0;
        for db in db_list {
            count += 1;
            println!(
                "{}: {}",
                count,
                db.to_str().unwrap_or("How the fuck did this method fail?")
            );
        }
    }
}

fn dbfmt(db_name: &str) -> String {
    if db_name.ends_with(".db") {
        db_name.to_string()
    } else {
        format!("{}.db", db_name)
    }
}

pub async fn create_db(db_name: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let name = dbfmt(db_name);
    //This check SHOULD be redundant since ideally the caller checks if a db exists
    //before calling create_db... But this makes it idiot proof
    if Sqlite::database_exists(&name).await? {
        return Err(DatabaseError::AlreadyExists);
    }
    Sqlite::create_database(&name).await?;
    let pool = SqlitePool::connect(&name).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn create_sqlite_pool(db_name: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    let name = dbfmt(db_name);
    if !Sqlite::database_exists(db_name).await? {
        create_db(db_name).await
    } else {
        Ok(SqlitePool::connect(db_name).await?)
    }
}

pub async fn database_info(db_name: &str) -> Result<Vec<String>, DatabaseError> {
    let name = dbfmt(db_name);
    let pool = SqlitePool::connect(&db_name).await?;

    let rows = sqlx::query(
        "SELECT sql FROM sqlite_master \
         WHERE sql IS NOT NULL \
         ORDER BY tbl_name, type DESC, name",
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        return Err(DatabaseError::NoTables);
    }

    let ddl = rows.iter().map(|row| row.get::<String, _>("sql")).collect();
    Ok(ddl)
}

async fn flush(pool: &SqlitePool, buffer: &mut Vec<IpCapture>) -> Result<(), DatabaseError> {
    let mut qb = QueryBuilder::<Sqlite>::new(
        "INSERT INTO packet_capture (timestamp, src_ip, dst_ip, protocol, length)",
    );
    // Not storing the ethertype (ipv4 or ipv6) because I'm converting both to bytes here
    qb.push_values(buffer.iter(), |mut row, packet| {
        row.push_bind(packet.timestamp.timestamp_nanos_opt().unwrap_or(0))
            .push_bind(ip_to_bytes(packet.source).to_vec())
            .push_bind(ip_to_bytes(packet.dest).to_vec())
            .push_bind(packet.transport_protocol.to_string())
            .push_bind(packet.length);
    });
    qb.build().execute(pool).await?;
    Ok(())
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
                        if buffer.len() >= 256 {
                            if let Err(e) = flush(&pool, &mut buffer).await {
                                tracing::error!("Flush failed. Moving on");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ = flush_timer.tick() => {
                    if let Err(e) = flush(&pool, &mut buffer).await {
                        tracing::error!("Flush failed. Moving on");
                    }
                }
        }
    }
}
