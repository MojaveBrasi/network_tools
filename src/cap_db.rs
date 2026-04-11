use bytesize::ByteSize;
use chrono::{DateTime, Local};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::UNIX_EPOCH;
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

fn has_sqlite_extension(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|s| s.to_str()),
        Some("db" | "sqlite" | "sqlite3")
    )
}

/* Every valid SQLite database file begins with the following 16 bytes (in hex):
53 51 4c 69 74 65 20 66 6f 72 6d 61 74 20 33 00.
This byte sequence corresponds to the UTF-8 string "SQLite format 3"
including the nul terminator character at the end.*/
fn is_sqlite_magic(p: &Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(p) else {
        return false;
    };
    let mut magic = [0u8; 16];
    matches!(f.read_exact(&mut magic), Ok(())) && magic == *b"SQLite format 3\0"
}

pub fn get_databases(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| has_sqlite_extension(e.path()) || is_sqlite_magic(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn list_databases(root: &Path) {
    let dbs = get_databases(root);
    println!("DATABASES: {:?}", dbs); //TODO: impl Display for Vec<PathBuf>
}