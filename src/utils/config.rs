use std::fs;
use std::path::PathBuf;

use dirs_next::config_dir;
use serde::{Deserialize, Serialize};
use toml;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConfigError {
    NoConfigDirectory,
    ConfigFileNotFound(PathBuf),
    ConfigFileReadError(String),
    ConfigFileParseError(String),
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    pub email: String,
}

pub fn get_config_file_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("pigeon").join("config.toml"))
}



pub fn create_config_file() {
    let config_dir = match config_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Could not find config directory.");
            return;
        }
    };
    let config_path = config_dir.join("pigeon");

    if !config_path.exists() {
        if let Err(e) = fs::create_dir_all(&config_path) {
            eprintln!("Failed to create config directory: {}", e);
            return;
        }
    }
    let config_file_path = config_path.join("config.toml");
    if !config_file_path.exists() {
        if let Err(e) = fs::write(&config_file_path, "") {
            eprintln!("Failed to create config file: {}", e);
            return;
        }
    }

    println!("Config file created at: {}", config_file_path.display());
}



pub fn load_user_config() -> Result<UserConfig, ConfigError> {
    let config_dir = match config_dir() {
        Some(dir) => dir,
        None => return Err(ConfigError::NoConfigDirectory),
    };
    let config_path = config_dir.join("pigeon").join("config.toml");

    if !config_path.exists() {
        return Err(ConfigError::ConfigFileNotFound(config_path));
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            return Err(ConfigError::ConfigFileReadError(e.to_string()));
        }
    };

    let user_config: UserConfig = match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config file: {}", e);
            return Err(ConfigError::ConfigFileParseError(e.to_string()));
        }
    };

    Ok(user_config)
}
