use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Could not load settings")]
    SettingsNotLoaded,
    #[error("Could not call 'create_new' because settings file already exists")]
    AlreadyExists,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    JSON(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_settings_dir")]
    pub path: String,
    #[serde(default = "default_db_dir")]
    pub db_dir: String,
    #[serde(default = "default_db_name")]
    pub db_name: String,
    #[serde(default = "default_print_to_console")]
    pub print_to_console: bool,
}
fn default_settings_dir() -> String {
    "settings".to_string()
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

impl Settings {
    /// Create New settings file in default settings directory.
    /// Will return an error if the file exists already.
    /// Settings path is const for now.
    pub fn create_new() -> Result<(), StateError> {
        let dir = default_settings_dir();
        fs::create_dir_all(&dir)?;
        let path = dir + "/settings.json";
        if let Ok(true) = fs::exists(&path) {
            return Err(StateError::AlreadyExists);
        }
        let s = Settings {
            path: path,
            db_dir: default_db_dir(),
            db_name: default_db_name(),
            print_to_console: true,
        };
        let json = serde_json::to_string(&s)?;
        fs::write(&s.path, json)?;
        println!("Created settings file at {}", &s.path);
        Ok(())
    }
    /// Get settings. Will create default if not exists
    pub fn get() -> Result<Settings, StateError> {
        let path = default_settings_dir() + "/settings.json";
        match fs::exists(&path) {
            Ok(false) => {
                Settings::create_new()?;
                let json = fs::read_to_string(&path)?;
                let settings: Settings = serde_json::from_str(&json)?;
                Ok(settings)
            }
            Ok(true) => {
                let json = fs::read_to_string(&path)?;
                let settings: Settings = serde_json::from_str(&json)?;

                Ok(settings)
            }
            Err(e) => return Err(StateError::Io(e))
        }
    }
}


pub struct State {
    pub settings: Settings,
    pub capture_count: usize,
}

impl State {
    pub fn init(settings: Settings) -> State {
        State {
            settings,
            capture_count: 0,
        }
    }
}
