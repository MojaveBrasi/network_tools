use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    default_db_path: String, //Always exists. If not set, then value is "."
}

pub fn create_settings() {
    let dbpath = String::from(".");
    let settings = Settings {
        default_db_path: dbpath,
    };
}
