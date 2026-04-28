use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_db_dir")]
    db_dir: String,
    #[serde(default = "default_db_name")]
    db_name: String,
    #[serde(default = "default_print_to_console")]
    print_to_console: bool,
}

fn default_db_dir() -> String {
    ".".to_string()
}
fn default_db_name() -> String {
    "main.db".to_string()
}
fn default_print_to_console() -> bool {
    false
}

pub fn create_settings() {
    let dbpath = String::from("databases");
    let dbname = String::from("hello");
    let settings = Settings {
        db_dir: dbpath,
        db_name: dbname,
        print_to_console: true,
    };
}

pub struct State {
    pub default_db_dir: String,
    pub default_db_name: String,
    pub capture_count: usize,
}

impl State {
    pub fn init() -> State {
        State {
            default_db_dir: String::from("databases"),
            default_db_name: String::from("hello"),
            capture_count: 0,
        }
    }
}
