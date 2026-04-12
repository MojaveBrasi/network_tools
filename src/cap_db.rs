use chrono::{DateTime, Local};
use sqlx::{Database, Sqlite, migrate::MigrateDatabase};
use std::path::{Path, PathBuf};
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

pub async fn create_database(filename: &str) {
    match Sqlite::create_database(&filename).await {
        Ok(_) => println!("SQlite database created: {}", filename,),
        Err(e) => println!("ERROR: {}", e),
    }
}

pub async fn write_captures_to_db(path: &Path) {
    todo!()
}
