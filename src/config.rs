use serde::{Deserialize, Serialize};
use std::fs;
use xdg::BaseDirectories;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub terminal: Option<String>,
    pub opacity: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub browser: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: None,
            opacity: Some(0.92),
            width: Some(800),
            height: Some(500),
            theme: Some("TokyoNight".to_string()),
            language: Some("EN".to_string()),
            browser: None,
        }
    }
}

impl Config {
    /// Loads configuration from ~/.config/castit/config.toml.
    /// If not present, creates it with defaults.
    pub fn load() -> Self {
        let xdg_dirs = BaseDirectories::with_prefix("castit");

        let config_path = match xdg_dirs.place_config_file("config.toml") {
            Ok(path) => path,
            Err(_) => return Self::default(),
        };

        if !config_path.exists() {
            let default_config = Self::default();
            if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
                let comments = "# CastIt Configuration File\n\
                                # Customize terminal, size, opacity, and other features.\n\n";
                let _ = fs::write(&config_path, format!("{}{}", comments, toml_str));
            }
            return default_config;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        toml::from_str(&content).unwrap_or_else(|_| Self::default())
    }

    /// Saves the current configuration back to ~/.config/castit/config.toml.
    pub fn save(&self) {
        let xdg_dirs = BaseDirectories::with_prefix("castit");

        let config_path = match xdg_dirs.place_config_file("config.toml") {
            Ok(path) => path,
            Err(_) => return,
        };

        if let Ok(toml_str) = toml::to_string_pretty(self) {
            let _ = fs::write(config_path, toml_str);
        }
    }
}
