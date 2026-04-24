use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    default_db_dir: String,  //Always exists. If not set, then value is "."
    default_db_name: String, //Always exists. If not set, then value is "."
}

pub fn create_settings() {
    let dbpath = String::from("databases");
    let dbname = String::from("hello");
    let settings = Settings {
        default_db_dir: dbpath,
        default_db_name: dbname,
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
